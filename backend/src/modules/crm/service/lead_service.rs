use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::crm::model::lead::{LeadDetailVO, LeadListQuery, LeadListVO, LeadModel, LeadSaveDTO, LeadSaveRequest, LeadUpdateRequest};
use sea_orm::DbConn;

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
        Some(item) => Ok(item.into()),
        None => Err(Error::from("线索不存在".to_string())),
    }
}

pub async fn list(db: &DbConn, query: &LeadListQuery) -> Result<ResultPage<Vec<LeadListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    
    let (list, total) = LeadModel::select_in_page(
        &db,
        page,
        page_size,
        query.keywords.clone(),
        query.status.clone(),
        query.level.clone(),
        query.source.clone(),
        query.assigned_to,
    ).await?;
    
    let data: Vec<LeadListVO> = list.into_iter().map(|item| item.into()).collect();
    Ok(ResultPage::new(data, total, page, page_size))
}

pub async fn update_status(db: &DbConn, id: i64, status: &str, updated_by: Option<i64>) -> Result<i64> {
    let status_enum = match status {
        "unchecked" => crate::core::r#enum::lead_status_enum::LeadStatus::Unchecked,
        "checking" => crate::core::r#enum::lead_status_enum::LeadStatus::Checking,
        "invalid" => crate::core::r#enum::lead_status_enum::LeadStatus::Invalid,
        "valid" => crate::core::r#enum::lead_status_enum::LeadStatus::Valid,
        _ => return Err(Error::from("无效的状态值".to_string())),
    };
    let result = LeadModel::update_status(db, id, status_enum, updated_by).await?;
    Ok(result)
}

pub async fn add_to_pool(db: &DbConn, id: i64, updated_by: Option<i64>) -> Result<i64> {
    let result = LeadModel::add_to_pool(db, id, updated_by).await?;
    Ok(result)
}