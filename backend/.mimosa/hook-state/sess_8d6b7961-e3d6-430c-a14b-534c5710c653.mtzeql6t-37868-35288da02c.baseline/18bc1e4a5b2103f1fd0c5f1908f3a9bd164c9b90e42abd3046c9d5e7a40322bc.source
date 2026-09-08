//!
//! 客户公海：管理员指派 / 冻结列表与恢复 / 自动分配兜底扫描（v57 全量设计 §5.3/§5.5/§5.7）
//!
//! 复用 workbench 的 insert_history / 通知 helper 与 lead_auto_assign_service 的 sweep 模式；
//! 全部归属变更使用条件 UPDATE 抢占（WHERE assigned_to IS NULL），rows_affected=1 才算成功。

use sea_orm::{
    ColumnTrait, Condition, DbConn, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
    TransactionTrait,
};
use std::collections::HashSet;

use crate::core::errors::error::{Error, Result};
use crate::modules::crm::entity::{customer, customer_pool_config, customer_pool_member};
use crate::modules::crm::model::customer_pool::CustomerPoolMemberSimpleVO;
use crate::modules::crm::service::customer_pool_config_service;
use crate::modules::crm::service::customer_pool_member_service;
use crate::modules::crm::service::customer_pool_service;
use crate::modules::crm::service::customer_pool_workbench_service::{
    insert_history, notify_pool_admins, ACTION_ASSIGN, ACTION_AUTO_ASSIGN, ACTION_UNFROZEN,
};
use crate::modules::crm::service::lead_pool_ops_service::send_pool_notice;

fn map_db_err(e: sea_orm::DbErr) -> Error {
    Error::from(e.to_string())
}

fn now() -> chrono::NaiveDateTime {
    chrono::Local::now().naive_local()
}

fn err<T>(msg: impl Into<String>) -> Result<T> {
    Err(Error::from(msg.into()))
}

/// 指派请求（单条 = 一个元素；批量 = 多元素，均摊由前端拆条）
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerPoolAssignItem {
    #[serde(deserialize_with = "crate::utils::string_utils::deserialize_string_or_number_to_i64")]
    pub customer_id: i64,
    #[serde(deserialize_with = "crate::utils::string_utils::deserialize_string_or_number_to_i64")]
    pub to_user_id: i64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerPoolAssignRequest {
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_to_u64")]
    pub pool_id: Option<i64>,
    #[serde(default)]
    pub items: Vec<CustomerPoolAssignItem>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CustomerPoolFreezePageQuery {
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_to_u64")]
    pub pool_id: Option<i64>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// 候选成员视图（启用 + 在池；含当前私海持有数，供指派/自动分配校验与展示）
async fn active_members(db: &DbConn, pool_id: i64) -> Result<Vec<CustomerPoolMemberSimpleVO>> {
    let all = customer_pool_member_service::list_candidates(db, pool_id).await?;
    Ok(all)
}

/// 目标保有量校验（对齐 workbench claim：include_self_built 决定是否计自建客户）
async fn check_hold_limit(
    db: &DbConn,
    pool_id: i64,
    to_user_id: i64,
) -> Result<()> {
    let config = customer_pool_config_service::get_vo(db, pool_id).await?;
    let hold_limit = config.hold_limit.unwrap_or(0);
    if hold_limit <= 0 {
        return Ok(());
    }
    let mut holding_cond = Condition::all()
        .add(customer::Column::AssignedTo.eq(to_user_id))
        .add(customer::Column::Deleted.eq(0));
    if config.include_self_built != Some(1) {
        holding_cond = holding_cond.add(
            Condition::any()
                .add(customer::Column::FromPool.eq(1))
                .add(customer::Column::FromPool.is_null()),
        );
    }
    let holding = customer::Entity::find()
        .filter(holding_cond)
        .count(db)
        .await
        .map_err(map_db_err)?;
    if holding >= hold_limit as u64 {
        return err(format!("目标成员已达私海保有量上限（{} 条）", hold_limit));
    }
    Ok(())
}

/// 管理员指派（池内客户 → 指定池成员；批量逐条，部分成功不阻断）
///
/// 校验链：操作人为池管理员/超管 → 池启用 → 目标是启用状态的池成员 → 目标保有量 → 条件抢占
/// （WHERE assigned_to IS NULL AND release_frozen=0）→ 级联 → 留痕(3=管理员分配) → 通知。
pub async fn assign(
    db: &DbConn,
    req: &CustomerPoolAssignRequest,
    operator_id: i64,
) -> Result<Vec<crate::modules::crm::model::customer_pool::CustomerPoolOpResultVO>> {
    use crate::modules::crm::model::customer_pool::CustomerPoolOpResultVO;

    let pool_id = req.pool_id.ok_or_else(|| Error::from("请指定公海池".to_string()))?;
    if req.items.is_empty() {
        return err("请选择要指派的客户");
    }
    let pool = customer_pool_service::find_by_id(db, pool_id)
        .await?
        .ok_or_else(|| Error::from("客户公海池不存在".to_string()))?;
    if pool.status != Some(1) {
        return err("该公海池已停用");
    }
    let is_super = crate::modules::crm::service::delete_guard_service::is_super_admin(db, operator_id).await?;
    if !is_super && !customer_pool_service::is_pool_manager(db, pool_id, operator_id).await? {
        return err("仅池管理员或超级管理员可以指派");
    }

    let members = active_members(db, pool_id).await?;
    let member_map: HashSet<i64> = members
        .iter()
        .filter_map(|m| m.user_id)
        .collect();

    let mut results: Vec<CustomerPoolOpResultVO> = Vec::with_capacity(req.items.len());
    for item in &req.items {
        let one = assign_one(db, pool_id, item.customer_id, item.to_user_id, operator_id, &member_map).await;
        match one {
            Ok(title) => {
                results.push(CustomerPoolOpResultVO {
                    customer_id: Some(item.customer_id),
                    success: true,
                    message: "指派成功".to_string(),
                });
                let content = format!(
                    "<p>管理员向您指派了公海客户 <strong>{}</strong>，请及时跟进，避免超期被自动回收。</p>",
                    title
                );
                let _ = send_pool_notice(db, item.to_user_id, "公海客户指派通知", &content, operator_id).await;
            },
            Err(e) => {
                results.push(CustomerPoolOpResultVO {
                    customer_id: Some(item.customer_id),
                    success: false,
                    message: e.to_string(),
                });
            },
        }
    }
    Ok(results)
}

/// 单条指派（事务内条件 UPDATE 抢占）
async fn assign_one(
    db: &DbConn,
    pool_id: i64,
    customer_id: i64,
    to_user_id: i64,
    operator_id: i64,
    member_map: &HashSet<i64>,
) -> Result<String> {
    if !member_map.contains(&to_user_id) {
        return err("指派目标不是该公海池成员");
    }
    let customer_model = customer::Entity::find_by_id(customer_id)
        .filter(customer::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| Error::from("客户不存在".to_string()))?;
    if customer_model.release_frozen == Some(1) {
        return err("该客户已被冻结，无法指派");
    }
    if customer_model.assigned_to.is_some() {
        return err("该客户已被他人领取");
    }
    check_hold_limit(db, pool_id, to_user_id).await?;

    let title = customer_pool_workbench_display(&customer_model);
    let ts = now();
    let txn: sea_orm::DatabaseTransaction = db.begin().await.map_err(map_db_err)?;
    let fresh = customer::Entity::find_by_id(customer_id)
        .filter(customer::Column::Deleted.eq(0))
        .one(&txn)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| Error::from("客户不存在".to_string()))?;
    if fresh.assigned_to.is_some() || fresh.release_frozen == Some(1) {
        txn.rollback().await.ok();
        return err("该客户已被他人操作，请刷新后重试");
    }

    let payload = customer::ActiveModel {
        assigned_to: Set(Some(to_user_id)),
        assigned_at: Set(Some(ts.clone())),
        pool_id: Set(None),
        from_pool: Set(Some(1)),
        reminded_at: Set(None),
        recycle_extend_days: Set(Some(0)),
        updated_by: Set(Some(operator_id)),
        update_time: Set(Some(ts.clone())),
        ..Default::default()
    };
    let res = customer::Entity::update_many()
        .set(payload)
        .filter(customer::Column::Id.eq(customer_id))
        .filter(customer::Column::Deleted.eq(0))
        .filter(customer::Column::AssignedTo.is_null())
        .exec(&txn)
        .await
        .map_err(map_db_err)?;
    if res.rows_affected == 0 {
        txn.rollback().await.ok();
        return err("该客户已被他人操作，请刷新后重试");
    }

    crate::modules::crm::service::customer_service::cascade_update_related_assignees(
        &txn,
        customer_id,
        to_user_id,
    )
    .await?;

    insert_history(
        &txn,
        customer_id,
        to_user_id,
        ACTION_ASSIGN,
        Some(ts.clone()),
        None,
        Some(pool_id),
        "管理员指派".to_string(),
        None,
        None,
        operator_id,
    )
    .await?;

    txn.commit().await.map_err(map_db_err)?;
    Ok(title)
}

fn customer_pool_workbench_display(c: &customer::Model) -> String {
    c.company_name
        .clone()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| c.person_name.clone().filter(|s| !s.trim().is_empty()))
        .or_else(|| c.short_name.clone().filter(|s| !s.trim().is_empty()))
        .unwrap_or_else(|| format!("客户#{}", c.id))
}

// ==================== 冻结列表与恢复 ====================

/// 冻结客户分页
pub async fn frozen_page(
    db: &DbConn,
    query: &CustomerPoolFreezePageQuery,
) -> Result<ResultPageVO> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let mut cond = Condition::all()
        .add(customer::Column::Deleted.eq(0))
        .add(customer::Column::ReleaseFrozen.eq(1))
        .add(customer::Column::AssignedTo.is_null());
    if let Some(pid) = query.pool_id {
        cond = cond.add(customer::Column::PoolId.eq(pid));
    }
    let total = customer::Entity::find()
        .filter(cond.clone())
        .count(db)
        .await
        .map_err(map_db_err)?;
    let items = customer::Entity::find()
        .filter(cond)
        .order_by_desc(customer::Column::UpdateTime)
        .paginate(db, page_size as u64)
        .fetch_page((page - 1) as u64)
        .await
        .map_err(map_db_err)?;

    let rows = items
        .into_iter()
        .map(|c| FrozenCustomerVO {
            id: c.id,
            company_name: c.company_name,
            short_name: c.short_name,
            person_name: c.person_name,
            pool_id: c.pool_id,
            release_count: c.release_count,
            entered_pool_at: c.entered_pool_at,
        })
        .collect();
    Ok(ResultPageVO { total: total as i64, items: rows })
}

/// 恢复冻结客户（release_count 保留，仅解除冻结位）
pub async fn restore(
    db: &DbConn,
    customer_id: i64,
    operator_id: i64,
) -> Result<()> {
    let customer_model = customer::Entity::find_by_id(customer_id)
        .filter(customer::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| Error::from("客户不存在".to_string()))?;
    if customer_model.release_frozen != Some(1) {
        return err("该客户未处于冻结状态");
    }
    let pool_id = customer_model
        .pool_id
        .or(customer_model.source_pool_id)
        .unwrap_or(customer_pool_service::DEFAULT_CUSTOMER_POOL_ID);
    let is_super = crate::modules::crm::service::delete_guard_service::is_super_admin(db, operator_id).await?;
    if !is_super && !customer_pool_service::is_pool_manager(db, pool_id, operator_id).await? {
        return err("仅池管理员或超级管理员可以恢复冻结客户");
    }

    let ts = now();
    let payload = customer::ActiveModel {
        release_frozen: Set(Some(0)),
        updated_by: Set(Some(operator_id)),
        update_time: Set(Some(ts.clone())),
        ..Default::default()
    };
    let res = customer::Entity::update_many()
        .set(payload)
        .filter(customer::Column::Id.eq(customer_id))
        .filter(customer::Column::ReleaseFrozen.eq(1))
        .exec(db)
        .await
        .map_err(map_db_err)?;
    if res.rows_affected == 0 {
        return err("该客户冻结状态已变化，请刷新后重试");
    }

    insert_history(
        db,
        customer_id,
        customer_model.assigned_to.unwrap_or(0),
        ACTION_UNFROZEN,
        None,
        None,
        Some(pool_id),
        "管理员恢复冻结客户".to_string(),
        None,
        None,
        operator_id,
    )
    .await?;

    if let Some(owner) = customer_model.assigned_to {
        let _ = send_pool_notice(
            db,
            owner,
            "公海客户解冻通知",
            &format!("<p>客户 <strong>{}</strong> 已被管理员解除冻结，重新进入公海可被领取。</p>", customer_pool_workbench_display(&customer_model)),
            operator_id,
        )
        .await;
    }
    Ok(())
}

// ==================== 自动分配兜底扫描（job: customer_pool_auto_assign） ====================

/// 全部开启自动分配的池逐池扫描（对齐 lead_auto_assign_service::sweep_all 模式）
pub async fn sweep_all(db: &DbConn) -> Result<String> {
    let configs = customer_pool_config::Entity::find()
        .filter(customer_pool_config::Column::AutoAssignEnabled.eq(1i16))
        .all(db)
        .await
        .map_err(map_db_err)?;

    let mut total: usize = 0;
    let mut details: Vec<String> = Vec::new();
    for config in &configs {
        match sweep_pool(db, config.pool_id).await {
            Ok(n) if n > 0 => {
                total += n;
                details.push(format!("池#{} 分配 {} 条", config.pool_id, n));
            },
            Ok(_) => {},
            Err(e) => log::warn!("客户公海池 {} 兜底自动分配失败: {}", config.pool_id, e),
        }
    }

    if total > 0 {
        Ok(format!("公海未分配客户自动分配完成，共 {} 条（{}）", total, details.join("；")))
    } else {
        Ok("公海未分配客户自动分配完成，无待分配客户".to_string())
    }
}

/// 单池扫描：按轮询游标把未分配客户分给启用成员（权重模式 V1 退化为轮询，待规则条目引擎落地后生效）
async fn sweep_pool(db: &DbConn, pool_id: i64) -> Result<usize> {
    let pool = customer_pool_service::find_by_id(db, pool_id)
        .await?
        .ok_or_else(|| Error::from("客户公海池不存在".to_string()))?;
    if pool.status != Some(1) {
        return Ok(0);
    }
    let config = customer_pool_config_service::get_raw(db, pool_id)
        .await?
        .ok_or_else(|| Error::from(format!("客户公海池 {} 配置缺失", pool_id)))?;

    // 候选成员：启用状态，按用户 ID 稳定排序
    let member_rows = customer_pool_member::Entity::find()
        .filter(customer_pool_member::Column::PoolId.eq(pool_id))
        .all(db)
        .await
        .map_err(map_db_err)?;
    let mut member_ids: Vec<i64> = Vec::new();
    for m in member_rows {
        if let Some(uid) = m.user_id {
            let active = crate::modules::system::entity::admin::Entity::find_by_id(uid)
                .one(db)
                .await
                .map_err(map_db_err)?
                .map(|a| a.deleted.unwrap_or(0) == 0 && a.status.unwrap_or(0) == 1)
                .unwrap_or(false);
            if active {
                member_ids.push(uid);
            }
        }
    }
    member_ids.sort_unstable();
    if member_ids.is_empty() {
        return Ok(0);
    }

    // 未分配且未冻结的在池客户
    let pending = customer::Entity::find()
        .filter(
            Condition::all()
                .add(customer::Column::PoolId.eq(pool_id))
                .add(customer::Column::AssignedTo.is_null())
                .add(customer::Column::ReleaseFrozen.eq(0))
                .add(customer::Column::Deleted.eq(0)),
        )
        .all(db)
        .await
        .map_err(map_db_err)?;
    if pending.is_empty() {
        return Ok(0);
    }

    let mut assigned: usize = 0;
    let mut assigned_titles: Vec<String> = Vec::new();
    let mut new_cursor = config.assign_cursor.unwrap_or(0);
    for c in &pending {
        // 保有量校验：任一成员满载则跳过该成员，全部满载则停止
        // 规则引擎优先：首个匹配条目给出目标，否则回退池级轮询
        let mut target: Option<i64> = match_rule_target(db, pool_id, c).await.unwrap_or(None);
        if target.is_none() {
        for attempt in 0..member_ids.len() {
            let idx = (new_cursor as usize + attempt) % member_ids.len();
            let uid = member_ids[idx];
            if check_hold_limit(db, pool_id, uid).await.is_ok() {
                target = Some(uid);
                new_cursor = (new_cursor as usize + idx + 1) as i32;
                break;
            }
        }
        }
        let Some(to_user_id) = target else {
            break;
        };

        let ts = now();
        let payload = customer::ActiveModel {
            assigned_to: Set(Some(to_user_id)),
            assigned_at: Set(Some(ts.clone())),
            pool_id: Set(None),
            from_pool: Set(Some(1)),
            reminded_at: Set(None),
            recycle_extend_days: Set(Some(0)),
            update_time: Set(Some(ts.clone())),
            ..Default::default()
        };
        let res = customer::Entity::update_many()
            .set(payload)
            .filter(customer::Column::Id.eq(c.id))
            .filter(customer::Column::Deleted.eq(0))
            .filter(customer::Column::AssignedTo.is_null())
            .filter(customer::Column::ReleaseFrozen.eq(0))
            .exec(db)
            .await
            .map_err(map_db_err)?;
        if res.rows_affected == 0 {
            continue;
        }

        insert_history(
            db,
            c.id,
            to_user_id,
            ACTION_AUTO_ASSIGN,
            Some(ts.clone()),
            None,
            Some(pool_id),
            "自动分配".to_string(),
            None,
            None,
            0,
        )
        .await?;
        let _ = send_pool_notice(
            db,
            to_user_id,
            "公海客户自动分配通知",
            &format!(
                "<p>系统自动向您分配了公海客户 <strong>{}</strong>（来源：公海池自动分配规则），请及时跟进。</p>",
                customer_pool_workbench_display(c)
            ),
            0,
        )
        .await;
        assigned += 1;
        assigned_titles.push(customer_pool_workbench_display(c));
    }

    // 游标落库（乐观推进）
    if assigned > 0 {
        let _ = customer_pool_config::Entity::update_many()
            .col_expr(
                customer_pool_config::Column::AssignCursor,
                sea_orm::sea_query::Expr::value(new_cursor),
            )
            .filter(customer_pool_config::Column::PoolId.eq(pool_id))
            .filter(customer_pool_config::Column::AssignCursor.eq(config.assign_cursor.unwrap_or(0)))
            .exec(db)
            .await;
        let _ = notify_pool_admins(
            db,
            pool_id,
            "公海客户自动分配通知",
            &format!(
                "<p>本池自动分配了 <strong>{}</strong> 条客户：{}</p>",
                assigned,
                assigned_titles.iter().take(5).cloned().collect::<Vec<_>>().join("、")
            ),
            0,
        )
        .await;
    }
    Ok(assigned)
}

// ==================== 简单分页 VO（冻结列表） ====================

#[derive(Debug, serde::Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct FrozenCustomerVO {
    pub id: i64,
    pub company_name: Option<String>,
    pub short_name: Option<String>,
    pub person_name: Option<String>,
    pub pool_id: Option<i64>,
    pub release_count: Option<i32>,
    pub entered_pool_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ResultPageVO {
    pub total: i64,
    pub items: Vec<FrozenCustomerVO>,
}


// ==================== 自动分配规则引擎（B9：条件 + 目标 + 轮询/权重/指定） ====================

use crate::modules::crm::entity::{customer_pool_assign_rule, customer_pool_assign_rule_entry};
use crate::modules::crm::model::customer_pool::{
    AssignRuleListQuery, AssignRuleSaveRequest, AssignRuleVO,
};

/// 规则分页（含条目，条目按 sort 升序）
pub async fn rule_page(db: &DbConn, query: &AssignRuleListQuery) -> Result<Vec<AssignRuleVO>> {
    let mut cond = Condition::all().add(customer_pool_assign_rule::Column::Deleted.eq(0));
    if let Some(pid) = query.pool_id {
        cond = cond.add(customer_pool_assign_rule::Column::PoolId.eq(pid));
    }
    if let Some(en) = query.enabled {
        cond = cond.add(customer_pool_assign_rule::Column::Enabled.eq(en));
    }
    if let Some(kw) = query.keywords.as_deref().filter(|s| !s.trim().is_empty()) {
        cond = cond.add(customer_pool_assign_rule::Column::Name.contains(kw));
    }
    let rules = customer_pool_assign_rule::Entity::find()
        .filter(cond)
        .order_by_asc(customer_pool_assign_rule::Column::Priority)
        .all(db)
        .await
        .map_err(map_db_err)?;

    let mut vos: Vec<AssignRuleVO> = Vec::with_capacity(rules.len());
    for r in rules {
        let entries = customer_pool_assign_rule_entry::Entity::find()
            .filter(customer_pool_assign_rule_entry::Column::RuleId.eq(r.id))
            .order_by_asc(customer_pool_assign_rule_entry::Column::Sort)
            .all(db)
            .await
            .map_err(map_db_err)?;
        vos.push(AssignRuleVO {
            id: Some(r.id),
            pool_id: r.pool_id,
            pool_name: customer_pool_service::find_by_id(db, r.pool_id.unwrap_or(0))
                .await?
                .and_then(|p| p.name),
            name: r.name,
            trigger_event: r.trigger_event,
            priority: r.priority,
            enabled: r.enabled,
            create_time: r.create_time,
            update_time: r.update_time,
            entries: entries
                .into_iter()
                .map(|e| crate::modules::crm::model::customer_pool::AssignRuleEntryVO {
                    id: Some(e.id),
                    rule_id: e.rule_id,
                    conditions: e.conditions,
                    target_type: e.target_type,
                    target_ids: e.target_ids.and_then(|v| serde_json::from_value(v).ok()),
                    mode: e.mode,
                    weight: e.weight,
                    sort: e.sort,
                    create_time: e.create_time,
                })
                .collect(),
        });
    }
    Ok(vos)
}

/// 规则保存（整体覆盖条目；最多 10 条）
pub async fn rule_save(db: &DbConn, req: &AssignRuleSaveRequest, operator_id: i64) -> Result<i64> {
    if req.name.trim().is_empty() {
        return err("请输入规则名称");
    }
    if req.entries.len() > 10 {
        return err("每个规则最多 10 个条目");
    }
    let ts = now();
    let rule_id: i64 = if let Some(id) = req.id {
        let payload = customer_pool_assign_rule::ActiveModel {
            pool_id: Set(Some(req.pool_id)),
            name: Set(Some(req.name.clone())),
            trigger_event: Set(req.trigger_event),
            priority: Set(req.priority),
            enabled: Set(req.enabled),
            updated_by: Set(Some(operator_id)),
            update_time: Set(Some(ts.clone())),
            ..Default::default()
        };
        let res = customer_pool_assign_rule::Entity::update_many()
            .set(payload)
            .filter(customer_pool_assign_rule::Column::Id.eq(id))
            .filter(customer_pool_assign_rule::Column::Deleted.eq(0))
            .exec(db)
            .await
            .map_err(map_db_err)?;
        if res.rows_affected == 0 {
            return err("规则不存在或已删除");
        }
        customer_pool_assign_rule_entry::Entity::delete_many()
            .filter(customer_pool_assign_rule_entry::Column::RuleId.eq(id))
            .exec(db)
            .await
            .map_err(map_db_err)?;
        id
    } else {
        let payload = customer_pool_assign_rule::ActiveModel {
            pool_id: Set(Some(req.pool_id)),
            name: Set(Some(req.name.clone())),
            trigger_event: Set(req.trigger_event),
            priority: Set(Some(req.priority.unwrap_or(100))),
            enabled: Set(Some(req.enabled.unwrap_or(1))),
            created_by: Set(Some(operator_id)),
            create_time: Set(Some(ts.clone())),
            updated_by: Set(Some(operator_id)),
            update_time: Set(Some(ts)),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        customer_pool_assign_rule::Entity::insert(payload).exec(db).await.map_err(map_db_err)?.last_insert_id
    };

    for (i, e) in req.entries.iter().enumerate() {
        let payload = customer_pool_assign_rule_entry::ActiveModel {
            rule_id: Set(Some(rule_id)),
            conditions: Set(e.conditions.clone()),
            target_type: Set(e.target_type),
            target_ids: Set(e
                .target_ids
                .as_ref()
                .map(|ids| serde_json::to_value(ids).unwrap_or_default())),
            mode: Set(e.mode),
            weight: Set(e.weight.clone()),
            sort: Set(e.sort.or(Some(i as i32))),
            create_time: Set(Some(ts.clone())),
            ..Default::default()
        };
        customer_pool_assign_rule_entry::Entity::insert(payload).exec(db).await.map_err(map_db_err)?;
    }
    Ok(rule_id)
}

/// 规则删除（逻辑删规则 + 硬删条目）
pub async fn rule_delete(db: &DbConn, ids: &[i64], operator_id: i64) -> Result<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let ts = now();
    let payload = customer_pool_assign_rule::ActiveModel {
        deleted: Set(Some(1)),
        delete_time: Set(Some(ts.clone())),
        delete_by: Set(Some(operator_id)),
        update_time: Set(Some(ts)),
        ..Default::default()
    };
    customer_pool_assign_rule::Entity::update_many()
        .set(payload)
        .filter(customer_pool_assign_rule::Column::Id.is_in(ids.to_vec()))
        .exec(db)
        .await
        .map_err(map_db_err)?;
    customer_pool_assign_rule_entry::Entity::delete_many()
        .filter(customer_pool_assign_rule_entry::Column::RuleId.is_in(ids.to_vec()))
        .exec(db)
        .await
        .map_err(map_db_err)?;
    Ok(())
}

/// 条件匹配（field: source/industry/level 数值列、country 字符串；op: eq / in；空条件=全命中）
fn conditions_match(conditions: &serde_json::Value, c: &customer::Model) -> bool {
    let Some(list) = conditions.as_array() else {
        return true;
    };
    for cond in list {
        let field = cond.get("field").and_then(|v| v.as_str()).unwrap_or("");
        let op = cond.get("op").and_then(|v| v.as_str()).unwrap_or("eq");
        let value = cond.get("value");
        let matched = match field {
            "source" => c
                .source
                .map(|v| value_matches(&serde_json::json!(v), op, value))
                .unwrap_or(false),
            "industry" => c
                .industry
                .map(|v| value_matches(&serde_json::json!(v), op, value))
                .unwrap_or(false),
            "level" => c
                .level
                .map(|v| value_matches(&serde_json::json!(v), op, value))
                .unwrap_or(false),
            "country" => c
                .country
                .as_ref()
                .map(|v| value_matches(&serde_json::json!(v), op, value))
                .unwrap_or(false),
            _ => false,
        };
        if !matched {
            return false;
        }
    }
    true
}

fn value_matches(
    actual: &serde_json::Value,
    op: &str,
    expected: Option<&serde_json::Value>,
) -> bool {
    let Some(expected) = expected else { return false };
    match op {
        "eq" => actual == expected,
        "in" => expected
            .as_array()
            .map(|arr| arr.contains(actual))
            .unwrap_or(false),
        _ => false,
    }
}

/// 规则引擎：对池内单个未分配客户求目标负责人（规则按 priority 升序、条目按 sort 升序，首个匹配生效）
///
/// 返回 None = 无规则/无命中（调用方回退池级轮询）。目标可用性：必须为该池启用成员。
async fn match_rule_target(db: &DbConn, pool_id: i64, c: &customer::Model) -> Result<Option<i64>> {
    let rules = customer_pool_assign_rule::Entity::find()
        .filter(customer_pool_assign_rule::Column::PoolId.eq(pool_id))
        .filter(customer_pool_assign_rule::Column::Enabled.eq(1))
        .filter(customer_pool_assign_rule::Column::Deleted.eq(0))
        .order_by_asc(customer_pool_assign_rule::Column::Priority)
        .all(db)
        .await
        .map_err(map_db_err)?;
    if rules.is_empty() {
        return Ok(None);
    }

    let members = active_members(db, pool_id).await?;
    let member_ids: HashSet<i64> = members.iter().filter_map(|m| m.user_id).collect();
    if member_ids.is_empty() {
        return Ok(None);
    }

    for rule in rules {
        let entries = customer_pool_assign_rule_entry::Entity::find()
            .filter(customer_pool_assign_rule_entry::Column::RuleId.eq(rule.id))
            .order_by_asc(customer_pool_assign_rule_entry::Column::Sort)
            .all(db)
            .await
            .map_err(map_db_err)?;
        for entry in entries {
            if !conditions_match(
                entry.conditions.as_ref().unwrap_or(&serde_json::Value::Null),
                c,
            ) {
                continue;
            }
            let target_type = entry.target_type.unwrap_or(4);
            let target_ids: Vec<i64> = entry
                .target_ids
                .as_ref()
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            let mut candidates: Vec<i64> = match target_type {
                1 => target_ids.clone(),
                4 => member_ids.iter().cloned().collect(),
                _ => Vec::new(), // 职位/群组 V1 不支持
            };
            candidates.retain(|uid| member_ids.contains(uid));
            if candidates.is_empty() {
                continue;
            }

            let mode = entry.mode.unwrap_or(1);
            let picked = match mode {
                1 => candidates.first().cloned(),
                2 => {
                    // 轮询（V1 无独立游标存储，按当前私海持有数最小者近似公平）
                    let counts =
                        customer_pool_member_service::private_customer_counts(db, &candidates)
                            .await
                            .unwrap_or_default();
                    candidates
                        .into_iter()
                        .min_by_key(|uid| counts.get(uid).cloned().unwrap_or(0))
                },
                3 => {
                    // 权重：按 weight 加权随机（weight = {userId: 权重}）
                    let weights: Vec<(i64, i64)> = candidates
                        .iter()
                        .map(|uid| {
                            let w = entry
                                .weight
                                .as_ref()
                                .and_then(|w| w.get(&uid.to_string()))
                                .and_then(|v| v.as_i64())
                                .unwrap_or(1)
                                .max(1);
                            (*uid, w)
                        })
                        .collect();
                    let total: i64 = weights.iter().map(|(_, w)| *w).sum();
                    if total == 0 {
                        candidates.first().cloned()
                    } else {
                        let mut pick = chrono::Utc::now().timestamp_subsec_nanos() as i64 % total;
                        let mut chosen = None;
                        for (uid, w) in weights {
                            if pick < w {
                                chosen = Some(uid);
                                break;
                            }
                            pick -= w;
                        }
                        chosen.or_else(|| candidates.first().cloned())
                    }
                },
                _ => candidates.first().cloned(),
            };
            if let Some(uid) = picked {
                return Ok(Some(uid));
            }
        }
    }
    Ok(None)
}
