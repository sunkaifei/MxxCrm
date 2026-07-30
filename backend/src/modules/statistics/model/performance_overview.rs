//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！

use rust_decimal::Decimal;
use sea_orm::prelude::DateTime;
use serde::{Deserialize, Serialize};

/// 业绩概览通用查询参数（同比环比/预测/漏斗/拆解/行为等共用）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PerformanceOverviewQuery {
    #[serde(default)]
    pub year: Option<i32>,
    #[serde(default)]
    pub month: Option<i32>,
    /// time_dimension: year | month | day
    #[serde(default, rename = "time_dimension")]
    pub time_dimension: Option<String>,
    /// employee_id：个人维度（个人成长/里程碑）
    #[serde(default)]
    pub employee_id: Option<i64>,
}

// ============== 4.2.1 业绩对比 ==============
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ComparisonItemVO {
    /// 同比 %
    pub yoy: Option<f64>,
    /// 环比 %
    pub mom: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceComparisonVO {
    pub contract: ComparisonItemVO,
    pub payment: ComparisonItemVO,
    pub contract_count: ComparisonItemVO,
    pub avg_deal_size: ComparisonItemVO,
}

// ============== 4.2.2 业绩预测 ==============
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceForecastVO {
    pub completed_amount: Decimal,
    pub pipeline_amount: Decimal,
    /// 历史成交率 0~1
    pub historical_win_rate: f64,
    pub forecast_amount: Decimal,
    pub target_amount: Decimal,
    pub gap_amount: Decimal,
    /// green | yellow | red | warning
    pub status: String,
    /// Pipeline 覆盖率 = 在途商机 / 缺口
    pub pipeline_coverage: f64,
}

// ============== 4.2.3 销售漏斗 ==============
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FunnelStageVO {
    pub stage: String,
    pub count: i64,
    pub amount: Decimal,
    /// 转化率 %（相对于上一阶段）
    pub conversion_rate: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SalesFunnelVO {
    pub stages: Vec<FunnelStageVO>,
    pub avg_cycle_days: i64,
    /// 赢单率 % = 成交 / (成交 + 丢单)
    pub win_rate: f64,
    pub total_leads: i64,
}

// ============== 4.2.4 客户拆解 ==============
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BreakdownItemVO {
    pub name: String,
    pub value: i64,
    pub amount: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TopCustomerVO {
    pub rank: i32,
    pub customer_name: String,
    pub amount: Decimal,
    /// 同比 %
    pub growth: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CustomerBreakdownVO {
    /// 新老客户
    pub new_vs_old: Vec<BreakdownItemVO>,
    /// ABC 分级
    pub abc_distribution: Vec<BreakdownItemVO>,
    /// Top10
    pub top10: Vec<TopCustomerVO>,
}

// ============== 4.2.5 产品拆解 ==============
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductRankVO {
    pub rank: i32,
    pub product_name: String,
    pub amount: Decimal,
    pub count: i64,
    /// 占比 %
    pub share: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductBreakdownVO {
    pub products: Vec<ProductRankVO>,
    pub categories: Vec<BreakdownItemVO>,
}

// ============== 4.2.6 行为指标 ==============
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BehaviorSummaryVO {
    pub visit_count: i64,
    pub phone_count: i64,
    pub follow_up_count: i64,
    /// 转化率 %
    pub conversion_rate: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BehaviorTrendItemVO {
    pub period: String,
    pub visit_count: i64,
    pub phone_count: i64,
    pub follow_up_count: i64,
    pub amount: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BehaviorMetricsVO {
    pub summary: BehaviorSummaryVO,
    pub trend: Vec<BehaviorTrendItemVO>,
}

// ============== 4.2.7 区域拆解 ==============
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegionItemVO {
    pub rank: i32,
    pub province: String,
    pub amount: Decimal,
    pub customer_count: i64,
    /// 占比 %
    pub share: f64,
}

// ============== 4.2.8 个人成长 ==============
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BestMonthVO {
    pub month: String,
    pub amount: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GrowthTrendItemVO {
    pub month: String,
    pub amount: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PersonalGrowthVO {
    pub hire_date: Option<String>,
    pub total_amount: Decimal,
    pub total_contract_count: i64,
    pub best_month: BestMonthVO,
    pub monthly_trend: Vec<GrowthTrendItemVO>,
}

// ============== 4.2.9 里程碑 ==============
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneItemVO {
    pub label: String,
    pub amount: Decimal,
    pub achieved: bool,
    pub achieved_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceMilestoneVO {
    pub current_milestone: Option<MilestoneItemVO>,
    pub next_milestone: Option<MilestoneItemVO>,
    pub future_milestone: Option<MilestoneItemVO>,
    /// future_remaining 字段：距离下一未达成档位的剩余金额
    pub remaining: Option<Decimal>,
    pub milestones: Vec<MilestoneItemVO>,
}

// ============== 4.2.10 页面配置 ==============
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceConfigVO {
    pub visible_cards: Vec<String>,
    pub default_time_dimension: String,
    pub rank_scope: String,
    pub show_sensitive_data: bool,
    pub refresh_interval: i64,
    pub custom_milestones: Vec<Decimal>,
}
