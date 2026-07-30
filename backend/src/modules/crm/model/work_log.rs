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
use serde::{Deserialize, Serialize};

/// 工作日志展示对象（序列化 camelCase）
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct WorkLogVO {
    pub id: i64,
    pub user_id: i64,
    pub user_name: Option<String>,
    pub action_type: Option<i32>,
    pub action_name: Option<String>,
    pub business_type: Option<String>,
    pub business_id: Option<i64>,
    pub business_title: Option<String>,
    pub description: Option<String>,
    pub result: Option<i32>,
    pub work_date: Option<NaiveDate>,
    pub create_time: Option<NaiveDateTime>,
}

/// 工作日志创建 DTO（内部创建使用，同时作为 /work-log/create 请求体，反序列化 camelCase）
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkLogCreateDTO {
    pub user_id: i64,
    pub user_name: Option<String>,
    pub action_type: Option<i32>,
    pub action_name: Option<String>,
    pub business_type: Option<String>,
    pub business_id: Option<i64>,
    pub business_title: Option<String>,
    pub description: Option<String>,
    pub result: Option<i32>,
    pub work_date: Option<NaiveDate>,
}

/// 本周工作负载统计（每天处理的工作数量）
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct WeekWorkloadVO {
    /// 日期字符串（如 2024-01-01）
    pub date: String,
    /// 当天工作数量
    pub count: i64,
}

/// 今日待办汇总（已处理数来自 work_log 持久化，剩余数实时查询）
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct TodaySummaryVO {
    /// 今日已处理待办数（来自 mxx_work_log，action_type=1/2/3）
    pub todo_processed: i64,
    /// 当前剩余待办数（实时查询：审批+跟进+回款）
    pub todo_remaining: i64,
    /// 今日待办总数 = todo_processed + todo_remaining
    pub todo_total: i64,
    /// 完成率（0-100）
    pub completion_rate: i32,
    /// 按业务类型分类统计
    pub by_type: TodoTypeSummary,
}

/// 按业务类型分类的待办统计
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct TodoTypeSummary {
    /// 审批：已处理 / 剩余
    pub approval_processed: i64,
    pub approval_remaining: i64,
    /// 跟进：已处理 / 剩余
    pub follow_up_processed: i64,
    pub follow_up_remaining: i64,
    /// 回款：已处理 / 剩余
    pub payment_processed: i64,
    pub payment_remaining: i64,
}
