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
use crate::modules::crm::entity::customer;
use crate::modules::crm::model::customer::{CustomerModel, CustomerSaveDTO};
use crate::modules::crm::model::followup::FollowupModel;
use crate::modules::crm::model::lead::{LeadDetailVO, LeadListQuery, LeadListVO, LeadModel, LeadSaveDTO, LeadSaveRequest, LeadTagVO, LeadUpdateRequest};
use crate::modules::system::entity::{admin, admin::Entity as Admin, tag, tag_merge};
use crate::modules::system::model::admin_dept_merge::AdminDeptMergeModel;
use crate::modules::system::model::dept::DeptModel;
use crate::modules::system::service::role_service;
use sea_orm::{DbConn, DbErr, TransactionTrait, ColumnTrait, EntityTrait, QueryFilter};
use std::collections::{HashMap, HashSet};

pub async fn insert(db: &DbConn, form_data: &LeadSaveRequest, created_by: i64) -> Result<i64> {
    let mut dto: LeadSaveDTO = form_data.clone().into();
    dto.created_by = Some(created_by);
    let result = LeadModel::insert(&db, &dto).await?;
    Ok(result)
}

pub async fn update(db: &DbConn, form_data: &LeadUpdateRequest, updated_by: i64) -> Result<i64> {
    let mut dto: LeadSaveDTO = form_data.clone().into();
    dto.updated_by = Some(updated_by);
    let result = LeadModel::update_by_id(&db, &form_data.id, &dto).await?;
    Ok(result)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let result = LeadModel::batch_delete_by_ids(&db, &ids_vec).await?;
    Ok(result)
}

pub async fn find_by_id(db: &DbConn, id: i64) -> Result<LeadDetailVO> {
    let result = LeadModel::find_by_id(&db, id).await?;
    match result {
        Some(item) => {
            let mut vo: LeadDetailVO = item.into();
            let followups = crate::modules::crm::model::followup::FollowupModel::select_by_lead_id(&db, id).await?;

            // 批量查询跟进人名称 + 线索创建人名称（统一调用共用方法）
            let mut creator_ids: Vec<i64> = followups.iter()
                .filter_map(|f| f.created_by)
                .collect();
            if let Some(cbid) = vo.created_by {
                creator_ids.push(cbid);
            }
            let creator_map = crate::modules::system::service::admin_service::build_admin_name_map(db, creator_ids).await;

            if let Some(cbid) = vo.created_by {
                vo.created_by_name = creator_map.get(&cbid).cloned();
            }

            let followup_vo_list: Vec<crate::modules::crm::model::followup::FollowupListVO> = followups.into_iter().map(|f| {
                let mut followup_vo: crate::modules::crm::model::followup::FollowupListVO = f.into();
                if let Some(created_by) = followup_vo.created_by {
                    followup_vo.created_by_name = creator_map.get(&created_by).cloned();
                }
                followup_vo
            }).collect();

            vo.followups = Some(followup_vo_list);
            Ok(vo)
        }
        None => Err(Error::from("线索不存在".to_string())),
    }
}

pub async fn list(db: &DbConn, query: &LeadListQuery, current_user_id: i64) -> Result<ResultPage<Vec<LeadListVO>>> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let list_type = query.list_type.as_deref().unwrap_or("my");

    // 合并搜索关键词：companyName 和 keywords 都支持
    let search_keywords = query.company_name.clone().or_else(|| query.keywords.clone());

    let (list, total) = match list_type {
        "all" => {
            // 全部线索：根据 data_scope 过滤（管理员/总经理/老板查看所有已分配数据）
            let roles = role_service::select_by_admin_id(db, &Some(current_user_id)).await?;
            let data_scope = roles.iter().filter_map(|r| r.data_scope).min();

            match get_accessible_user_ids(db, current_user_id, data_scope).await? {
                None => {
                    // 全部数据权限 - 不过滤负责人
                    LeadModel::select_in_page(
                        &db, page, page_size,
                        search_keywords, query.status, query.level.clone(), query.source.clone(),
                        None,
                        query.contact_name.clone(), query.mobile.clone(), query.industry,
                    ).await?
                }
                Some(user_ids) => {
                    let assigned_ids = if user_ids.is_empty() { None } else { Some(user_ids) };
                    LeadModel::select_in_page_by_assigned_ids(
                        &db, page, page_size,
                        search_keywords, query.status, query.level.clone(), query.source.clone(),
                        assigned_ids,
                        query.contact_name.clone(), query.mobile.clone(), query.industry,
                    ).await?
                }
            }
        }
        "subordinate" => {
            // 下属线索：显示用户 data_scope 范围内的其他人的线索（排除自己）
            let roles = role_service::select_by_admin_id(db, &Some(current_user_id)).await?;
            let data_scope = roles.iter().filter_map(|r| r.data_scope).min();

            let user_ids = match data_scope {
                Some(5) => {
                    // 仅本人数据权限的人，无法看到下属线索
                    Vec::new()
                }
                Some(1) | None => {
                    // 全部数据权限：获取所有用户（排除自己）
                    let all_admins = Admin::find()
                        .filter(admin::Column::Id.ne(current_user_id))
                        .all(db)
                        .await
                        .map_err(|e| Error::from(format!("查询用户列表失败: {}", e)))?;
                    all_admins.iter().map(|u| u.id).collect()
                }
                _ => {
                    get_accessible_user_ids(db, current_user_id, data_scope).await?
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|id| *id != current_user_id)
                        .collect::<Vec<_>>()
                }
            };

            let assigned_ids = if user_ids.is_empty() { None } else { Some(user_ids) };
            LeadModel::select_in_page_by_assigned_ids(
                &db, page, page_size,
                search_keywords, query.status, query.level.clone(), query.source.clone(),
                assigned_ids,
                query.contact_name.clone(), query.mobile.clone(), query.industry,
            ).await?
        }
        "pool" => {
            // 公海线索：显示未领取（assigned_to IS NULL）的线索
            LeadModel::select_pool_page(
                &db, page, page_size,
                search_keywords, query.level.clone(), query.source.clone(),
            ).await?
        }
        "todayFollow" => {
            // 今日跟进线索：关联 followup 表过滤
            let roles = role_service::select_by_admin_id(db, &Some(current_user_id)).await?;
            let data_scope = roles.iter().filter_map(|r| r.data_scope).min();
            let user_ids = get_accessible_user_ids(db, current_user_id, data_scope).await?;

            LeadModel::select_today_follow_page(
                &db, page, page_size,
                search_keywords, query.status, query.level.clone(), query.source.clone(),
                user_ids,
                query.contact_name.clone(), query.mobile.clone(), query.industry,
            ).await?
        }
        _ => {
            // my（默认）：只看自己负责的线索
            LeadModel::select_in_page(
                &db,
                page,
                page_size,
                search_keywords,
                query.status,
                query.level.clone(),
                query.source.clone(),
                Some(current_user_id),
                query.contact_name.clone(), query.mobile.clone(), query.industry,
            ).await?
        }
    };

    // 收集所有 created_by + assigned_to 的 id，合并为一次 IN 查询 admin 用户名
    let user_ids: Vec<i64> = list.iter()
        .flat_map(|item| [item.created_by, item.assigned_to])
        .flatten()
        .collect();
    let user_map = crate::modules::system::service::admin_service::build_admin_name_map(db, user_ids).await;

    // 批量查询线索标签
    let lead_ids: Vec<i64> = list.iter().map(|l| l.id).collect();
    let tag_map = batch_query_lead_tags(db, &lead_ids).await?;
    log::info!("[线索列表] 线索IDs={:?}, 查询到的标签映射={:?}", lead_ids, tag_map.keys().collect::<Vec<_>>());

    let data: Vec<LeadListVO> = list.into_iter().map(|item| {
        let created_by = item.created_by;
        let created_by_name = created_by.and_then(|id| user_map.get(&id).cloned());
        let assigned_to = item.assigned_to;
        let assignee = assigned_to.and_then(|id| user_map.get(&id).cloned());
        let lid = item.id;
        let mut vo: LeadListVO = item.into();
        vo.created_by_name = created_by_name;
        vo.assignee = assignee;
        vo.tags = tag_map.get(&lid).cloned();
        vo
    }).collect();

    Ok(ResultPage::new(data, total, page, page_size))
}

/// 批量查询多个线索的标签关联，返回 lead_id -> Vec<LeadTagVO> 的映射
async fn batch_query_lead_tags(
    db: &DbConn,
    lead_ids: &[i64],
) -> Result<HashMap<i64, Vec<LeadTagVO>>> {
    if lead_ids.is_empty() {
        return Ok(HashMap::new());
    }

    // 一次 IN 查询所有关联记录
    let merges = tag_merge::Entity::find()
        .filter(tag_merge::Column::EntityType.eq("lead"))
        .filter(tag_merge::Column::EntityId.is_in(lead_ids.to_vec()))
        .all(db)
        .await?;

    log::info!("[线索标签查询] lead_ids={:?}, entityType=lead, 找到关联记录数={}", lead_ids, merges.len());

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

    // 组装 lead_id -> Vec<LeadTagVO>
    let mut result: HashMap<i64, Vec<LeadTagVO>> = HashMap::new();
    for m in merges {
        if let (Some(lid), Some(tid)) = (m.entity_id, m.tag_id) {
            if let Some(t) = tag_detail_map.get(&tid) {
                result.entry(lid).or_default().push(LeadTagVO {
                    id: Some(t.id),
                    tag_name: t.tag_name.clone(),
                    tag_color: t.tag_color.clone(),
                });
            }
        }
    }

    Ok(result)
}

/// 根据当前用户的数据权限，计算可见的用户ID列表
/// 返回 None 表示全部数据（不限制负责人）；Some(vec) 表示仅这些用户的数据
async fn get_accessible_user_ids(
    db: &DbConn,
    current_user_id: i64,
    data_scope: Option<i32>,
) -> Result<Option<Vec<i64>>> {
    match data_scope {
        Some(1) => {
            // 全部数据 - 不限制
            Ok(None)
        }
        Some(5) => {
            // 仅本人数据
            Ok(Some(vec![current_user_id]))
        }
        Some(3) | Some(4) | Some(2) => {
            // 获取用户的部门
            let user_depts = AdminDeptMergeModel::find_by_admin_id(db, current_user_id).await
                .map_err(|e| Error::from(format!("查询用户部门失败: {}", e)))?;

            let mut target_dept_ids = Vec::new();

            if data_scope == Some(2) {
                // 自定义数据权限 - 查询角色关联的部门
                let roles = role_service::select_by_admin_id(db, &Some(current_user_id)).await?;
                for role in roles {
                    if role.data_scope == Some(2) {
                        if let Some(role_id) = role.id {
                            let dept_result = crate::modules::system::model::role_dept_merge::RoleDeptMergeModel::find_by_role_id(db, &Some(role_id)).await
                                .map_err(|e| Error::from(format!("查询角色部门关联失败: {}", e)))?;
                            for merge in dept_result {
                                if let Some(dept_id) = merge.dept_id {
                                    target_dept_ids.push(dept_id);
                                }
                            }
                        }
                    }
                }
            } else {
                // data_scope = 3 或 4：基于用户所在部门
                for merge in &user_depts {
                    if let Some(dept_id) = merge.dept_id {
                        target_dept_ids.push(dept_id);
                    }
                }
            }

            if target_dept_ids.is_empty() {
                return Ok(Some(vec![current_user_id]));
            }

            let all_depts = DeptModel::find_all(db).await
                .map_err(|e| Error::from(format!("查询部门列表失败: {}", e)))?;

            // 收集所有目标部门ID（含子部门）
            let mut all_target_ids = Vec::new();
            for dept_id in &target_dept_ids {
                if data_scope == Some(4) || data_scope == Some(2) {
                    // 本部门及以下 / 自定义：包含子部门
                    all_target_ids.extend(collect_child_dept_ids(&all_depts, *dept_id));
                } else {
                    // data_scope = 3：仅本部门
                    all_target_ids.push(*dept_id);
                }
            }

            // 去重
            all_target_ids.sort();
            all_target_ids.dedup();

            // 查询这些部门下的所有用户
            let dept_merges = AdminDeptMergeModel::find_by_dept_id(db, all_target_ids).await
                .map_err(|e| Error::from(format!("查询部门用户失败: {}", e)))?;

            let mut user_ids: Vec<i64> = dept_merges.iter()
                .filter_map(|m| m.admin_id)
                .collect();
            user_ids.sort();
            user_ids.dedup();

            if user_ids.is_empty() {
                Ok(Some(vec![current_user_id]))
            } else {
                Ok(Some(user_ids))
            }
        }
        _ => {
            // 默认仅本人
            Ok(Some(vec![current_user_id]))
        }
    }
}

/// 递归收集部门下的所有子部门ID（含自身）
fn collect_child_dept_ids(all_depts: &[crate::modules::system::entity::dept::Model], parent_id: i64) -> Vec<i64> {
    let mut ids = Vec::new();
    for dept in all_depts {
        if dept.parent_id == Some(parent_id) {
            ids.push(dept.id);
            ids.extend(collect_child_dept_ids(all_depts, dept.id));
        }
    }
    ids
}

pub async fn update_status(db: &DbConn, id: i64, status: i32, updated_by: Option<i64>) -> Result<i64> {
    // 已转客户的线索不允许修改状态，防止状态不一致
    let lead = LeadModel::find_by_id(db, id)
        .await?
        .ok_or_else(|| Error::from("线索不存在".to_string()))?;
    if lead.converted_to_customer_id.is_some() && status != 3 {
        return Err(Error::from("该线索已转为客户，不能修改状态".to_string()));
    }

    let result = LeadModel::update_status(db, id, status, updated_by).await?;
    Ok(result)
}

pub async fn add_to_pool(db: &DbConn, id: i64, updated_by: Option<i64>) -> Result<i64> {
    // 已转客户的线索不允许加入线索池
    let lead = LeadModel::find_by_id(db, id)
        .await?
        .ok_or_else(|| Error::from("线索不存在".to_string()))?;
    if lead.converted_to_customer_id.is_some() {
        return Err(Error::from("该线索已转为客户，不能加入线索池".to_string()));
    }
    let result = LeadModel::add_to_pool(db, id, updated_by).await?;
    Ok(result)
}

/// 领取线索：从线索创建客户，并将线索标记为已转客户
pub async fn claim(db: &DbConn, id: i64, user_id: i64) -> Result<i64> {
    let lead = LeadModel::find_by_id(db, id)
        .await?
        .ok_or_else(|| Error::from("线索不存在".to_string()))?;

    if lead.converted_to_customer_id.is_some() {
        return Err(Error::from("该线索已被领取".to_string()));
    }

    let customer_dto = CustomerSaveDTO {
        id: None,
        customer_no: None,
        customer_type: Some(1),
        company_name: lead.company_name.clone(),
        short_name: None,
        person_name: None,
        gender: None,
        birthday: None,
        wechat: None,
        qq: None,
        personal_mobile: None,
        personal_email: None,
        nickname: None,
        occupation: None,
        country: lead.country.clone(),
        region: lead.region.clone(),
        address: lead.address.clone(),
        website: lead.website.clone(),
        industry: lead.industry,
        level: lead.level,
        source: lead.source.clone().map(|s| s.to_i32()),
        currency: lead.currency.clone(),
        credit_limit: None,
        credit_days: None,
        assigned_to: Some(user_id),
        cooperated_at: None,
        birthday_month: None,
        description: lead.description.clone(),
        custom_fields: lead.custom_fields.clone(),
        deleted: None,
        created_by: Some(user_id),
        create_time: None,
        updated_by: None,
        update_time: None,
    };

    // 使用事务确保创建客户和更新线索原子执行
    let customer_id = db.transaction::<_, _, DbErr>(|txn| {
        Box::pin(async move {
            let customer_id = CustomerModel::insert(txn, &customer_dto).await?;
            LeadModel::claim(txn, id, user_id, customer_id).await?;
            Ok(customer_id)
        })
    }).await.map_err(|e| Error::from(e.to_string()))?;

    Ok(customer_id)
}

/// 验证手机号格式（支持中国大陆手机号和国际号码格式）
fn is_valid_mobile(mobile: &str) -> bool {
    use regex::Regex;
    let trimmed = mobile.trim();
    if trimmed.is_empty() {
        return false;
    }
    // 中国大陆手机号：1开头，第二位3-9，共11位
    let cn_re = Regex::new(r"^1[3-9]\d{9}$").unwrap();
    // 国际号码：带+号，后面跟着数字和空格，长度7-20位
    let intl_re = Regex::new(r"^\+?[\d\s-]{7,20}$").unwrap();
    cn_re.is_match(trimmed) || intl_re.is_match(trimmed)
}

/// 一键转客户：创建客户+联系人，并更新线索状态
pub async fn convert_to_customer(db: &DbConn, id: i64, user_id: i64) -> Result<i64> {
    use crate::modules::crm::model::contact::{ContactModel, ContactSaveDTO};
    use crate::modules::crm::entity::contact;
    use sea_orm::ConnectionTrait;
    use crate::modules::company::service::code_rule_service;
    use crate::modules::crm::service::assign_history_service;

    let lead = LeadModel::find_by_id(db, id)
        .await?
        .ok_or_else(|| Error::from("线索不存在".to_string()))?;

    // 检查是否已转客户
    if lead.converted_to_customer_id.is_some() {
        return Err(Error::from("该线索已转为客户".to_string()));
    }

    // 检查公司名称是否存在
    let company_name = lead.company_name.as_deref().unwrap_or("").trim();
    if company_name.is_empty() {
        return Err(Error::from("公司名称不能为空".to_string()));
    }

    // 检查联系人和手机号是否完整
    let contact_name = lead.contact_name.as_deref().unwrap_or("").trim();
    let mobile = lead.mobile.as_deref().unwrap_or("").trim();
    if contact_name.is_empty() {
        return Err(Error::from("联系人姓名不能为空".to_string()));
    }
    if mobile.is_empty() {
        return Err(Error::from("联系人手机号不能为空".to_string()));
    }

    // 验证手机号格式
    if !is_valid_mobile(mobile) {
        return Err(Error::from("手机号格式不正确".to_string()));
    }

    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;

    // 检查公司名称是否已存在
    let existing_customer = customer::Entity::find()
        .filter(customer::Column::Deleted.eq(0))
        .filter(customer::Column::CompanyName.eq(company_name))
        .one(&txn)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    if existing_customer.is_some() {
        txn.rollback().await.ok();
        return Err(Error::from(format!("公司名称「{}」已存在", company_name)));
    }

    // 检查手机号是否已存在于联系人表
    let existing_contact = contact::Entity::find()
        .filter(contact::Column::Deleted.eq(0))
        .filter(contact::Column::Mobile.eq(mobile))
        .one(&txn)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    if existing_contact.is_some() {
        txn.rollback().await.ok();
        return Err(Error::from(format!("手机号「{}」已存在", mobile)));
    }

    // 创建客户
    let customer_dto = CustomerSaveDTO {
        id: None,
        customer_no: None,
        customer_type: Some(1),
        company_name: Some(company_name.to_string()),
        short_name: None,
        person_name: None,
        gender: None,
        birthday: None,
        wechat: None,
        qq: None,
        personal_mobile: None,
        personal_email: None,
        nickname: None,
        occupation: None,
        country: lead.country.clone(),
        region: lead.region.clone(),
        address: lead.address.clone(),
        website: lead.website.clone(),
        industry: lead.industry,
        level: lead.level,
        source: lead.source.clone().map(|s| s.to_i32()),
        currency: lead.currency.clone(),
        credit_limit: None,
        credit_days: None,
        assigned_to: Some(user_id),
        cooperated_at: None,
        birthday_month: None,
        description: lead.description.clone(),
        custom_fields: lead.custom_fields.clone(),
        deleted: None,
        created_by: Some(user_id),
        create_time: None,
        updated_by: None,
        update_time: None,
    };

    // 生成客户编号
    let mut customer_dto = customer_dto;
    if let Ok(code) = code_rule_service::generate_code(&txn, "customer", None, None, None).await {
        customer_dto.customer_no = Some(code);
    }

    let customer_id = CustomerModel::insert(&txn, &customer_dto)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    // 记录分配历史
    let _ = assign_history_service::record_claim(&txn, customer_id, user_id).await;

    // 创建联系人
    let contact_dto = ContactSaveDTO {
        id: None,
        name: Some(contact_name.to_string()),
        title: lead.title.clone(),
        email: lead.email.clone(),
        phone: lead.phone.clone(),
        mobile: Some(mobile.to_string()),
        whatsapp: None,
        wechat: None,
        qq: None,
        gender: None,
        birthday: None,
        notes: None,
        customer_id: Some(customer_id),
        role_type: Some(0),
        is_primary: Some(true),
        is_billing: None,
        is_shipping: None,
        bound_at: None,
        deleted: None,
        created_by: Some(user_id),
        create_time: None,
        updated_by: None,
        update_time: None,
    };

    let contact_id = ContactModel::insert(&txn, &contact_dto)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    // 绑定联系人到客户
    let _ = ContactModel::insert_merge(&txn, customer_id, contact_id, &contact_dto)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    // 同步该线索下的所有背调记录到新客户
    use crate::modules::ai::entity::company_background_check;
    company_background_check::Entity::update_many()
        .set(company_background_check::ActiveModel {
            company_id: sea_orm::Set(Some(customer_id)),
            ..Default::default()
        })
        .filter(company_background_check::Column::LeadId.eq(id))
        .exec(&txn)
        .await
        .ok();

    // 同步该线索下的所有跟进记录到新客户（保留 lead_id 用于追溯）
    // source_type 升级为 2（客户跟进），但 lead_id 保留可识别原线索来源
    let _ = FollowupModel::inherit_to_customer(&txn, id, customer_id).await;

    // 更新线索状态为已成交，标记已转客户
    LeadModel::claim(&txn, id, user_id, customer_id)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;

    Ok(customer_id)
}