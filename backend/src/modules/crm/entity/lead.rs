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
use crate::core::r#enum::industry_enum::IndustryType;
use crate::core::r#enum::lead_source_enum::LeadSource;
use crate::core::r#enum::currency_code_enum::CurrencyCode;

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_crm_lead")]
pub struct Model {
    /// 线索ID（主键）
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,

    /// 公司名称
    pub company_name: Option<String>,

    /// 联系人姓名
    pub contact_name: Option<String>,

    /// 职位/头衔
    pub title: Option<String>,

    /// 邮箱地址
    pub email: Option<String>,

    /// 固定电话
    pub phone: Option<String>,

    /// 手机号码
    pub mobile: Option<String>,

    /// 国家
    pub country: Option<String>,

    /// 地区/省份
    pub region: Option<String>,

    /// 详细地址
    pub address: Option<String>,

    /// 公司官网
    pub website: Option<String>,

    /// 所属行业
    pub industry: Option<IndustryType>,

    /// 线索来源
    pub source: Option<LeadSource>,

    /// 来源详情
    pub source_detail: Option<String>,

    /// 线索状态
    pub status: Option<i32>,

    /// 线索等级（1-无级别、2-重点客户、3-优质客户、4-普通客户、5-其他）
    pub level: Option<i32>,

    /// 预算金额
    pub budget: Option<Decimal>,

    /// 币种
    pub currency: Option<CurrencyCode>,

    /// 下次跟进时间
    pub next_follow_at: Option<DateTime>,

    /// 负责人ID
    pub assigned_to: Option<i64>,

    /// 转换客户ID（线索转客户后的关联客户ID）
    pub converted_to_customer_id: Option<i64>,

    /// 转换时间
    pub converted_at: Option<DateTime>,

    /// 描述/备注
    pub description: Option<String>,

    /// 自定义字段（JSON格式）
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
