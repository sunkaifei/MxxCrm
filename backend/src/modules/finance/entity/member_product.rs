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
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "mxx_member_product")]
pub struct Model {
    /// 涓婚敭ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 浜у搧鏍囪瘑ID
    pub product_id: String,

    /// 浜у搧鍚嶇О
    pub product_name: String,

    /// 浜у搧绫诲瀷
    pub product_type: String,

    /// 鏃堕暱绫诲瀷: 1=鏈堝害, 2=瀛ｅ害, 3=骞村害, 4=姘镐箙
    pub duration_type: i32,

    /// 鏈堜唤鍊硷紝1灏辨槸1涓湀锛?灏辨槸2涓湀
    pub duration_value: i32,

    /// 璐拱闄愬埗绫诲瀷: 0=鏃犻檺鍒? 1=鑷劧骞村唴闄愬埗, 2=姘镐箙闄愬埗(鏁翠釜鐢熷懡鍛ㄦ湡鍐?
    pub purchase_limit_type: Option<i32>,

    /// 璐拱闄愬埗娆℃暟锛堝湪闄愬埗鍛ㄦ湡鍐呮渶澶氬彲璐拱鐨勬鏁帮級
    pub purchase_limit_count: Option<i32>,

    /// 浠锋牸
    pub price: rust_decimal::Decimal,

    /// 鍘熶环
    pub original_price: Option<rust_decimal::Decimal>,

    /// 鎶樻墸
    pub discount: Option<rust_decimal::Decimal>,

    /// 浼氬憳绫诲瀷: 1=鏅€氫細鍛? 2=鍏绘畺鎴? 3=鍟嗘埛
    pub member_type: i32,

    /// 鐘舵€? 0=涓嬫灦, 1=涓婃灦
    pub status: i32,

    /// 鎺掑簭鍊?
    pub sort_order: i32,

    /// 浜у搧鎻忚堪
    pub description: Option<String>,

    /// 浜у搧鐗规€э紙JSON鏍煎紡锛?
    pub features: Option<serde_json::Value>,

    /// 鍒涘缓鏃堕棿
    pub create_time: Option<DateTime<Utc>>,

    /// 鏇存柊鏃堕棿
    pub update_time: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
