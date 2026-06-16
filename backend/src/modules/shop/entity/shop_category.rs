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
#[sea_orm(table_name = "mxx_shop_category")]
pub struct Model {
    /// 涓婚敭ID
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 涓婄骇鍒嗙被ID, 0=椤剁骇
    pub parent_id: i64,
    /// 鍒嗙被鍚嶇О
    pub name: String,
    /// 鍒嗙被鍥炬爣
    pub icon: Option<String>,
    /// 鎺掑簭鍊?鍗囧簭)
    pub sort_order: i32,
    /// 鏄惁鏄剧ず: 0=闅愯棌, 1=鏄剧ず
    pub is_show: i16,
    /// 灞傜骇: 1/2/3
    pub level: i16,
    /// 鍒涘缓鏃堕棿
    pub create_time: Option<DateTime>,
    /// 鏇存柊鏃堕棿
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}