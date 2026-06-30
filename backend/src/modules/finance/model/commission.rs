//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use serde::{Deserialize, Serialize};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use crate::modules::finance::entity::{commission_rule, commission_tier};

/// 提成阶梯DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionTierDTO {
    pub id: Option<i64>,
    pub rule_id: Option<i64>,
    pub min_amount: f64,
    pub max_amount: Option<f64>,
    pub commission_rate: f64,
    pub sort: Option<i32>,
}

impl From<commission_tier::Model> for CommissionTierDTO {
    fn from(model: commission_tier::Model) -> Self {
        Self {
            id: Some(model.id),
            rule_id: Some(model.rule_id),
            min_amount: model.min_amount.to_f64().unwrap_or_default(),
            max_amount: model.max_amount.map(|d| d.to_f64().unwrap_or_default()),
            commission_rate: model.commission_rate.to_f64().unwrap_or_default(),
            sort: model.sort,
        }
    }
}

/// 提成规则列表返回DTO（含阶梯）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionRuleDTO {
    pub id: i64,
    pub rule_name: Option<String>,
    pub department_id: Option<i64>,
    pub department_name: Option<String>,
    pub post_id: Option<i64>,
    pub post_name: Option<String>,
    pub trigger_condition: Option<i32>,
    pub effective_date: Option<String>,
    pub expiry_date: Option<String>,
    pub enabled: Option<i32>,
    pub description: Option<String>,
    pub created_by: Option<i64>,
    pub create_time: Option<String>,
    pub updated_by: Option<i64>,
    pub update_time: Option<String>,
    pub tiers: Vec<CommissionTierDTO>,
}

impl From<commission_rule::Model> for CommissionRuleDTO {
    fn from(model: commission_rule::Model) -> Self {
        Self {
            id: model.id,
            rule_name: model.rule_name,
            department_id: model.department_id,
            department_name: None,
            post_id: model.post_id,
            post_name: None,
            trigger_condition: model.trigger_condition,
            effective_date: model.effective_date.map(|d| d.format("%Y-%m-%d").to_string()),
            expiry_date: model.expiry_date.map(|d| d.format("%Y-%m-%d").to_string()),
            enabled: model.enabled,
            description: model.description,
            created_by: model.created_by,
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            updated_by: model.updated_by,
            update_time: model.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            tiers: Vec::new(),
        }
    }
}

/// 提成规则查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionRuleQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub rule_name: Option<String>,
    pub enabled: Option<i32>,
    pub department_id: Option<i64>,
    pub post_id: Option<i64>,
}

/// 提成规则保存请求体（含阶梯）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionRuleSaveDTO {
    pub id: Option<i64>,
    pub rule_name: String,
    pub department_id: Option<i64>,
    pub post_id: Option<i64>,
    pub trigger_condition: Option<i32>,
    pub effective_date: String,
    pub expiry_date: Option<String>,
    pub enabled: Option<i32>,
    pub description: Option<String>,
    pub tiers: Vec<CommissionTierSaveDTO>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
}

/// 阶梯保存DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionTierSaveDTO {
    pub id: Option<i64>,
    pub min_amount: f64,
    pub max_amount: Option<f64>,
    pub commission_rate: f64,
    pub sort: Option<i32>,
}

/// 将 f64 转为 Decimal
pub fn to_decimal(value: f64) -> Decimal {
    Decimal::from_f64(value).unwrap_or_default()
}
