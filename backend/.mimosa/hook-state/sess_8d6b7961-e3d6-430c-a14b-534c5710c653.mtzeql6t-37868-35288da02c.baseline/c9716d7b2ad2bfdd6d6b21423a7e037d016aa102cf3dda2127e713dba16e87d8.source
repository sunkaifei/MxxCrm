//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::DatabaseConnection;
use crate::modules::finance::model::payment_record::{PaymentRecordDTO, PaymentRecordSaveRequest, PaymentRecordQuery, PaymentRecordModel};

pub async fn get_list(db: &DatabaseConnection, query: PaymentRecordQuery) -> Result<(Vec<PaymentRecordDTO>, i64), String> {
    PaymentRecordModel::find_list(db, query).await.map_err(|e| e.to_string())
}

pub async fn get_by_id(db: &DatabaseConnection, id: i64) -> Result<Option<PaymentRecordDTO>, String> {
    PaymentRecordModel::find_by_id(db, id).await.map_err(|e| e.to_string())
}

pub async fn insert(db: &DatabaseConnection, req: PaymentRecordSaveRequest) -> Result<PaymentRecordDTO, String> {
    PaymentRecordModel::insert(db, req).await.map_err(|e| e.to_string())
}

pub async fn update(db: &DatabaseConnection, id: i64, req: PaymentRecordSaveRequest) -> Result<PaymentRecordDTO, String> {
    PaymentRecordModel::update(db, id, req).await.map_err(|e| e.to_string())
}

pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<bool, String> {
    PaymentRecordModel::delete(db, id).await.map_err(|e| e.to_string())
}

pub async fn get_by_order_id(db: &DatabaseConnection, order_id: &str) -> Result<Option<PaymentRecordDTO>, String> {
    PaymentRecordModel::find_by_order_id(db, order_id).await.map_err(|e| e.to_string())
}

pub async fn update_paid(db: &DatabaseConnection, order_id: &str, transaction_id: &str) -> Result<bool, String> {
    PaymentRecordModel::update_paid(db, order_id, transaction_id).await.map_err(|e| e.to_string())
}

pub async fn update_failed(db: &DatabaseConnection, order_id: &str, remark: &str) -> Result<bool, String> {
    PaymentRecordModel::update_failed(db, order_id, remark).await.map_err(|e| e.to_string())
}

pub async fn update_refunded(db: &DatabaseConnection, order_id: &str, remark: &str) -> Result<bool, String> {
    PaymentRecordModel::update_refunded(db, order_id, remark).await.map_err(|e| e.to_string())
}
