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
#[sea_orm(table_name = "mxx_shop_settlement")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 店铺ID
    pub shop_id: i64,
    /// 结算单号
    pub settlement_no: String,
    /// 结算周期开始
    pub period_start: DateTime,
    /// 结算周期结束
    pub period_end: DateTime,
    /// 订单数量
    pub order_count: i32,
    /// 总成交额(元)
    pub total_amount: Decimal,
    /// 总佣金(元)
    pub commission_amount: Decimal,
    /// 结算金额(元)
    pub settlement_amount: Decimal,
    /// 状态: 0=待结算, 1=已结算, 2=已打款
    pub settlement_status: i16,
    /// 放款时间
    pub settle_time: Option<DateTime>,
    /// 备注
    pub remark: Option<String>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
