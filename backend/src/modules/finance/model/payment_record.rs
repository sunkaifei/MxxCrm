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
use crate::modules::finance::entity::payment_record;
use crate::modules::finance::entity::payment_record::Entity as PaymentRecordEntity;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use chrono::{DateTime, NaiveDateTime, Utc, TimeZone, Datelike};

/// 支付状态常量
pub const PAY_STATUS_PENDING: i32 = 0;      /// 待支付
pub const PAY_STATUS_SUCCESS: i32 = 1;      /// 支付成功
pub const PAY_STATUS_FAILED: i32 = 2;       /// 支付失败
pub const PAY_STATUS_REFUNDED: i32 = 3;     /// 已退款

/// 支付方式常量
pub const PAY_METHOD_WECHAT: i32 = 1;       /// 微信支付
pub const PAY_METHOD_ALIPAY: i32 = 2;       /// 支付宝
pub const PAY_METHOD_BANK: i32 = 3;         /// 银行卡

/// 支付类型常量
pub const PAY_TYPE_MEMBER: i32 = 1;         /// 会员费用
pub const PAY_TYPE_PRODUCT: i32 = 2;        /// 商品购买
pub const PAY_TYPE_RECHARGE: i32 = 3;       /// 充值
pub const PAY_TYPE_OTHER: i32 = 4;          /// 其他

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRecordDTO {
    pub id: i64,
    pub user_id: i64,
    pub member_product_id: Option<i64>,
    pub order_id: Option<String>,
    pub payment_type: Option<i32>,
    pub amount: f64,
    pub pay_method: Option<i32>,
    pub status: Option<i32>,
    pub transaction_id: Option<String>,
    pub pay_time: Option<String>,
    pub remark: Option<String>,
    pub create_time: Option<String>,
}

impl From<payment_record::Model> for PaymentRecordDTO {
    fn from(model: payment_record::Model) -> Self {
        Self {
            id: model.id,
            user_id: model.user_id,
            member_product_id: model.member_product_id,
            order_id: model.order_id,
            payment_type: model.payment_type,
            amount: model.amount.to_f64().unwrap_or_default(),
            pay_method: model.pay_method,
            status: model.status,
            transaction_id: model.transaction_id,
            pay_time: model.pay_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            remark: model.remark,
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRecordSaveRequest {
    pub user_id: i64,
    pub member_product_id: Option<i64>,
    pub order_id: Option<String>,
    pub payment_type: Option<i32>,
    pub amount: f64,
    pub pay_method: Option<i32>,
    pub status: Option<i32>,
    pub transaction_id: Option<String>,
    pub pay_time: Option<String>,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRecordQuery {
    pub user_id: Option<i64>,
    pub payment_type: Option<i32>,
    pub status: Option<i32>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub struct PaymentRecordModel;

impl PaymentRecordModel {
    pub async fn find_list(db: &DbConn, query: PaymentRecordQuery) -> Result<(Vec<PaymentRecordDTO>, i64), DbErr> {
        let mut stmt = PaymentRecordEntity::find();
        
        if let Some(user_id) = query.user_id {
            stmt = stmt.filter(payment_record::Column::UserId.eq(user_id));
        }
        
        if let Some(payment_type) = query.payment_type {
            stmt = stmt.filter(payment_record::Column::PaymentType.eq(payment_type));
        }
        
        if let Some(status) = query.status {
            stmt = stmt.filter(payment_record::Column::Status.eq(status));
        }
        
        if let Some(start_time) = query.start_time {
            if let Ok(dt) = NaiveDateTime::parse_from_str(&start_time, "%Y-%m-%d %H:%M:%S") {
                let dt: DateTime<Utc> = Utc.from_utc_datetime(&dt);
                stmt = stmt.filter(payment_record::Column::CreateTime.gte(dt));
            }
        }

        if let Some(end_time) = query.end_time {
            if let Ok(dt) = NaiveDateTime::parse_from_str(&end_time, "%Y-%m-%d %H:%M:%S") {
                let dt: DateTime<Utc> = Utc.from_utc_datetime(&dt);
                stmt = stmt.filter(payment_record::Column::CreateTime.lte(dt));
            }
        }
        
        stmt = stmt.order_by_desc(payment_record::Column::CreateTime);
        
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        
        let paginator = stmt.paginate(db, page_size as u64);
        let total = paginator.num_items().await? as i64;
        let items = paginator.fetch_page((page - 1) as u64).await?;
        
        let dto_list: Vec<PaymentRecordDTO> = items.into_iter().map(PaymentRecordDTO::from).collect();
        
        Ok((dto_list, total))
    }

    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<PaymentRecordDTO>, DbErr> {
        let model = PaymentRecordEntity::find_by_id(id)
            .one(db)
            .await?;
        
        Ok(model.map(PaymentRecordDTO::from))
    }

    pub async fn insert(db: &DbConn, req: PaymentRecordSaveRequest) -> Result<PaymentRecordDTO, DbErr> {
        let now = Some(Utc::now());
        
        let pay_time = req.pay_time.as_ref()
            .and_then(|s| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok())
            .map(|dt| Utc.from_utc_datetime(&dt));
        
        let model = payment_record::ActiveModel {
            user_id: Set(req.user_id),
            member_product_id: Set(req.member_product_id),
            order_id: Set(req.order_id),
            payment_type: Set(req.payment_type),
            amount: Set(Decimal::from_f64(req.amount).unwrap_or_default()),
            pay_method: Set(req.pay_method),
            status: Set(req.status),
            transaction_id: Set(req.transaction_id),
            pay_time: Set(pay_time),
            remark: Set(req.remark),
            create_time: Set(now),
            update_time: Set(now),
            ..Default::default()
        };
        
        let result = model.insert(db).await?;
        
        Ok(PaymentRecordDTO::from(result))
    }

    pub async fn update(db: &DbConn, id: i64, req: PaymentRecordSaveRequest) -> Result<PaymentRecordDTO, DbErr> {
        let mut model: payment_record::ActiveModel = PaymentRecordEntity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("记录不存在".to_string()))?
            .into();
        
        model.user_id = Set(req.user_id);
        model.member_product_id = Set(req.member_product_id);
        model.order_id = Set(req.order_id);
        model.payment_type = Set(req.payment_type);
        model.amount = Set(Decimal::from_f64(req.amount).unwrap_or_default());
        model.pay_method = Set(req.pay_method);
        model.status = Set(req.status);
        model.transaction_id = Set(req.transaction_id);
        
        if let Some(pay_time_str) = &req.pay_time {
            if let Ok(dt) = NaiveDateTime::parse_from_str(pay_time_str, "%Y-%m-%d %H:%M:%S") {
                model.pay_time = Set(Some(DateTime::from_naive_utc_and_offset(dt, Utc)));
            }
        }
        
        model.remark = Set(req.remark);
        model.update_time = Set(Some(Utc::now()));
        
        let result = model.update(db).await?;
        
        Ok(PaymentRecordDTO::from(result))
    }

    pub async fn delete(db: &DbConn, id: i64) -> Result<bool, DbErr> {
        let count = PaymentRecordEntity::delete_by_id(id)
            .exec(db)
            .await?;
        
        Ok(count.rows_affected > 0)
    }

    pub async fn find_by_order_id(db: &DbConn, order_id: &str) -> Result<Option<PaymentRecordDTO>, DbErr> {
        let model = PaymentRecordEntity::find()
            .filter(payment_record::Column::OrderId.eq(order_id))
            .one(db)
            .await?;
        
        Ok(model.map(PaymentRecordDTO::from))
    }

    pub async fn update_status(
        db: &DbConn, 
        order_id: &str, 
        status: i32, 
        transaction_id: Option<&str>, 
        remark: Option<&str>
    ) -> Result<bool, DbErr> {
        let mut model: payment_record::ActiveModel = PaymentRecordEntity::find()
            .filter(payment_record::Column::OrderId.eq(order_id))
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("订单不存在".to_string()))?
            .into();
        
        model.status = Set(Some(status));
        
        if let Some(tx_id) = transaction_id {
            model.transaction_id = Set(Some(tx_id.to_string()));
        }
        
        if status == PAY_STATUS_SUCCESS || status == PAY_STATUS_FAILED {
            model.pay_time = Set(Some(Utc::now()));
        }
        
        if let Some(r) = remark {
            model.remark = Set(Some(r.to_string()));
        }
        
        model.update_time = Set(Some(Utc::now()));
        
        model.update(db).await?;
        
        Ok(true)
    }

    pub async fn update_paid(db: &DbConn, order_id: &str, transaction_id: &str) -> Result<bool, DbErr> {
        Self::update_status(db, order_id, PAY_STATUS_SUCCESS, Some(transaction_id), None).await
    }

    pub async fn update_failed(db: &DbConn, order_id: &str, remark: &str) -> Result<bool, DbErr> {
        Self::update_status(db, order_id, PAY_STATUS_FAILED, None, Some(remark)).await
    }

    pub async fn update_refunded(db: &DbConn, order_id: &str, remark: &str) -> Result<bool, DbErr> {
        Self::update_status(db, order_id, PAY_STATUS_REFUNDED, None, Some(remark)).await
    }

    pub async fn check_purchase_limit(
        db: &DbConn,
        user_id: i64,
        member_product_id: i64,
        purchase_limit_type: Option<i32>,
        purchase_limit_count: Option<i32>,
    ) -> Result<bool, DbErr> {
        let (limit_type, limit_count) = match (purchase_limit_type, purchase_limit_count) {
            (Some(t), Some(c)) => (t, c),
            _ => return Ok(true),
        };

        if limit_type == 0 || limit_count <= 0 {
            return Ok(true);
        }

        let count = match limit_type {
            1 => {
                let now = Utc::now();
                let year_start = match now.date_naive().with_month(1).and_then(|d| d.with_day(1)) {
                    Some(d) => {
                        let time = chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap_or_default();
                        NaiveDateTime::new(d, time)
                    }
                    None => {
                        return Ok(true);
                    }
                };
                let start_time = Utc.from_utc_datetime(&year_start);
                
                PaymentRecordEntity::find()
                    .filter(payment_record::Column::UserId.eq(user_id))
                    .filter(payment_record::Column::MemberProductId.eq(member_product_id))
                    .filter(payment_record::Column::CreateTime.gte(start_time))
                    .count(db)
                    .await?
            }
            2 => {
                PaymentRecordEntity::find()
                    .filter(payment_record::Column::UserId.eq(user_id))
                    .filter(payment_record::Column::MemberProductId.eq(member_product_id))
                    .count(db)
                    .await?
            }
            _ => {
                return Ok(true);
            }
        };

        Ok((count as i64) < (limit_count as i64))
    }
}
