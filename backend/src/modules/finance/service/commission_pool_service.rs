//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 团建资金池服务
//! 负责资金池的 CRUD、支出登记、流水查询，以及提成归集存入
//!

use sea_orm::*;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::modules::finance::entity::{commission_pool, commission_pool_log};
use crate::modules::system::entity::{admin, dept};

/// 资金池列表 VO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommissionPoolVO {
    pub id: i64,
    pub pool_name: String,
    pub department_id: Option<i64>,
    pub department_name: Option<String>,
    pub manager_id: Option<i64>,
    pub manager_name: Option<String>,
    pub total_amount: f64,
    pub used_amount: f64,
    pub balance: f64,
    pub status: i16,
    pub description: Option<String>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
}

/// 资金池保存 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommissionPoolSaveDTO {
    pub id: Option<i64>,
    pub pool_name: String,
    pub department_id: Option<i64>,
    pub manager_id: Option<i64>,
    pub status: Option<i16>,
    pub description: Option<String>,
}

/// 资金池支出登记 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolExpenseDTO {
    pub pool_id: i64,
    pub amount: f64,
    pub usage_date: String,
    pub usage_description: String,
    pub operator_id: Option<i64>,
}

/// 资金池流水 VO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolLogVO {
    pub id: i64,
    pub pool_id: i64,
    pub log_type: i16,
    pub log_type_name: String,
    pub amount: f64,
    pub source_rule_id: Option<i64>,
    pub source_employee_id: Option<i64>,
    pub source_employee_name: Option<String>,
    pub source_year: Option<i32>,
    pub source_month: Option<i32>,
    pub usage_description: Option<String>,
    pub usage_date: Option<String>,
    pub operator_id: Option<i64>,
    pub create_time: Option<String>,
}

/// 资金池列表查询参数
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub pool_name: Option<String>,
    pub department_id: Option<i64>,
    pub status: Option<i16>,
}

/// 分页查询资金池列表
pub async fn get_pool_list(
    db: &DatabaseConnection,
    query: PoolQuery,
) -> Result<(Vec<CommissionPoolVO>, i64), String> {
    let page = std::cmp::max(query.page.unwrap_or(1), 1);
    let page_size = std::cmp::max(query.page_size.unwrap_or(20), 1);

    let mut stmt = commission_pool::Entity::find()
        .filter(commission_pool::Column::Deleted.eq(0));

    if let Some(name) = query.pool_name {
        stmt = stmt.filter(commission_pool::Column::PoolName.contains(name));
    }
    if let Some(dept_id) = query.department_id {
        stmt = stmt.filter(commission_pool::Column::DepartmentId.eq(dept_id));
    }
    if let Some(status) = query.status {
        stmt = stmt.filter(commission_pool::Column::Status.eq(status));
    }

    stmt = stmt.order_by_desc(commission_pool::Column::Id);

    let paginator = stmt.paginate(db, page_size as u64);
    let total = paginator.num_items().await.map_err(|e| e.to_string())? as i64;
    let items = paginator
        .fetch_page((page - 1) as u64)
        .await
        .map_err(|e| e.to_string())?;

    // 批量查询部门和管理人名称
    let dept_ids: Vec<i64> = items.iter().filter_map(|p| p.department_id).collect();
    let mut dept_map: HashMap<i64, String> = HashMap::new();
    if !dept_ids.is_empty() {
        let depts = dept::Entity::find()
            .filter(dept::Column::Id.is_in(dept_ids))
            .all(db)
            .await
            .map_err(|e| e.to_string())?;
        for d in depts {
            if let Some(name) = d.dept_name {
                dept_map.insert(d.id, name);
            }
        }
    }

    let manager_ids: Vec<i64> = items.iter().filter_map(|p| p.manager_id).collect();
    let mut manager_map: HashMap<i64, String> = HashMap::new();
    if !manager_ids.is_empty() {
        let admins = admin::Entity::find()
            .filter(admin::Column::Id.is_in(manager_ids))
            .all(db)
            .await
            .map_err(|e| e.to_string())?;
        for a in admins {
            let name = a.nick_name.or(a.user_name);
            if let Some(n) = name {
                manager_map.insert(a.id, n);
            }
        }
    }

    let vo_list = items
        .into_iter()
        .map(|m| {
            let balance = m.total_amount - m.used_amount;
            CommissionPoolVO {
                pool_name: m.pool_name.clone(),
                department_name: m.department_id.and_then(|id| dept_map.get(&id).cloned()),
                manager_name: m.manager_id.and_then(|id| manager_map.get(&id).cloned()),
                total_amount: m.total_amount.to_f64().unwrap_or_default(),
                used_amount: m.used_amount.to_f64().unwrap_or_default(),
                balance: balance.to_f64().unwrap_or_default(),
                create_time: m.create_time.format("%Y-%m-%d %H:%M:%S").to_string().into(),
                update_time: m.update_time.format("%Y-%m-%d %H:%M:%S").to_string().into(),
                id: m.id,
                department_id: m.department_id,
                manager_id: m.manager_id,
                status: m.status,
                description: m.description,
            }
        })
        .collect();

    Ok((vo_list, total))
}

/// 资金池详情
pub async fn get_pool_detail(db: &DatabaseConnection, id: i64) -> Result<CommissionPoolVO, String> {
    let pool = commission_pool::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "资金池不存在".to_string())?;

    let balance = pool.total_amount - pool.used_amount;
    let dept_name = if let Some(dept_id) = pool.department_id {
        dept::Entity::find_by_id(dept_id)
            .one(db)
            .await
            .map_err(|e| e.to_string())?
            .and_then(|d| d.dept_name)
    } else {
        None
    };
    let manager_name = if let Some(mid) = pool.manager_id {
        admin::Entity::find_by_id(mid)
            .one(db)
            .await
            .map_err(|e| e.to_string())?
            .and_then(|a| a.nick_name.or(a.user_name))
    } else {
        None
    };

    Ok(CommissionPoolVO {
        id: pool.id,
        pool_name: pool.pool_name,
        department_id: pool.department_id,
        department_name: dept_name,
        manager_id: pool.manager_id,
        manager_name,
        total_amount: pool.total_amount.to_f64().unwrap_or_default(),
        used_amount: pool.used_amount.to_f64().unwrap_or_default(),
        balance: balance.to_f64().unwrap_or_default(),
        status: pool.status,
        description: pool.description,
        create_time: Some(pool.create_time.format("%Y-%m-%d %H:%M:%S").to_string()),
        update_time: Some(pool.update_time.format("%Y-%m-%d %H:%M:%S").to_string()),
    })
}

/// 新建/编辑资金池
pub async fn save_pool(
    db: &DatabaseConnection,
    dto: CommissionPoolSaveDTO,
) -> Result<i64, String> {
    let now = chrono::Utc::now().naive_utc();

    if let Some(id) = dto.id {
        // 更新
        let existing: commission_pool::ActiveModel = commission_pool::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "资金池不存在".to_string())?
            .into();

        let mut existing = existing;
        existing.pool_name = Set(dto.pool_name);
        existing.department_id = Set(dto.department_id);
        existing.manager_id = Set(dto.manager_id);
        existing.status = Set(dto.status.unwrap_or(1));
        existing.description = Set(dto.description);
        existing.update_time = Set(now);
        let result = existing.update(db).await.map_err(|e| e.to_string())?;
        Ok(result.id)
    } else {
        // 新增
        let model = commission_pool::ActiveModel {
            pool_name: Set(dto.pool_name),
            department_id: Set(dto.department_id),
            manager_id: Set(dto.manager_id),
            total_amount: Set(Decimal::ZERO),
            used_amount: Set(Decimal::ZERO),
            status: Set(dto.status.unwrap_or(1)),
            description: Set(dto.description),
            create_time: Set(now),
            update_time: Set(now),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        let result = model.insert(db).await.map_err(|e| e.to_string())?;
        Ok(result.id)
    }
}

/// 资金池支出登记
///
/// 从资金池余额中扣除支出金额，写入流水（log_type=2=支出）
pub async fn expense(db: &DatabaseConnection, dto: PoolExpenseDTO) -> Result<i64, String> {
    let amount = Decimal::from_f64(dto.amount)
        .ok_or_else(|| "支出金额格式错误".to_string())?;
    if amount <= Decimal::ZERO {
        return Err("支出金额必须大于0".to_string());
    }

    let usage_date = chrono::NaiveDate::parse_from_str(&dto.usage_date, "%Y-%m-%d")
        .map_err(|_| "支出日期格式错误，应为 YYYY-MM-DD".to_string())?;

    let txn = db.begin().await.map_err(|e| e.to_string())?;

    let pool = commission_pool::Entity::find_by_id(dto.pool_id)
        .one(&txn)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "资金池不存在".to_string())?;

    if pool.status != 1 {
        return Err("资金池状态异常，无法支出".to_string());
    }

    let balance = pool.total_amount - pool.used_amount;
    if balance < amount {
        return Err(format!(
            "资金池余额不足，当前余额 {:.2}，申请支出 {:.2}",
            balance, amount
        ));
    }

    // 更新已使用金额
    let current_used = pool.used_amount;
    let mut pool_active: commission_pool::ActiveModel = pool.into();
    pool_active.used_amount = Set(current_used + amount);
    pool_active.update_time = Set(chrono::Utc::now().naive_utc());
    pool_active.update(&txn).await.map_err(|e| e.to_string())?;

    // 写入流水
    let now = chrono::Utc::now().naive_utc();
    let log = commission_pool_log::ActiveModel {
        pool_id: Set(dto.pool_id),
        log_type: Set(2), // 2=支出
        amount: Set(amount),
        source_rule_id: Set(None),
        source_employee_id: Set(None),
        source_year: Set(None),
        source_month: Set(None),
        usage_description: Set(Some(dto.usage_description)),
        usage_date: Set(Some(usage_date)),
        operator_id: Set(dto.operator_id),
        create_time: Set(now),
        deleted: Set(Some(0)),
        ..Default::default()
    };
    let log_result = log.insert(&txn).await.map_err(|e| e.to_string())?;

    txn.commit().await.map_err(|e| e.to_string())?;
    Ok(log_result.id)
}

/// 查询资金池流水
pub async fn get_pool_log(
    db: &DatabaseConnection,
    pool_id: i64,
    page: i64,
    page_size: i64,
) -> Result<(Vec<PoolLogVO>, i64), String> {
    let page = std::cmp::max(page, 1);
    let page_size = std::cmp::max(page_size, 1);

    let stmt = commission_pool_log::Entity::find()
        .filter(commission_pool_log::Column::PoolId.eq(pool_id))
        .filter(commission_pool_log::Column::Deleted.eq(0))
        .order_by_desc(commission_pool_log::Column::CreateTime);

    let paginator = stmt.paginate(db, page_size as u64);
    let total = paginator.num_items().await.map_err(|e| e.to_string())? as i64;
    let items = paginator
        .fetch_page((page - 1) as u64)
        .await
        .map_err(|e| e.to_string())?;

    // 批量查询来源员工姓名
    let employee_ids: Vec<i64> = items.iter().filter_map(|l| l.source_employee_id).collect();
    let mut emp_map: HashMap<i64, String> = HashMap::new();
    if !employee_ids.is_empty() {
        let admins = admin::Entity::find()
            .filter(admin::Column::Id.is_in(employee_ids))
            .all(db)
            .await
            .map_err(|e| e.to_string())?;
        for a in admins {
            if let Some(n) = a.nick_name.or(a.user_name) {
                emp_map.insert(a.id, n);
            }
        }
    }

    let vo_list = items
        .into_iter()
        .map(|m| PoolLogVO {
            id: m.id,
            pool_id: m.pool_id,
            log_type: m.log_type,
            log_type_name: if m.log_type == 1 { "存入".to_string() } else { "支出".to_string() },
            amount: m.amount.to_f64().unwrap_or_default(),
            source_rule_id: m.source_rule_id,
            source_employee_id: m.source_employee_id,
            source_employee_name: m.source_employee_id.and_then(|id| emp_map.get(&id).cloned()),
            source_year: m.source_year,
            source_month: m.source_month,
            usage_description: m.usage_description,
            usage_date: m.usage_date.map(|d| d.format("%Y-%m-%d").to_string()),
            operator_id: m.operator_id,
            create_time: Some(m.create_time.format("%Y-%m-%d %H:%M:%S").to_string()),
        })
        .collect();

    Ok((vo_list, total))
}

/// 提成归集存入资金池（内部调用，非 HTTP 接口）
///
/// 由月度结算引擎在 category=4 时调用：
/// 1. 更新资金池 total_amount
/// 2. 写入流水（log_type=1=存入）
pub async fn deposit_from_commission<C: ConnectionTrait>(
    db: &C,
    pool_id: i64,
    amount: Decimal,
    source_rule_id: Option<i64>,
    source_employee_id: Option<i64>,
    source_year: Option<i32>,
    source_month: Option<i32>,
) -> Result<(), String> {
    if amount <= Decimal::ZERO {
        return Ok(());
    }

    let pool = commission_pool::Entity::find_by_id(pool_id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("资金池 {} 不存在", pool_id))?;

    if pool.status != 1 {
        return Err(format!("资金池 {} 状态异常，无法存入", pool_id));
    }

    // 更新累计存入金额
    let current_total = pool.total_amount;
    let mut pool_active: commission_pool::ActiveModel = pool.into();
    pool_active.total_amount = Set(current_total + amount);
    pool_active.update_time = Set(chrono::Utc::now().naive_utc());
    pool_active.update(db).await.map_err(|e| e.to_string())?;

    // 写入流水
    let now = chrono::Utc::now().naive_utc();
    let log = commission_pool_log::ActiveModel {
        pool_id: Set(pool_id),
        log_type: Set(1), // 1=存入
        amount: Set(amount),
        source_rule_id: Set(source_rule_id),
        source_employee_id: Set(source_employee_id),
        source_year: Set(source_year),
        source_month: Set(source_month),
        usage_description: Set(None),
        usage_date: Set(None),
        operator_id: Set(None),
        create_time: Set(now),
        deleted: Set(Some(0)),
        ..Default::default()
    };
    log.insert(db).await.map_err(|e| e.to_string())?;

    Ok(())
}
