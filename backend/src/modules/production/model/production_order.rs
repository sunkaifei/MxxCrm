//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::modules::production::entity::production_order::{self, Entity as ProductionOrder};
use sea_orm::prelude::{Date, DateTime, Decimal};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, UpdateResult,
};
use serde::{Deserialize, Serialize};

// ==================== 状态常量 ====================

pub mod production_order_status {
    pub const DRAFT: i32 = 0;
    pub const RELEASED: i32 = 1;
    pub const IN_PROGRESS: i32 = 2;
    pub const COMPLETED: i32 = 3;
    pub const INBOUNDED: i32 = 4;
    pub const CLOSED: i32 = 5;
    pub const CANCELLED: i32 = 6;
}

// ==================== DTO ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductionOrderSaveRequest {
    pub mo_no: Option<String>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub quantity: Option<Decimal>,
    pub completed_quantity: Option<Decimal>,
    pub plan_start_date: Option<Date>,
    pub plan_complete_date: Option<Date>,
    pub actual_complete_date: Option<Date>,
    pub source_type: Option<String>,
    pub source_id: Option<i64>,
    pub source_no: Option<String>,
    pub workshop_id: Option<i64>,
    pub production_lead_time: Option<i32>,
    pub status: Option<i32>,
    pub cost_amount: Option<Decimal>,
    pub remark: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductionOrderListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub keywords: Option<String>,
    pub status: Option<i32>,
    pub product_id: Option<i64>,
}

// ==================== VO ====================

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductionOrderDetailVO {
    pub id: Option<i64>,
    pub mo_no: Option<String>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub quantity: Option<Decimal>,
    pub completed_quantity: Option<Decimal>,
    pub plan_start_date: Option<Date>,
    pub plan_complete_date: Option<Date>,
    pub actual_complete_date: Option<Date>,
    pub source_type: Option<String>,
    pub source_id: Option<i64>,
    pub source_no: Option<String>,
    pub workshop_id: Option<i64>,
    pub production_lead_time: Option<i32>,
    pub status: Option<i32>,
    pub cost_amount: Option<Decimal>,
    pub remark: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

impl From<production_order::Model> for ProductionOrderDetailVO {
    fn from(model: production_order::Model) -> Self {
        ProductionOrderDetailVO {
            id: Some(model.id),
            mo_no: model.mo_no,
            product_id: model.product_id,
            product_name: model.product_name,
            quantity: model.quantity,
            completed_quantity: model.completed_quantity,
            plan_start_date: model.plan_start_date,
            plan_complete_date: model.plan_complete_date,
            actual_complete_date: model.actual_complete_date,
            source_type: model.source_type,
            source_id: model.source_id,
            source_no: model.source_no,
            workshop_id: model.workshop_id,
            production_lead_time: model.production_lead_time,
            status: model.status,
            cost_amount: model.cost_amount,
            remark: model.remark,
            created_by: model.created_by,
            updated_by: model.updated_by,
            create_time: model.create_time,
            update_time: model.update_time,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductionOrderListVO {
    pub id: Option<i64>,
    pub mo_no: Option<String>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub quantity: Option<Decimal>,
    pub completed_quantity: Option<Decimal>,
    pub plan_start_date: Option<Date>,
    pub plan_complete_date: Option<Date>,
    pub status: Option<i32>,
    pub create_time: Option<DateTime>,
}

impl From<production_order::Model> for ProductionOrderListVO {
    fn from(model: production_order::Model) -> Self {
        ProductionOrderListVO {
            id: Some(model.id),
            mo_no: model.mo_no,
            product_id: model.product_id,
            product_name: model.product_name,
            quantity: model.quantity,
            completed_quantity: model.completed_quantity,
            plan_start_date: model.plan_start_date,
            plan_complete_date: model.plan_complete_date,
            status: model.status,
            create_time: model.create_time,
        }
    }
}

// ==================== Model ====================

pub struct ProductionOrderModel;

impl ProductionOrderModel {
    pub async fn insert<C: ConnectionTrait>(db: &C, req: &ProductionOrderSaveRequest) -> std::result::Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let mo_no = req.mo_no.clone().unwrap_or_else(|| {
            format!("MO{}", now.and_utc().timestamp_millis())
        });
        let payload = production_order::ActiveModel {
            mo_no: Set(Some(mo_no)),
            product_id: Set(req.product_id),
            product_name: Set(req.product_name.clone()),
            quantity: Set(req.quantity.clone()),
            completed_quantity: Set(req.completed_quantity.clone()),
            plan_start_date: Set(req.plan_start_date),
            plan_complete_date: Set(req.plan_complete_date),
            actual_complete_date: Set(req.actual_complete_date),
            source_type: Set(req.source_type.clone()),
            source_id: Set(req.source_id),
            source_no: Set(req.source_no.clone()),
            workshop_id: Set(req.workshop_id),
            production_lead_time: Set(req.production_lead_time),
            status: Set(req.status.or(Some(production_order_status::DRAFT))),
            cost_amount: Set(req.cost_amount.clone()),
            remark: Set(req.remark.clone()),
            deleted: Set(Some(0)),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            ..Default::default()
        };

        ProductionOrder::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    pub async fn update_by_id<C: ConnectionTrait>(db: &C, id: i64, req: &ProductionOrderSaveRequest) -> std::result::Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = production_order::ActiveModel {
            mo_no: Set(req.mo_no.clone()),
            product_id: Set(req.product_id),
            product_name: Set(req.product_name.clone()),
            quantity: Set(req.quantity.clone()),
            completed_quantity: Set(req.completed_quantity.clone()),
            plan_start_date: Set(req.plan_start_date),
            plan_complete_date: Set(req.plan_complete_date),
            actual_complete_date: Set(req.actual_complete_date),
            source_type: Set(req.source_type.clone()),
            source_id: Set(req.source_id),
            source_no: Set(req.source_no.clone()),
            workshop_id: Set(req.workshop_id),
            production_lead_time: Set(req.production_lead_time),
            status: Set(req.status),
            cost_amount: Set(req.cost_amount.clone()),
            remark: Set(req.remark.clone()),
            update_time: Set(Some(now)),
            ..Default::default()
        };

        let update_result: UpdateResult = ProductionOrder::update_many()
            .set(payload)
            .filter(production_order::Column::Id.eq(id))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    pub async fn batch_delete_by_ids(db: &DatabaseConnection, ids: &Vec<i64>) -> std::result::Result<i64, DbErr> {
        ProductionOrder::update_many()
            .set(production_order::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(production_order::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn find_by_id(db: &DatabaseConnection, id: i64) -> std::result::Result<Option<production_order::Model>, DbErr> {
        ProductionOrder::find_by_id(id)
            .filter(production_order::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    pub async fn select_in_page(
        db: &DatabaseConnection,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        status: Option<i32>,
        product_id: Option<i64>,
    ) -> std::result::Result<(Vec<production_order::Model>, i64), DbErr> {
        let mut query = ProductionOrder::find()
            .filter(production_order::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            query = query.filter(
                sea_orm::Condition::any()
                    .add(production_order::Column::MoNo.contains(k.clone()))
                    .add(production_order::Column::ProductName.contains(k))
            );
        }
        if let Some(s) = status {
            query = query.filter(production_order::Column::Status.eq(s));
        }
        if let Some(p) = product_id {
            query = query.filter(production_order::Column::ProductId.eq(p));
        }

        let paginator = query.order_by_desc(production_order::Column::CreateTime).paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }

    pub async fn select_count(
        db: &DatabaseConnection,
        keywords: Option<String>,
        status: Option<i32>,
        product_id: Option<i64>,
    ) -> std::result::Result<i64, DbErr> {
        let mut query = ProductionOrder::find()
            .filter(production_order::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            query = query.filter(
                sea_orm::Condition::any()
                    .add(production_order::Column::MoNo.contains(k.clone()))
                    .add(production_order::Column::ProductName.contains(k))
            );
        }
        if let Some(s) = status {
            query = query.filter(production_order::Column::Status.eq(s));
        }
        if let Some(p) = product_id {
            query = query.filter(production_order::Column::ProductId.eq(p));
        }

        query.count(db).await.map(|c| c as i64)
    }

    pub async fn update_status<C: ConnectionTrait>(db: &C, id: i64, status: i32) -> std::result::Result<(), DbErr> {
        let now = chrono::Local::now().naive_local();
        ProductionOrder::update_many()
            .set(production_order::ActiveModel {
                status: Set(Some(status)),
                update_time: Set(Some(now)),
                ..Default::default()
            })
            .filter(production_order::Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(())
    }
}