use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::crm::entity::customer_contact_merge;
use crate::modules::crm::model::contact::{
    CareerHistoryItem, ContactBindRequest, ContactCompanyInfo, ContactDetailVO, ContactListQuery,
    ContactListVO, ContactModel, ContactSaveDTO, ContactSaveRequest, ContactSetRoleRequest,
    ContactUnbindRequest, ContactUpdateRequest, CustomerContactVO,
};
use sea_orm::DbConn;
use sea_orm::EntityTrait;

pub async fn insert(db: &DbConn, form_data: &ContactSaveRequest, created_by: i64) -> Result<i64> {
    let mut dto: ContactSaveDTO = form_data.clone().into();
    dto.created_by = Some(created_by);
    let contact_id = ContactModel::insert(&db, &dto).await?;

    // 如果创建时指定了客户ID，同时创建关联
    if let Some(customer_id) = form_data.customer_id {
        ContactModel::insert_merge(&db, customer_id, contact_id, &dto).await?;
    }

    Ok(contact_id)
}

pub async fn update(db: &DbConn, form_data: &ContactUpdateRequest, updated_by: i64) -> Result<i64> {
    let mut dto: ContactSaveDTO = form_data.clone().into();
    dto.updated_by = Some(updated_by);
    let result = ContactModel::update_by_id(&db, &form_data.id, &dto).await?;
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
                birthday: item.birthday,
                notes: item.notes,
                current_company,
                career_history: if career_history.is_empty() { None } else { Some(career_history) },
                created_at: item.created_at,
                updated_at: item.updated_at,
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

    let data: Vec<ContactListVO> = list
        .into_iter()
        .map(|item| {
            // 简单列表不查询关联公司，保持性能
            ContactListVO {
                id: Some(item.id),
                name: item.name,
                title: item.title,
                email: item.email,
                phone: item.phone,
                mobile: item.mobile,
                customer_id: None,
                company_name: None,
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