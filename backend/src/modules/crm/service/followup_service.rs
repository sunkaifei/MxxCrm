use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::crm::model::followup::{FollowupDetailVO, FollowupListQuery, FollowupListVO, FollowupModel, FollowupSaveDTO, FollowupSaveRequest, FollowupUpdateRequest};
use sea_orm::DbConn;

pub async fn insert(db: &DbConn, form_data: &FollowupSaveRequest, created_by: i64) -> Result<i64> {
    let mut dto: FollowupSaveDTO = form_data.clone().into();
    dto.created_by = Some(created_by);
    let result = FollowupModel::insert(&db, &dto).await?;
    Ok(result)
}

pub async fn update(db: &DbConn, form_data: &FollowupUpdateRequest, updated_by: i64) -> Result<i64> {
    let mut dto: FollowupSaveDTO = form_data.clone().into();
    dto.updated_by = Some(updated_by);
    let result = FollowupModel::update_by_id(&db, &form_data.id, &dto).await?;
    Ok(result)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let result = FollowupModel::batch_delete_by_ids(&db, &ids_vec).await?;
    Ok(result)
}

pub async fn find_by_id(db: &DbConn, id: i64) -> Result<FollowupDetailVO> {
    let result = FollowupModel::find_by_id(&db, id).await?;
    match result {
        Some(item) => Ok(item.into()),
        None => Err(Error::from("跟进记录不存在".to_string())),
    }
}

pub async fn list(db: &DbConn, query: &FollowupListQuery) -> Result<ResultPage<Vec<FollowupListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    
    let (list, total) = FollowupModel::select_in_page(
        &db,
        page,
        page_size,
        query.customer_id,
        query.lead_id,
        query.opportunity_id,
    ).await?;
    
    let data: Vec<FollowupListVO> = list.into_iter().map(|item| item.into()).collect();
    Ok(ResultPage::new(data, total, page, page_size))
}