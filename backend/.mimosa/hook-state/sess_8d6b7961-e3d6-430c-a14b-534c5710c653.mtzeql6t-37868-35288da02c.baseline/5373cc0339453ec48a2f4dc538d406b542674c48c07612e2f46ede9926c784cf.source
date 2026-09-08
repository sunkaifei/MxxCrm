//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::*;
use crate::core::errors::error::{Error, Result};
use crate::modules::inventory::model::stock_log::*;
use crate::modules::inventory::model::stock_log as stock_log_model;
use crate::modules::inventory::entity::warehouse;
use crate::modules::product::entity::product as product_entity;
use crate::modules::system::entity::admin;

/// 库存流水列表查询
pub async fn get_list(
    db: &DatabaseConnection,
    query: &StockLogListQuery,
) -> Result<StockLogListVO> {
    let (models, total) = stock_log_model::select_page(db, query).await
        .map_err(|e| Error::from(e.to_string()))?;

    let mut list: Vec<StockLogListItem> = models.into_iter().map(|m| m.into()).collect();

    // 补充产品名称、仓库名称和操作人名称
    for item in &mut list {
        if let Some(pid) = item.product_id {
            if let Ok(Some(prod)) = product_entity::Entity::find_by_id(pid).one(db).await {
                item.product_name = prod.name;
            }
        }
        if let Some(wid) = item.warehouse_id {
            if let Ok(Some(wh)) = warehouse::Entity::find_by_id(wid)
                .filter(warehouse::Column::Deleted.eq(0))
                .one(db).await
            {
                item.warehouse_name = wh.name;
            }
        }
        if let Some(oid) = item.operator_id {
            if let Ok(Some(admin)) = admin::Entity::find_by_id(oid).one(db).await {
                item.operator_name = admin.nick_name.or(admin.user_name);
            }
        }
    }

    Ok(StockLogListVO { list, total })
}