use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::sale::model::quotation::{QuotationDetailVO, QuotationListQuery, QuotationListVO, QuotationModel, QuotationSaveDTO, QuotationSaveRequest, QuotationUpdateRequest};
use sea_orm::DbConn;

pub async fn insert(db: &DbConn, form_data: &QuotationSaveRequest, created_by: i64) -> Result<i64> {
    let mut dto: QuotationSaveDTO = form_data.clone().into();
    dto.created_by = Some(created_by);
    let result = QuotationModel::insert(db, &dto).await?;
    Ok(result)
}

pub async fn update(db: &DbConn, form_data: &QuotationUpdateRequest, updated_by: i64) -> Result<i64> {
    let mut dto: QuotationSaveDTO = form_data.clone().into();
    dto.updated_by = Some(updated_by);
    let result = QuotationModel::update_by_id(db, &form_data.id, &dto).await?;
    Ok(result)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let result = QuotationModel::batch_delete_by_ids(db, ids_vec).await?;
    Ok(result)
}

pub async fn find_by_id(db: &DbConn, id: i64) -> Result<QuotationDetailVO> {
    let result = QuotationModel::find_by_id(db, id).await?;
    match result {
        Some(item) => Ok(item.into()),
        None => Err(Error::from("报价单不存在".to_string())),
    }
}

pub async fn list(db: &DbConn, query: &QuotationListQuery) -> Result<ResultPage<Vec<QuotationListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let (list, total) = QuotationModel::select_in_page(
        db,
        page,
        page_size,
        query.keywords.clone(),
        query.customer_id,
        query.status.clone(),
        query.start_date.clone(),
        query.end_date.clone(),
    ).await?;

    let data: Vec<QuotationListVO> = list.into_iter().map(|item| item.into()).collect();
    Ok(ResultPage { items: data, total, current_page: page, page_size, total_pages: 0 })
}

pub async fn update_status(db: &DbConn, id: i64, status: &str) -> Result<i64> {
    let result = QuotationModel::update_status(db, id, status).await?;
    Ok(result)
}