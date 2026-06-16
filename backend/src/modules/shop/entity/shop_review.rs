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
#[sea_orm(table_name = "mxx_shop_review")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 订单ID
    pub order_id: i64,
    /// 商品ID
    pub spu_id: i64,
    /// SKU ID
    pub sku_id: i64,
    /// 买家用户ID
    pub user_id: i64,
    /// 店铺ID
    pub shop_id: i64,
    /// 评分: 1~5
    pub score: i16,
    /// 评价内容
    pub content: Option<String>,
    /// 评价图片列表(JSON)
    pub images: Option<Json>,
    /// 是否匿名: 0=否, 1=是
    pub is_anonymous: i16,
    /// 供货商回复
    pub reply_content: Option<String>,
    /// 回复时间
    pub reply_time: Option<DateTime>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
