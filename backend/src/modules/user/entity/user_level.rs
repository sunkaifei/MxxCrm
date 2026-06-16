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
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "mxx_user_level")]
pub struct Model {
    /// 涓婚敭ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 绛夌骇鏍囪瘑锛堝敮涓€锛?
    pub level_code: String,

    /// 绛夌骇鍚嶇О
    pub level_name: String,

    /// 绛夌骇鎻忚堪
    pub level_desc: Option<String>,

    /// 涔岄緹鏁伴噺閰嶉锛?1琛ㄧず鏃犻檺鍒讹級
    pub turtle_quota: i32,

    /// 浜岀淮鐮佹暟閲忛厤棰濓紙-1琛ㄧず鏃犻檺鍒讹級
    pub qrcode_quota: i32,

    /// 鏍囩鏁伴噺閰嶉锛?1琛ㄧず鏃犻檺鍒讹級
    pub tag_quota: i32,

    /// 鍔熻兘鏉冮檺閰嶇疆锛圝SON鏍煎紡锛?
    #[sea_orm(column_type = "Json")]
    pub permissions: Option<serde_json::Value>,

    /// 鎺掑簭浼樺厛绾?
    pub sort_order: i32,

    /// 鐘舵€? 0=绂佺敤, 1=鍚敤
    pub status: i32,

    /// 鍒涘缓鏃堕棿
    pub create_time: Option<DateTime>,

    /// 鏇存柊鏃堕棿
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}