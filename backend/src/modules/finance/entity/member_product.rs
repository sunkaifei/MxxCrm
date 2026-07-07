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
#[sea_orm(table_name = "mxx_member_product")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 产品标识ID
    pub product_id: String,

    /// 产品名称
    pub product_name: String,

    /// 产品类型
    pub product_type: String,

    /// 时长类型: 1=月度, 2=季度, 3=年度, 4=永久
    pub duration_type: i32,

    /// 月份值，1就是1个月，2就是2个月
    pub duration_value: i32,

    /// 购买限制类型: 0=无限制, 1=自然年内限制, 2=永久限制(整个生命周期内)
    pub purchase_limit_type: Option<i32>,

    /// 购买限制次数（在限制周期内最多可购买的数量）
    pub purchase_limit_count: Option<i32>,

    /// 价格
    pub price: rust_decimal::Decimal,

    /// 原价
    pub original_price: Option<rust_decimal::Decimal>,

    /// 折扣
    pub discount: Option<rust_decimal::Decimal>,

    /// 会员类型: 1=普通会员, 2=养户, 3=商户
    pub member_type: i32,

    /// 状态: 0=下架, 1=上架
    pub status: i32,

    /// 排序值
    pub sort_order: i32,

    /// 产品描述
    pub description: Option<String>,

    /// 产品特性（JSON格式）
    pub features: Option<serde_json::Value>,

    /// 创建人ID
    pub created_by: Option<i64>,

    /// 创建时间
    pub create_time: Option<DateTime>,

    /// 更新人ID
    pub updated_by: Option<i64>,

    /// 更新时间
    pub update_time: Option<DateTime>,

    /// 删除标识: 0=未删除 1=已删除
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
