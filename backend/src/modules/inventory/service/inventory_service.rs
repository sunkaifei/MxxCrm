//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::Result;
use crate::modules::inventory::entity::stock::{Column, Entity};
use crate::modules::inventory::model::stock::{InventoryDetailVO, InventoryListData, InventoryListQuery, InventoryListVO};
use crate::modules::product::entity::product as product_entity;
use crate::modules::inventory::entity::warehouse as warehouse_entity;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter};

pub async fn get_list(db: &DatabaseConnection, query: &InventoryListQuery) -> Result<InventoryListData> {
    let page_num = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(10);

    let mut condition = Entity::find()
        .filter(Column::Deleted.eq(0));

    if let Some(warehouse_id) = query.warehouse_id {
        condition = condition.filter(Column::WarehouseId.eq(warehouse_id));
    }

    let paginator = condition.paginate(db, page_size as u64);
    let total = paginator.num_items().await?;
    let models = paginator.fetch_page((page_num - 1) as u64).await?;

    let mut result: Vec<InventoryListVO> = Vec::new();
    for stock in models {
        let product: Option<product_entity::Model> = product_entity::Entity::find_by_id(stock.product_id.unwrap_or(0))
            .one(db)
            .await
            .ok()
            .flatten();
        let warehouse: Option<warehouse_entity::Model> = warehouse_entity::Entity::find_by_id(stock.warehouse_id.unwrap_or(0))
            .one(db)
            .await
            .ok()
            .flatten();

        let mut vo = InventoryListVO {
            id: Some(stock.id),
            product_id: stock.product_id,
            product_name: product.as_ref().and_then(|p| p.name.clone()),
            product_code: product.as_ref().and_then(|p| p.product_no.clone()),
            warehouse_id: stock.warehouse_id,
            warehouse_name: warehouse.as_ref().and_then(|w| w.name.clone()),
            quantity: stock.quantity,
            reserved_quantity: stock.reserved_quantity,
            available_quantity: stock.available_quantity,
            update_time: stock.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        };

        if let Some(name) = &query.product_name {
            if let Some(pn) = &vo.product_name {
                if pn.contains(name) {
                    result.push(vo);
                }
            }
        } else {
            result.push(vo);
        }
    }

    Ok(InventoryListData { total: total as i64, items: result })
}

pub async fn get_detail(db: &DatabaseConnection, id: i64) -> Result<InventoryDetailVO> {
    let stock = Entity::find_by_id(id)
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| format!("库存不存在，ID: {}", id))?;

    let product: Option<product_entity::Model> = product_entity::Entity::find_by_id(stock.product_id.unwrap_or(0))
        .one(db)
        .await
        .ok()
        .flatten();
    let warehouse: Option<warehouse_entity::Model> = warehouse_entity::Entity::find_by_id(stock.warehouse_id.unwrap_or(0))
        .one(db)
        .await
        .ok()
        .flatten();

    Ok(InventoryDetailVO {
        id: Some(stock.id),
        product_id: stock.product_id,
        product_name: product.as_ref().and_then(|p| p.name.clone()),
        product_code: product.as_ref().and_then(|p| p.product_no.clone()),
        spec: product.as_ref().and_then(|p| p.sku.clone()),
        unit: product.as_ref().and_then(|p| p.unit.clone()),
        warehouse_id: stock.warehouse_id,
        warehouse_name: warehouse.as_ref().and_then(|w| w.name.clone()),
        warehouse_code: warehouse.as_ref().and_then(|w| w.code.clone()),
        quantity: stock.quantity,
        reserved_quantity: stock.reserved_quantity,
        available_quantity: stock.available_quantity,
        update_time: stock.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
    })
}
