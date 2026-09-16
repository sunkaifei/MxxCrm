//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 设计器字段树元数据（设计文档 §4.2 + §32.11 样本预设列）。

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_pdf_field_meta")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 单据类型：quotation/order/contract/shipment/outbound/purchase/invoice/payment
    pub doc_type: String,
    /// 分组：header | item | company | customer | sys | custom
    pub group_code: String,
    /// 分组显示名
    pub group_name: String,
    /// 绑定路径，如 order.order_no
    pub field_path: String,
    /// 显示名，如 订单编号
    pub field_name: String,
    /// 数据类型：string|number|date|money|image
    pub data_type: String,
    /// 典型样例值（typical 预设）
    pub sample: Option<String>,
    /// 超长样例值（long 预设）
    pub sample_long: Option<String>,
    /// 按预设键覆盖的样例值 {"long":"…","empty":"","zero":0}
    pub sample_presets: Option<Json>,
    /// 可否为空
    pub nullable: bool,
    /// 取样来源：meta | seed | fallback
    pub sample_source: String,
    /// 排序
    pub sort: i32,
    /// 状态（1启用 0禁用）
    pub status: i32,
    /// 删除标志
    pub deleted: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
