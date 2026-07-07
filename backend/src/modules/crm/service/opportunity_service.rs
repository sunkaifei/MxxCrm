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
use crate::modules::crm::entity::customer;
use crate::modules::crm::model::opportunity::{OpportunityDetailVO, OpportunityListQuery, OpportunityListVO, OpportunityModel, OpportunitySaveDTO, OpportunitySaveRequest, OpportunityUpdateRequest};
use crate::modules::system::entity::{admin, admin::Entity as Admin};
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter};
use std::collections::HashMap;

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
    
    // 批量查询客户名称
    let customer_ids: Vec<i64> = list.iter()
        .filter_map(|item| item.customer_id)
        .collect();
    let mut customer_map: HashMap<i64, String> = HashMap::new();
    if !customer_ids.is_empty() {
        let customers = customer::Entity::find()
            .filter(customer::Column::Id.is_in(customer_ids))
            .all(db)
            .await?;
        for c in customers {
            if let Some(name) = c.company_name {
                customer_map.insert(c.id, name);
            }
        }
    }
    
    // 批量查询创建人名称
    let creator_ids: Vec<i64> = list.iter()
        .filter_map(|item| item.created_by)
        .collect();
    let mut creator_map: HashMap<i64, String> = HashMap::new();
    if !creator_ids.is_empty() {
        let admins = Admin::find()
            .filter(admin::Column::Id.is_in(creator_ids))
            .all(db)
            .await?;
        for a in admins {
            if let Some(name) = a.nick_name.or(a.user_name) {
                creator_map.insert(a.id, name);
            }
        }
    }
    
    let data: Vec<OpportunityListVO> = list.into_iter().map(|item| {
        let customer_id = item.customer_id;
        let created_by = item.created_by;
        let mut vo: OpportunityListVO = item.into();
        vo.customer_name = customer_id.and_then(|id| customer_map.get(&id).cloned());
        vo.created_by_name = created_by.and_then(|id| creator_map.get(&id).cloned());
        vo
    }).collect();
    
    Ok(ResultPage::new(data, total, page, page_size))
}