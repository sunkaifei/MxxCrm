//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::crm::model::customer::{CustomerDetailVO, CustomerListQuery, CustomerListVO, CustomerModel, CustomerSaveDTO, CustomerSaveRequest, CustomerUpdateRequest};
use crate::modules::crm::entity::customer;
use crate::modules::crm::entity::{opportunity, opportunity::Entity as Opportunity, contact, contact::Entity as Contact};
use crate::modules::crm::service::assign_history_service;
use crate::modules::crm::service::customer_edit_log_service;
use crate::modules::company::service::code_rule_service;
use crate::modules::system::entity::{admin::Entity as Admin, tag, tag_merge};
use crate::modules::system::model::admin_dept_merge::AdminDeptMergeModel;
use crate::modules::system::model::dept::DeptModel;
use crate::modules::system::service::role_service;
use sea_orm::{DbConn, ColumnTrait, DatabaseTransaction, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect, TransactionTrait, ConnectionTrait};
use std::collections::{HashMap, HashSet};

/// 递归获取指定部门及其所有子部门的ID列表
/// 根据用户ID获取其数据权限范围内的所有用户ID
///
/// 已迁移至 [`data_scope_service::get_accessible_user_ids`]，支持多角色合并。
/// 保留此函数签名是为了兼容现有调用方（如 followup_service）。
/// 参数 `data_scope` 已弃用，内部会自动查询用户所有角色并合并权限。
pub async fn get_accessible_user_ids(
    db: &DbConn,
    current_user_id: i64,
    _data_scope: Option<i32>,
) -> Result<Option<Vec<i64>>> {
    crate::modules::system::service::data_scope_service::get_accessible_user_ids(db, current_user_id).await
}

/// 检查客户名称是否已存在（按 customer_type 区分查重字段）
/// customer_type: 1=企业（按 company_name 查重），2=个人（按 person_name 查重）
/// name: 名称
/// exclude_id: 排除的客户ID（编辑时传入当前客户ID，新建时传 None）
pub async fn check_customer_name(
    db: &impl ConnectionTrait,
    customer_type: i32,
    name: &str,
    exclude_id: Option<i64>,
) -> Result<bool> {
    let mut query = customer::Entity::find()
        .filter(customer::Column::Deleted.eq(0))
        .filter(customer::Column::CustomerType.eq(customer_type));
    query = match customer_type {
        1 => query.filter(customer::Column::CompanyName.eq(name)),
        2 => query.filter(customer::Column::PersonName.eq(name)),
        _ => return Err(Error::from("无效的客户类型，仅支持 1=企业, 2=个人")),
    };
    if let Some(id) = exclude_id {
        query = query.filter(customer::Column::Id.ne(id));
    }
    let count = query
        .count(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(count > 0)
}

pub async fn insert(db: &DbConn, form_data: &CustomerSaveRequest, created_by: i64) -> Result<i64> {
    let txn = db.begin().await?;

    // 1. 客户类型默认值
    let customer_type = form_data.customer_type.unwrap_or(1);
    if customer_type != 1 && customer_type != 2 {
        txn.rollback().await?;
        return Err(Error::from("无效的客户类型，仅支持 1=企业, 2=个人"));
    }

    // 2. 按类型校验必填字段与查重
    match customer_type {
        1 => {
            // 企业客户：公司名称必填且不重复
            let name = form_data.company_name.as_deref().unwrap_or("").trim();
            if name.is_empty() {
                txn.rollback().await?;
                return Err(Error::from("企业客户必须填写公司名称"));
            }
            if check_customer_name(&txn, 1, name, None).await? {
                txn.rollback().await?;
                return Err(Error::from(format!("公司名称「{}」已存在", name)));
            }
        }
        2 => {
            // 个人客户：姓名必填且不重复
            let pname = form_data.person_name.as_deref().unwrap_or("").trim();
            if pname.is_empty() {
                txn.rollback().await?;
                return Err(Error::from("个人客户必须填写姓名"));
            }
            if check_customer_name(&txn, 2, pname, None).await? {
                txn.rollback().await?;
                return Err(Error::from(format!("个人姓名「{}」已存在", pname)));
            }
        }
        _ => unreachable!(),
    }

    let mut dto: CustomerSaveDTO = form_data.clone().into();
    dto.customer_type = Some(customer_type);
    dto.created_by = Some(created_by);
    // 新建客户时，若未指定负责人，默认归属当前登录用户（当前销售）
    if dto.assigned_to.is_none() {
        dto.assigned_to = Some(created_by);
    }

    // 3. 客户编号：优先使用编码规则；若无规则，按类型生成简易编号（ENT- / PER-）
    if let Ok(code) = code_rule_service::generate_code(&txn, "customer", None, None, None).await {
        dto.customer_no = Some(code);
    } else if dto.customer_no.as_deref().unwrap_or("").trim().is_empty() {
        let prefix = if customer_type == 2 { "PER" } else { "ENT" };
        let date_part = chrono::Local::now().format("%Y%m%d").to_string();
        // 用计数方式生成尾号（同日同类型数量+1）
        let today_count = customer::Entity::find()
            .filter(customer::Column::CustomerType.eq(customer_type))
            .filter(customer::Column::CreateTime.gte(
                chrono::NaiveDateTime::parse_from_str(
                    &format!("{} 00:00:00", chrono::Local::now().format("%Y-%m-%d")),
                    "%Y-%m-%d %H:%M:%S"
                ).ok()
            ))
            .count(&txn)
            .await
            .unwrap_or(0);
        dto.customer_no = Some(format!("{}-{}-{:04}", prefix, date_part, today_count + 1));
    }

    let result = CustomerModel::insert(&txn, &dto).await?;

    // 4. 新建客户时，如果有负责人，记录初始分配历史
    if let Some(aid) = dto.assigned_to {
        let _ = assign_history_service::record_claim(&txn, result, aid).await;
    }

    txn.commit().await?;
    Ok(result)
}

pub async fn update(db: &DbConn, form_data: &CustomerUpdateRequest, updated_by: i64) -> Result<i64> {
    let customer_id = form_data.id.unwrap_or_default();
    let txn = db.begin().await?;

    // 1. 查询旧数据
    let old_model = customer::Entity::find_by_id(customer_id)
        .filter(customer::Column::Deleted.eq(0))
        .one(&txn)
        .await?
        .ok_or_else(|| Error::from("客户不存在"))?;

    // 2. 类型以旧数据为准，禁止通过 update 修改类型
    let customer_type = old_model.customer_type.unwrap_or(1);

    // 3. 按类型校验必填字段与查重
    match customer_type {
        1 => {
            let name = form_data.company_name.as_deref().unwrap_or("").trim();
            if name.is_empty() {
                txn.rollback().await?;
                return Err(Error::from("企业客户必须填写公司名称"));
            }
            if check_customer_name(&txn, 1, name, Some(customer_id)).await? {
                txn.rollback().await?;
                return Err(Error::from(format!("公司名称「{}」已存在", name)));
            }
        }
        2 => {
            let pname = form_data.person_name.as_deref().unwrap_or("").trim();
            if pname.is_empty() {
                txn.rollback().await?;
                return Err(Error::from("个人客户必须填写姓名"));
            }
            if check_customer_name(&txn, 2, pname, Some(customer_id)).await? {
                txn.rollback().await?;
                return Err(Error::from(format!("个人姓名「{}」已存在", pname)));
            }
        }
        _ => {}
    }

    // 4. 执行更新
    let mut dto: CustomerSaveDTO = form_data.clone().into();
    // 强制保持原类型，防止前端传值覆盖
    dto.customer_type = Some(customer_type);
    dto.updated_by = Some(updated_by);
    let result = CustomerModel::update_by_id(&txn, &form_data.id, &dto).await?;

    // 4.1 如果负责人(assigned_to)发生变化，级联更新关联业务数据的负责人
    let old_assignee = old_model.assigned_to;
    let new_assignee = dto.assigned_to;
    if old_assignee != new_assignee && new_assignee.is_some() {
        let new_uid = new_assignee.unwrap();
        log::info!(
            "[customer_update] 客户(id={})负责人变更: {:?} -> {}, 级联更新关联业务数据",
            customer_id, old_assignee, new_uid
        );

        // 直接调用级联更新（不需要重复校验/记录历史，因为这是编辑的一部分）
        cascade_update_related_assignees(&txn, customer_id, new_uid).await?;
    }

    // 5. 记录修改日志（如有差异）
    let old_json = serde_json::to_value(&old_model).unwrap_or_default();
    let new_json = serde_json::to_value(&dto).unwrap_or_default();
    let editor_name = Admin::find_by_id(updated_by)
        .one(&txn)
        .await
        .ok()
        .flatten()
        .and_then(|a| a.nick_name.or(a.user_name));
    let _ = customer_edit_log_service::log_update(
        &txn, customer_id, updated_by, editor_name, &old_json, &new_json, Some(0),
    ).await;

    txn.commit().await?;
    Ok(result)
}

/// 级联更新客户关联业务数据的负责人（商机/合同/报价单/订单/发票/回款/回款计划）
async fn cascade_update_related_assignees(
    txn: &DatabaseTransaction,
    customer_id: i64,
    new_user_id: i64,
) -> Result<()> {
    use crate::modules::crm::entity::{opportunity, contract};
    use crate::modules::sale::entity::{quotation, order, invoice, payment};
    use crate::modules::crm::entity::contract_payment_plan;
    use sea_orm::sea_query::Expr;

    // 1. 商机 assigned_to
    let _ = opportunity::Entity::update_many()
        .col_expr(opportunity::Column::AssignedTo, Expr::value(Some(new_user_id)))
        .filter(opportunity::Column::CustomerId.eq(customer_id))
        .filter(opportunity::Column::Deleted.eq(0))
        .exec(txn).await;

    // 2. 合同 assigned_to
    let _ = contract::Entity::update_many()
        .col_expr(contract::Column::AssignedTo, Expr::value(Some(new_user_id)))
        .filter(contract::Column::CustomerId.eq(customer_id))
        .filter(contract::Column::Deleted.eq(0))
        .exec(txn).await;

    // 2.1 收集合同ID供回款计划使用
    let contract_ids: Vec<i64> = contract::Entity::find()
        .filter(contract::Column::CustomerId.eq(customer_id))
        .filter(contract::Column::Deleted.eq(0))
        .all(txn).await.unwrap_or_default()
        .into_iter().map(|c| c.id).collect();

    // 3. 回款计划 owner_user_id
    if !contract_ids.is_empty() {
        let _ = contract_payment_plan::Entity::update_many()
            .col_expr(contract_payment_plan::Column::OwnerUserId, Expr::value(Some(new_user_id)))
            .filter(contract_payment_plan::Column::ContractId.is_in(contract_ids))
            .filter(contract_payment_plan::Column::Deleted.eq(0))
            .exec(txn).await;
    }

    // 4. 报价单 owner_user_id
    let _ = quotation::Entity::update_many()
        .col_expr(quotation::Column::OwnerUserId, Expr::value(Some(new_user_id)))
        .filter(quotation::Column::CustomerId.eq(customer_id))
        .filter(quotation::Column::Deleted.eq(0))
        .exec(txn).await;

    // 5. 订单 owner_user_id
    let _ = order::Entity::update_many()
        .col_expr(order::Column::OwnerUserId, Expr::value(Some(new_user_id)))
        .filter(order::Column::CustomerId.eq(customer_id))
        .filter(order::Column::Deleted.eq(0))
        .exec(txn).await;

    // 6. 发票 owner_user_id
    let _ = invoice::Entity::update_many()
        .col_expr(invoice::Column::OwnerUserId, Expr::value(Some(new_user_id)))
        .filter(invoice::Column::CustomerId.eq(customer_id))
        .filter(invoice::Column::Deleted.eq(0))
        .exec(txn).await;

    // 7. 回款 owner_user_id
    let _ = payment::Entity::update_many()
        .col_expr(payment::Column::OwnerUserId, Expr::value(Some(new_user_id)))
        .filter(payment::Column::CustomerId.eq(customer_id))
        .filter(payment::Column::Deleted.eq(0))
        .exec(txn).await;

    Ok(())
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<i64>, deleted_by: i64) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let txn = db.begin().await?;

    // 查询待删除的旧数据（用于日志）
    let old_models = customer::Entity::find()
        .filter(customer::Column::Id.is_in(ids_vec.clone()))
        .filter(customer::Column::Deleted.eq(0))
        .all(&txn)
        .await?;
    let editor_name = Admin::find_by_id(deleted_by)
        .one(&txn)
        .await
        .ok()
        .flatten()
        .and_then(|a| a.nick_name.or(a.user_name));
    // 先逐条记录删除日志
    for m in &old_models {
        let old_json = serde_json::to_value(m).unwrap_or_default();
        let _ = customer_edit_log_service::log_delete(
            &txn, m.id, deleted_by, editor_name.clone(), &old_json,
        ).await;
    }
    let result = CustomerModel::batch_delete_by_ids(&txn, &ids_vec).await?;

    txn.commit().await?;
    Ok(result)
}

pub async fn find_by_id(db: &DbConn, id: i64) -> Result<CustomerDetailVO> {
    let result = CustomerModel::find_by_id(db, id).await?;
    match result {
        Some(item) => {
            let mut vo: CustomerDetailVO = item.into();
            let assignee_id = vo.assigned_to;

            // 查询客户的跟进记录
            let followups = crate::modules::crm::model::followup::FollowupModel::select_by_customer_id(&db, id).await?;

            // 合并 客户负责人 + 创建人 + 所有跟进记录创建人 一次 IN 查询用户名
            let mut user_ids: Vec<i64> = followups.iter()
                .filter_map(|f| f.created_by)
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();
            if let Some(aid) = assignee_id {
                user_ids.push(aid);
            }
            if let Some(cbid) = vo.created_by {
                user_ids.push(cbid);
            }
            let name_map = crate::modules::system::service::admin_service::build_admin_name_map(db, user_ids).await;

            if let Some(aid) = assignee_id {
                vo.assigned_to_name = name_map.get(&aid).cloned();
            }
            if let Some(cbid) = vo.created_by {
                vo.created_by_name = name_map.get(&cbid).cloned();
            }

            let followup_vo_list: Vec<crate::modules::crm::model::followup::FollowupListVO> = followups.into_iter().map(|f| {
                let mut f_vo: crate::modules::crm::model::followup::FollowupListVO = f.into();
                if let Some(created_by) = f_vo.created_by {
                    f_vo.created_by_name = name_map.get(&created_by).cloned();
                }
                f_vo
            }).collect();

            vo.followups = Some(followup_vo_list);
            Ok(vo)
        },
        None => Err(Error::from("客户不存在")),
    }
}

/// 批量填充客户列表的负责人名称和创建人名称（避免 N+1 查询）
async fn fill_assignee_and_creator_names(
    db: &DbConn,
    list: Vec<customer::Model>,
    total: i64,
    page: i64,
    page_size: i64,
) -> Result<ResultPage<Vec<CustomerListVO>>> {
    // 收集所有需要查询名称的用户ID（负责人 + 创建人，去重）
    let user_ids: Vec<i64> = list.iter()
        .flat_map(|c| [c.assigned_to, c.created_by])
        .flatten()
        .collect();

    // 统一调用共用方法（内部已去重 + deleted=0 过滤）
    let user_map = crate::modules::system::service::admin_service::build_admin_name_map(db, user_ids).await;

    // 批量查询客户标签（IN 查询，避免 N+1）
    let customer_ids: Vec<i64> = list.iter().map(|c| c.id).collect();
    let tag_map = batch_query_customer_tags(db, &customer_ids).await?;

    // 批量统计商机数量
    let mut opportunity_count_map: HashMap<i64, i64> = HashMap::new();
    if !customer_ids.is_empty() {
        let opp_rows = Opportunity::find()
            .select_only()
            .column(opportunity::Column::CustomerId)
            .column_as(opportunity::Column::Id.count(), "cnt")
            .filter(opportunity::Column::CustomerId.is_in(customer_ids.clone()))
            .filter(opportunity::Column::Deleted.eq(0))
            .group_by(opportunity::Column::CustomerId)
            .into_tuple::<(i64, i64)>()
            .all(db)
            .await?;
        for (cid, cnt) in opp_rows {
            opportunity_count_map.insert(cid, cnt);
        }
    }

    // 批量统计联系人数量
    let mut contact_count_map: HashMap<i64, i64> = HashMap::new();
    if !customer_ids.is_empty() {
        let contact_rows = Contact::find()
            .select_only()
            .column(contact::Column::CustomerId)
            .column_as(contact::Column::Id.count(), "cnt")
            .filter(contact::Column::CustomerId.is_in(customer_ids.clone()))
            .filter(contact::Column::Deleted.eq(0))
            .group_by(contact::Column::CustomerId)
            .into_tuple::<(i64, i64)>()
            .all(db)
            .await?;
        for (cid, cnt) in contact_rows {
            contact_count_map.insert(cid, cnt);
        }
    }

    let data: Vec<CustomerListVO> = list.into_iter().map(|item| {
        let assignee_id = item.assigned_to;
        let creator_id = item.created_by;
        let cid = item.id;
        let mut vo: CustomerListVO = item.into();
        vo.assignee_name = assignee_id.and_then(|id| user_map.get(&id).cloned());
        vo.created_by_name = creator_id.and_then(|id| user_map.get(&id).cloned());
        vo.tags = tag_map.get(&cid).cloned();
        vo.opportunity_count = opportunity_count_map.get(&cid).copied();
        vo.contact_count = contact_count_map.get(&cid).copied();
        vo
    }).collect();

    Ok(ResultPage::new(data, total, page, page_size))
}

/// 批量查询多个客户的标签关联，返回 customer_id -> Vec<CustomerTagVO> 的映射
async fn batch_query_customer_tags(
    db: &DbConn,
    customer_ids: &[i64],
) -> Result<HashMap<i64, Vec<crate::modules::crm::model::customer::CustomerTagVO>>> {
    if customer_ids.is_empty() {
        return Ok(HashMap::new());
    }

    // 一次 IN 查询所有关联记录
    let merges = tag_merge::Entity::find()
        .filter(tag_merge::Column::EntityType.eq("customer"))
        .filter(tag_merge::Column::EntityId.is_in(customer_ids.to_vec()))
        .all(db)
        .await?;

    if merges.is_empty() {
        return Ok(HashMap::new());
    }

    // 收集去重的 tag_id
    let tag_ids: Vec<i64> = merges.iter()
        .filter_map(|m| m.tag_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    // 一次 IN 查询所有标签详情
    let tags = tag::Entity::find()
        .filter(tag::Column::Id.is_in(tag_ids))
        .filter(tag::Column::Deleted.eq(0))
        .all(db)
        .await?;

    let tag_detail_map: HashMap<i64, tag::Model> = tags.into_iter().map(|t| (t.id, t)).collect();

    // 组装 customer_id -> Vec<CustomerTagVO>
    let mut result: HashMap<i64, Vec<crate::modules::crm::model::customer::CustomerTagVO>> = HashMap::new();
    for m in merges {
        if let (Some(cid), Some(tid)) = (m.entity_id, m.tag_id) {
            if let Some(t) = tag_detail_map.get(&tid) {
                result.entry(cid).or_default().push(
                    crate::modules::crm::model::customer::CustomerTagVO {
                        id: Some(t.id),
                        tag_name: t.tag_name.clone(),
                        tag_color: t.tag_color.clone(),
                    }
                );
            }
        }
    }

    Ok(result)
}

pub async fn list(db: &DbConn, query: &CustomerListQuery, current_user_id: i64) -> Result<ResultPage<Vec<CustomerListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let list_type = query.list_type.as_deref().unwrap_or("all");

    match list_type {
        "my" => {
            // 我的客户：只看自己负责的
            let (list, total) = CustomerModel::select_in_page(
                &db, page, page_size,
                query.keywords.clone(), query.customer_type.clone(),
                query.level.clone(), query.country.clone(), query.source.clone(),
                Some(current_user_id),
            ).await?;
            fill_assignee_and_creator_names(db, list, total, page, page_size).await
        }
        "subordinate" => {
            // 下属客户：按汇报关系（direct_manager_id）递归查找所有下属，含跨级别
            let subordinate_ids = crate::modules::system::service::subordinate_service
                ::get_subordinate_ids_default(db, current_user_id).await?;
            let (list, total) = CustomerModel::select_in_page_by_assigned_ids(
                &db, page, page_size,
                query.keywords.clone(), query.customer_type.clone(),
                query.level.clone(), query.country.clone(), query.source.clone(),
                Some(subordinate_ids),
            ).await?;
            fill_assignee_and_creator_names(db, list, total, page, page_size).await
        }
        "todayFollow" => {
            // 今日跟进客户：关联 followup 表过滤
            // 使用用户实际的数据权限范围
            let roles = role_service::select_by_admin_id(db, &Some(current_user_id)).await?;
            let data_scope = roles.iter()
                .filter_map(|r| r.data_scope)
                .min();
            let user_ids = get_accessible_user_ids(db, current_user_id, data_scope).await?;

            let (list, total) = CustomerModel::select_today_follow_page(
                &db, page, page_size,
                query.keywords.clone(), query.customer_type.clone(),
                query.level.clone(), query.country.clone(), query.source.clone(),
                user_ids,
            ).await?;
            fill_assignee_and_creator_names(db, list, total, page, page_size).await
        }
        _ => {
            // all：根据数据权限过滤全部客户
            match crate::modules::system::service::data_scope_service
                ::get_accessible_user_ids(db, current_user_id).await? {
                None => {
                    // 全部数据 - 不过滤负责人
                    let (list, total) = CustomerModel::select_in_page(
                        &db, page, page_size,
                        query.keywords.clone(), query.customer_type.clone(),
                        query.level.clone(), query.country.clone(), query.source.clone(),
                        None,
                    ).await?;
                    fill_assignee_and_creator_names(db, list, total, page, page_size).await
                }
                Some(user_ids) => {
                    let assigned_ids = if user_ids.is_empty() { None } else { Some(user_ids) };
                    let (list, total) = CustomerModel::select_in_page_by_assigned_ids(
                        &db, page, page_size,
                        query.keywords.clone(), query.customer_type.clone(),
                        query.level.clone(), query.country.clone(), query.source.clone(),
                        assigned_ids,
                    ).await?;
                    fill_assignee_and_creator_names(db, list, total, page, page_size).await
                }
            }
        }
    }
}

/// 公海客户列表
pub async fn pool_list(db: &DbConn, query: &CustomerListQuery) -> Result<ResultPage<Vec<CustomerListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let (list, total) = CustomerModel::select_pool_in_page(
        &db,
        page,
        page_size,
        query.keywords.clone(),
        query.customer_type.clone(),
        query.level.clone(),
        query.country.clone(),
        query.source.clone(),
        query.industry.clone(),
    ).await?;

    // 批量查询创建人名称（统一调用共用方法）
    let creator_ids: Vec<i64> = list.iter()
        .filter_map(|item| item.created_by)
        .collect();
    let creator_map = crate::modules::system::service::admin_service::build_admin_name_map(db, creator_ids).await;

    // 批量查询客户标签
    let customer_ids: Vec<i64> = list.iter().map(|c| c.id).collect();
    let tag_map = batch_query_customer_tags(db, &customer_ids).await?;

    let data: Vec<CustomerListVO> = list.into_iter().map(|item| {
        let created_by = item.created_by;
        let cid = item.id;
        let created_by_name = created_by.and_then(|id| creator_map.get(&id).cloned());
        let mut vo: CustomerListVO = item.into();
        vo.created_by_name = created_by_name;
        vo.tags = tag_map.get(&cid).cloned();
        vo
    }).collect();

    Ok(ResultPage::new(data, total, page, page_size))
}

/// 领取公海客户
pub async fn claim(db: &DbConn, id: i64, user_id: i64) -> Result<i64> {
    let txn = db.begin().await?;

    let customer = CustomerModel::find_by_id(&txn, id).await?;
    if customer.is_none() {
        txn.rollback().await?;
        return Err(Error::from("客户不存在"));
    }
    if customer.unwrap().assigned_to.is_some() {
        txn.rollback().await?;
        return Err(Error::from("该客户已被领取，无法重复领取"));
    }
    let result = CustomerModel::claim(&txn, id, user_id).await?;
    if result == 0 {
        txn.rollback().await?;
        return Err(Error::from("领取失败，该客户可能已被他人领取"));
    }
    // 记录分配历史
    let _ = assign_history_service::record_claim(&txn, id, user_id).await;

    txn.commit().await?;
    Ok(result)
}

/// 退回公海
pub async fn add_to_pool(db: &DbConn, id: i64, user_id: i64) -> Result<i64> {
    let txn = db.begin().await?;

    let customer = CustomerModel::find_by_id(&txn, id).await?;
    if customer.is_none() {
        txn.rollback().await?;
        return Err(Error::from("客户不存在"));
    }
    let assigned_to = customer.unwrap().assigned_to;
    let result = CustomerModel::add_to_pool(&txn, id, user_id).await?;
    // 记录退回历史
    if let Some(aid) = assigned_to {
        let _ = assign_history_service::record_release(&txn, id, aid).await;
    }

    txn.commit().await?;
    Ok(result)
}
