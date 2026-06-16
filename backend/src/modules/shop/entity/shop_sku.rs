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
#[sea_orm(table_name = "mxx_shop_sku")]
pub struct Model {
    /// 涓婚敭ID
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 鎵€灞濻PU ID
    pub spu_id: i64,
    /// 瑙勬牸鎻忚堪(濡?棰滆壊:鐧借壊 灏虹爜:M")
    pub spec_desc: String,
    /// SKU缂栫爜
    pub sku_code: Option<String>,
    /// SKU鍥剧墖
    pub image: Option<String>,
    /// 渚涜揣搴曚环(鍒?
    pub base_price: Decimal,
    /// 闆跺敭浠?鍒?
    pub retail_price: Decimal,
    /// 瀹為檯鍞环(鍒?
    pub sale_price: Decimal,
    /// 鍒掔嚎浠?鍒?
    pub line_price: Decimal,
    /// 搴撳瓨
    pub stock: i32,
    /// 瀹夊叏搴撳瓨
    pub safe_stock: i32,
    /// 宸插敭鏁伴噺
    pub sold_num: i32,
    /// 閲嶉噺(kg)
    pub weight: Option<Decimal>,
    /// 鍒涘缓鏃堕棿
    pub create_time: Option<DateTime>,
    /// 鏇存柊鏃堕棿
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}