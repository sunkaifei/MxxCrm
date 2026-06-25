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
    #[sea_orm(ignore)]
    pub contact_id: Option<i64>,

    /// 关联线索ID
    #[sea_orm(ignore)]
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
    #[sea_orm(ignore)]
    pub source: Option<LeadSource>,

    /// 标签列表（仅用于展示，关联关系存于 tag_merge 表）
    #[sea_orm(ignore)]
    pub tags: Option<Vec<String>>,

    /// 竞争对手信息
    pub competitor_info: Option<String>,

    /// 丢单原因
    pub loss_reason: Option<String>,

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

    /// 软删除标识（0-未删除，1-已删除）
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
