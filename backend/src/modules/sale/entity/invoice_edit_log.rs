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

/// 发票修改留痕实体
/// 表 mxx_sale_invoice_edit_log：按字段粒度记录发票在"已驳回/已撤回 → 重新提交"之间的修改，
/// 保证审批记录连续性（完整链路追溯：提交→驳回→修改→再提交→再审批）。
/// 实现参考 CRM mxx_crm_customer_edit_log。
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_sale_invoice_edit_log")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 发票ID
    pub invoice_id: Option<i64>,
    /// 编辑人ID
    pub editor_id: Option<i64>,
    /// 编辑人姓名（冗余，避免联查）
    pub editor_name: Option<String>,
    /// 变更内容 JSON 数组（EditLogItem：field + field_label + old + new）
    pub content: Option<Json>,
    /// 编辑时间
    pub edit_time: Option<DateTime>,
    /// 关联审批实例ID（该次驳回/撤回所对应的实例）
    pub instance_id: Option<i64>,
    /// 软删除标记
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
