//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 服务权益配额流水（mxx_sale_entitlement_usage，P1.3）
//!

use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_sale_entitlement_usage")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub entitlement_id: i64,
    /// 正=消耗，负=返还
    pub change_amount: Decimal,
    pub before_quota: Option<Decimal>,
    pub after_quota: Option<Decimal>,
    /// 1工单 2人工 9其他
    pub biz_type: Option<i32>,
    pub biz_id: Option<i64>,
    pub operator_id: Option<i64>,
    pub remark: Option<String>,
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
