//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::modules::purchase::entity::purchase_stock_plan::{self, Entity as PurchaseStockPlan};
use sea_orm::prelude::{Date, DateTime, Decimal};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, UpdateResult,
};
use serde::{Deserialize, Serialize};

// ==================== 状态常量 ====================

pub mod stock_plan_status {
    pub const DRAFT: i32 = 0;
    pub const GENERATED: i32 = 1;
    pub const CLOSED: i32 = 2;
    pub const DELAYED: i32 = 3;
    pub const CANCELLED: i32 = 4;
}

// ==================== DTO ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StockPlanSaveRequest {
    pub plan_no: Option<String>,
    pub product_id: Option<i64>,
    pub plan_date: Option<Date>,
    pub demand_quantity: Option<Decimal>,
    pub demand_source: Option<String>,
    pub source_type: Option<String>,
    pub source_id: Option<i64>,
    pub available_quantity: Option<Decimal>,
    pub net_demand: Option<Decimal>,
    pub safety_stock: Option<Decimal>,
    pub suggested_order_date: Option<Date>,
    pub suggested_quantity: Option<Decimal>,
    pub supplier_id: Option<i64>,
    pub lead_time_days: Option<i32>,
    pub status: Option<i32>,
    pub actual_pr_id: Option<i64>,
    pub remark: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StockPlanListQuery {
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
pub struct StockPlanDetailVO {
    pub id: Option<i64>,
    pub plan_no: Option<String>,
    pub product_id: Option<i64>,
    pub plan_date: Option<Date>,
    pub demand_quantity: Option<Decimal>,
    pub demand_source: Option<String>,
    pub available_quantity: Option<Decimal>,
    pub net_demand: Option<Decimal>,
    pub safety_stock: Option<Decimal>,
    pub suggested_order_date: Option<Date>,
    pub suggested_quantity: Option<Decimal>,
    pub supplier_id: Option<i64>,
    pub lead_time_days: Option<i32>,
    pub status: Option<i32>,
    pub actual_pr_id: Option<i64>,
    pub remark: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

impl From<purchase_stock_plan::Model> for StockPlanDetailVO {
    fn from(model: purchase_stock_plan::Model) -> Self {
        StockPlanDetailVO {
            id: Some(model.id),
            plan_no: model.plan_no,
            product_id: model.product_id,
            plan_date: model.plan_date,
            demand_quantity: model.demand_quantity,
            demand_source: model.demand_source,
            available_quantity: model.available_quantity,
            net_demand: model.net_demand,
            safety_stock: model.safety_stock,
            suggested_order_date: model.suggested_order_date,
            suggested_quantity: model.suggested_quantity,
            supplier_id: model.supplier_id,
            lead_time_days: model.lead_time_days,
            status: model.status,
            actual_pr_id: model.actual_pr_id,
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
pub struct StockPlanListVO {
    pub id: Option<i64>,
    pub plan_no: Option<String>,
    pub product_id: Option<i64>,
    pub plan_date: Option<Date>,
    pub demand_quantity: Option<Decimal>,
    pub net_demand: Option<Decimal>,
    pub safety_stock: Option<Decimal>,
    pub suggested_order_date: Option<Date>,
    pub status: Option<i32>,
    pub create_time: Option<DateTime>,
}

impl From<purchase_stock_plan::Model> for StockPlanListVO {
    fn from(model: purchase_stock_plan::Model) -> Self {
        StockPlanListVO {
            id: Some(model.id),
            plan_no: model.plan_no,
            product_id: model.product_id,
            plan_date: model.plan_date,
            demand_quantity: model.demand_quantity,
            net_demand: model.net_demand,
            safety_stock: model.safety_stock,
            suggested_order_date: model.suggested_order_date,
            status: model.status,
            create_time: model.create_time,
        }
    }
}

// ==================== Model ====================

pub struct StockPlanModel;

impl StockPlanModel {
    pub async fn insert<C: ConnectionTrait>(db: &C, req: &StockPlanSaveRequest) -> std::result::Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let plan_no = req.plan_no.clone().unwrap_or_else(|| {
            format!("SP{}", now.timestamp_millis())
        });
        let payload = purchase_stock_plan::ActiveModel {
            plan_no: Set(Some(plan_no)),
            product_id: Set(req.product_id),
            plan_date: Set(req.plan_date),
            demand_quantity: Set(req.demand_quantity.clone()),
            demand_source: Set(req.demand_source.clone()),
            source_type: Set(req.source_type.clone()),
            source_id: Set(req.source_id),
            available_quantity: Set(req.available_quantity.clone()),
            net_demand: Set(req.net_demand.clone()),
            safety_stock: Set(req.safety_stock.clone()),
            suggested_order_date: Set(req.suggested_order_date),
            suggested_quantity: Set(req.suggested_quantity.clone()),
            supplier_id: Set(req.supplier_id),
            lead_time_days: Set(req.lead_time_days),
            status: Set(req.status.or(Some(stock_plan_status::DRAFT))),
            actual_pr_id: Set(req.actual_pr_id),
            remark: Set(req.remark.clone()),
            deleted: Set(Some(0)),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            ..Default::default()
        };

        PurchaseStockPlan::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    pub async fn update_by_id<C: ConnectionTrait>(db: &C, id: i64, req: &StockPlanSaveRequest) -> std::result::Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = purchase_stock_plan::ActiveModel {
            plan_no: Set(req.plan_no.clone()),
            product_id: Set(req.product_id),
            plan_date: Set(req.plan_date),
            demand_quantity: Set(req.demand_quantity.clone()),
            demand_source: Set(req.demand_source.clone()),
            source_type: Set(req.source_type.clone()),
            source_id: Set(req.source_id),
            available_quantity: Set(req.available_quantity.clone()),
            net_demand: Set(req.net_demand.clone()),
            safety_stock: Set(req.safety_stock.clone()),
            suggested_order_date: Set(req.suggested_order_date),
            suggested_quantity: Set(req.suggested_quantity.clone()),
            supplier_id: Set(req.supplier_id),
            lead_time_days: Set(req.lead_time_days),
            status: Set(req.status),
            actual_pr_id: Set(req.actual_pr_id),
            remark: Set(req.remark.clone()),
            update_time: Set(Some(now)),
            ..Default::default()
        };

        let update_result: UpdateResult = PurchaseStockPlan::update_many()
            .set(payload)
            .filter(purchase_stock_plan::Column::Id.eq(id))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    pub async fn batch_delete_by_ids(db: &DatabaseConnection, ids: &Vec<i64>) -> std::result::Result<i64, DbErr> {
        PurchaseStockPlan::update_many()
            .set(purchase_stock_plan::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(purchase_stock_plan::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn find_by_id(db: &DatabaseConnection, id: i64) -> std::result::Result<Option<purchase_stock_plan::Model>, DbErr> {
        PurchaseStockPlan::find_by_id(id)
            .filter(purchase_stock_plan::Column::Deleted.eq(0))
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
    ) -> std::result::Result<(Vec<purchase_stock_plan::Model>, i64), DbErr> {
        let mut query = PurchaseStockPlan::find()
            .filter(purchase_stock_plan::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            query = query.filter(purchase_stock_plan::Column::PlanNo.contains(k));
        }
        if let Some(s) = status {
            query = query.filter(purchase_stock_plan::Column::Status.eq(s));
        }
        if let Some(p) = product_id {
            query = query.filter(purchase_stock_plan::Column::ProductId.eq(p));
        }

        let paginator = query.order_by_desc(purchase_stock_plan::Column::CreateTime).paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }

    pub async fn select_count(
        db: &DatabaseConnection,
        keywords: Option<String>,
        status: Option<i32>,
        product_id: Option<i64>,
    ) -> std::result::Result<i64, DbErr> {
        let mut query = PurchaseStockPlan::find()
            .filter(purchase_stock_plan::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            query = query.filter(purchase_stock_plan::Column::PlanNo.contains(k));
        }
        if let Some(s) = status {
            query = query.filter(purchase_stock_plan::Column::Status.eq(s));
        }
        if let Some(p) = product_id {
            query = query.filter(purchase_stock_plan::Column::ProductId.eq(p));
        }

        query.count(db).await.map(|c| c as i64)
    }

    pub async fn update_status<C: ConnectionTrait>(db: &C, id: i64, status: i32) -> std::result::Result<(), DbErr> {
        let now = chrono::Local::now().naive_local();
        PurchaseStockPlan::update_many()
            .set(purchase_stock_plan::ActiveModel {
                status: Set(Some(status)),
                update_time: Set(Some(now)),
                ..Default::default()
            })
            .filter(purchase_stock_plan::Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(())
    }

    /// 回填 actual_pr_id 并更新状态
    pub async fn update_actual_pr_id_and_status<C: ConnectionTrait>(db: &C, id: i64, actual_pr_id: i64, status: i32) -> std::result::Result<(), DbErr> {
        let now = chrono::Local::now().naive_local();
        PurchaseStockPlan::update_many()
            .set(purchase_stock_plan::ActiveModel {
                actual_pr_id: Set(Some(actual_pr_id)),
                status: Set(Some(status)),
                update_time: Set(Some(now)),
                ..Default::default()
            })
            .filter(purchase_stock_plan::Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(())
    }

    /// 计算净需求 = 需求数量 - 可用数量
    pub fn calculate_net_demand(demand: Option<Decimal>, available: Option<Decimal>) -> Option<Decimal> {
        let d = demand.unwrap_or(Decimal::ZERO);
        let a = available.unwrap_or(Decimal::ZERO);
        let net = d - a;
        if net > Decimal::ZERO { Some(net) } else { Some(Decimal::ZERO) }
    }
}