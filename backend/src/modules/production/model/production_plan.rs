//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::modules::production::entity::production_plan::{self, Entity as ProductionPlan};
use sea_orm::prelude::{Date, DateTime, Decimal};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, UpdateResult,
};
use serde::{Deserialize, Serialize};

// ==================== 状态常量 ====================

pub mod production_plan_status {
    pub const PENDING: i32 = 0;
    pub const GENERATED: i32 = 1;
    pub const CLOSED: i32 = 2;
    pub const CANCELLED: i32 = 3;
}

// ==================== DTO ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductionPlanSaveRequest {
    pub plan_no: Option<String>,
    pub product_id: Option<i64>,
    pub plan_date: Option<Date>,
    pub demand_quantity: Option<Decimal>,
    pub demand_source: Option<String>,
    pub available_quantity: Option<Decimal>,
    pub net_demand: Option<Decimal>,
    pub suggested_start_date: Option<Date>,
    pub suggested_quantity: Option<Decimal>,
    pub status: Option<i32>,
    pub actual_mo_id: Option<i64>,
    pub remark: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductionPlanListQuery {
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
pub struct ProductionPlanDetailVO {
    pub id: Option<i64>,
    pub plan_no: Option<String>,
    pub product_id: Option<i64>,
    pub plan_date: Option<Date>,
    pub demand_quantity: Option<Decimal>,
    pub demand_source: Option<String>,
    pub available_quantity: Option<Decimal>,
    pub net_demand: Option<Decimal>,
    pub suggested_start_date: Option<Date>,
    pub suggested_quantity: Option<Decimal>,
    pub status: Option<i32>,
    pub actual_mo_id: Option<i64>,
    pub remark: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

impl From<production_plan::Model> for ProductionPlanDetailVO {
    fn from(model: production_plan::Model) -> Self {
        ProductionPlanDetailVO {
            id: Some(model.id),
            plan_no: model.plan_no,
            product_id: model.product_id,
            plan_date: model.plan_date,
            demand_quantity: model.demand_quantity,
            demand_source: model.demand_source,
            available_quantity: model.available_quantity,
            net_demand: model.net_demand,
            suggested_start_date: model.suggested_start_date,
            suggested_quantity: model.suggested_quantity,
            status: model.status,
            actual_mo_id: model.actual_mo_id,
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
pub struct ProductionPlanListVO {
    pub id: Option<i64>,
    pub plan_no: Option<String>,
    pub product_id: Option<i64>,
    pub plan_date: Option<Date>,
    pub demand_quantity: Option<Decimal>,
    pub net_demand: Option<Decimal>,
    pub suggested_start_date: Option<Date>,
    pub status: Option<i32>,
    pub create_time: Option<DateTime>,
}

impl From<production_plan::Model> for ProductionPlanListVO {
    fn from(model: production_plan::Model) -> Self {
        ProductionPlanListVO {
            id: Some(model.id),
            plan_no: model.plan_no,
            product_id: model.product_id,
            plan_date: model.plan_date,
            demand_quantity: model.demand_quantity,
            net_demand: model.net_demand,
            suggested_start_date: model.suggested_start_date,
            status: model.status,
            create_time: model.create_time,
        }
    }
}

// ==================== Model ====================

pub struct ProductionPlanModel;

impl ProductionPlanModel {
    pub async fn insert<C: ConnectionTrait>(db: &C, req: &ProductionPlanSaveRequest) -> std::result::Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let plan_no = req.plan_no.clone().unwrap_or_else(|| {
            format!("PP{}", now.and_utc().timestamp_millis())
        });
        let payload = production_plan::ActiveModel {
            plan_no: Set(Some(plan_no)),
            product_id: Set(req.product_id),
            plan_date: Set(req.plan_date),
            demand_quantity: Set(req.demand_quantity.clone()),
            demand_source: Set(req.demand_source.clone()),
            available_quantity: Set(req.available_quantity.clone()),
            net_demand: Set(req.net_demand.clone()),
            suggested_start_date: Set(req.suggested_start_date),
            suggested_quantity: Set(req.suggested_quantity.clone()),
            status: Set(req.status.or(Some(production_plan_status::PENDING))),
            actual_mo_id: Set(req.actual_mo_id),
            remark: Set(req.remark.clone()),
            deleted: Set(Some(0)),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            ..Default::default()
        };

        ProductionPlan::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    pub async fn update_by_id<C: ConnectionTrait>(db: &C, id: i64, req: &ProductionPlanSaveRequest) -> std::result::Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = production_plan::ActiveModel {
            plan_no: Set(req.plan_no.clone()),
            product_id: Set(req.product_id),
            plan_date: Set(req.plan_date),
            demand_quantity: Set(req.demand_quantity.clone()),
            demand_source: Set(req.demand_source.clone()),
            available_quantity: Set(req.available_quantity.clone()),
            net_demand: Set(req.net_demand.clone()),
            suggested_start_date: Set(req.suggested_start_date),
            suggested_quantity: Set(req.suggested_quantity.clone()),
            status: Set(req.status),
            actual_mo_id: Set(req.actual_mo_id),
            remark: Set(req.remark.clone()),
            update_time: Set(Some(now)),
            ..Default::default()
        };

        let update_result: UpdateResult = ProductionPlan::update_many()
            .set(payload)
            .filter(production_plan::Column::Id.eq(id))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    pub async fn batch_delete_by_ids(db: &DatabaseConnection, ids: &Vec<i64>) -> std::result::Result<i64, DbErr> {
        ProductionPlan::update_many()
            .set(production_plan::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(production_plan::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn find_by_id(db: &DatabaseConnection, id: i64) -> std::result::Result<Option<production_plan::Model>, DbErr> {
        ProductionPlan::find_by_id(id)
            .filter(production_plan::Column::Deleted.eq(0))
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
    ) -> std::result::Result<(Vec<production_plan::Model>, i64), DbErr> {
        let mut query = ProductionPlan::find()
            .filter(production_plan::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            query = query.filter(production_plan::Column::PlanNo.contains(k));
        }
        if let Some(s) = status {
            query = query.filter(production_plan::Column::Status.eq(s));
        }
        if let Some(p) = product_id {
            query = query.filter(production_plan::Column::ProductId.eq(p));
        }

        let paginator = query.order_by_desc(production_plan::Column::CreateTime).paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }

    pub async fn select_count(
        db: &DatabaseConnection,
        keywords: Option<String>,
        status: Option<i32>,
        product_id: Option<i64>,
    ) -> std::result::Result<i64, DbErr> {
        let mut query = ProductionPlan::find()
            .filter(production_plan::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            query = query.filter(production_plan::Column::PlanNo.contains(k));
        }
        if let Some(s) = status {
            query = query.filter(production_plan::Column::Status.eq(s));
        }
        if let Some(p) = product_id {
            query = query.filter(production_plan::Column::ProductId.eq(p));
        }

        query.count(db).await.map(|c| c as i64)
    }

    pub async fn update_status<C: ConnectionTrait>(db: &C, id: i64, status: i32) -> std::result::Result<(), DbErr> {
        let now = chrono::Local::now().naive_local();
        ProductionPlan::update_many()
            .set(production_plan::ActiveModel {
                status: Set(Some(status)),
                update_time: Set(Some(now)),
                ..Default::default()
            })
            .filter(production_plan::Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(())
    }

    /// 回填 actual_mo_id 并更新状态
    pub async fn update_actual_mo_id_and_status<C: ConnectionTrait>(db: &C, id: i64, actual_mo_id: i64, status: i32) -> std::result::Result<(), DbErr> {
        let now = chrono::Local::now().naive_local();
        ProductionPlan::update_many()
            .set(production_plan::ActiveModel {
                actual_mo_id: Set(Some(actual_mo_id)),
                status: Set(Some(status)),
                update_time: Set(Some(now)),
                ..Default::default()
            })
            .filter(production_plan::Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(())
    }
}