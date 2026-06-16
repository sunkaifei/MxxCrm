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
#[sea_orm(table_name = "mxx_shop_promotion")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 店铺ID(NULL=平台活动)
    pub shop_id: i64,
    /// 活动标题
    pub title: String,
    /// 类型: 1=满减, 2=折扣, 3=限时抢购, 4=新人专享
    pub promotion_type: i16,
    /// 优惠值(满减金额/折扣率/特价价格)
    pub discount_value: Decimal,
    /// 条件值(满减条件金额)
    pub condition_value: Option<Decimal>,
    /// 开始时间
    pub start_time: DateTime,
    /// 结束时间
    pub end_time: DateTime,
    /// 状态: 0=未开始, 1=进行中, 2=已结束, 3=已关闭
    pub status: i16,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
