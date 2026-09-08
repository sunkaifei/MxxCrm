//!
//! 客户公海考核统计（v57 全量设计 §3.5，V1 报表口径）
//!
//! 个人视图：在持/本月领取/退回/被回收/跟进及时率/平均持有天数/首次响应时长
//! 管理员视图：按池透视（在池/冻结/流入/流出）+ 全局口径
//!
//! 口径说明：存量私海客户 `assigned_at IS NULL` 不计入考核（迁移声明）。

use chrono::{Datelike, Local};
use sea_orm::{ColumnTrait, Condition, DbConn, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};

use crate::core::errors::error::{Error, Result};
use crate::modules::crm::entity::{customer, customer_assign_history, customer_pool_config};

#[derive(Debug, serde::Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct MyPoolStatsVO {
    /// 在持客户数（私海，deleted=0）
    pub holding_count: i64,
    /// 本月领取（action 1 领取 + 5 自动分配）
    pub month_claimed: i64,
    /// 本月退回（action 2）
    pub month_released: i64,
    /// 本月被自动回收（action 6）
    pub month_recycled: i64,
    /// 跟进及时率 %（领取后 48h 内产生首跟进的占比，本年口径）
    pub followup_timely_rate: f64,
    /// 平均持有天数（在持客户 assigned_at 至今）
    pub avg_holding_days: f64,
    /// 首次响应时长（小时，均值；领取→首跟进）
    pub first_response_avg_hours: f64,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct PoolAdminRowVO {
    pub pool_id: i64,
    pub pool_name: Option<String>,
    /// 在池（未分配）客户数
    pub pool_customer_count: i64,
    /// 冻结客户数
    pub frozen_count: i64,
    /// 本月流入（退回 2 + 自动回收 6 进入本池）
    pub month_inflow: i64,
    /// 本月流出（领取/申领 1 + 自动分配 5 离开本池）
    pub month_outflow: i64,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct AdminPoolStatsVO {
    pub pools: Vec<PoolAdminRowVO>,
    /// 全局冻结数
    pub total_frozen: i64,
    /// 全局在池
    pub total_in_pool: i64,
}

fn month_start() -> chrono::NaiveDateTime {
    let now = Local::now().naive_local();
    chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
}

async fn history_count(db: &DbConn, admin_id: i64, actions: Vec<i16>, from: chrono::NaiveDateTime) -> Result<i64> {
    let n = customer_assign_history::Entity::find()
        .filter(customer_assign_history::Column::AdminId.eq(admin_id))
        .filter(customer_assign_history::Column::ActionType.is_in(actions))
        .filter(customer_assign_history::Column::CreateTime.gte(from))
        .count(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(n as i64)
}

/// 个人公海成绩
pub async fn mine(db: &DbConn, user_id: i64) -> Result<MyPoolStatsVO> {
    let month_start = month_start();
    let holding_count = customer::Entity::find()
        .filter(
            Condition::all()
                .add(customer::Column::AssignedTo.eq(user_id))
                .add(customer::Column::Deleted.eq(0)),
        )
        .count(db)
        .await
        .map_err(|e| Error::from(e.to_string()))? as i64;

    let month_claimed = history_count(db, user_id, vec![1, 5], month_start).await?;
    let month_released = history_count(db, user_id, vec![2], month_start).await?;
    let month_recycled = history_count(db, user_id, vec![6], month_start).await?;

    // 跟进及时率/首次响应：本年领取记录（action 1/5），对比首条跟进时间
    let claims = customer_assign_history::Entity::find()
        .filter(customer_assign_history::Column::AdminId.eq(user_id))
        .filter(customer_assign_history::Column::ActionType.is_in(vec![1, 5]))
        .filter(customer_assign_history::Column::StartTime.is_not_null())
        .filter(customer_assign_history::Column::StartTime.gte(month_start))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    let mut timely = 0i64;
    let mut total_hours: Vec<f64> = Vec::new();
    for c in &claims {
        let Some(start) = c.start_time else { continue };
        let first = crate::modules::crm::entity::followup::Entity::find()
            .filter(crate::modules::crm::entity::followup::Column::CustomerId.eq(c.customer_id.unwrap_or(0)))
            .filter(crate::modules::crm::entity::followup::Column::Deleted.eq(0))
            .filter(crate::modules::crm::entity::followup::Column::CreateTime.gte(start))
            .order_by_asc(crate::modules::crm::entity::followup::Column::CreateTime)
            .one(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        if let Some(f) = first {
            if let Some(ft) = f.create_time {
                let hours = (ft - start).num_hours() as f64;
                total_hours.push(hours.max(0.0));
                if hours <= 48.0 {
                    timely += 1;
                }
            }
        }
    }
    let claim_total = claims.len() as f64;
    let followup_timely_rate = if claim_total > 0.0 { timely as f64 / claim_total * 100.0 } else { 0.0 };
    let first_response_avg_hours = if !total_hours.is_empty() {
        total_hours.iter().sum::<f64>() / total_hours.len() as f64
    } else {
        0.0
    };

    // 平均持有天数（在持且 assigned_at 非空）
    let holding_rows = customer::Entity::find()
        .filter(
            Condition::all()
                .add(customer::Column::AssignedTo.eq(user_id))
                .add(customer::Column::Deleted.eq(0))
                .add(customer::Column::AssignedAt.is_not_null()),
        )
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    let now = Local::now().naive_local();
    let avg_holding_days = if holding_rows.is_empty() {
        0.0
    } else {
        holding_rows
            .iter()
            .filter_map(|c| c.assigned_at.map(|a| (now - a).num_days() as f64))
            .sum::<f64>()
            / holding_rows.len() as f64
    };

    Ok(MyPoolStatsVO {
        holding_count,
        month_claimed,
        month_released,
        month_recycled,
        followup_timely_rate: (followup_timely_rate * 10.0).round() / 10.0,
        avg_holding_days: (avg_holding_days * 10.0).round() / 10.0,
        first_response_avg_hours: (first_response_avg_hours * 10.0).round() / 10.0,
    })
}

/// 管理员按池透视
pub async fn admin_view(db: &DbConn) -> Result<AdminPoolStatsVO> {
    use crate::modules::crm::entity::customer_pool;
    let pools = customer_pool::Entity::find()
        .filter(customer_pool::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    let month_start = month_start();

    let mut rows: Vec<PoolAdminRowVO> = Vec::new();
    let mut total_frozen = 0i64;
    let mut total_in_pool = 0i64;

    for p in &pools {
        let pid = p.id;
        let in_pool = customer::Entity::find()
            .filter(
                Condition::all()
                    .add(customer::Column::PoolId.eq(pid))
                    .add(customer::Column::AssignedTo.is_null())
                    .add(customer::Column::Deleted.eq(0)),
            )
            .count(db)
            .await
            .map_err(|e| Error::from(e.to_string()))? as i64;
        let frozen = customer::Entity::find()
            .filter(
                Condition::all()
                    .add(customer::Column::PoolId.eq(pid))
                    .add(customer::Column::ReleaseFrozen.eq(1))
                    .add(customer::Column::Deleted.eq(0)),
            )
            .count(db)
            .await
            .map_err(|e| Error::from(e.to_string()))? as i64;
        // 流入/流出按归属历史的池维度 + 时间
        let inflow = customer_assign_history::Entity::find()
            .filter(customer_assign_history::Column::PoolId.eq(pid))
            .filter(customer_assign_history::Column::ActionType.is_in(vec![2, 6]))
            .filter(customer_assign_history::Column::CreateTime.gte(month_start))
            .count(db)
            .await
            .map_err(|e| Error::from(e.to_string()))? as i64;
        let outflow = customer_assign_history::Entity::find()
            .filter(customer_assign_history::Column::PoolId.eq(pid))
            .filter(customer_assign_history::Column::ActionType.is_in(vec![1, 5]))
            .filter(customer_assign_history::Column::CreateTime.gte(month_start))
            .count(db)
            .await
            .map_err(|e| Error::from(e.to_string()))? as i64;

        total_frozen += frozen;
        total_in_pool += in_pool;
        rows.push(PoolAdminRowVO {
            pool_id: pid,
            pool_name: p.name.clone(),
            pool_customer_count: in_pool,
            frozen_count: frozen,
            month_inflow: inflow,
            month_outflow: outflow,
        });
    }

    // 压制未使用告警（config 引入预留给超期预警 V2）
    let _ = customer_pool_config::Entity::find().count(db).await;

    Ok(AdminPoolStatsVO { pools: rows, total_frozen, total_in_pool })
}
