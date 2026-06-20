use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::sale::model::invoice::{InvoiceDetailVO, InvoiceListQuery, InvoiceListVO, InvoiceModel, InvoiceSaveDTO, InvoiceSaveRequest, InvoiceUpdateRequest};
use sea_orm::DbConn;

pub async fn insert(db: &DbConn, form_data: &InvoiceSaveRequest, created_by: i64) -> Result<i64> {
    let mut dto: InvoiceSaveDTO = form_data.clone().into();
    dto.created_by = Some(created_by);
    let result = InvoiceModel::insert(db, &dto).await?;
    Ok(result)
}

pub async fn update(db: &DbConn, form_data: &InvoiceUpdateRequest, updated_by: i64) -> Result<i64> {
    let mut dto: InvoiceSaveDTO = form_data.clone().into();
    dto.updated_by = Some(updated_by);
    let result = InvoiceModel::update_by_id(db, &form_data.id, &dto).await?;
    Ok(result)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let result = InvoiceModel::batch_delete_by_ids(db, ids_vec).await?;
    Ok(result)
}

pub async fn find_by_id(db: &DbConn, id: i64) -> Result<InvoiceDetailVO> {
    let result = InvoiceModel::find_by_id(db, id).await?;
    match result {
        Some(item) => Ok(item.into()),
        None => Err(Error::from("发票不存在".to_string())),
    }
}

pub async fn list(db: &DbConn, query: &InvoiceListQuery) -> Result<ResultPage<Vec<InvoiceListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let (list, total) = InvoiceModel::select_in_page(
        db,
        page,
        page_size,
        query.keywords.clone(),
        query.invoice_type.clone(),
        query.status.clone(),
        query.customer_id,
    ).await?;

    let data: Vec<InvoiceListVO> = list.into_iter().map(|item| item.into()).collect();
    Ok(ResultPage { items: data, total, current_page: page, page_size, total_pages: 0 })
}