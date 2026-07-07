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
                let result = ContactModel::update_by_id(txn, &contact_id, &dto).await?;
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
                                let update_payload = customer_contact_merge::ActiveModel {
                                    id: Set(current_record.id),
                                    title: Set(title_opt.clone()),
                                    role_type: Set(dto.role_type.clone()),
                                    is_primary: Set(None),
                                    is_billing: Set(None),
                                    is_shipping: Set(None),
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

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let result = ContactModel::batch_delete_by_ids(&db, &ids_vec).await?;
    Ok(result)
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
            let roles = role_service::select_by_admin_id(db, &Some(current_user_id)).await?;
            let data_scope = roles.iter()
                .filter_map(|r| r.data_scope)
                .min();

            if data_scope == Some(5) {
                return Ok(ResultPage::new(Vec::<ContactListVO>::new(), 0, page, page_size));
            }

            let user_ids = get_accessible_user_ids(db, current_user_id, data_scope).await?
                .unwrap_or_default()
                .into_iter()
                .filter(|id| *id != current_user_id)
                .collect::<Vec<_>>();

            if user_ids.is_empty() {
                return Ok(ResultPage::new(Vec::<ContactListVO>::new(), 0, page, page_size));
            }

            let ids: Vec<i64> = customer::Entity::find()
                .filter(customer::Column::Deleted.eq(0))
                .filter(customer::Column::AssignedTo.is_in(user_ids))
                .all(db)
                .await
                .map_err(|e| Error::from(e.to_string()))?
                .iter()
                .map(|c| c.id)
                .collect();
            Some(ids)
        }
        _ => None,
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

    // 通过客户ID获取联系人ID
    let contact_ids = if let Some(cids) = &final_customer_ids {
        if cids.is_empty() {
            return Ok(ResultPage::new(Vec::<ContactListVO>::new(), 0, page, page_size));
        }
        let ids: Vec<i64> = customer_contact_merge::Entity::find()
            .filter(customer_contact_merge::Column::CustomerId.is_in(cids.clone()))
            .filter(customer_contact_merge::Column::IsCurrent.eq(true))
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
                role_type,
                create_time: item.create_time,
            }
        })
        .collect();
    Ok(ResultPage::new(data, total, page, page_size))
}

async fn get_accessible_user_ids(
    db: &DbConn,
    current_user_id: i64,
    data_scope: Option<i32>,
) -> Result<Option<Vec<i64>>> {
    use crate::modules::system::model::admin_dept_merge::AdminDeptMergeModel;
    use crate::modules::system::model::dept::DeptModel;
    use crate::modules::system::entity::dept;

    match data_scope {
        Some(1) => Ok(None),
        Some(5) => Ok(Some(vec![current_user_id])),
        Some(3) | Some(4) | Some(2) => {
            let user_depts = AdminDeptMergeModel::find_by_admin_id(db, current_user_id).await
                .map_err(|e| Error::from(format!("查询用户部门失败: {}", e)))?;

            let mut target_dept_ids = Vec::new();

            if data_scope == Some(2) {
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

            let mut all_target_ids = Vec::new();
            for dept_id in &target_dept_ids {
                if data_scope == Some(4) || data_scope == Some(2) {
                    all_target_ids.extend(collect_child_dept_ids(&all_depts, *dept_id));
                } else {
                    all_target_ids.push(*dept_id);
                }
            }

            all_target_ids.sort();
            all_target_ids.dedup();

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
        _ => Ok(Some(vec![current_user_id])),
    }
}

fn collect_child_dept_ids(all_depts: &[crate::modules::system::entity::dept::Model], parent_id: i64) -> Vec<i64> {
    let mut ids = vec![parent_id];
    for dept in all_depts {
        if dept.parent_id == Some(parent_id) {
            ids.extend(collect_child_dept_ids(all_depts, dept.id));
        }
    }
    ids
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