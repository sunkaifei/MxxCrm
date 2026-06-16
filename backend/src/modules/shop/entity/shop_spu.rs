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
#[sea_orm(table_name = "mxx_shop_spu")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 所属店铺ID
    pub shop_id: i64,
    /// 分类ID
    pub category_id: i64,
    /// 商品标题
    pub title: String,
    /// 副标题/卖点
    pub subtitle: Option<String>,
    /// 主图
    pub primary_image: String,
    /// 商品轮播图列表(JSON)
    pub images: Option<Json>,
    /// 商品视频
    pub video: Option<String>,
    /// 商品详情(富文本/图片列表JSON)
    pub desc_content: Option<Json>,
    /// 是否参与佣金: 0=否, 1=是
    pub is_commission: i16,
    /// 状态: 0=待审核, 1=已上架, 2=下架, 3=审核退回
    pub status: i16,
    /// 总库存
    pub stock_total: i32,
    /// 已售数量
    pub sold_num: i32,
    /// 最低售价(元)
    pub min_sale_price: Decimal,
    /// 最高售价(元)
    pub max_sale_price: Decimal,
    /// 最低划线价(元)
    pub min_line_price: Decimal,
    /// 最高划线价(元)
    pub max_line_price: Decimal,
    /// 审核备注
    pub audit_remark: Option<String>,
    /// 运费模板ID
    pub freight_template_id: Option<i64>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
