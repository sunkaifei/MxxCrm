//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_sale_online_payment")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub payment_no: Option<String>,
    pub order_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub amount: Option<Decimal>,
    pub currency: Option<i32>,
    pub payment_channel: Option<i32>,
    pub channel_trade_no: Option<String>,
    pub prepay_id: Option<String>,
    pub pay_url: Option<String>,
    pub qr_code: Option<String>,
    pub status: Option<i32>,
    pub paid_time: Option<DateTime>,
    pub expire_time: Option<DateTime>,
    pub callback_data: Option<String>,
    pub remark: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
