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
use crate::modules::finance::entity::{payment_record, member_fee, refund_record, finance_statistics};
use crate::modules::finance::entity::finance_statistics::Entity as FinanceStatisticsEntity;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use chrono::{DateTime, NaiveDateTime, Utc, Duration, TimeZone};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinanceStatisticsDTO {
    pub id: i64,
    pub stat_date: Option<String>,
    pub stat_type: Option<i32>,
    pub total_income: f64,
    pub success_amount: f64,
    pub refund_amount: f64,
    pub member_fee_amount: f64,
    pub order_count: Option<i64>,
    pub success_count: Option<i64>,
    pub refund_count: Option<i64>,
    pub create_time: Option<String>,
}

impl From<finance_statistics::Model> for FinanceStatisticsDTO {
    fn from(model: finance_statistics::Model) -> Self {
        Self {
            id: model.id,
            stat_date: model.stat_date.map(|dt| dt.format("%Y-%m-%d").to_string()),
            stat_type: model.stat_type,
            total_income: model.total_income.to_f64().unwrap_or_default(),
            success_amount: model.success_amount.to_f64().unwrap_or_default(),
            refund_amount: model.refund_amount.to_f64().unwrap_or_default(),
            member_fee_amount: model.member_fee_amount.to_f64().unwrap_or_default(),
            order_count: model.order_count,
            success_count: model.success_count,
            refund_count: model.refund_count,
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinanceStatisticsQuery {
    pub stat_type: Option<i32>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinanceSummary {
    pub total_income: f64,
    pub success_amount: f64,
    pub refund_amount: f64,
    pub member_fee_amount: f64,
    pub order_count: i64,
    pub success_count: i64,
    pub refund_count: i64,
}

pub struct FinanceStatisticsModel;

impl FinanceStatisticsModel {
    pub async fn find_list(db: &DbConn, query: FinanceStatisticsQuery) -> Result<Vec<FinanceStatisticsDTO>, DbErr> {
        let mut stmt = FinanceStatisticsEntity::find();
        
        if let Some(stat_type) = query.stat_type {
            stmt = stmt.filter(finance_statistics::Column::StatType.eq(stat_type));
        }
        
        if let Some(start_date) = query.start_date {
            if let Ok(dt) = NaiveDateTime::parse_from_str(&format!("{} 00:00:00", start_date), "%Y-%m-%d %H:%M:%S") {
                let dt: DateTime<Utc> = Utc.from_utc_datetime(&dt);
                stmt = stmt.filter(finance_statistics::Column::StatDate.gte(dt));
            }
        }

        if let Some(end_date) = query.end_date {
            if let Ok(dt) = NaiveDateTime::parse_from_str(&format!("{} 23:59:59", end_date), "%Y-%m-%d %H:%M:%S") {
                let dt: DateTime<Utc> = Utc.from_utc_datetime(&dt);
                stmt = stmt.filter(finance_statistics::Column::StatDate.lte(dt));
            }
        }
        
        stmt = stmt.order_by_desc(finance_statistics::Column::StatDate);
        
        let items = stmt.all(db).await?;
        
        let dto_list: Vec<FinanceStatisticsDTO> = items.into_iter().map(FinanceStatisticsDTO::from).collect();
        
        Ok(dto_list)
    }

    pub async fn get_summary(db: &DbConn) -> Result<FinanceSummary, DbErr> {
        let today_start: DateTime<Utc> = Utc::now().date_naive().and_hms_opt(0, 0, 0)
            .map(|dt| Utc.from_utc_datetime(&dt))
            .ok_or(DbErr::Custom("日期格式化失败".to_string()))?;

        let today_end: DateTime<Utc> = Utc::now().date_naive().and_hms_opt(23, 59, 59)
            .map(|dt| Utc.from_utc_datetime(&dt))
            .ok_or(DbErr::Custom("日期格式化失败".to_string()))?;
        
        let total_income_result = payment_record::Entity::find()
            .filter(payment_record::Column::CreateTime.between(today_start, today_end))
            .all(db)
            .await?;
        
        let total_income: Decimal = total_income_result.iter().map(|r| r.amount).sum();
        
        let success_result = payment_record::Entity::find()
            .filter(payment_record::Column::Status.eq(1))
            .filter(payment_record::Column::CreateTime.between(today_start, today_end))
            .all(db)
            .await?;
        
        let success_amount: Decimal = success_result.iter().map(|r| r.amount).sum();
        let success_count = success_result.len() as i64;
        
        let refund_result = refund_record::Entity::find()
            .filter(refund_record::Column::Status.eq(2))
            .filter(refund_record::Column::CreateTime.between(today_start, today_end))
            .all(db)
            .await?;
        
        let refund_amount: Decimal = refund_result.iter().map(|r| r.amount).sum();
        let refund_count = refund_result.len() as i64;
        
        let member_fee_result = member_fee::Entity::find()
            .filter(member_fee::Column::Status.eq(1))
            .filter(member_fee::Column::CreateTime.between(today_start, today_end))
            .all(db)
            .await?;
        
        let member_fee_amount: Decimal = member_fee_result.iter().map(|r| r.amount).sum();
        
        Ok(FinanceSummary {
            total_income: total_income.to_f64().unwrap_or_default(),
            success_amount: success_amount.to_f64().unwrap_or_default(),
            refund_amount: refund_amount.to_f64().unwrap_or_default(),
            member_fee_amount: member_fee_amount.to_f64().unwrap_or_default(),
            order_count: total_income_result.len() as i64,
            success_count,
            refund_count,
        })
    }

    pub async fn generate_daily_statistics(db: &DbConn) -> Result<FinanceStatisticsDTO, DbErr> {
        let yesterday = Utc::now() - Duration::days(1);
        let stat_date = yesterday.date_naive().and_hms_opt(0, 0, 0)
            .map(|dt| Utc.from_utc_datetime(&dt))
            .ok_or(DbErr::Custom("日期格式化失败".to_string()))?;
        
        let day_start = stat_date;
        let day_end = stat_date + Duration::days(1) - Duration::seconds(1);
        
        let total_result = payment_record::Entity::find()
            .filter(payment_record::Column::CreateTime.between(day_start, day_end))
            .all(db)
            .await?;
        
        let total_income: Decimal = total_result.iter().map(|r| r.amount).sum();
        let order_count = total_result.len() as i64;
        
        let success_result = payment_record::Entity::find()
            .filter(payment_record::Column::Status.eq(1))
            .filter(payment_record::Column::CreateTime.between(day_start, day_end))
            .all(db)
            .await?;
        
        let success_amount: Decimal = success_result.iter().map(|r| r.amount).sum();
        let success_count = success_result.len() as i64;
        
        let refund_result = refund_record::Entity::find()
            .filter(refund_record::Column::Status.eq(2))
            .filter(refund_record::Column::CreateTime.between(day_start, day_end))
            .all(db)
            .await?;
        
        let refund_amount: Decimal = refund_result.iter().map(|r| r.amount).sum();
        let refund_count = refund_result.len() as i64;
        
        let member_fee_result = member_fee::Entity::find()
            .filter(member_fee::Column::Status.eq(1))
            .filter(member_fee::Column::CreateTime.between(day_start, day_end))
            .all(db)
            .await?;
        
        let member_fee_amount: Decimal = member_fee_result.iter().map(|r| r.amount).sum();
        
        let model = finance_statistics::ActiveModel {
            stat_date: Set(Some(stat_date.naive_utc())),
            stat_type: Set(Some(1)),
            total_income: Set(total_income),
            success_amount: Set(success_amount),
            refund_amount: Set(refund_amount),
            member_fee_amount: Set(member_fee_amount),
            order_count: Set(Some(order_count)),
            success_count: Set(Some(success_count)),
            refund_count: Set(Some(refund_count)),
            create_time: Set(Some(Utc::now().naive_utc())),
            update_time: Set(Some(Utc::now().naive_utc())),
            ..Default::default()
        };
        
        let result = model.insert(db).await?;
        
        Ok(FinanceStatisticsDTO::from(result))
    }
}
