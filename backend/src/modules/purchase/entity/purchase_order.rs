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
use serde::{Deserialize, Serialize};
use crate::core::r#enum::currency_code_enum::CurrencyCode;

/// 采购订单表实体
/// 存储采购订单信息，包含订单金额、供应商状态、收货地址等
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_purchase_order")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 订单编号
    pub order_no: Option<String>,
    /// 供应商ID
    pub supplier_id: Option<i64>,
    /// 订单类型
    pub order_type: Option<String>,
    /// 订单状态
    pub status: Option<String>,
    /// 订单金额（不含税）
    pub amount: Option<Decimal>,
    /// 币种
    pub currency: Option<CurrencyCode>,
    /// 税额
    pub tax_amount: Option<Decimal>,
    /// 订单总金额（含税）
    pub total_amount: Option<Decimal>,
    /// 付款状态
    pub payment_status: Option<String>,
    /// 付款方式
    pub payment_method: Option<String>,
    /// 已付款金额
    pub paid_amount: Option<Decimal>,
    /// 收货地址
    pub delivery_address: Option<String>,
    /// 预计到货日期
    pub expected_delivery_date: Option<DateTime>,
    /// 备注
    pub remark: Option<String>,
    /// 创建人ID
    pub created_by: Option<i64>,
    /// 创建时间
    pub created_at: Option<DateTime>,
    /// 更新人ID
    pub updated_by: Option<i64>,
    /// 更新时间
    pub updated_at: Option<DateTime>,
    /// 软删除标记（0未删除，1已删除）
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}