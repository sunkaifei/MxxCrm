//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究。
//!

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// 退款单（mxx_website_refund）
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_website_refund")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,

    /// 退款单号（唯一）
    pub refund_no: String,

    /// 订单ID
    pub order_id: i64,

    /// 订单号
    pub order_no: Option<String>,

    /// 用户ID
    pub user_id: i64,

    /// 订单项ID（部分退款时必填，整单退款为空）
    pub order_item_id: Option<i64>,

    /// 退款类型：1仅退款 2退货退款
    #[serde(default)]
    pub refund_type: Option<i32>,

    /// 退款原因
    pub refund_reason: Option<String>,

    /// 退款金额
    #[serde(default)]
    pub refund_amount: Decimal,

    /// 状态：0待审核 1已通过 2已拒绝 3已退款 4已取消
    #[serde(default)]
    pub status: Option<i32>,

    /// 退款方式：1原路退回 2余额
    pub refund_way: Option<i32>,

    /// 第三方退款流水号
    pub transaction_id: Option<String>,

    /// 处理备注
    pub handle_remark: Option<String>,

    /// 处理人ID（管理员）
    pub handle_by: Option<i64>,

    /// 处理时间
    pub handle_time: Option<DateTime>,

    /// 创建时间
    pub create_time: Option<DateTime>,

    /// 更新时间
    pub update_time: Option<DateTime>,

    /// 软删除：0未删除 1已删除
    #[serde(default)]
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
