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
use crate::modules::finance::entity::member_fee;
use crate::modules::finance::entity::member_fee::Entity as MemberFeeEntity;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use chrono::{DateTime, NaiveDateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberFeeDTO {
    pub id: i64,
    pub user_id: i64,
    pub member_type: Option<i32>,
    pub amount: f64,
    pub valid_start_time: Option<String>,
    pub valid_end_time: Option<String>,
    pub status: Option<i32>,
    pub payment_record_id: Option<i64>,
    pub remark: Option<String>,
    pub create_time: Option<String>,
}

impl From<member_fee::Model> for MemberFeeDTO {
    fn from(model: member_fee::Model) -> Self {
        Self {
            id: model.id,
            user_id: model.user_id,
            member_type: model.member_type,
            amount: model.amount.to_f64().unwrap_or_default(),
            valid_start_time: model.valid_start_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            valid_end_time: model.valid_end_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            status: model.status,
            payment_record_id: model.payment_record_id,
            remark: model.remark,
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberFeeSaveRequest {
    pub user_id: i64,
    pub member_type: Option<i32>,
    pub amount: f64,
    pub valid_start_time: Option<String>,
    pub valid_end_time: Option<String>,
    pub status: Option<i32>,
    pub payment_record_id: Option<i64>,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberFeeQuery {
    pub user_id: Option<i64>,
    pub member_type: Option<i32>,
    pub status: Option<i32>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub struct MemberFeeModel;

impl MemberFeeModel {
    pub async fn find_list(db: &DbConn, query: MemberFeeQuery) -> Result<(Vec<MemberFeeDTO>, i64), DbErr> {
        let mut stmt = MemberFeeEntity::find();
        
        if let Some(user_id) = query.user_id {
            stmt = stmt.filter(member_fee::Column::UserId.eq(user_id));
        }
        
        if let Some(member_type) = query.member_type {
            stmt = stmt.filter(member_fee::Column::MemberType.eq(member_type));
        }
        
        if let Some(status) = query.status {
            stmt = stmt.filter(member_fee::Column::Status.eq(status));
        }
        
        stmt = stmt.order_by_desc(member_fee::Column::CreateTime);
        
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        
        let paginator = stmt.paginate(db, page_size as u64);
        let total = paginator.num_items().await? as i64;
        let items = paginator.fetch_page((page - 1) as u64).await?;
        
        let dto_list: Vec<MemberFeeDTO> = items.into_iter().map(MemberFeeDTO::from).collect();
        
        Ok((dto_list, total))
    }

    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<MemberFeeDTO>, DbErr> {
        let model = MemberFeeEntity::find_by_id(id)
            .one(db)
            .await?;
        
        Ok(model.map(MemberFeeDTO::from))
    }

    pub async fn find_by_user_id(db: &DbConn, user_id: i64) -> Result<Option<MemberFeeDTO>, DbErr> {
        let model = MemberFeeEntity::find()
            .filter(member_fee::Column::UserId.eq(user_id))
            .filter(member_fee::Column::Status.eq(1))
            .order_by_desc(member_fee::Column::CreateTime)
            .one(db)
            .await?;
        
        Ok(model.map(MemberFeeDTO::from))
    }

    pub async fn insert(db: &DbConn, req: MemberFeeSaveRequest) -> Result<MemberFeeDTO, DbErr> {
        let now = Some(Utc::now().naive_utc());

        let valid_start_time = req.valid_start_time.as_ref()
            .and_then(|s| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok());

        let valid_end_time = req.valid_end_time.as_ref()
            .and_then(|s| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok());
        
        let model = member_fee::ActiveModel {
            user_id: Set(req.user_id),
            member_type: Set(req.member_type),
            amount: Set(Decimal::from_f64(req.amount).unwrap_or_default()),
            valid_start_time: Set(valid_start_time),
            valid_end_time: Set(valid_end_time),
            status: Set(req.status),
            payment_record_id: Set(req.payment_record_id),
            remark: Set(req.remark),
            create_time: Set(now),
            update_time: Set(now),
            ..Default::default()
        };
        
        let result = model.insert(db).await?;
        
        Ok(MemberFeeDTO::from(result))
    }

    pub async fn update(db: &DbConn, id: i64, req: MemberFeeSaveRequest) -> Result<MemberFeeDTO, DbErr> {
        let mut model: member_fee::ActiveModel = MemberFeeEntity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("记录不存在".to_string()))?
            .into();
        
        model.user_id = Set(req.user_id);
        model.member_type = Set(req.member_type);
        model.amount = Set(Decimal::from_f64(req.amount).unwrap_or_default());
        
        if let Some(start_time_str) = &req.valid_start_time {
            if let Ok(dt) = NaiveDateTime::parse_from_str(start_time_str, "%Y-%m-%d %H:%M:%S") {
                model.valid_start_time = Set(Some(dt));
            }
        }

        if let Some(end_time_str) = &req.valid_end_time {
            if let Ok(dt) = NaiveDateTime::parse_from_str(end_time_str, "%Y-%m-%d %H:%M:%S") {
                model.valid_end_time = Set(Some(dt));
            }
        }

        model.status = Set(req.status);
        model.payment_record_id = Set(req.payment_record_id);
        model.remark = Set(req.remark);
        model.update_time = Set(Some(Utc::now().naive_utc()));
        
        let result = model.update(db).await?;
        
        Ok(MemberFeeDTO::from(result))
    }

    pub async fn delete(db: &DbConn, id: i64) -> Result<bool, DbErr> {
        let count = MemberFeeEntity::delete_by_id(id)
            .exec(db)
            .await?;
        
        Ok(count.rows_affected > 0)
    }
}
