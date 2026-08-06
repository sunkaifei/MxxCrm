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
use crate::modules::purchase::model::purchase_supplier_product::{
    SupplierProductDTO, SupplierProductModel,
};
use sea_orm::DbConn;

pub async fn insert(db: &DbConn, form_data: &SupplierProductDTO) -> Result<i64> {
    let result = SupplierProductModel::insert(db, form_data).await?;
    Ok(result)
}

pub async fn update(db: &DbConn, form_data: &SupplierProductDTO) -> Result<i64> {
    let id = form_data.id.unwrap_or_default();
    if id == 0 {
        return Err(Error::from("供应商产品关联ID不能为空".to_string()));
    }
    let result = SupplierProductModel::update_by_id(db, id, form_data).await?;
    Ok(result)
}

pub async fn batch_delete(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let result = SupplierProductModel::batch_delete_by_ids(db, ids_vec).await?;
    Ok(result)
}

pub async fn get_info(db: &DbConn, id: i64) -> Result<SupplierProductDTO> {
    let model = SupplierProductModel::find_by_id(db, id)
        .await?
        .ok_or_else(|| Error::from("供应商产品关联不存在"))?;
    Ok(SupplierProductDTO {
        id: Some(model.id),
        supplier_id: model.supplier_id,
        product_id: model.product_id,
        lead_time_days: model.lead_time_days,
        moq: model.moq,
        supplier_sku: model.supplier_sku,
        agreement_price: model.agreement_price,
        is_preferred: model.is_preferred,
        remark: model.remark,
    })
}

pub async fn get_list(db: &DbConn) -> Result<Vec<SupplierProductDTO>> {
    let list = SupplierProductModel::find_all(db).await?;
    let data: Vec<SupplierProductDTO> = list
        .into_iter()
        .map(|m| SupplierProductDTO {
            id: Some(m.id),
            supplier_id: m.supplier_id,
            product_id: m.product_id,
            lead_time_days: m.lead_time_days,
            moq: m.moq,
            supplier_sku: m.supplier_sku,
            agreement_price: m.agreement_price,
            is_preferred: m.is_preferred,
            remark: m.remark,
        })
        .collect();
    Ok(data)
}

pub async fn list_by_supplier(db: &DbConn, supplier_id: i64) -> Result<Vec<SupplierProductDTO>> {
    let list = SupplierProductModel::find_by_supplier_id(db, supplier_id).await?;
    let data: Vec<SupplierProductDTO> = list
        .into_iter()
        .map(|m| SupplierProductDTO {
            id: Some(m.id),
            supplier_id: m.supplier_id,
            product_id: m.product_id,
            lead_time_days: m.lead_time_days,
            moq: m.moq,
            supplier_sku: m.supplier_sku,
            agreement_price: m.agreement_price,
            is_preferred: m.is_preferred,
            remark: m.remark,
        })
        .collect();
    Ok(data)
}

pub async fn list_by_product(db: &DbConn, product_id: i64) -> Result<Vec<SupplierProductDTO>> {
    let list = SupplierProductModel::find_by_product_id(db, product_id).await?;
    let data: Vec<SupplierProductDTO> = list
        .into_iter()
        .map(|m| SupplierProductDTO {
            id: Some(m.id),
            supplier_id: m.supplier_id,
            product_id: m.product_id,
            lead_time_days: m.lead_time_days,
            moq: m.moq,
            supplier_sku: m.supplier_sku,
            agreement_price: m.agreement_price,
            is_preferred: m.is_preferred,
            remark: m.remark,
        })
        .collect();
    Ok(data)
}
