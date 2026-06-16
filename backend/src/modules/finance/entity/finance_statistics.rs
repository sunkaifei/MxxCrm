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
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_finance_statistics")]
pub struct Model {
    /// 涓婚敭ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 缁熻鏃ユ湡
    pub stat_date: Option<DateTime<Utc>>,

    /// 缁熻绫诲瀷: 1=鏃ョ粺璁? 2=鍛ㄧ粺璁? 3=鏈堢粺璁?
    pub stat_type: Option<i32>,

    /// 鎬绘敹鍏ラ噾棰?
    pub total_income: Decimal,

    /// 鎴愬姛鏀粯閲戦
    pub success_amount: Decimal,

    /// 閫€娆鹃噾棰?
    pub refund_amount: Decimal,

    /// 浼氬憳璐规敹鍏ラ噾棰?
    pub member_fee_amount: Decimal,

    /// 璁㈠崟鏁伴噺
    pub order_count: Option<i64>,

    /// 鎴愬姛鏀粯璁㈠崟鏁?
    pub success_count: Option<i64>,

    /// 閫€娆捐鍗曟暟
    pub refund_count: Option<i64>,

    /// 鍒涘缓鏃堕棿
    pub create_time: Option<DateTime<Utc>>,

    /// 鏇存柊鏃堕棿
    pub update_time: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
