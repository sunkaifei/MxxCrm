//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSummaryVO {
    pub stock_alert: StockAlertSummary,
    pub stock_doc_todo: StockDocTodoSummary,
    pub purchase_approval: PurchaseApprovalSummary,
    pub payment_reminder: PaymentReminderSummary,
    pub payslip_stat: PayslipStatSummary,
    pub hr_todo: HrTodoSummary,
    pub sales_performance: SalesPerformanceSummary,
    pub announcement: AnnouncementSummary,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct StockAlertSummary {
    pub total: i64,
    pub items: Vec<StockAlertItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct StockAlertItem {
    pub product_name: Option<String>,
    pub warehouse_name: Option<String>,
    pub quantity: Option<Decimal>,
    pub alert_type: Option<String>,
    pub alert_min_quantity: Option<Decimal>,
    pub alert_max_quantity: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct StockDocTodoSummary {
    pub inbound_total: i64,
    pub outbound_total: i64,
    pub items: Vec<StockDocTodoItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct StockDocTodoItem {
    pub biz_type: Option<String>,
    pub id: Option<i64>,
    pub order_no: Option<String>,
    pub status: Option<i32>,
    pub total_quantity: Option<Decimal>,
    pub total_amount: Option<Decimal>,
    pub created_by_name: Option<String>,
    pub create_time: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseApprovalSummary {
    pub total: i64,
    pub items: Vec<PurchaseApprovalItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseApprovalItem {
    pub id: Option<i64>,
    pub pr_no: Option<String>,
    pub title: Option<String>,
    pub status: Option<i32>,
    pub total_amount: Option<Decimal>,
    pub urgency: Option<String>,
    pub create_time: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct PaymentReminderSummary {
    pub total: i64,
    pub items: Vec<PaymentReminderItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct PaymentReminderItem {
    pub id: Option<i64>,
    pub contract_id: Option<i64>,
    pub stage_name: Option<String>,
    pub plan_amount: Option<Decimal>,
    pub received_amount: Option<Decimal>,
    pub plan_date: Option<NaiveDate>,
    pub status: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct PayslipStatSummary {
    pub sent_count: i64,
    pub read_count: i64,
    pub unread_count: i64,
    pub total_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct HrTodoSummary {
    pub total: i64,
    pub onboarding: i64,
    pub resign: i64,
    pub items: Vec<HrTodoItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct HrTodoItem {
    pub id: i64,
    pub business_type: String,
    pub business_title: Option<String>,
    pub submitter_name: Option<String>,
    pub submitted_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct SalesPerformanceSummary {
    pub new_customers: i64,
    pub follow_ups: i64,
    pub new_opportunities: i64,
    pub deal_amount: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AnnouncementSummary {
    pub total: i64,
    pub unread: i64,
    pub items: Vec<AnnouncementItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AnnouncementItem {
    pub id: Option<i64>,
    pub title: Option<String>,
    pub level: Option<String>,
    pub publish_name: Option<String>,
    pub publish_time: Option<String>,
    pub is_read: Option<i32>,
}
