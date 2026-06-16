use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::crm::model::opportunity::{OpportunityDetailVO, OpportunityListQuery, OpportunityListVO, OpportunityModel, OpportunitySaveDTO, OpportunitySaveRequest, OpportunityUpdateRequest};
use sea_orm::DbConn;

pub async fn insert(db: &DbConn, form_data: &OpportunitySaveRequest, created_by: i64) -> Result<i64> {
    let mut dto: OpportunitySaveDTO = form_data.clone().into();
    dto.created_by = Some(created_by);
    let result = OpportunityModel::insert(&db, &dto).await?;
    Ok(result)
}

pub async fn update(db: &DbConn, form_data: &OpportunityUpdateRequest, updated_by: i64) -> Result<i64> {
    let mut dto: OpportunitySaveDTO = form_data.clone().into();
    dto.updated_by = Some(updated_by);
    let result = OpportunityModel::update_by_id(&db, &form_data.id, &dto).await?;
    Ok(result)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let result = OpportunityModel::batch_delete_by_ids(&db, &ids_vec).await?;
    Ok(result)
}

pub async fn find_by_id(db: &DbConn, id: i64) -> Result<OpportunityDetailVO> {
    let result = OpportunityModel::find_by_id(&db, id).await?;
    match result {
        Some(item) => Ok(item.into()),
        None => Err(Error::from("商机不存在".to_string())),
    }
}

pub async fn list(db: &DbConn, query: &OpportunityListQuery) -> Result<ResultPage<Vec<OpportunityListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    
    let (list, total) = OpportunityModel::select_in_page(
        &db,
        page,
        page_size,
        query.keywords.clone(),
        query.stage.clone(),
        query.assigned_to,
        query.customer_id,
    ).await?;
    
    let data: Vec<OpportunityListVO> = list.into_iter().map(|item| item.into()).collect();
    Ok(ResultPage::new(data, total, page, page_size))
}