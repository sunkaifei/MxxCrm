//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use chrono::NaiveDate;
use sea_orm::prelude::Date;
use sea_orm::{Condition, ColumnTrait};

/// 统计时间范围。start/end 均为 None 时表示不过滤（全部）。
#[derive(Debug, Clone, Copy)]
pub struct StatsRange {
    pub start: Option<NaiveDate>,
    pub end: Option<NaiveDate>,
}

impl StatsRange {
    pub fn is_all(&self) -> bool {
        self.start.is_none() && self.end.is_none()
    }

    /// 是否覆盖今天（含今天则查询需合并当日实时数据）
    pub fn covers_today(&self) -> bool {
        match self.end {
            None => true,
            Some(e) => e >= chrono::Local::now().date_naive(),
        }
    }

    /// 历史部分区间（到昨天为止），用于汇总表查询
    pub fn history_part(&self) -> StatsRange {
        let yesterday = chrono::Local::now().date_naive() - chrono::Duration::days(1);
        StatsRange {
            start: self.start,
            end: self.end.map_or(Some(yesterday), |e| Some(e.min(yesterday))),
        }
    }
}

/// 判断日期（Option<Date>）是否落在范围内。
/// 范围为全部时恒为 true；指定范围后无日期的记录不计入。
pub fn date_in_range(d: Option<Date>, range: &StatsRange) -> bool {
    if range.is_all() {
        return true;
    }
    match d {
        None => false,
        Some(d) => range.start.map_or(true, |s| d >= s) && range.end.map_or(true, |e| d <= e),
    }
}

/// 判断日期时间（Option<NaiveDateTime>）是否落在范围内（按日期部分比较）。
pub fn datetime_in_range(dt: Option<chrono::NaiveDateTime>, range: &StatsRange) -> bool {
    if range.is_all() {
        return true;
    }
    match dt {
        None => false,
        Some(dt) => date_in_range(Some(dt.date()), range),
    }
}

/// 数据权限范围：None = 全部（超管）；Some(空) = 无可见数据（调用方应短路返回空结果）
pub type StatsScope = Option<Vec<i64>>;

/// scope 是否为"空范围"（有权限约束但无任何可见用户）
pub fn scope_is_empty(scope: &StatsScope) -> bool {
    matches!(scope, Some(ids) if ids.is_empty())
}

/// DATE 列的区间过滤条件（SeaORM 构建器用；range 为 None 时不加条件）
pub fn date_cond<C: ColumnTrait>(col: C, range: &StatsRange) -> Condition {
    let mut cond = Condition::all();
    if let Some(s) = range.start {
        cond = cond.add(col.gte(s));
    }
    if let Some(e) = range.end {
        cond = cond.add(col.lte(e));
    }
    cond
}

/// TIMESTAMP 列的区间过滤条件（start 日 00:00:00 ~ end 日 23:59:59）
pub fn datetime_cond<C: ColumnTrait>(col: C, range: &StatsRange) -> Condition {
    let mut cond = Condition::all();
    if let Some(s) = range.start.and_then(|d| d.and_hms_opt(0, 0, 0)) {
        cond = cond.add(col.gte(s));
    }
    if let Some(e) = range.end.and_then(|d| d.and_hms_opt(23, 59, 59)) {
        cond = cond.add(col.lte(e));
    }
    cond
}

/// 负责人列的数据权限过滤条件
pub fn scope_cond<C: ColumnTrait>(col: C, scope: &StatsScope) -> Condition {
    match scope {
        None => Condition::all(),
        Some(ids) if ids.is_empty() => Condition::all(), // 调用方应先用 scope_is_empty 短路
        Some(ids) => Condition::all().add(col.is_in(ids.clone())),
    }
}

// ==================== 原生 SQL 参数辅助（SeaORM 2.0 兼容）====================
// SeaORM 2.0-rc 的 Value 无 Date/BigIntArray 公开变体（项目现有代码仅用 String/BigInt 等），
// 统一策略：日期与 id 数组均以 text 参数传入，SQL 端用 ::date / ::int8[] 转换。

/// 日期参数（text 形式，SQL 端 ::date 转换；None → SQL NULL）
pub fn date_param(d: Option<NaiveDate>) -> sea_orm::Value {
    sea_orm::Value::String(d.map(|x| x.format("%Y-%m-%d").to_string()))
}

/// scope 数组参数（text '{1,2}' 形式，SQL 端 ::int8[] 转换；空数组 → '{}' 恒无匹配）
pub fn ids_param(scope: &StatsScope) -> sea_orm::Value {
    match scope {
        None => sea_orm::Value::String(None),
        Some(ids) if ids.is_empty() => sea_orm::Value::String(Some("{}".to_string())),
        Some(ids) => sea_orm::Value::String(Some(format!(
            "{{{}}}",
            ids.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(",")
        ))),
    }
}
