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
use crate::core::r#enum::lead_source_enum::LeadSource;

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_crm_opportunity")]
pub struct Model {
    /// 商机ID（主键）
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,

    /// 商机编号
    #[sea_orm(ignore)]
    pub opportunity_no: Option<String>,

    /// 关联客户ID
    pub customer_id: Option<i64>,

    /// 关联联系人ID
    pub contact_id: Option<i64>,

    /// 关联线索ID
    pub lead_id: Option<i64>,

    /// 商机名称
    #[sea_orm(column_name = "name")]
    pub title: Option<String>,

    /// 描述/备注
    pub description: Option<String>,

    /// 销售阶段
    pub stage: Option<i32>,

    /// 成交概率（百分比）
    pub probability: Option<i32>,

    /// 商机金额
    pub amount: Option<Decimal>,

    /// 币种
    pub currency: Option<CurrencyCode>,

    /// 预计成交日期
    pub expected_close_date: Option<Date>,

    /// 实际关闭日期
    pub actual_close_date: Option<Date>,

    /// 负责人ID
    pub assigned_to: Option<i64>,

    /// 商机来源
    pub source: Option<LeadSource>,

    /// 标签列表（仅用于展示，关联关系存于 tag_merge 表）
    #[sea_orm(ignore)]
    pub tags: Option<Vec<String>>,

    /// 竞争对手信息
    pub competitor_info: Option<String>,

    /// 丢单原因
    pub loss_reason: Option<String>,

    /// 作废原因（作废必填，stage=6 时展示）
    pub void_reason: Option<String>,

    /// 作废前阶段（恢复时回滚到该阶段）
    pub prev_stage: Option<i32>,

    /// 需求确认内容（阶段2）
    pub requirement_summary: Option<String>,

    /// 方案沟通内容（阶段3）
    pub solution_summary: Option<String>,

    /// 报价状态（0=未报价, 1=已报价, 2=已确认）
    pub quote_status: Option<i32>,

    /// 订单状态（0=未下单, 1=已下单, 2=已完成）
    pub order_status: Option<i32>,

    /// 合同状态（0=未签, 1=已签）
    pub contract_status: Option<i32>,

    /// 发货状态（0=未发货, 1=部分发货, 2=已发货）
    pub shipment_status: Option<i32>,

    /// 回款状态（0=未回款, 1=部分回款, 2=全部回款）
    pub payment_status: Option<i32>,

    /// 发票状态（0=未开, 1=已开）
    pub invoice_status: Option<i32>,

    /// 自定义字段（JSON格式）
    #[sea_orm(ignore)]
    pub custom_fields: Option<serde_json::Value>,

    /// 创建人ID
    pub created_by: Option<i64>,

    /// 创建时间
    pub create_time: Option<DateTime>,

    /// 更新人ID
    pub updated_by: Option<i64>,

    /// 更新时间
    pub update_time: Option<DateTime>,

    /// 删除人ID（回收站）
    pub delete_by: Option<i64>,
    /// 删除时间（回收站保留期计算）
    pub delete_time: Option<DateTime>,
    /// 软删除标识（0-未删除，1-已删除）
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
