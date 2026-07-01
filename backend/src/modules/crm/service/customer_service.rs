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
use crate::modules::system::entity::{admin, admin::Entity as Admin};
use sea_orm::{DbConn, ColumnTrait, EntityTrait, QueryFilter};
use std::collections::{HashMap, HashSet};

pub async fn insert(db: &DbConn, form_data: &CustomerSaveRequest, created_by: i64) -> Result<i64> {
    let mut dto: CustomerSaveDTO = form_data.clone().into();
    dto.created_by = Some(created_by);
    let result = CustomerModel::insert(db, &dto).await?;
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

/// 公海客户列表
pub async fn pool_list(db: &DbConn, query: &CustomerListQuery) -> Result<ResultPage<Vec<CustomerListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let (list, total) = CustomerModel::select_pool_in_page(
        &db,
        page,
        page_size,
        query.keywords.clone(),
        query.level.clone(),
        query.country.clone(),
        query.source.clone(),
    ).await?;

    // 批量查询创建人名称
    let creator_ids: Vec<i64> = list.iter()
        .filter_map(|item| item.created_by)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let mut creator_map: HashMap<i64, String> = HashMap::new();
    if !creator_ids.is_empty() {
        let admins = Admin::find()
            .filter(admin::Column::Id.is_in(creator_ids))
            .all(db)
            .await?;
        for a in admins {
            if let Some(name) = a.user_name {
                creator_map.insert(a.id, name);
            }
        }
    }

    let data: Vec<CustomerListVO> = list.into_iter().map(|item| {
        let created_by = item.created_by;
        let created_by_name = created_by.and_then(|id| creator_map.get(&id).cloned());
        let mut vo: CustomerListVO = item.into();
        vo.created_by_name = created_by_name;
        vo
    }).collect();

    Ok(ResultPage::new(data, total, page, page_size))
}

/// 领取公海客户
pub async fn claim(db: &DbConn, id: i64, user_id: i64) -> Result<i64> {
    let customer = CustomerModel::find_by_id(&db, id).await?;
    if customer.is_none() {
        return Err(Error::from("客户不存在"));
    }
    if customer.unwrap().assigned_to.is_some() {
        return Err(Error::from("该客户已被领取，无法重复领取"));
    }
    let result = CustomerModel::claim(&db, id, user_id).await?;
    if result == 0 {
        return Err(Error::from("领取失败，该客户可能已被他人领取"));
    }
    Ok(result)
}

/// 退回公海
pub async fn add_to_pool(db: &DbConn, id: i64, user_id: i64) -> Result<i64> {
    let customer = CustomerModel::find_by_id(&db, id).await?;
    if customer.is_none() {
        return Err(Error::from("客户不存在"));
    }
    let result = CustomerModel::add_to_pool(&db, id, user_id).await?;
    Ok(result)
}
