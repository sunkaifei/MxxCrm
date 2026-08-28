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
use crate::modules::crm::entity::customer_contact_merge;
use crate::modules::crm::entity::customer;
use crate::modules::crm::entity::contact;
use crate::modules::crm::entity::opportunity;
use crate::modules::crm::service::contact_edit_log_service;
use crate::modules::crm::service::delete_guard_service;
use crate::modules::crm::model::contact::{
    CareerHistoryItem, ContactBindRequest, ContactCheckRequest, ContactCheckResult, ContactCompanyInfo, ContactDetailVO, ContactListQuery,
    ContactListVO, ContactModel, ContactSaveDTO, ContactSaveRequest, ContactSetRoleRequest,
    ContactUnbindRequest, ContactUpdateRequest, CustomerContactVO,
};
use crate::modules::system::service::role_service;
use sea_orm::DbConn;
use sea_orm::DbErr;
use sea_orm::EntityTrait;
use sea_orm::TransactionTrait;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::Condition;
use sea_orm::Set;
use sea_orm::ActiveModelTrait;
use sea_orm::PaginatorTrait;

pub async fn insert(db: &DbConn, form_data: &ContactSaveRequest, created_by: i64) -> Result<i64> {
    let mut dto: ContactSaveDTO = form_data.clone().into();
    dto.created_by = Some(created_by);
    let customer_id_opt = form_data.customer_id;

    // 联系人主表与客户关联表需原子写入，避免产生无关联的联系人
    let contact_id = db
        .transaction::<_, _, DbErr>(|txn| {
            Box::pin(async move {
                let contact_id = ContactModel::insert(txn, &dto).await?;
                if let Some(customer_id) = customer_id_opt {
                    ContactModel::insert_merge(txn, customer_id, contact_id, &dto).await?;
                }
                Ok(contact_id)
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(contact_id)
}

pub async fn update(db: &DbConn, form_data: &ContactUpdateRequest, updated_by: i64) -> Result<i64> {
    let mut dto: ContactSaveDTO = form_data.clone().into();
    dto.updated_by = Some(updated_by);
    let contact_id = form_data.id;
    let customer_id_opt = form_data.customer_id;
    let title_opt = form_data.title.clone();

    // 联系人更新与客户绑定需原子操作
    let result = db
        .transaction::<_, _, DbErr>(|txn| {
            Box::pin(async move {
                // 更新前先获取旧数据，用于记录修改日志
                let old_model = contact::Entity::find_by_id(contact_id.unwrap_or_default())
                    .one(txn)
                    .await?;

                let result = ContactModel::update_by_id(txn, &contact_id, &dto).await?;

                // 记录修改日志（如有差异）
                if let Some(old) = &old_model {
                    let old_json = serde_json::to_value(old).unwrap_or_default();
                    // 用更新后的模型做对比，避免 DTO 未携带的字段（首要/账单/收货联系人标志）被误报为"删除"
                    let new_model = contact::Entity::find_by_id(contact_id.unwrap_or_default())
                        .one(txn)
                        .await?;
                    let new_json = serde_json::to_value(&new_model).unwrap_or_default();
                    let editor_name = crate::modules::system::entity::admin::Entity::find_by_id(updated_by)
                        .one(txn)
                        .await
                        .ok()
                        .flatten()
                        .and_then(|a| a.nick_name.or(a.user_name));
                    let _ = contact_edit_log_service::log_update(
                        txn,
                        contact_id.unwrap_or_default(),
                        updated_by,
                        editor_name,
                        &old_json,
                        &new_json,
                    ).await;
                }

                if let Some(cid) = contact_id {
                    if let Some(customer_id) = customer_id_opt {
                        // 查询当前任职
                        let current = customer_contact_merge::Entity::find()
                            .filter(customer_contact_merge::Column::ContactId.eq(cid))
                            .filter(customer_contact_merge::Column::IsCurrent.eq(true))
                            .one(txn)
                            .await?;

                        let now = chrono::Local::now().naive_local().to_owned();

                        if let Some(ref current_record) = current {
                            if current_record.customer_id == customer_id {
                                // 同一公司：只更新信息，不新增履历
                                // 注意：is_primary/is_billing/is_shipping 不在此处设置（NotSet=不修改），
                                // Set(None) 会把已有标志清空为 NULL
                                let update_payload = customer_contact_merge::ActiveModel {
                                    id: Set(current_record.id),
                                    title: Set(title_opt.clone()),
                                    role_type: Set(dto.role_type.clone()),
                                    update_time: Set(Some(now)),
                                    ..Default::default()
                                };
                                customer_contact_merge::Entity::update(update_payload).exec(txn).await?;
                                return Ok(result);
                            }

                            // 不同公司：当前关联降级为历史
                            let downgrade = customer_contact_merge::ActiveModel {
                                id: Set(current_record.id),
                                is_current: Set(Some(false)),
                                unbound_at: Set(Some(now)),
                                update_time: Set(Some(now)),
                                ..Default::default()
                            };
                            customer_contact_merge::Entity::update(downgrade).exec(txn).await?;
                        }

                        // 插入新关联
                        let payload = customer_contact_merge::ActiveModel {
                            customer_id: Set(customer_id),
                            contact_id: Set(cid),
                            title: Set(title_opt.clone()),
                            role_type: Set(dto.role_type.clone()),
                            is_current: Set(Some(true)),
                            is_primary: Set(None),
                            is_billing: Set(None),
                            is_shipping: Set(None),
                            bound_at: Set(Some(now)),
                            create_time: Set(Some(now)),
                            update_time: Set(Some(now)),
                            ..Default::default()
                        };
                        customer_contact_merge::Entity::insert(payload).exec(txn).await?;
                    }
                }
                Ok(result)
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(result)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<i64>, deleted_by: i64) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    // 查询待删除的旧数据（用于校验）
    let old_models = contact::Entity::find()
        .filter(contact::Column::Id.is_in(ids_vec.clone()))
        .filter(contact::Column::Deleted.eq(0))
        .all(db)
        .await?;
    if old_models.len() != ids_vec.len() {
        return Err(Error::from("部分联系人不存在或已删除，请刷新后重试"));
    }

    // 逐条前置校验，任一失败整体拒绝并返回失败明细（全有全无语义）
    let mut failures: Vec<String> = Vec::new();
    for m in &old_models {
        if let Err(e) = check_contact_deletable(db, m, deleted_by).await {
            let name = m.name.clone().unwrap_or_else(|| format!("#{}", m.id));
            failures.push(format!("【{}】{}", name, e));
        }
    }
    if !failures.is_empty() {
        return Err(Error::from(format!("删除失败：{}", failures.join("；"))));
    }

    let ids_clone = ids_vec.clone();
    let result = db.transaction::<_, i64, DbErr>(|txn| {
        Box::pin(async move {
            // 注意：不清理 customer_contact_merge，保留任职关系（当前+历史）：
            // 联系人仅软删进回收站，还原后任职关系应完整恢复；彻底删除（回收站 purge）时再级联清理。
            contact::Entity::update_many()
                .set(contact::ActiveModel {
                    deleted: Set(Some(1)),
                    delete_by: Set(Some(deleted_by)),
                    delete_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
                    ..Default::default()
                })
                .filter(contact::Column::Id.is_in(ids_clone))
                .exec(txn)
                .await
                .map(|r| r.rows_affected as i64)
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    Ok(result)
}

/// 联系人删除前置校验（规划方案 5.4）：
/// 商机直接挂接的联系人硬禁删（opportunity.contact_id）；非创建人须管理员；
/// 创建人 24 小时内可删，超窗仅管理员（口径见 4.1 #6）。
async fn check_contact_deletable(db: &DbConn, m: &contact::Model, current_user_id: i64) -> Result<()> {
    let count = opportunity::Entity::find()
        .filter(opportunity::Column::ContactId.eq(m.id))
        .filter(opportunity::Column::Deleted.eq(0))
        .count(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    if count > 0 {
        return Err(Error::from("该联系人已被商机挂接，无法删除，请先处理相关商机"));
    }
    if m.created_by != Some(current_user_id) {
        let ok = delete_guard_service::is_manager(db, current_user_id, "crm:contact:delete").await?;
        if !ok {
            return Err(Error::from("仅联系人创建人可删除"));
        }
        return Ok(());
    }
    if let Err(e) = delete_guard_service::check_delete_window(
        m.create_time,
        delete_guard_service::DEFAULT_DELETE_WINDOW_HOURS,
        "联系人",
        "",
    ) {
        let ok = delete_guard_service::is_manager(db, current_user_id, "crm:contact:delete").await?;
        if !ok {
            return Err(e);
        }
    }
    Ok(())
}

/// 当前用户可见的联系人 ID 集合（按数据权限）
///
/// 可见性口径（三层并集）：
/// 1. **我的人脉**：contact.created_by = 我（人脉资产归属，独立于客户，客户转移/删除不影响）
/// 2. 现任/曾任于我权限范围内客户的联系人（历史任职沉淀，离职换绑后仍可见）
/// 3. 超管/系统管理员：全部（返回 None）
pub async fn visible_contact_ids(db: &DbConn, user_id: i64) -> Result<Option<Vec<i64>>> {
    let accessible = crate::modules::system::service::data_scope_service
        ::get_accessible_user_ids(db, user_id).await?;
    match accessible {
        None => Ok(None),
        Some(user_ids) => {
            // 数据权限内的用户（含本人）
            let mut scope_users = user_ids;
            if !scope_users.contains(&user_id) {
                scope_users.push(user_id);
            }

            // 1) 我的人脉
            let mut contact_ids: Vec<i64> = crate::modules::crm::entity::contact::Entity::find()
                .filter(crate::modules::crm::entity::contact::Column::Deleted.eq(0))
                .filter(crate::modules::crm::entity::contact::Column::CreatedBy.eq(user_id))
                .all(db)
                .await
                .map_err(|e| Error::from(e.to_string()))?
                .iter()
                .map(|c| c.id)
                .collect();

            // 2) 权限客户（现任或曾任）的联系人
            let customer_ids: Vec<i64> = customer::Entity::find()
                .filter(customer::Column::Deleted.eq(0))
                .filter(customer::Column::AssignedTo.is_in(scope_users))
                .all(db)
                .await
                .map_err(|e| Error::from(e.to_string()))?
                .iter()
                .map(|c| c.id)
                .collect();
            if !customer_ids.is_empty() {
                let customer_contact_ids: Vec<i64> = customer_contact_merge::Entity::find()
                    .filter(customer_contact_merge::Column::CustomerId.is_in(customer_ids))
                    .all(db)
                    .await
                    .map_err(|e| Error::from(e.to_string()))?
                    .iter()
                    .map(|m| m.contact_id)
                    .collect();
                contact_ids.extend(customer_contact_ids);
            }

            contact_ids.sort_unstable();
            contact_ids.dedup();
            Ok(Some(contact_ids))
        }
    }
}

/// 单个联系人是否对当前用户可见
async fn is_contact_visible(db: &DbConn, id: i64, user_id: i64) -> Result<bool> {
    Ok(match visible_contact_ids(db, user_id).await? {
        None => true,
        Some(ids) => ids.contains(&id),
    })
}

/// 详情查询（带数据权限校验）：无权返回错误
pub async fn find_by_id_checked(db: &DbConn, id: i64, user_id: i64) -> Result<ContactDetailVO> {
    if !is_contact_visible(db, id, user_id).await? {
        return Err(Error::from("无权访问该联系人"));
    }
    find_by_id(db, id).await
}

/// 更新（带数据权限校验）：无权返回错误
pub async fn update_checked(db: &DbConn, form_data: &ContactUpdateRequest, updated_by: i64) -> Result<i64> {
    if let Some(id) = form_data.id {
        if !is_contact_visible(db, id, updated_by).await? {
            return Err(Error::from("无权修改该联系人"));
        }
    }
    update(db, form_data, updated_by).await
}

/// 批量删除（带数据权限校验）：仅删除可见联系人；全部不可见时报错
pub async fn batch_delete_by_ids_checked(db: &DbConn, ids_vec: &Vec<i64>, user_id: i64) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    match visible_contact_ids(db, user_id).await? {
        None => batch_delete_by_ids(db, ids_vec, user_id).await,
        Some(visible) => {
            let filtered: Vec<i64> = ids_vec.iter().filter(|id| visible.contains(id)).copied().collect();
            if filtered.is_empty() {
                return Err(Error::from("无权删除该联系人"));
            }
            batch_delete_by_ids(db, &filtered, user_id).await
        }
    }
}

pub async fn find_by_id(db: &DbConn, id: i64) -> Result<ContactDetailVO> {
    let contact = ContactModel::find_by_id(&db, id).await?;
    match contact {
        Some(item) => {
            // 查询当前任职
            let current_merge = ContactModel::find_current_merge(&db, id).await?;
            let current_company = if let Some(ref m) = current_merge {
                let cu = crate::modules::crm::entity::customer::Entity::find_by_id(m.customer_id)
                    .one(db)
                    .await?;
                Some(ContactCompanyInfo {
                    customer_id: Some(m.customer_id),
                    company_name: cu.as_ref().and_then(|c| c.company_name.clone()),
                    short_name: cu.as_ref().and_then(|c| c.short_name.clone()),
                    title: m.title.clone(),
                    role_type: m.role_type.clone(),
                    is_primary: m.is_primary,
                    bound_at: m.bound_at,
                })
            } else {
                None
            };

            // 查询职业生涯履历
            let career_history = ContactModel::find_career_history(&db, id).await?;

            Ok(ContactDetailVO {
                id: Some(item.id),
                name: item.name,
                title: item.title,
                email: item.email,
                phone: item.phone,
                mobile: item.mobile,
                whatsapp: item.whatsapp,
                wechat: item.wechat,
                qq: item.qq,
                country: item.country,
                region: item.region,
                address: item.address,
                gender: item.gender,
                birthday: item.birthday,
                notes: item.notes,
                current_company,
                career_history: if career_history.is_empty() { None } else { Some(career_history) },
                create_time: item.create_time,
                update_time: item.update_time,
            })
        }
        None => Err(Error::from("联系人不存在".to_string())),
    }
}

/// 绑定联系人到客户（带数据权限校验）
///
/// 权限规则（人脉资产化）：
/// - 联系人必须对当前用户可见（我的人脉 / 曾任我客户 / 权限范围）
/// - 目标客户必须对当前用户可见（负责人在数据权限内）
/// 满足后即可将联系人绑定到自己负责的客户（跨销售人脉复用）
pub async fn bind_contact_checked(db: &DbConn, req: &ContactBindRequest, user_id: i64) -> Result<i64> {
    // 1) 联系人可见性
    if !is_contact_visible(db, req.contact_id, user_id).await? {
        return Err(Error::from("无权操作该联系人"));
    }

    // 2) 目标客户可见性（负责人在数据权限内，或本人负责）
    let target = customer::Entity::find_by_id(req.customer_id)
        .filter(customer::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    let target = target.ok_or_else(|| Error::from("目标客户不存在"))?;
    let visible = match crate::modules::system::service::data_scope_service
        ::get_accessible_user_ids(db, user_id).await? {
        None => true,
        Some(ids) => target.assigned_to.map(|owner| ids.contains(&owner)).unwrap_or(false),
    };
    if !visible {
        return Err(Error::from("无权将联系人绑定到该客户"));
    }

    bind_contact(db, req).await
}

pub async fn bind_contact(db: &DbConn, req: &ContactBindRequest) -> Result<i64> {
    let result = ContactModel::bind_contact(&db, req).await?;
    Ok(result)
}

pub async fn unbind_contact(db: &DbConn, req: &ContactUnbindRequest) -> Result<i64> {
    let result = ContactModel::unbind_contact(&db, req).await?;
    Ok(result)
}

pub async fn set_role(db: &DbConn, req: &ContactSetRoleRequest) -> Result<i64> {
    let result = ContactModel::set_role(&db, req).await?;
    Ok(result)
}

pub async fn list(db: &DbConn, query: &ContactListQuery, current_user_id: i64) -> Result<ResultPage<Vec<ContactListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let list_type = query.list_type.as_deref().unwrap_or("my");

    // 根据 list_type 获取可见的客户ID列表
    let customer_ids = match list_type {
        "my" => {
            let ids: Vec<i64> = customer::Entity::find()
                .filter(customer::Column::Deleted.eq(0))
                .filter(customer::Column::AssignedTo.eq(current_user_id))
                .all(db)
                .await
                .map_err(|e| Error::from(e.to_string()))?
                .iter()
                .map(|c| c.id)
                .collect();
            Some(ids)
        }
        "subordinate" => {
            // 下属联系人：按汇报关系（direct_manager_id）递归查找所有下属，含跨级别
            let subordinate_ids = crate::modules::system::service::subordinate_service
                ::get_subordinate_ids_default(db, current_user_id).await?;

            if subordinate_ids.is_empty() {
                return Ok(ResultPage::new(Vec::<ContactListVO>::new(), 0, page, page_size));
            }

            let ids: Vec<i64> = customer::Entity::find()
                .filter(customer::Column::Deleted.eq(0))
                .filter(customer::Column::AssignedTo.is_in(subordinate_ids))
                .all(db)
                .await
                .map_err(|e| Error::from(e.to_string()))?
                .iter()
                .map(|c| c.id)
                .collect();
            Some(ids)
        }
        _ => {
            // all（含任意未知值兜底）：按数据权限过滤，绝不允许无权限标识落到"不过滤"分支
            // 对齐 customer_service::list 的 all 分支（防越权：前端 tab 只是 UI，权限必须后端强制）
            let accessible = crate::modules::system::service::data_scope_service
                ::get_accessible_user_ids(db, current_user_id).await?;
            match accessible {
                None => None, // 全部数据权限（超管/系统管理员）
                Some(ids) => {
                    if ids.is_empty() {
                        return Ok(ResultPage::new(Vec::<ContactListVO>::new(), 0, page, page_size));
                    }
                    let cid_list: Vec<i64> = customer::Entity::find()
                        .filter(customer::Column::Deleted.eq(0))
                        .filter(customer::Column::AssignedTo.is_in(ids))
                        .all(db)
                        .await
                        .map_err(|e| Error::from(e.to_string()))?
                        .iter()
                        .map(|c| c.id)
                        .collect();
                    Some(cid_list)
                }
            }
        }
    };

    // 客户名称过滤
    let mut final_customer_ids = customer_ids;
    if let Some(cn) = &query.customer_name {
        if !cn.is_empty() {
            let name_filtered: Vec<i64> = customer::Entity::find()
                .filter(customer::Column::Deleted.eq(0))
                .filter(customer::Column::CompanyName.contains(cn))
                .all(db)
                .await
                .map_err(|e| Error::from(e.to_string()))?
                .iter()
                .map(|c| c.id)
                .collect();
            if let Some(existing) = final_customer_ids.take() {
                let merged: Vec<i64> = existing.into_iter()
                    .filter(|id| name_filtered.contains(id))
                    .collect();
                final_customer_ids = Some(merged);
            } else {
                final_customer_ids = Some(name_filtered);
            }
        }
    }

    // 指定客户ID过滤：直接锁定到该客户，覆盖 list_type/customer_name 的过滤结果
    // 用于订单/合同等场景选择特定客户下的联系人
    if let Some(cid) = query.customer_id {
        if cid > 0 {
            final_customer_ids = Some(vec![cid]);
        }
    }

    // 通过客户ID获取联系人ID（含历史任职：曾在我客户工作过的人脉沉淀，离职/换绑后仍可见可换绑）
    let contact_ids = if let Some(cids) = &final_customer_ids {
        if cids.is_empty() {
            return Ok(ResultPage::new(Vec::<ContactListVO>::new(), 0, page, page_size));
        }
        let ids: Vec<i64> = customer_contact_merge::Entity::find()
            .filter(customer_contact_merge::Column::CustomerId.is_in(cids.clone()))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?
            .iter()
            .map(|m| m.contact_id)
            .collect();
        if ids.is_empty() {
            return Ok(ResultPage::new(Vec::<ContactListVO>::new(), 0, page, page_size));
        }
        Some(ids)
    } else {
        None
    };

    let (list, total) = ContactModel::select_in_page_with_filters(
        db,
        page,
        page_size,
        contact_ids,
        query.name.clone(),
        query.mobile.clone(),
        query.phone.clone(),
        query.wechat.clone(),
        query.email.clone(),
    ).await?;

    // 批量查询当前任职公司
    let contact_id_list: Vec<i64> = list.iter().filter_map(|c| Some(c.id)).collect();
    let mut company_map: std::collections::HashMap<i64, (Option<i64>, Option<String>, Option<i32>)> = std::collections::HashMap::new();

    if !contact_id_list.is_empty() {
        let merges = customer_contact_merge::Entity::find()
            .filter(customer_contact_merge::Column::ContactId.is_in(contact_id_list.clone()))
            .filter(customer_contact_merge::Column::IsCurrent.eq(true))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        let customer_id_list: Vec<i64> = merges.iter().map(|m| m.customer_id).collect();
        let customer_map = if !customer_id_list.is_empty() {
            customer::Entity::find()
                .filter(customer::Column::Id.is_in(customer_id_list.clone()))
                .all(db)
                .await
                .map_err(|e| Error::from(e.to_string()))?
                .into_iter()
                .map(|c| (c.id, c.company_name))
                .collect::<std::collections::HashMap<i64, Option<String>>>()
        } else {
            std::collections::HashMap::new()
        };

        for m in merges {
            let company_name = customer_map.get(&m.customer_id).and_then(|n| n.clone());
            company_map.insert(m.contact_id, (Some(m.customer_id), company_name, m.role_type));
        }
    }

    // 批量查询归属人姓名（管理员视角：人脉属于谁管理）
    let owner_ids: Vec<i64> = list.iter().filter_map(|c| c.created_by.filter(|v| *v > 0)).collect();
    let owner_name_map: std::collections::HashMap<i64, Option<String>> = if owner_ids.is_empty() {
        std::collections::HashMap::new()
    } else {
        crate::modules::system::entity::admin::Entity::find()
            .filter(crate::modules::system::entity::admin::Column::Id.is_in(owner_ids))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?
            .into_iter()
            .map(|a| (a.id, a.nick_name.or(a.user_name)))
            .collect()
    };

    let data: Vec<ContactListVO> = list
        .into_iter()
        .map(|item| {
            let (customer_id, company_name, role_type) = company_map.remove(&item.id).unwrap_or((None, None, None));
            ContactListVO {
                id: Some(item.id),
                name: item.name,
                title: item.title,
                email: item.email,
                phone: item.phone,
                mobile: item.mobile,
                customer_id,
                company_name,
                created_by: item.created_by,
                owner_name: item.created_by.and_then(|oid| owner_name_map.get(&oid).cloned().flatten()),
                role_type,
                create_time: item.create_time,
            }
        })
        .collect();
    Ok(ResultPage::new(data, total, page, page_size))
}

/// 获取客户下的联系人列表
pub async fn find_by_customer(
    db: &DbConn,
    customer_id: i64,
) -> Result<(Vec<CustomerContactVO>, Vec<CustomerContactVO>)> {
    let result = ContactModel::find_by_customer(&db, customer_id).await?;
    Ok(result)
}

/// 联系人查重：检查手机、电话、微信、QQ、邮箱是否已存在
pub async fn check_duplicate(db: &DbConn, req: &ContactCheckRequest) -> Result<Vec<ContactCheckResult>> {
    use crate::modules::crm::entity::contact::{Entity as Contact, Column};
    let mut results = Vec::new();

    let exclude_id = req.id;

    // 逐个字段检查（空值跳过）
    let checks: Vec<(&str, Option<&String>, Column)> = vec![
        ("mobile", req.mobile.as_ref(), Column::Mobile),
        ("phone", req.phone.as_ref(), Column::Phone),
        ("wechat", req.wechat.as_ref(), Column::Wechat),
        ("qq", req.qq.as_ref(), Column::Qq),
        ("email", req.email.as_ref(), Column::Email),
    ];

    for (field_name, value, column) in checks {
        if let Some(v) = value {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                continue;
            }
            let mut query = Contact::find()
                .filter(Column::Deleted.eq(0))
                .filter(column.eq(trimmed));
            if let Some(eid) = exclude_id {
                query = query.filter(Column::Id.ne(eid));
            }
            let existing = query
                .one(db)
                .await
                .map_err(|e| Error::from(e.to_string()))?;
            if let Some(c) = existing {
                results.push(ContactCheckResult {
                    field: field_name.to_string(),
                    duplicated: true,
                    contact_name: c.name,
                });
            } else {
                results.push(ContactCheckResult {
                    field: field_name.to_string(),
                    duplicated: false,
                    contact_name: None,
                });
            }
        }
    }

    Ok(results)
}