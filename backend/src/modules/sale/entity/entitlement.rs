//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 服务权益实体（mxx_sale_entitlement）
//!

use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_sale_entitlement")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub entitlement_no: Option<String>,
    pub order_id: Option<i64>,
    pub order_item_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    /// 权益类型：1=服务期, 2=订阅周期, 3=技术支持, 4=资源包, 5=SLA
    pub entitlement_type: Option<i32>,
    /// 状态：1=待激活, 2=生效中, 3=已暂停, 4=已到期, 5=已取消
    pub status: Option<i32>,
    pub start_date: Option<Date>,
    pub end_date: Option<Date>,
    pub duration_months: Option<i32>,
    /// 是否自动续约（0=否，1=是）
    pub auto_renew: Option<i32>,
    pub renew_count: Option<i32>,
    pub total_quota: Option<Decimal>,
    pub used_quota: Option<Decimal>,
    pub remaining_quota: Option<Decimal>,
    pub sla_level: Option<String>,
    pub response_time_hours: Option<i32>,
    pub resolution_time_hours: Option<i32>,
    /// 续约前序权益（链式）
    pub parent_entitlement_id: Option<i64>,
    pub next_renew_date: Option<Date>,
    pub remark: Option<String>,
    pub create_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
