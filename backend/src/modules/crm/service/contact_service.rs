use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::crm::entity::customer_contact_merge;
use crate::modules::crm::entity::customer;
use crate::modules::crm::model::contact::{
    CareerHistoryItem, ContactBindRequest, ContactCompanyInfo, ContactDetailVO, ContactListQuery,
    ContactListVO, ContactModel, ContactSaveDTO, ContactSaveRequest, ContactSetRoleRequest,
    ContactUnbindRequest, ContactUpdateRequest, CustomerContactVO,
};
use sea_orm::DbConn;
use sea_orm::DbErr;
use sea_orm::EntityTrait;
use sea_orm::TransactionTrait;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
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

pub async fn list(db: &DbConn, query: &ContactListQuery) -> Result<ResultPage<Vec<ContactListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let (list, total) = ContactModel::select_in_page(
        &db,
        page,
        page_size,
        query.keywords.clone(),
        query.customer_id,
    )
    .await?;

    // 批量查询当前任职公司
    let contact_ids: Vec<i64> = list.iter().filter_map(|c| Some(c.id)).collect();
    let mut company_map: std::collections::HashMap<i64, (Option<i64>, Option<String>, Option<i32>)> = std::collections::HashMap::new();

    if !contact_ids.is_empty() {
        let merges = customer_contact_merge::Entity::find()
            .filter(customer_contact_merge::Column::ContactId.is_in(contact_ids.clone()))
            .filter(customer_contact_merge::Column::IsCurrent.eq(true))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        // 收集客户ID
        let customer_ids: Vec<i64> = merges.iter().map(|m| m.customer_id).collect();
        let customer_map = if !customer_ids.is_empty() {
            customer::Entity::find()
                .filter(customer::Column::Id.is_in(customer_ids.clone()))
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

/// 获取客户下的联系人列表
pub async fn find_by_customer(
    db: &DbConn,
    customer_id: i64,
) -> Result<(Vec<CustomerContactVO>, Vec<CustomerContactVO>)> {
    let result = ContactModel::find_by_customer(&db, customer_id).await?;
    Ok(result)
}