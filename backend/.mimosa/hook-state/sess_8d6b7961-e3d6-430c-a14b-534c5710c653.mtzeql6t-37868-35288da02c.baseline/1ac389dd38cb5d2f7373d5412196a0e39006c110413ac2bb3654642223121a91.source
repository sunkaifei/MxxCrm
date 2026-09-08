//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_purchase_stock_plan")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
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
    pub deleted: Option<i32>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}