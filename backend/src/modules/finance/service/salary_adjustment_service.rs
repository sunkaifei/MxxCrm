//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 调薪记录服务
//! 负责调薪记录的 CRUD、审批流，审批通过后同步更新 salary_config
//!

use sea_orm::*;
use sea_orm::sea_query::Expr;
use chrono::Utc;
use chrono::Datelike;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};

use crate::modules::finance::entity::{salary_adjustment, salary_config};
use crate::modules::message::service::notification_service::NotificationService;
use crate::modules::system::entity::{admin, admin_post_merge, post};

use std::collections::HashMap;

// ==================== DTO ====================

/// 创建调薪记录 DTO
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SalaryAdjustmentCreateDTO {
    pub employee_id: i64,
    /// 调薪日期（ISO 8601 字符串，如 "2026-07-01T00:00:00"）。若不传则取当前时间
    pub adjustment_date: Option<String>,
    pub adjustment_type: Option<i32>,
    pub old_base_salary: Option<f64>,
    pub new_base_salary: Option<f64>,
    pub old_position_allowance: Option<f64>,
    pub new_position_allowance: Option<f64>,
    pub old_performance_base: Option<f64>,
    pub new_performance_base: Option<f64>,
    pub adjustment_reason: Option<String>,
}

/// 驳回请求 DTO
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectDTO {
    pub id: i64,
    pub reason: String,
}

// ==================== CRUD ====================

/// 分页查询调薪记录（附带员工姓名、岗位名称）
pub async fn get_adjustment_list(
    db: &DatabaseConnection,
    employee_id: Option<i64>,
    emp_status: Option<String>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<serde_json::Value>, i64), String> {
    let mut stmt = salary_adjustment::Entity::find();
    if let Some(eid) = employee_id {
        stmt = stmt.filter(salary_adjustment::Column::EmployeeId.eq(eid));
    }

    // 员工身份过滤（G-全量）：
    //   基线（所有视图生效）：正常员工 = 非超管(user_type≠1) + 已审核(audit_status=1) + 已入职(hire_date 为空视为老员工或 ≤ 今天)
    //   active：在职 = 基线 + status=1 + 未离职(leave_date 为空)
    //   resigned：离职 = 基线 + 已离职(leave_date 非空)
    // 超管/未入职/无档案账号的调薪记录在任何视图都不出现
    let mode = emp_status.as_deref().unwrap_or("all");
    let mut cond = Condition::all()
        .add(Expr::col((admin::Entity, admin::Column::Deleted)).eq(0))
        .add(Expr::col((admin::Entity, admin::Column::AuditStatus)).eq(1))
        .add(Condition::any()
            .add(Expr::col((admin::Entity, admin::Column::UserType)).ne(1))
            .add(Expr::col((admin::Entity, admin::Column::UserType)).is_null()))
        .add(Condition::any()
            .add(Expr::col((admin::Entity, admin::Column::HireDate)).is_null())
            .add(Expr::col((admin::Entity, admin::Column::HireDate))
                .lte(Expr::current_date())));
    match mode {
        "active" => {
            cond = cond
                .add(Expr::col((admin::Entity, admin::Column::Status)).eq(1))
                .add(Expr::col((admin::Entity, admin::Column::LeaveDate)).is_null());
        }
        "resigned" => {
            cond = cond.add(Expr::col((admin::Entity, admin::Column::LeaveDate)).is_not_null());
        }
        _ => {}
    }
    let emp_sub = sea_query::Query::select()
        .column(admin::Column::Id)
        .from(admin::Entity)
        .cond_where(cond)
        .to_owned();
    stmt = stmt.filter(salary_adjustment::Column::EmployeeId.in_subquery(emp_sub));

    stmt = stmt
        .order_by_desc(salary_adjustment::Column::AdjustmentDate)
        .order_by_desc(salary_adjustment::Column::CreateTime);

    let page = std::cmp::max(page, 1);
    let page_size = std::cmp::max(page_size, 1);
    let paginator = stmt.paginate(db, page_size as u64);
    let total = paginator.num_items().await.map_err(|e| e.to_string())? as i64;
    let items = paginator
        .fetch_page((page - 1) as u64)
        .await
        .map_err(|e| e.to_string())?;

    if items.is_empty() {
        return Ok((Vec::new(), total));
    }

    // 批量补员工姓名（nick_name 优先，退回 user_name）
    let employee_ids: Vec<i64> = items.iter().map(|r| r.employee_id).collect();
    let admins = admin::Entity::find()
        .filter(admin::Column::Id.is_in(employee_ids.clone()))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;
    let name_map: HashMap<i64, String> = admins
        .iter()
        .map(|a| {
            (
                a.id,
                a.nick_name
                    .clone()
                    .or_else(|| a.user_name.clone())
                    .unwrap_or_default(),
            )
        })
        .collect();

    // 员工在职状态（是否在职列）：active=在职 resigned=离职 none=非员工/未入职/异常档案
    let today = chrono::Local::now().date_naive();
    let mut emp_state_map: HashMap<i64, &str> = HashMap::new();
    for a in &admins {
        let state = if a.user_type == Some(1) {
            "none"
        } else if a.leave_date.is_some() {
            "resigned"
        } else if a.status == Some(1)
            && a.audit_status == Some(1)
            && a.hire_date.map(|h| h <= today).unwrap_or(true)
        {
            "active"
        } else {
            "none"
        };
        emp_state_map.insert(a.id, state);
    }

    // 批量补岗位名称（一人多岗取第一个）
    let merges = admin_post_merge::Entity::find()
        .filter(admin_post_merge::Column::AdminId.is_in(employee_ids))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;
    let mut admin_post_map: HashMap<i64, i64> = HashMap::new();
    for pm in merges {
        if let (Some(admin_id), Some(post_id)) = (pm.admin_id, pm.post_id) {
            admin_post_map.entry(admin_id).or_insert(post_id);
        }
    }
    let post_ids: Vec<i64> = admin_post_map.values().copied().collect();
    let post_name_map: HashMap<i64, String> = if post_ids.is_empty() {
        HashMap::new()
    } else {
        post::Entity::find()
            .filter(post::Column::Id.is_in(post_ids))
            .all(db)
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .filter_map(|p| p.post_name.map(|n| (p.id, n)))
            .collect()
    };

    let mut list: Vec<serde_json::Value> = Vec::with_capacity(items.len());
    for r in items {
        let mut v = serde_json::to_value(&r).map_err(|e| e.to_string())?;
        if let Some(obj) = v.as_object_mut() {
            obj.insert(
                "empStatus".to_string(),
                serde_json::Value::String(
                    emp_state_map
                        .get(&r.employee_id)
                        .copied()
                        .unwrap_or("none")
                        .to_string(),
                ),
            );
            obj.insert(
                "employeeName".to_string(),
                serde_json::Value::String(
                    name_map
                        .get(&r.employee_id)
                        .cloned()
                        .unwrap_or_default(),
                ),
            );
            obj.insert(
                "postName".to_string(),
                serde_json::Value::String(
                    admin_post_map
                        .get(&r.employee_id)
                        .and_then(|pid| post_name_map.get(pid))
                        .cloned()
                        .unwrap_or_default(),
                ),
            );
        }
        list.push(v);
    }
    Ok((list, total))
}

/// 查员工调薪历史（时间轴，按调薪日期倒序）
pub async fn get_employee_history(
    db: &DatabaseConnection,
    employee_id: i64,
) -> Result<Vec<salary_adjustment::Model>, String> {
    salary_adjustment::Entity::find()
        .filter(salary_adjustment::Column::EmployeeId.eq(employee_id))
        .filter(salary_adjustment::Column::Status.eq(1)) // 只看已通过的
        .order_by_desc(salary_adjustment::Column::AdjustmentDate)
        .all(db)
        .await
        .map_err(|e| e.to_string())
}

/// 创建调薪记录（状态默认 0=待审批）
pub async fn create_adjustment(
    db: &DatabaseConnection,
    dto: SalaryAdjustmentCreateDTO,
) -> Result<i64, String> {
    let now = Utc::now().naive_utc();

    // 解析调薪日期，未传则用当前日期
    let adjustment_date = match dto.adjustment_date.as_deref() {
        Some(s) => chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
                    .map(|d| d.date())
            })
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").map(|d| d.date())
            })
            .map_err(|e| format!("调薪日期格式错误: {}", e))?,
        None => now.date(),
    };

    // 自动从 salary_config 读取 old 值（如果未提供）
    let (old_base_salary, old_position_allowance, old_performance_base) =
        if dto.old_base_salary.is_some()
            && dto.old_position_allowance.is_some()
            && dto.old_performance_base.is_some()
        {
            (
                dto.old_base_salary,
                dto.old_position_allowance,
                dto.old_performance_base,
            )
        } else {
            // 查 salary_config 当前生效配置（工资档案模型，跨年延续）
            let current = get_current_salary_config(
                db,
                dto.employee_id,
                adjustment_date.year(),
                adjustment_date.month() as i32,
            )
            .await?;
            let cfg_old_base = current.as_ref().map(|c| c.base_salary);
            let cfg_old_position = current.as_ref().and_then(|c| c.position_allowance);
            let cfg_old_perf = current.as_ref().and_then(|c| c.performance_base);
            (
                dto.old_base_salary.or_else(|| {
                    cfg_old_base.map(|v| v.to_f64().unwrap_or_default())
                }),
                dto.old_position_allowance.or_else(|| {
                    cfg_old_position.map(|v| v.to_f64().unwrap_or_default())
                }),
                dto.old_performance_base.or_else(|| {
                    cfg_old_perf.map(|v| v.to_f64().unwrap_or_default())
                }),
            )
        };

    let active = salary_adjustment::ActiveModel {
        employee_id: Set(dto.employee_id),
        adjustment_date: Set(Some(adjustment_date)),
        adjustment_type: Set(dto.adjustment_type),
        old_base_salary: Set(old_base_salary.map(|v| Decimal::from_f64(v).unwrap_or_default())),
        new_base_salary: Set(dto.new_base_salary.map(|v| Decimal::from_f64(v).unwrap_or_default())),
        old_position_allowance: Set(old_position_allowance.map(|v| Decimal::from_f64(v).unwrap_or_default())),
        new_position_allowance: Set(dto.new_position_allowance.map(|v| Decimal::from_f64(v).unwrap_or_default())),
        old_performance_base: Set(old_performance_base.map(|v| Decimal::from_f64(v).unwrap_or_default())),
        new_performance_base: Set(dto.new_performance_base.map(|v| Decimal::from_f64(v).unwrap_or_default())),
        adjustment_reason: Set(dto.adjustment_reason),
        approver_id: Set(None),
        approver_name: Set(None),
        approve_time: Set(None),
        status: Set(Some(0)),
        create_time: Set(Some(now)),
        ..Default::default()
    };

    let txn = db.begin().await.map_err(|e| e.to_string())?;
    let inserted = active.insert(&txn).await.map_err(|e| e.to_string())?;
    txn.commit().await.map_err(|e| e.to_string())?;
    Ok(inserted.id)
}

/// 审批通过
/// - 将调薪记录状态置为 1
/// - 同步更新 salary_config（按生效年月）
pub async fn approve_adjustment(
    db: &DatabaseConnection,
    id: i64,
    approver_id: i64,
    approver_name: &str,
) -> Result<(), String> {
    let now = Utc::now().naive_utc();
    let txn = db.begin().await.map_err(|e| e.to_string())?;

    let record = salary_adjustment::Entity::find_by_id(id)
        .one(&txn)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "调薪记录不存在".to_string())?;

    let status = record.status.unwrap_or(0);
    if status != 0 {
        return Err("只有待审批状态的调薪记录才能审批".to_string());
    }

    // T2.7: 审批流进行中禁止手动旁路（结果以引擎同步为准）
    if crate::modules::approval::service::approval_service::ApprovalService::has_active_instance(
        db, "salary_adjustment", id,
    )
    .await
    .map_err(|e| e.to_string())?
    {
        return Err("该调薪记录正在审批流中，请在审批流内处理或先撤销当前流程".to_string());
    }

    // 更新调薪记录
    let mut active: salary_adjustment::ActiveModel = record.clone().into();
    active.status = Set(Some(1));
    active.approver_id = Set(Some(approver_id));
    active.approver_name = Set(Some(approver_name.to_string()));
    active.approve_time = Set(Some(now));
    active.update(&txn).await.map_err(|e| e.to_string())?;

    // 同步到 salary_config
    // 取生效年月（adjustment_date 的年月）
    let adjustment_date = record.adjustment_date.unwrap_or(now.date());
    let year = adjustment_date.year();
    let month = Some(adjustment_date.month() as i32);

    let new_base = record.new_base_salary.unwrap_or_else(|| {
        record.old_base_salary.unwrap_or(Decimal::ZERO)
    });
    let new_position = record.new_position_allowance.or(record.old_position_allowance);
    let new_perf = record.new_performance_base.or(record.old_performance_base);

    upsert_salary_config(
        &txn,
        record.employee_id,
        year,
        month,
        new_base,
        new_position,
        new_perf,
        now,
    )
    .await?;

    txn.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

/// 审批驳回
pub async fn reject_adjustment(
    db: &DatabaseConnection,
    id: i64,
    approver_id: i64,
    approver_name: &str,
    reason: &str,
) -> Result<(), String> {
    let now = Utc::now().naive_utc();
    let txn = db.begin().await.map_err(|e| e.to_string())?;

    let record = salary_adjustment::Entity::find_by_id(id)
        .one(&txn)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "调薪记录不存在".to_string())?;

    let status = record.status.unwrap_or(0);
    if status != 0 {
        return Err("只有待审批状态的调薪记录才能驳回".to_string());
    }

    // T2.7: 审批流进行中禁止手动旁路
    if crate::modules::approval::service::approval_service::ApprovalService::has_active_instance(
        db, "salary_adjustment", id,
    )
    .await
    .map_err(|e| e.to_string())?
    {
        return Err("该调薪记录正在审批流中，请在审批流内处理或先撤销当前流程".to_string());
    }

    let emp_id = record.employee_id;
    let mut active: salary_adjustment::ActiveModel = record.into();
    active.status = Set(Some(2));
    active.approver_id = Set(Some(approver_id));
    active.approver_name = Set(Some(approver_name.to_string()));
    active.approve_time = Set(Some(now));
    active.reject_reason = Set(Some(reason.to_string()));
    active.update(&txn).await.map_err(|e| e.to_string())?;

    txn.commit().await.map_err(|e| e.to_string())?;

    // 通知申请人调薪被驳回
    let _ = NotificationService::send_system_notification(
        db, emp_id,
        "调薪申请已驳回".to_string(),
        format!("您的调薪申请已被驳回，原因：{}", reason),
        8, // 8=财务信息
        Some("/finance/salary-adjustment".to_string()),
    ).await;

    Ok(())
}

/// 调薪前后对比（最近一次调薪）
pub async fn get_comparison(
    db: &DatabaseConnection,
    employee_id: i64,
) -> Result<salary_adjustment::Model, String> {
    salary_adjustment::Entity::find()
        .filter(salary_adjustment::Column::EmployeeId.eq(employee_id))
        .order_by_desc(salary_adjustment::Column::AdjustmentDate)
        .order_by_desc(salary_adjustment::Column::CreateTime)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "暂无调薪记录".to_string())
}

// ==================== T2.7c 审批流对接 ====================

/// 提交调薪审批流（flow_code=salary_adjustment_approval，v60 预置）
/// 提交后 approval_status=2（审批中），手动审批入口被拦截
pub async fn submit_adjustment_approval(
    db: &DatabaseConnection,
    id: i64,
    submitter_id: i64,
    submitter_name: &str,
) -> Result<i64, String> {
    use crate::modules::approval::model::approval::ApprovalSubmitRequest;
    use crate::modules::approval::service::approval_service::ApprovalService;

    let record = salary_adjustment::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "调薪记录不存在".to_string())?;

    if record.status.unwrap_or(0) != 0 {
        return Err("只有待审批状态的调薪记录才能提交审批".to_string());
    }
    if record.approval_status == Some(2) {
        return Err("该调薪记录已在审批流中".to_string());
    }

    let employee_name = crate::modules::system::entity::admin::Entity::find_by_id(record.employee_id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .and_then(|a| a.nick_name.or(a.user_name))
        .unwrap_or_default();

    let submit_req = ApprovalSubmitRequest {
        flow_code: "salary_adjustment_approval".to_string(),
        business_type: "salary_adjustment".to_string(),
        business_id: id,
        business_title: Some(format!("{} 调薪审批", employee_name)),
        submitter_id,
        submitter_name: Some(submitter_name.to_string()),
        extra_data: Some(serde_json::json!({
            "employeeId": record.employee_id,
            "employeeName": employee_name,
            "newBaseSalary": record.new_base_salary,
        })),
        cc_user_ids: None,
        cc_reason: None,
    };
    let instance_id = ApprovalService::submit(db, &submit_req)
        .await
        .map_err(|e| e.to_string())?;

    let mut active: salary_adjustment::ActiveModel = record.into();
    active.approval_status = Set(Some(2));
    active.instance_id = Set(Some(instance_id));
    active.update(db).await.map_err(|e| e.to_string())?;
    Ok(instance_id)
}

/// 同步调薪审批流结果（引擎实例 status=3 通过 / 4 驳回 → 回写业务状态）
/// 通过：复用 approve_adjustment（同步 salary_config）；驳回：复用 reject_adjustment
pub async fn sync_approval_status(db: &DatabaseConnection) -> Result<i64, String> {
    use crate::modules::approval::entity::approval_instance::{Column as InstanceColumn, Entity as InstanceEntity};

    let instances = InstanceEntity::find()
        .filter(InstanceColumn::BusinessType.eq("salary_adjustment".to_string()))
        .filter(InstanceColumn::Status.is_in(vec![3, 4]))
        .order_by_desc(InstanceColumn::SubmittedAt)
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let mut synced: i64 = 0;
    for inst in instances {
        let adj_id = match inst.business_id {
            Some(bid) => bid,
            None => continue,
        };
        let record = match salary_adjustment::Entity::find_by_id(adj_id).one(db).await {
            Ok(Some(r)) => r,
            _ => continue,
        };
        // 只同步处于"审批中"的记录，幂等
        if record.approval_status != Some(2) {
            continue;
        }
        match inst.status {
            Some(3) => {
                approve_adjustment(db, adj_id, 0, "审批流终审").await?;
                mark_adjustment_approval(db, adj_id, 3).await?;
                synced += 1;
            }
            Some(4) => {
                let reason = inst
                    .cancel_reason
                    .clone()
                    .unwrap_or_else(|| format!("审批流 #{} 驳回", inst.id));
                reject_adjustment(db, adj_id, 0, "审批流终审", &reason).await?;
                mark_adjustment_approval(db, adj_id, 4).await?;
                synced += 1;
            }
            _ => {}
        }
    }
    Ok(synced)
}

/// 回写 approval_status（approve/reject 已改写 status，此处仅同步审批字典位）
async fn mark_adjustment_approval(db: &DatabaseConnection, id: i64, approval_status: i32) -> Result<(), String> {
    let record = salary_adjustment::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "调薪记录不存在".to_string())?;
    let mut active: salary_adjustment::ActiveModel = record.into();
    active.approval_status = Set(Some(approval_status));
    active.update(db).await.map_err(|e| e.to_string())?;
    Ok(())
}

// ==================== 内部工具 ====================

/// 查询员工当前生效的 salary_config（工资档案模型）
/// 生效时间 = (year, month.unwrap_or(1))，取生效时间 <= (year, month) 中最近的一条
async fn get_current_salary_config(
    db: &DatabaseConnection,
    employee_id: i64,
    year: i32,
    month: i32,
) -> Result<Option<salary_config::Model>, String> {
    let configs = salary_config::Entity::find()
        .filter(salary_config::Column::EmployeeId.eq(employee_id))
        .filter(salary_config::Column::Year.lte(year))
        .filter(salary_config::Column::Status.eq(1))
        .filter(salary_config::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    configs
        .iter()
        .filter(|c| (c.year, c.month.unwrap_or(1)) <= (year, month))
        .max_by_key(|c| (c.year, c.month.unwrap_or(1)))
        .cloned()
        .map_or(Ok(None), |c| Ok(Some(c)))
}

/// 新增或更新 salary_config
async fn upsert_salary_config<C: ConnectionTrait>(
    db: &C,
    employee_id: i64,
    year: i32,
    month: Option<i32>,
    base_salary: Decimal,
    position_allowance: Option<Decimal>,
    performance_base: Option<Decimal>,
    now: chrono::NaiveDateTime,
) -> Result<(), String> {
    // 查找现有配置（同 employee_id + year + month）
    let existing = salary_config::Entity::find()
        .filter(salary_config::Column::EmployeeId.eq(employee_id))
        .filter(salary_config::Column::Year.eq(year))
        .filter(salary_config::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let matched = existing.iter().find(|c| c.month == month).cloned();

    if let Some(model) = matched {
        let mut active: salary_config::ActiveModel = model.into();
        active.base_salary = Set(base_salary);
        active.position_allowance = Set(position_allowance);
        active.performance_base = Set(performance_base);
        active.status = Set(Some(1));
        active.update_time = Set(Some(now));
        active.update(db).await.map_err(|e| e.to_string())?;
    } else {
        let active = salary_config::ActiveModel {
            employee_id: Set(employee_id),
            year: Set(year),
            month: Set(month),
            base_salary: Set(base_salary),
            position_allowance: Set(position_allowance),
            performance_base: Set(performance_base),
            status: Set(Some(1)),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        active.insert(db).await.map_err(|e| e.to_string())?;
    }
    Ok(())
}
