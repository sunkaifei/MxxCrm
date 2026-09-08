//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 销售退货退款记录实体层
//!

use sea_orm::entity::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_sale_refund_payment")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub refund_id: Option<i64>,
    pub payment_no: Option<String>,
    /// 退款方式：1=原路退回, 2=银行转账, 3=现金, 4=其他
    pub payment_method: Option<i32>,
    pub payment_amount: Option<Decimal>,
    pub payment_time: Option<DateTime>,
    pub payment_account: Option<String>,
    pub transaction_no: Option<String>,
    pub remark: Option<String>,
    pub create_by: Option<i64>,
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
