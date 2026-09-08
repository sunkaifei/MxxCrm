use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::purchase::model::supplier::{SupplierDetailVO, SupplierListQuery, SupplierListVO, SupplierModel, SupplierSaveRequest, SupplierUpdateRequest};
use sea_orm::DbConn;

pub async fn insert(db: &DbConn, form_data: &SupplierSaveRequest, created_by: i64) -> Result<i64> {
    let result = SupplierModel::insert(db, form_data, created_by).await?;
    Ok(result)
}

pub async fn update(db: &DbConn, form_data: &SupplierUpdateRequest, updated_by: i64) -> Result<i64> {
    let id = form_data.id.unwrap_or_default();
    if id == 0 {
        return Err(Error::from("供应商ID不能为空".to_string()));
    }
    let result = SupplierModel::update_by_id(db, id, form_data, updated_by).await?;
    Ok(result)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let result = SupplierModel::batch_delete_by_ids(db, ids_vec).await?;
    Ok(result)
}

pub async fn find_by_id(db: &DbConn, id: i64) -> Result<SupplierDetailVO> {
    let result = SupplierModel::find_by_id(db, id).await?;
    match result {
        Some(item) => Ok(item.into()),
        None => Err(Error::from("供应商不存在".to_string())),
    }
}

pub async fn list(db: &DbConn, query: &SupplierListQuery) -> Result<ResultPage<Vec<SupplierListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    
    let (list, total) = SupplierModel::select_in_page(
        db,
        page,
        page_size,
        query.keywords.clone(),
        query.level,
        query.country.clone(),
        query.status,
    ).await?;
    
    let data: Vec<SupplierListVO> = list.into_iter().map(|item| item.into()).collect();
    Ok(ResultPage::new(data, total, page, page_size))
}
