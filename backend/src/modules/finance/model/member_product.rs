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
use log;
use crate::modules::finance::entity::member_product;
use crate::modules::finance::entity::member_product::Entity as MemberProductEntity;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use chrono::{DateTime, NaiveDateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberProductDTO {
    pub id: i64,
    pub product_id: String,
    pub product_name: String,
    pub product_type: String,
    pub duration_type: i32,
    pub duration_value: i32,
    /// 购买限制类型: 0=无限制, 1=自然年内限制, 2=终身限制
    pub purchase_limit_type: Option<i32>,
    /// 购买限制次数(在限制周期内最多可购买的次数)
    pub purchase_limit_count: Option<i32>,
    pub price: f64,
    pub original_price: Option<f64>,
    pub discount: Option<f64>,
    pub member_type: i32,
    pub status: i32,
    pub sort_order: i32,
    pub description: Option<String>,
    pub features: Option<String>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
}

impl From<member_product::Model> for MemberProductDTO {
    fn from(model: member_product::Model) -> Self {
        Self {
            id: model.id,
            product_id: model.product_id,
            product_name: model.product_name,
            product_type: model.product_type,
            duration_type: model.duration_type,
            duration_value: model.duration_value,
            purchase_limit_type: model.purchase_limit_type,
            purchase_limit_count: model.purchase_limit_count,
            price: model.price.to_f64().unwrap_or_default(),
            original_price: model.original_price.and_then(|d| d.to_f64()),
            discount: model.discount.and_then(|d| d.to_f64()),
            member_type: model.member_type,
            status: model.status,
            sort_order: model.sort_order,
            description: model.description,
            features: model.features.map(|v| serde_json::to_string(&v).unwrap_or_default()),
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: model.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberProductSaveRequest {
    pub product_id: String,
    pub product_name: String,
    pub product_type: String,
    pub duration_type: i32,
    pub duration_value: i32,
    /// 购买限制类型: 0=无限制, 1=自然年内限制, 2=终身限制
    pub purchase_limit_type: Option<i32>,
    /// 购买限制次数(在限制周期内最多可购买的次数)
    pub purchase_limit_count: Option<i32>,
    pub price: f64,
    pub original_price: Option<f64>,
    pub discount: Option<f64>,
    pub member_type: i32,
    pub status: Option<i32>,
    pub sort_order: Option<i32>,
    pub description: Option<String>,
    pub features: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberProductQuery {
    pub product_type: Option<String>,
    pub member_type: Option<i32>,
    pub status: Option<i32>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub struct MemberProductModel;

impl MemberProductModel {
    pub async fn find_list(db: &DbConn, query: MemberProductQuery) -> Result<(Vec<MemberProductDTO>, i64), DbErr> {
        let mut stmt = MemberProductEntity::find();

        if let Some(product_type) = query.product_type {
            stmt = stmt.filter(member_product::Column::ProductType.eq(product_type));
        }
        if let Some(member_type) = query.member_type {
            stmt = stmt.filter(member_product::Column::MemberType.eq(member_type));
        }
        if let Some(status) = query.status {
            stmt = stmt.filter(member_product::Column::Status.eq(status));
        }

        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(10);

        let paginator = stmt
            .order_by_asc(member_product::Column::SortOrder)
            .paginate(db, page_size as u64);
        let total = paginator.num_items().await? as i64;
        let items = paginator.fetch_page((page - 1) as u64).await?;

        let dto_list = items.into_iter().map(MemberProductDTO::from).collect();

        Ok((dto_list, total))
    }

    pub async fn find_by_product_id(db: &DbConn, product_id: &str) -> Result<Option<MemberProductDTO>, DbErr> {
        log::debug!("[MemberProductModel] 开始查询商品, product_id={}", product_id);
        
        let model = MemberProductEntity::find()
            .filter(member_product::Column::ProductId.eq(product_id))
            .filter(member_product::Column::Status.eq(1))
            .one(db)
            .await?;
        
        match &model {
            Some(p) => log::debug!("[MemberProductModel] 查询到商品, id={}, name={}", p.id, p.product_name),
            None => log::debug!("[MemberProductModel] 未查询到商品: product_id={}", product_id),
        }
        
        Ok(model.map(MemberProductDTO::from))
    }

    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<MemberProductDTO>, DbErr> {
        let model = MemberProductEntity::find_by_id(id).one(db).await?;
        Ok(model.map(MemberProductDTO::from))
    }

    pub async fn insert(db: &DbConn, req: MemberProductSaveRequest) -> Result<MemberProductDTO, DbErr> {
        let now = Some(Utc::now());
        
        let model = member_product::ActiveModel {
            product_id: Set(req.product_id),
            product_name: Set(req.product_name),
            product_type: Set(req.product_type),
            duration_type: Set(req.duration_type),
            duration_value: Set(req.duration_value),
            purchase_limit_type: Set(req.purchase_limit_type),
            purchase_limit_count: Set(req.purchase_limit_count),
            price: Set(Decimal::from_f64(req.price).unwrap_or_default()),
            original_price: Set(req.original_price.map(|d| Decimal::from_f64(d).unwrap_or_default())),
            discount: Set(req.discount.map(|d| Decimal::from_f64(d).unwrap_or_default())),
            member_type: Set(req.member_type),
            status: Set(req.status.unwrap_or(1)),
            sort_order: Set(req.sort_order.unwrap_or(0)),
            description: Set(req.description),
            features: Set(req.features.map(|f| serde_json::from_str(&f).unwrap_or_default())),
            create_time: Set(now),
            update_time: Set(now),
            ..Default::default()
        };
        
        let result = model.insert(db).await?;
        
        Ok(MemberProductDTO::from(result))
    }

    pub async fn update(db: &DbConn, id: i64, req: MemberProductSaveRequest) -> Result<MemberProductDTO, DbErr> {
        let mut model: member_product::ActiveModel = MemberProductEntity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("商品不存在".to_string()))?
            .into();
        
        model.product_id = Set(req.product_id);
        model.product_name = Set(req.product_name);
        model.product_type = Set(req.product_type);
        model.duration_type = Set(req.duration_type);
        model.duration_value = Set(req.duration_value);
        model.purchase_limit_type = Set(req.purchase_limit_type);
        model.purchase_limit_count = Set(req.purchase_limit_count);
        model.price = Set(Decimal::from_f64(req.price).unwrap_or_default());
        model.original_price = Set(req.original_price.map(|d| Decimal::from_f64(d).unwrap_or_default()));
        model.discount = Set(req.discount.map(|d| Decimal::from_f64(d).unwrap_or_default()));
        model.member_type = Set(req.member_type);
        model.status = Set(req.status.unwrap_or(1));
        model.sort_order = Set(req.sort_order.unwrap_or(0));
        model.description = Set(req.description);
        model.features = Set(req.features.map(|f| serde_json::from_str(&f).unwrap_or_default()));
        model.update_time = Set(Some(Utc::now()));
        
        let result = model.update(db).await?;
        
        Ok(MemberProductDTO::from(result))
    }

    pub async fn delete(db: &DbConn, id: i64) -> Result<(), DbErr> {
        MemberProductEntity::delete_by_id(id).exec(db).await?;
        Ok(())
    }

    pub fn get_valid_end_time(start_time: &DateTime<Utc>, duration_type: i32, duration_value: i32) -> DateTime<Utc> {
        match duration_type {
            1 => *start_time + chrono::Duration::days(duration_value as i64 * 30),
            2 => *start_time + chrono::Duration::days(duration_value as i64 * 90),
            3 => *start_time + chrono::Duration::days(duration_value as i64 * 365),
            4 => DateTime::<Utc>::MAX_UTC,
            _ => *start_time + chrono::Duration::days(30),
        }
    }
}
