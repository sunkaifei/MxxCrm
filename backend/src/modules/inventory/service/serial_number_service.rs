//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 序列号管理业务逻辑层
//!

use crate::core::errors::error::{Error, Result};
use crate::modules::inventory::entity::serial_number::{self, Entity, Column};
use chrono::NaiveDate;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DbConn, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SerialByProductQuery {
    pub product_id: i64,
    pub status: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ExpiryAlertVO {
    #[serde(flatten)]
    pub serial: serial_number::Model,
    pub days_until_expiry: i64,
    pub product_name: Option<String>,
}

/// 批量导入序列号
pub async fn import_serials(
    db: &DbConn,
    product_id: i64,
    warehouse_id: i64,
    serials: Vec<(String, Option<NaiveDate>, Option<NaiveDate>)>,
) -> Result<i64> {
    if serials.is_empty() {
        return Err(Error::from("序列号列表不能为空"));
    }

    let now = chrono::Local::now().naive_local();
    let txn = db.begin().await?;
    let mut count = 0i64;

    for (serial_no, production_date, expiry_date) in serials {
        let model = serial_number::ActiveModel {
            product_id: Set(Some(product_id)),
            warehouse_id: Set(Some(warehouse_id)),
            serial_no: Set(Some(serial_no)),
            production_date: Set(production_date),
            expiry_date: Set(expiry_date),
            status: Set(Some(1)),
            create_time: Set(Some(now)),
            ..Default::default()
        };
        model.insert(&txn).await?;
        count += 1;
    }

    txn.commit().await?;
    Ok(count)
}

/// 出库绑定
pub async fn bind_serial_to_order(db: &DbConn, serial_id: i64, order_item_id: i64) -> Result<i64> {
    let existing = Entity::find_by_id(serial_id)
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("序列号不存在"))?;

    let status = existing.status.unwrap_or(0);
    if status != 1 {
        return Err(Error::from(format!("当前状态({})不允许绑定，仅库存中(1)状态可绑定", status)));
    }

    let now = chrono::Local::now().naive_local();
    let txn = db.begin().await?;
    let mut active: serial_number::ActiveModel = existing.into();
    active.order_item_id = Set(Some(order_item_id));
    active.status = Set(Some(2));
    active.update_time = Set(Some(now));
    active.update(&txn).await?;
    txn.commit().await?;

    Ok(serial_id)
}

/// 退货解绑
pub async fn unbind_serial(db: &DbConn, serial_id: i64) -> Result<i64> {
    let existing = Entity::find_by_id(serial_id)
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("序列号不存在"))?;

    let status = existing.status.unwrap_or(0);
    if status != 2 {
        return Err(Error::from(format!("当前状态({})不允许解绑，仅已出库(2)状态可解绑", status)));
    }

    let now = chrono::Local::now().naive_local();
    let txn = db.begin().await?;
    let mut active: serial_number::ActiveModel = existing.into();
    active.order_item_id = Set(None);
    active.status = Set(Some(1));
    active.update_time = Set(Some(now));
    active.update(&txn).await?;
    txn.commit().await?;

    Ok(serial_id)
}

/// 按产品查询
pub async fn get_serials_by_product(
    db: &DbConn,
    product_id: i64,
    status: Option<i32>,
) -> Result<Vec<serial_number::Model>> {
    let mut cond = Condition::all()
        .add(Column::ProductId.eq(product_id))
        .add(Column::Deleted.eq(0));
    if let Some(s) = status {
        if s > 0 {
            cond = cond.add(Column::Status.eq(s));
        }
    }

    let list = Entity::find()
        .filter(cond)
        .order_by_desc(Column::Id)
        .all(db)
        .await?;

    Ok(list)
}

/// 临期预警（查询 expiry_date 在 days 天内的记录）
pub async fn check_expiry_alerts(db: &DbConn, days: i32) -> Result<Vec<ExpiryAlertVO>> {
    let today = chrono::Local::now().date_naive();
    let threshold = today + chrono::Duration::days(days as i64);

    let list = Entity::find()
        .filter(Column::Deleted.eq(0))
        .filter(Column::ExpiryDate.is_not_null())
        .filter(Column::ExpiryDate.lte(threshold))
        .filter(Column::Status.eq(1))
        .order_by_asc(Column::ExpiryDate)
        .all(db)
        .await?;

    let data: Vec<ExpiryAlertVO> = list
        .into_iter()
        .map(|m| {
            let days_until_expiry = m.expiry_date
                .map(|d| (d - today).num_days())
                .unwrap_or(0);
            ExpiryAlertVO {
                days_until_expiry,
                product_name: None,
                serial: m,
            }
        })
        .collect();

    Ok(data)
}

/// 按SN查询
pub async fn get_serial_info(db: &DbConn, serial_no: String) -> Result<serial_number::Model> {
    Entity::find()
        .filter(Column::SerialNo.eq(serial_no))
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("序列号不存在"))
}
