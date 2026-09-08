use crate::core::errors::error::{Error, Result};
use crate::modules::purchase::model::supplier_brand::{SupplierBrandDTO, SupplierBrandModel, SupplierBrandVO};
use sea_orm::DbConn;

pub async fn insert(db: &DbConn, form_data: &SupplierBrandDTO) -> Result<i64> {
    let result = SupplierBrandModel::insert(db, form_data).await?;
    Ok(result)
}

pub async fn update(db: &DbConn, form_data: &SupplierBrandDTO) -> Result<i64> {
    let id = form_data.id.unwrap_or_default();
    if id == 0 {
        return Err(Error::from("供应商品牌关联ID不能为空".to_string()));
    }
    let result = SupplierBrandModel::update(db, id, form_data).await?;
    Ok(result)
}

pub async fn batch_delete(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let result = SupplierBrandModel::batch_delete(db, ids_vec).await?;
    Ok(result)
}

pub async fn list_by_supplier(db: &DbConn, supplier_id: i64) -> Result<Vec<SupplierBrandVO>> {
    let list = SupplierBrandModel::find_by_supplier_id(db, supplier_id).await?;
    let data: Vec<SupplierBrandVO> = list.into_iter().map(|item| item.into()).collect();
    Ok(data)
}

pub async fn list_by_brand(db: &DbConn, brand_id: i64) -> Result<Vec<SupplierBrandVO>> {
    let list = SupplierBrandModel::find_by_brand_id(db, brand_id).await?;
    let data: Vec<SupplierBrandVO> = list.into_iter().map(|item| item.into()).collect();
    Ok(data)
}