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
#[sea_orm(table_name = "mxx_shop_refund")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 订单ID
    pub order_id: i64,
    /// 订单明细ID
    pub order_item_id: i64,
    /// 退款单号
    pub refund_no: String,
    /// 买家用户ID
    pub user_id: i64,
    /// 店铺ID
    pub shop_id: i64,
    /// 退款类型: 1=退货退款, 2=仅退款
    pub refund_type: i16,
    /// 退款原因
    pub refund_reason: String,
    /// 退款金额(元)
    pub refund_amount: Decimal,
    /// 退款状态: 0=待审核, 1=已同意, 2=已拒绝, 3=退款中, 4=已退款
    pub refund_status: i16,
    /// 审核备注
    pub audit_remark: Option<String>,
    /// 审核时间
    pub audit_time: Option<DateTime>,
    /// 退回原因
    pub reject_reason: Option<String>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
