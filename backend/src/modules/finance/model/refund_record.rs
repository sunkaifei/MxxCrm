//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use serde::{Deserialize, Serialize};
use sea_orm::*;
use crate::modules::finance::entity::refund_record;
use crate::modules::finance::entity::refund_record::Entity as RefundRecordEntity;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use chrono::{DateTime, NaiveDateTime, Utc, TimeZone};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundRecordDTO {
    pub id: i64,
    pub user_id: i64,
    pub payment_record_id: i64,
    pub amount: f64,
    pub status: Option<i32>,
    pub transaction_id: Option<String>,
    pub refund_time: Option<String>,
    pub reason: Option<String>,
    pub remark: Option<String>,
    pub create_time: Option<String>,
}

impl From<refund_record::Model> for RefundRecordDTO {
    fn from(model: refund_record::Model) -> Self {
        Self {
            id: model.id,
            user_id: model.user_id,
            payment_record_id: model.payment_record_id,
            amount: model.amount.to_f64().unwrap_or_default(),
            status: model.status,
            transaction_id: model.transaction_id,
            refund_time: model.refund_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            reason: model.reason,
            remark: model.remark,
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundRecordSaveRequest {
    pub user_id: i64,
    pub payment_record_id: i64,
    pub amount: f64,
    pub status: Option<i32>,
    pub transaction_id: Option<String>,
    pub refund_time: Option<String>,
    pub reason: Option<String>,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundRecordQuery {
    pub user_id: Option<i64>,
    pub status: Option<i32>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub struct RefundRecordModel;

impl RefundRecordModel {
    pub async fn find_list(db: &DbConn, query: RefundRecordQuery) -> Result<(Vec<RefundRecordDTO>, i64), DbErr> {
        let mut stmt = RefundRecordEntity::find();
        
        if let Some(user_id) = query.user_id {
            stmt = stmt.filter(refund_record::Column::UserId.eq(user_id));
        }
        
        if let Some(status) = query.status {
            stmt = stmt.filter(refund_record::Column::Status.eq(status));
        }
        
        if let Some(start_time) = query.start_time {
            if let Ok(dt) = NaiveDateTime::parse_from_str(&start_time, "%Y-%m-%d %H:%M:%S") {
                let dt: DateTime<Utc> = Utc.from_utc_datetime(&dt);
                stmt = stmt.filter(refund_record::Column::CreateTime.gte(dt));
            }
        }

        if let Some(end_time) = query.end_time {
            if let Ok(dt) = NaiveDateTime::parse_from_str(&end_time, "%Y-%m-%d %H:%M:%S") {
                let dt: DateTime<Utc> = Utc.from_utc_datetime(&dt);
                stmt = stmt.filter(refund_record::Column::CreateTime.lte(dt));
            }
        }
        
        stmt = stmt.order_by_desc(refund_record::Column::CreateTime);
        
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        
        let paginator = stmt.paginate(db, page_size as u64);
        let total = paginator.num_items().await? as i64;
        let items = paginator.fetch_page((page - 1) as u64).await?;
        
        let dto_list: Vec<RefundRecordDTO> = items.into_iter().map(RefundRecordDTO::from).collect();
        
        Ok((dto_list, total))
    }

    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<RefundRecordDTO>, DbErr> {
        let model = RefundRecordEntity::find_by_id(id)
            .one(db)
            .await?;
        
        Ok(model.map(RefundRecordDTO::from))
    }

    pub async fn insert(db: &DbConn, req: RefundRecordSaveRequest) -> Result<RefundRecordDTO, DbErr> {
        let now = Some(Utc::now());
        
        let refund_time = req.refund_time.as_ref()
            .and_then(|s| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok())
            .map(|dt| Utc.from_utc_datetime(&dt));
        
        let model = refund_record::ActiveModel {
            user_id: Set(req.user_id),
            payment_record_id: Set(req.payment_record_id),
            amount: Set(Decimal::from_f64(req.amount).unwrap_or_default()),
            status: Set(req.status),
            transaction_id: Set(req.transaction_id),
            refund_time: Set(refund_time),
            reason: Set(req.reason),
            remark: Set(req.remark),
            create_time: Set(now),
            update_time: Set(now),
            ..Default::default()
        };
        
        let result = model.insert(db).await?;
        
        Ok(RefundRecordDTO::from(result))
    }

    pub async fn update(db: &DbConn, id: i64, req: RefundRecordSaveRequest) -> Result<RefundRecordDTO, DbErr> {
        let mut model: refund_record::ActiveModel = RefundRecordEntity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("记录不存在".to_string()))?
            .into();
        
        model.user_id = Set(req.user_id);
        model.payment_record_id = Set(req.payment_record_id);
        model.amount = Set(Decimal::from_f64(req.amount).unwrap_or_default());
        model.status = Set(req.status);
        model.transaction_id = Set(req.transaction_id);
        
        if let Some(refund_time_str) = &req.refund_time {
            if let Ok(dt) = NaiveDateTime::parse_from_str(refund_time_str, "%Y-%m-%d %H:%M:%S") {
                model.refund_time = Set(Some(DateTime::from_naive_utc_and_offset(dt, Utc)));
            }
        }
        
        model.reason = Set(req.reason);
        model.remark = Set(req.remark);
        model.update_time = Set(Some(Utc::now()));
        
        let result = model.update(db).await?;
        
        Ok(RefundRecordDTO::from(result))
    }

    pub async fn delete(db: &DbConn, id: i64) -> Result<bool, DbErr> {
        let count = RefundRecordEntity::delete_by_id(id)
            .exec(db)
            .await?;
        
        Ok(count.rows_affected > 0)
    }
}
