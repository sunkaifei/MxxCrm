//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 服务权益 Model 层
//!

use rust_decimal::Decimal;
use sea_orm::*;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::sale::entity::entitlement::{self, Entity as EntitlementEntity};

// ==================== 请求 DTO ====================

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct EntitlementSaveRequest {
    pub order_id: Option<i64>,
    pub order_item_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub entitlement_type: Option<i32>,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub duration_months: Option<i32>,
    pub auto_renew: Option<i32>,
    pub total_quota: Option<Decimal>,
    pub sla_level: Option<String>,
    pub response_time_hours: Option<i32>,
    pub resolution_time_hours: Option<i32>,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct EntitlementListQuery {
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_to_u64")]
    pub page_num: Option<i64>,
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_to_u64")]
    pub page_size: Option<i64>,
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_to_u64")]
    pub customer_id: Option<i64>,
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_to_u64")]
    pub order_id: Option<i64>,
    pub status: Option<i32>,
    pub entitlement_type: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct EntitlementRenewRequest {
    pub old_entitlement_id: i64,
    pub new_order_id: i64,
}

// ==================== VO ====================

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct EntitlementListVO {
    pub id: i64,
    pub entitlement_no: Option<String>,
    pub order_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub entitlement_type: Option<i32>,
    pub entitlement_type_name: Option<String>,
    pub status: Option<i32>,
    pub status_name: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub duration_months: Option<i32>,
    pub auto_renew: Option<i32>,
    pub renew_count: Option<i32>,
    pub remaining_days: Option<i32>,
    pub total_quota: Option<Decimal>,
    pub used_quota: Option<Decimal>,
    pub remaining_quota: Option<Decimal>,
    pub sla_level: Option<String>,
    pub parent_entitlement_id: Option<i64>,
    pub create_time: Option<String>,
}

// ==================== Model ====================

pub struct EntitlementModel;

impl EntitlementModel {
    pub async fn insert<C: ConnectionTrait>(
        db: &C, req: &EntitlementSaveRequest, entitlement_no: Option<String>, create_by: Option<i64>
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = entitlement::ActiveModel {
            entitlement_no: Set(entitlement_no),
            order_id: Set(req.order_id),
            order_item_id: Set(req.order_item_id),
            customer_id: Set(req.customer_id),
            product_id: Set(req.product_id),
            product_name: Set(req.product_name.clone()),
            entitlement_type: Set(req.entitlement_type),
            status: Set(Some(2)), // 生效中
            start_date: Set(req.start_date),
            end_date: Set(req.end_date),
            duration_months: Set(req.duration_months),
            auto_renew: Set(req.auto_renew.or(Some(0))),
            renew_count: Set(Some(0)),
            total_quota: Set(req.total_quota),
            used_quota: Set(Some(Decimal::from(0))),
            remaining_quota: Set(req.total_quota),
            sla_level: Set(req.sla_level.clone()),
            response_time_hours: Set(req.response_time_hours),
            resolution_time_hours: Set(req.resolution_time_hours),
            parent_entitlement_id: Set(None),
            next_renew_date: Set(req.end_date),
            remark: Set(req.remark.clone()),
            create_by: Set(create_by),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        let result = EntitlementEntity::insert(payload).exec(db).await?;
        Ok(result.last_insert_id)
    }

    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<entitlement::Model>, DbErr> {
        EntitlementEntity::find_by_id(id)
            .filter(entitlement::Column::Deleted.eq(0))
            .one(db).await
    }

    pub async fn find_by_order<C: ConnectionTrait>(db: &C, order_id: i64) -> Result<Vec<entitlement::Model>, DbErr> {
        EntitlementEntity::find()
            .filter(entitlement::Column::OrderId.eq(order_id))
            .filter(entitlement::Column::Deleted.eq(0))
            .all(db).await
    }

    pub async fn find_by_customer<C: ConnectionTrait>(db: &C, customer_id: i64) -> Result<Vec<entitlement::Model>, DbErr> {
        EntitlementEntity::find()
            .filter(entitlement::Column::CustomerId.eq(customer_id))
            .filter(entitlement::Column::Deleted.eq(0))
            .order_by_desc(entitlement::Column::Id)
            .all(db).await
    }

    pub async fn find_expiring<C: ConnectionTrait>(
        db: &C, target_date: chrono::NaiveDate
    ) -> Result<Vec<entitlement::Model>, DbErr> {
        EntitlementEntity::find()
            .filter(entitlement::Column::EndDate.eq(target_date))
            .filter(entitlement::Column::Status.eq(2)) // 生效中
            .filter(entitlement::Column::Deleted.eq(0))
            .all(db).await
    }

    pub async fn update_status<C: ConnectionTrait>(db: &C, id: i64, status: i32) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = entitlement::ActiveModel {
            status: Set(Some(status)),
            update_time: Set(Some(now)),
            ..Default::default()
        };
        let result = EntitlementEntity::update_many()
            .set(payload)
            .filter(entitlement::Column::Id.eq(id))
            .filter(entitlement::Column::Deleted.eq(0))
            .exec(db).await?;
        Ok(result.rows_affected as i64)
    }

    /// 设置续约链
    pub async fn link_renewal<C: ConnectionTrait>(db: &C, new_id: i64, parent_id: i64) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = entitlement::ActiveModel {
            parent_entitlement_id: Set(Some(parent_id)),
            renew_count: Set(Some(1)),
            update_time: Set(Some(now)),
            ..Default::default()
        };
        let result = EntitlementEntity::update_many()
            .set(payload)
            .filter(entitlement::Column::Id.eq(new_id))
            .filter(entitlement::Column::Deleted.eq(0))
            .exec(db).await?;
        Ok(result.rows_affected as i64)
    }

    /// 批量软删除
    pub async fn batch_delete<C: ConnectionTrait>(db: &C, ids: &[i64]) -> Result<i64, DbErr> {
        if ids.is_empty() {
            return Ok(0);
        }
        let result = EntitlementEntity::update_many()
            .col_expr(entitlement::Column::Deleted, sea_orm::sea_query::Expr::value(1))
            .filter(entitlement::Column::Id.is_in(ids.to_vec()))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn get_max_entitlement_no_today<C: ConnectionTrait>(
        db: &C, prefix: &str
    ) -> Result<Option<i64>, DbErr> {
        let pattern = format!("{}%", prefix);
        let rows: Vec<(String,)> = EntitlementEntity::find()
            .filter(entitlement::Column::EntitlementNo.like(&pattern))
            .select_only()
            .column(entitlement::Column::EntitlementNo)
            .into_tuple()
            .all(db).await?;
        let max_seq = rows.iter()
            .filter_map(|(no,)| no.trim_start_matches(prefix).parse::<i64>().ok())
            .max();
        Ok(max_seq)
    }
}

pub fn entitlement_type_name(t: i32) -> &'static str {
    match t {
        1 => "服务期",
        2 => "订阅周期",
        3 => "技术支持",
        4 => "资源包",
        5 => "SLA",
        _ => "未知",
    }
}

pub fn entitlement_status_name(s: i32) -> &'static str {
    match s {
        1 => "待激活",
        2 => "生效中",
        3 => "已暂停",
        4 => "已到期",
        5 => "已取消",
        _ => "未知",
    }
}

/// 计算剩余天数
pub fn remaining_days(end_date: Option<chrono::NaiveDate>) -> Option<i32> {
    end_date.map(|d| {
        let today = chrono::Local::now().date_naive();
        (d - today).num_days() as i32
    })
}
