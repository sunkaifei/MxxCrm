//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! ABC 分类分析业务逻辑层
//!

use crate::core::errors::error::{Error, Result};
use crate::modules::product::entity::product::{self, Entity, Column};
use crate::modules::sale::entity::order_item::{self, Entity as OrderItemEntity};
use rust_decimal::Decimal;
use rust_decimal::prelude::RoundingStrategy;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, QueryFilter, Set, TransactionTrait,
};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct AbcRunResultVO {
    pub total_products: i64,
    pub class_a_count: i64,
    pub class_b_count: i64,
    pub class_c_count: i64,
    pub total_sales_amount: Decimal,
}

#[derive(Debug, Serialize)]
pub struct AbcSummaryItemVO {
    pub abc_class: String,
    pub product_count: i64,
    pub percentage: Decimal,
}

fn round2(d: Decimal) -> Decimal {
    d.round_dp_with_strategy(2, RoundingStrategy::MidpointNearestEven)
}

/// 执行 ABC 分类
/// - 查所有有销售记录的产品
/// - 按累计销售额降序排列
/// - 前 70% 标记 A，70-90% 标记 B，90-100% 标记 C
/// - 更新 product 表的 abc_class, abc_score, abc_updated_at
/// - 返回分类结果统计
pub async fn run_abc_analysis(db: &DbConn) -> Result<AbcRunResultVO> {
    // 1. 聚合各产品销售额
    let order_items = OrderItemEntity::find()
        .filter(order_item::Column::Deleted.eq(0))
        .all(db)
        .await?;

    let mut sales_map: HashMap<i64, Decimal> = HashMap::new();
    for oi in &order_items {
        if let Some(pid) = oi.product_id {
            let amount = oi.total_amount.unwrap_or(oi.amount.unwrap_or(Decimal::ZERO));
            *sales_map.entry(pid).or_insert(Decimal::ZERO) += amount;
        }
    }

    if sales_map.is_empty() {
        return Ok(AbcRunResultVO {
            total_products: 0,
            class_a_count: 0,
            class_b_count: 0,
            class_c_count: 0,
            total_sales_amount: Decimal::ZERO,
        });
    }

    // 2. 按销售额降序排列
    let mut sorted: Vec<(i64, Decimal)> = sales_map.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    let total_sales: Decimal = sorted.iter().map(|(_, amt)| *amt).sum();
    if total_sales <= Decimal::ZERO {
        return Err(Error::from("总销售额为零，无法进行 ABC 分类"));
    }

    // 3. 计算累计百分比，分配 A/B/C
    let mut cumulative = Decimal::ZERO;
    let now = chrono::Local::now().naive_local();

    let mut class_a = 0i64;
    let mut class_b = 0i64;
    let mut class_c = 0i64;

    let txn = db.begin().await?;

    for (product_id, amount) in &sorted {
        cumulative += *amount;
        let cumulative_pct = cumulative / total_sales;
        let score = round2(cumulative_pct * Decimal::from(100));

        let (abc_class, is_a, is_b, is_c) = if cumulative_pct <= Decimal::new(70, 2) {
            ("A".to_string(), true, false, false)
        } else if cumulative_pct <= Decimal::new(90, 2) {
            ("B".to_string(), false, true, false)
        } else {
            ("C".to_string(), false, false, true)
        };

        if is_a {
            class_a += 1;
        } else if is_b {
            class_b += 1;
        } else {
            class_c += 1;
        }

        // 更新产品 ABC 分类
        let active = product::ActiveModel {
            id: Set(*product_id),
            abc_class: Set(Some(abc_class)),
            abc_score: Set(Some(score)),
            abc_updated_at: Set(Some(now)),
            ..Default::default()
        };
        active.update(&txn).await?;
    }

    txn.commit().await?;

    Ok(AbcRunResultVO {
        total_products: sorted.len() as i64,
        class_a_count: class_a,
        class_b_count: class_b,
        class_c_count: class_c,
        total_sales_amount: round2(total_sales),
    })
}

/// 返回 A/B/C 各类的产品数和占比
pub async fn get_abc_summary(db: &DbConn) -> Result<Vec<AbcSummaryItemVO>> {
    let products = Entity::find()
        .filter(Column::Deleted.eq(0))
        .filter(Column::AbcClass.is_not_null())
        .all(db)
        .await?;

    let total = products.len() as i64;
    let mut count_map: HashMap<String, i64> = HashMap::new();
    for p in &products {
        if let Some(class) = &p.abc_class {
            *count_map.entry(class.clone()).or_insert(0) += 1;
        }
    }

    let mut result: Vec<AbcSummaryItemVO> = count_map.into_iter()
        .map(|(class, count)| AbcSummaryItemVO {
            percentage: if total > 0 {
                round2(Decimal::from(count) / Decimal::from(total) * Decimal::from(100))
            } else {
                Decimal::ZERO
            },
            abc_class: class,
            product_count: count,
        })
        .collect();

    // 按 A, B, C 顺序排列
    result.sort_by(|a, b| {
        let order = |c: &str| match c {
            "A" => 0,
            "B" => 1,
            "C" => 2,
            _ => 3,
        };
        order(&a.abc_class).cmp(&order(&b.abc_class))
    });

    Ok(result)
}
