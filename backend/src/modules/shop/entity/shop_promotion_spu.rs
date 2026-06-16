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

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mxx_shop_promotion_spu")]
pub struct Model {
    /// 涓婚敭ID
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 娲诲姩ID
    pub promotion_id: i64,
    /// 鍟嗗搧ID
    pub spu_id: i64,
    /// SKU ID(鍙€?
    pub sku_id: Option<i64>,
    /// 鍒涘缓鏃堕棿
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}