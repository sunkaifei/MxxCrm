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
#[sea_orm(table_name = "mxx_shop_order_item")]
pub struct Model {
    /// 涓婚敭ID
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 璁㈠崟ID
    pub order_id: i64,
    /// 鍟嗗搧ID
    pub spu_id: i64,
    /// SKU ID
    pub sku_id: i64,
    /// 鍟嗗搧鏍囬(蹇収)
    pub goods_title: String,
    /// 鍟嗗搧鍥剧墖(蹇収)
    pub goods_image: String,
    /// 瑙勬牸鎻忚堪(濡?棰滆壊:鐧借壊 灏虹爜:M")
    pub spec_desc: Option<String>,
    /// 鎴愪氦鍗曚环(鍒?
    pub price: Decimal,
    /// 鏁伴噺
    pub quantity: i32,
    /// 渚涜揣搴曚环(鍒?
    pub base_price: Decimal,
    /// 璇KU浣ｉ噾(鍒?
    pub commission_amount: Decimal,
    /// 璇KU缁撶畻閲戦(鍒?
    pub settlement_amount: Decimal,
    /// 鍒涘缓鏃堕棿
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}