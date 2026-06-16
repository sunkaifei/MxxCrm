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
use crate::core::web::response::ResultPage;
use crate::modules::crm::model::customer::{CustomerDetailVO, CustomerListQuery, CustomerListVO, CustomerModel, CustomerSaveDTO, CustomerSaveRequest, CustomerUpdateRequest};
use sea_orm::DbConn;

pub async fn insert(db: &DbConn, form_data: &CustomerSaveRequest, created_by: i64) -> Result<i64> {
    let mut dto: CustomerSaveDTO = form_data.clone().into();
    dto.created_by = Some(created_by);
    let result = CustomerModel::insert(&db, &dto).await?;
    Ok(result)
}

pub async fn update(db: &DbConn, form_data: &CustomerUpdateRequest, updated_by: i64) -> Result<i64> {
    let mut dto: CustomerSaveDTO = form_data.clone().into();
    dto.updated_by = Some(updated_by);
    let result = CustomerModel::update_by_id(&db, &form_data.id, &dto).await?;
    Ok(result)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let result = CustomerModel::batch_delete_by_ids(&db, &ids_vec).await?;
    Ok(result)
}

pub async fn find_by_id(db: &DbConn, id: i64) -> Result<CustomerDetailVO> {
    let result = CustomerModel::find_by_id(&db, id).await?;
    match result {
        Some(item) => Ok(item.into()),
        None => Err(Error::from("客户不存在")),
    }
}

pub async fn list(db: &DbConn, query: &CustomerListQuery) -> Result<ResultPage<Vec<CustomerListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    
    let (list, total) = CustomerModel::select_in_page(
        &db,
        page,
        page_size,
        query.keywords.clone(),
        query.level.clone(),
        query.country.clone(),
        query.source.clone(),
        query.assigned_to,
    ).await?;
    
    let data: Vec<CustomerListVO> = list.into_iter().map(|item| item.into()).collect();
    Ok(ResultPage::new(data, total, page, page_size))
}
