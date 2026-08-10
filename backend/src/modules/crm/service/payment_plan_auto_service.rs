//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 回款计划自动生成 Service
//!
//! 根据合同的付款方式（一次性/分期/里程碑）自动生成 contract_payment_plan 记录。
//!

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, QueryFilter, Set, TransactionTrait};

use crate::core::errors::error::{Error, Result};
use crate::modules::crm::entity::contract::Entity as ContractEntity;
use crate::modules::crm::entity::contract_payment_plan::{self};

/// 付款方式：1=一次性, 2=分期, 3=里程碑
const PAYMENT_METHOD_ONE_TIME: i32 = 1;
const PAYMENT_METHOD_INSTALLMENT: i32 = 2;
const PAYMENT_METHOD_MILESTONE: i32 = 3;

/// 回款计划默认状态：1=待回款
const PLAN_STATUS_PENDING: i32 = 1;
/// 付款类型：1=首付款, 2=分期款, 3=尾款, 4=里程碑款
const PAYMENT_TYPE_INSTALLMENT: i32 = 2;

/// 根据合同自动生成回款计划
///
/// 根据合同 `payment_method_type` 字段决定生成策略：
/// - **一次性（1）**：生成 1 条计划，金额=total_amount，日期=签约日
/// - **分期（2）**：解析 `payment_terms` JSON `{ "periods": N, "intervalDays": D }`，
///   每期金额=total/N，首期日期=签约日，后续按 intervalDays 间隔递增
/// - **里程碑（3）**：解析 `payment_terms` JSON 数组，按里程碑节点生成
///   每个节点支持 `{ "name": "阶段名", "ratio": 0.3, "amount": "1000", "days": 30 }`
///
/// 返回生成的计划条数。
pub async fn generate_plans_for_contract(db: &DbConn, contract_id: i64) -> Result<i64> {
    if contract_id <= 0 {
        return Err(Error::from("合同ID不能为空"));
    }

    // 1. 查询合同
    let contract = ContractEntity::find()
        .filter(crate::modules::crm::entity::contract::Column::Id.eq(contract_id))
        .filter(crate::modules::crm::entity::contract::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("合同不存在"))?;

    let total_amount = contract.total_amount.unwrap_or(contract.amount.unwrap_or_default());
    if total_amount <= Decimal::ZERO {
        return Err(Error::from("合同金额不能为空或小于等于0"));
    }

    let payment_method_type = contract.payment_method_type.unwrap_or(PAYMENT_METHOD_ONE_TIME);
    let base_date = contract.sign_date
        .or(contract.start_date)
        .unwrap_or_else(|| chrono::Local::now().naive_local().date());
    let payment_terms = contract.payment_terms.clone();
    let owner_user_id = contract.assigned_to;

    // 2. 根据付款方式构造计划条目
    let entries = build_plan_entries(payment_method_type, total_amount, base_date, payment_terms.as_deref())?;

    if entries.is_empty() {
        return Err(Error::from("未能生成任何回款计划，请检查付款条款配置"));
    }

    // 3. 写入 contract_payment_plan 表（事务包裹）
    let entries_clone = entries.clone();
    let owner_user_id_clone = owner_user_id;
    let contract_id_val = contract_id;
    let inserted = db.transaction::<_, i64, sea_orm::DbErr>(|txn| {
        Box::pin(async move {
            let mut count: i64 = 0;
            for (idx, entry) in entries_clone.iter().enumerate() {
                let active = contract_payment_plan::ActiveModel {
                    contract_id: Set(Some(contract_id_val)),
                    stage_name: Set(Some(entry.name.clone())),
                    payment_type: Set(Some(entry.payment_type)),
                    plan_amount: Set(Some(entry.amount)),
                    received_amount: Set(Some(Decimal::ZERO)),
                    plan_date: Set(Some(entry.date)),
                    status: Set(Some(PLAN_STATUS_PENDING)),
                    sort: Set(Some(idx as i32 + 1)),
                    owner_user_id: Set(owner_user_id_clone),
                    create_by: Set(Some("system".to_string())),
                    create_time: Set(Some(chrono::Local::now().naive_local())),
                    ..Default::default()
                };
                active.insert(txn).await?;
                count += 1;
            }
            Ok(count)
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    Ok(inserted)
}

/// 单条回款计划构造项
#[derive(Clone)]
struct PlanEntry {
    name: String,
    payment_type: i32,
    amount: Decimal,
    date: NaiveDate,
}

/// 根据付款方式构造计划条目
fn build_plan_entries(
    method_type: i32,
    total_amount: Decimal,
    base_date: NaiveDate,
    payment_terms: Option<&str>,
) -> Result<Vec<PlanEntry>> {
    match method_type {
        PAYMENT_METHOD_ONE_TIME => {
            // 一次性：1 条，金额=total，日期=签约日
            Ok(vec![PlanEntry {
                name: "一次性付款".to_string(),
                payment_type: PAYMENT_TYPE_INSTALLMENT,
                amount: total_amount,
                date: base_date,
            }])
        }
        PAYMENT_METHOD_INSTALLMENT => {
            // 分期：解析 { "periods": N, "intervalDays": D }
            let json = parse_terms_object(payment_terms)?;
            let periods = json.get("periods")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| Error::from("分期付款缺少 periods 配置"))?;
            if periods <= 0 {
                return Err(Error::from("分期期数必须大于0"));
            }
            let interval_days = json.get("intervalDays")
                .and_then(|v| v.as_i64())
                .unwrap_or(30);
            let periods_u32 = periods as u32;
            let each = (total_amount / Decimal::from(periods_u32))
                .round_dp(2);

            let mut entries = Vec::new();
            for i in 0..periods {
                let offset = chrono::Duration::days(interval_days * i as i64);
                let date = base_date.checked_add_signed(offset)
                    .unwrap_or(base_date);
                entries.push(PlanEntry {
                    name: format!("第{}期", i + 1),
                    payment_type: PAYMENT_TYPE_INSTALLMENT,
                    amount: each,
                    date,
                });
            }
            Ok(entries)
        }
        PAYMENT_METHOD_MILESTONE => {
            // 里程碑：解析数组 [{ "name", "ratio", "amount", "days" }]
            let arr = parse_terms_array(payment_terms)?;
            if arr.is_empty() {
                return Err(Error::from("里程碑付款缺少节点配置"));
            }
            let mut entries = Vec::new();
            for (idx, node) in arr.iter().enumerate() {
                let name = node.get("name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("里程碑{}", idx + 1));
                let amount = if let Some(amt_str) = node.get("amount").and_then(|v| v.as_str()) {
                    Decimal::from_str_exact(amt_str)
                        .map_err(|e| Error::from(format!("里程碑金额解析失败: {}", e)))?
                } else if let Some(ratio) = node.get("ratio").and_then(|v| v.as_f64()) {
                    (total_amount * Decimal::from_f64_retain(ratio).unwrap_or(Decimal::ZERO))
                        .round_dp(2)
                } else {
                    return Err(Error::from(format!("里程碑[{}]缺少 amount 或 ratio", name)));
                };
                let days = node.get("days")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let date = base_date.checked_add_signed(chrono::Duration::days(days))
                    .unwrap_or(base_date);
                entries.push(PlanEntry {
                    name,
                    payment_type: PAYMENT_TYPE_INSTALLMENT,
                    amount,
                    date,
                });
            }
            Ok(entries)
        }
        other => Err(Error::from(format!("不支持的付款方式类型: {}", other))),
    }
}

/// 解析付款条款为 JSON 对象
fn parse_terms_object(payment_terms: Option<&str>) -> Result<serde_json::Value> {
    let raw = payment_terms
        .filter(|s| !s.is_empty())
        .ok_or_else(|| Error::from("付款条款(payment_terms)为空"))?;
    serde_json::from_str::<serde_json::Value>(raw)
        .map_err(|e| Error::from(format!("付款条款JSON解析失败: {}", e)))
}

/// 解析付款条款为 JSON 数组
fn parse_terms_array(payment_terms: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let raw = payment_terms
        .filter(|s| !s.is_empty())
        .ok_or_else(|| Error::from("付款条款(payment_terms)为空"))?;
    let value: serde_json::Value = serde_json::from_str(raw)
        .map_err(|e| Error::from(format!("付款条款JSON解析失败: {}", e)))?;
    value.as_array()
        .cloned()
        .ok_or_else(|| Error::from("里程碑付款条款必须是JSON数组"))
}

/// 返回付款方式配置说明（前端可据此构造 payment_terms JSON）
pub fn get_payment_terms_config() -> serde_json::Value {
    serde_json::json!({
        "paymentMethodTypes": [
            { "value": PAYMENT_METHOD_ONE_TIME, "label": "一次性付款", "desc": "生成1条计划，金额=合同总额，日期=签约日" },
            {
                "value": PAYMENT_METHOD_INSTALLMENT,
                "label": "分期付款",
                "desc": "按期数均分，每期金额=总额/期数，首期=签约日，后续按间隔天数递增",
                "termsExample": { "periods": 3, "intervalDays": 30 }
            },
            {
                "value": PAYMENT_METHOD_MILESTONE,
                "label": "里程碑付款",
                "desc": "按里程碑节点生成，每节点可用 ratio(比例) 或 amount(金额) 指定，days=距签约日的天数",
                "termsExample": [
                    { "name": "首付", "ratio": 0.3, "days": 0 },
                    { "name": "验收", "amount": "5000", "days": 30 },
                    { "name": "尾款", "ratio": 0.5, "days": 60 }
                ]
            }
        ]
    })
}
