//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::crm::entity::{
    contract, customer, customer_assign_history, customer_pool, customer_pool_apply, customer_pool_member,
    customer_retain_apply,
};
use crate::modules::crm::model::customer_pool::{
    CustomerAssignHistoryVO, CustomerPoolApplyVO, CustomerPoolAuditPageQuery, CustomerPoolAuditRequest, CustomerPoolOpResultVO, CustomerPoolWorkbenchCustomerVO, CustomerPoolWorkbenchPageVO,
    CustomerPoolWorkbenchQuery, CustomerRetainApplyVO, CustomerWorkbenchApplyRequest, CustomerWorkbenchClaimRequest,
    CustomerWorkbenchReleaseRequest, CustomerWorkbenchRetainRequest,
};
use crate::modules::crm::service::customer_pool_config_service;
use crate::modules::crm::service::customer_pool_member_service;
use crate::modules::crm::service::customer_pool_service;
use crate::modules::crm::service::customer_service;
use crate::modules::crm::service::delete_guard_service;
use crate::modules::crm::service::lead_pool_ops_service::send_pool_notice;
use crate::modules::crm::service::pool_service::mask_contact_value;
use crate::modules::system::service::admin_service::build_admin_name_map;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DatabaseTransaction, DbConn, DbErr, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
};
use std::collections::{BTreeMap, HashMap, HashSet};

/// 归属历史操作类型（方案 §4.2 定稿：1=领取 2=退回 3=管理员分配 4=转移 5=自动分配
/// 6=自动回收 7=管理员收回 8=延期获批 9=冻结 10=解冻）
pub const ACTION_CLAIM: i16 = 1;
pub const ACTION_RELEASE: i16 = 2;
pub const ACTION_ASSIGN: i16 = 3;
pub const ACTION_TRANSFER: i16 = 4;
pub const ACTION_AUTO_ASSIGN: i16 = 5;
pub const ACTION_AUTO_RECYCLE: i16 = 6;
pub const ACTION_MANAGER_RETRIEVE: i16 = 7;
pub const ACTION_RETAIN_APPROVED: i16 = 8;
pub const ACTION_FROZEN: i16 = 9;
pub const ACTION_UNFROZEN: i16 = 10;

/// 申领/延期申请状态：1=待审批 2=通过 3=拒绝
pub const APPLY_STATUS_PENDING: i16 = 1;
pub const APPLY_STATUS_APPROVED: i16 = 2;
pub const APPLY_STATUS_REJECTED: i16 = 3;

/// 退回去向：1=原池 2=选择分组 3=指定分组
pub const RELEASE_BACK_TO_ORIGIN: i16 = 1;
pub const RELEASE_BACK_TO_PICK: i16 = 2;
pub const RELEASE_BACK_TO_SPECIFY: i16 = 3;

/// 有效合同状态（对齐 contract_status_enum 数值：1=草拟 2=履行中 3=已完成 4=已收款/完成 5=终止）
const CONTRACT_STATUS_SIGNED: i32 = 2;
const CONTRACT_STATUS_EXECUTING: i32 = 3;
const CONTRACT_STATUS_COMPLETED: i32 = 4;

fn map_db_err(e: DbErr) -> Error {
    Error::from(e.to_string())
}

fn err<T>(msg: impl Into<String>) -> Result<T> {
    Err(Error::from(msg.into()))
}

fn now() -> chrono::NaiveDateTime {
    chrono::Local::now().naive_local()
}

/// 客户展示名（公司名 → 个人姓名 → 简称 → 客户#id）
pub fn customer_display_name(c: &customer::Model) -> String {
    c.company_name
        .clone()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| c.person_name.clone().filter(|s| !s.trim().is_empty()))
        .or_else(|| c.short_name.clone().filter(|s| !s.trim().is_empty()))
        .unwrap_or_else(|| format!("客户#{}", c.id))
}

/// 归属历史操作类型文案（§4.2 定稿文案，历史轨迹与动作留痕共用）
pub fn action_label(action_type: i16) -> &'static str {
    match action_type {
        ACTION_CLAIM => "领取客户",
        ACTION_RELEASE => "退回公海",
        ACTION_ASSIGN => "管理员分配",
        ACTION_TRANSFER => "客户转移",
        ACTION_AUTO_ASSIGN => "系统自动分配",
        ACTION_AUTO_RECYCLE => "超期自动回收",
        ACTION_MANAGER_RETRIEVE => "管理员收回",
        ACTION_RETAIN_APPROVED => "延期获批",
        ACTION_FROZEN => "冻结",
        ACTION_UNFROZEN => "解冻",
        _ => "入池",
    }
}

/// 批量联查客户展示名（避免 N+1）
async fn customer_name_map(db: &DbConn, customer_ids: &[i64]) -> Result<HashMap<i64, String>> {
    if customer_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = customer::Entity::find()
        .filter(customer::Column::Id.is_in(customer_ids.to_vec()))
        .all(db)
        .await
        .map_err(map_db_err)?;
    Ok(rows
        .into_iter()
        .map(|c| (c.id, customer_display_name(&c)))
        .collect())
}

/// 批量联查池名称（缺失回退 公海池#id）
async fn pool_name_map(db: &DbConn, pool_ids: &[i64]) -> Result<HashMap<i64, String>> {
    if pool_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = customer_pool::Entity::find()
        .filter(customer_pool::Column::Id.is_in(pool_ids.to_vec()))
        .all(db)
        .await
        .map_err(map_db_err)?;
    Ok(rows
        .into_iter()
        .map(|p| (p.id, p.name.unwrap_or_else(|| format!("公海池#{}", p.id))))
        .collect())
}

/// 指定池的管理员 ID 集合（成员行 member_type=1 ∪ 池的 pool_admin_id），去重升序
pub(crate) async fn pool_admin_ids(db: &DbConn, pool_id: i64) -> Result<Vec<i64>> {
    let mut ids: Vec<i64> = customer_pool_member::Entity::find()
        .filter(customer_pool_member::Column::PoolId.eq(pool_id))
        .filter(customer_pool_member::Column::MemberType.eq(1))
        .all(db)
        .await
        .map_err(map_db_err)?
        .into_iter()
        .filter_map(|m| m.user_id)
        .collect();
    if let Some(pool) = customer_pool_service::find_by_id(db, pool_id).await? {
        if let Some(aid) = pool.pool_admin_id {
            ids.push(aid);
        }
    }
    ids.sort_unstable();
    ids.dedup();
    Ok(ids)
}

/// 向指定池的管理员广播站内信（操作人本人跳过）
pub(crate) async fn notify_pool_admins(
    db: &DbConn,
    pool_id: i64,
    title: &str,
    content: &str,
    operator_id: i64,
) -> Result<()> {
    for uid in pool_admin_ids(db, pool_id).await? {
        if uid != operator_id {
            let _ = send_pool_notice(db, uid, title, content, operator_id).await;
        }
    }
    Ok(())
}

/// 归属历史插入（对齐 lead_pool_ops_service::insert_history 模式；create_time 内部落值）
#[allow(clippy::too_many_arguments)]
pub(crate) async fn insert_history(
    txn: &impl ConnectionTrait,
    customer_id: i64,
    admin_id: i64,
    action_type: i16,
    start_time: Option<chrono::NaiveDateTime>,
    end_time: Option<chrono::NaiveDateTime>,
    pool_id: Option<i64>,
    remark: String,
    reason_type: Option<i16>,
    reason: Option<String>,
    operated_by: i64,
) -> Result<()> {
    let ts = now();
    let payload = customer_assign_history::ActiveModel {
        customer_id: Set(Some(customer_id)),
        admin_id: Set(Some(admin_id)),
        action_type: Set(Some(action_type)),
        start_time: Set(start_time.or(Some(ts.clone()))),
        end_time: Set(end_time),
        remark: Set(Some(remark)),
        reason_type: Set(reason_type),
        reason: Set(reason),
        pool_id: Set(pool_id),
        operated_by: Set(Some(operated_by)),
        create_time: Set(Some(ts)),
        ..Default::default()
    };
    customer_assign_history::Entity::insert(payload)
        .exec(txn)
        .await
        .map_err(map_db_err)?;
    Ok(())
}

/// 关闭指定负责人在负归属记录（冷却期以 end_time 为锚点；无在负记录时补插关闭流水）
#[allow(clippy::too_many_arguments)]
pub(crate) async fn close_open_history(
    txn: &DatabaseTransaction,
    customer_id: i64,
    owner_id: i64,
    action_type: i16,
    reason_type: Option<i16>,
    reason: &Option<String>,
    operated_by: i64,
    pool_id: i64,
) -> Result<()> {
    let current = customer_assign_history::Entity::find()
        .filter(customer_assign_history::Column::CustomerId.eq(customer_id))
        .filter(customer_assign_history::Column::AdminId.eq(owner_id))
        .filter(customer_assign_history::Column::EndTime.is_null())
        .one(txn)
        .await
        .map_err(map_db_err)?;

    let ts = now();
    let reason_text = reason.clone().unwrap_or_default();
    match current {
        Some(record) => {
            let mut active: customer_assign_history::ActiveModel = record.into();
            active.end_time = Set(Some(ts));
            active.reason_type = Set(reason_type);
            active.reason = Set(reason.clone());
            active.remark = Set(Some(action_label(action_type).to_string()));
            active.pool_id = Set(Some(pool_id));
            active.operated_by = Set(Some(operated_by));
            active.update(txn).await.map_err(map_db_err)?;
        },
        None => {
            insert_history(
                txn,
                customer_id,
                owner_id,
                action_type,
                Some(ts.clone()),
                Some(ts),
                Some(pool_id),
                action_label(action_type).to_string(),
                reason_type,
                Some(reason_text),
                operated_by,
            )
            .await?;
        },
    }
    Ok(())
}

/// 在池未分配客户列表（工作台，§5.4 口径：pool_id 命中 + 未分配 + 未删除 + 未冻结）
///
/// 可见范围：指定池=池成员/超管；跨池=超管全池、成员=所在启用池。
/// 行级脱敏：普通成员看不到客户详情（无 allow_detail 且非该池管理员/超管）时，
/// 按所属池配置的 mask_fields 对联系方式与公司名脱敏。
pub async fn pool_customers(
    db: &DbConn,
    query: &CustomerPoolWorkbenchQuery,
    user_id: i64,
) -> Result<CustomerPoolWorkbenchPageVO> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let default_vo = CustomerPoolWorkbenchPageVO {
        total: 0,
        items: Vec::new(),
        claim_mode: None,
        mask_fields: Vec::new(),
        release_back_to: RELEASE_BACK_TO_ORIGIN,
        allow_detail: false,
        can_view_detail: false,
        can_claim: false,
    };

    let is_super = delete_guard_service::is_super_admin(db, user_id).await?;
    let member_rows = customer_pool_member::Entity::find()
        .filter(customer_pool_member::Column::UserId.eq(user_id))
        .all(db)
        .await
        .map_err(map_db_err)?;
    let member_pool_ids: HashSet<i64> = member_rows.iter().filter_map(|m| m.pool_id).collect();
    let managed_pool_ids: HashSet<i64> = member_rows
        .iter()
        .filter(|m| m.member_type == Some(1))
        .filter_map(|m| m.pool_id)
        .collect();

    // 可见池范围
    let mut scope: Vec<i64> = if let Some(pid) = query.pool_id {
        let pool = customer_pool_service::find_by_id(db, pid)
            .await?
            .ok_or_else(|| Error::from("客户公海池不存在".to_string()))?;
        if pool.status != Some(1) {
            return err("该客户公海池已停用，无法查看");
        }
        if !is_super && !member_pool_ids.contains(&pid) {
            return err("您不是该公海池的成员，无法查看");
        }
        vec![pid]
    } else if is_super {
        let pools = customer_pool::Entity::find()
            .filter(customer_pool::Column::Deleted.eq(0))
            .filter(customer_pool::Column::Status.eq(1))
            .all(db)
            .await
            .map_err(map_db_err)?;
        pools.into_iter().map(|p| p.id).collect()
    } else {
        let pools = customer_pool::Entity::find()
            .filter(customer_pool::Column::Deleted.eq(0))
            .filter(customer_pool::Column::Status.eq(1))
            .filter(customer_pool::Column::Id.is_in(member_pool_ids.iter().copied().collect::<Vec<i64>>()))
            .all(db)
            .await
            .map_err(map_db_err)?;
        pools.into_iter().map(|p| p.id).collect()
    };
    scope.sort_unstable();
    scope.dedup();
    if scope.is_empty() {
        return Ok(default_vo);
    }

    // 池配置聚合：跨池任一池开启即隐藏；deal_status 取首个非空
    let config_map = customer_pool_service::load_config_map(db, &scope).await?;
    let mut hide_claimed = false;
    let mut hide_converted = false;
    let mut deal_status: Vec<i32> = Vec::new();
    for pid in &scope {
        if let Some(cfg) = config_map.get(pid).cloned().flatten() {
            if cfg.hide_claimed == Some(1) {
                hide_claimed = true;
            }
            if cfg.hide_converted == Some(1) {
                hide_converted = true;
            }
            if deal_status.is_empty() {
                if let Some(ref ds) = cfg.deal_status {
                    if !ds.is_empty() {
                        deal_status = ds.iter().map(|v| *v as i32).collect();
                    }
                }
            }
        }
    }
    if deal_status.is_empty() {
        deal_status = vec![CONTRACT_STATUS_SIGNED, CONTRACT_STATUS_EXECUTING, CONTRACT_STATUS_COMPLETED];
    }

    let mut select = customer::Entity::find()
        .filter(customer::Column::Deleted.eq(0))
        .filter(customer::Column::AssignedTo.is_null())
        .filter(
            Condition::any()
                .add(customer::Column::ReleaseFrozen.is_null())
                .add(customer::Column::ReleaseFrozen.eq(0)),
        );
    let mut pool_cond = Condition::any().add(customer::Column::PoolId.is_in(scope.clone()));
    if scope.contains(&customer_pool_service::DEFAULT_CUSTOMER_POOL_ID) {
        // 存量数据 pool_id NULL ≡ 默认池
        pool_cond = pool_cond.add(customer::Column::PoolId.is_null());
    }
    select = select.filter(pool_cond);

    if let Some(kw) = query.keywords.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        let pattern = format!("%{}%", kw);
        select = select.filter(
            Condition::any()
                .add(customer::Column::CompanyName.like(pattern.clone()))
                .add(customer::Column::ShortName.like(pattern.clone()))
                .add(customer::Column::PersonName.like(pattern)),
        );
    }

    // 隐藏过滤（两段式 IN 查询，规避子查询 API 差异）
    let mut hidden: HashSet<i64> = HashSet::new();
    if hide_claimed {
        let claimed_rows = customer_assign_history::Entity::find()
            .select_only()
            .column(customer_assign_history::Column::CustomerId)
            .column(customer_assign_history::Column::Id)
            .filter(customer_assign_history::Column::AdminId.eq(user_id))
            .filter(customer_assign_history::Column::CustomerId.is_not_null())
            .into_tuple::<(Option<i64>, i64)>()
            .all(db)
            .await
            .map_err(map_db_err)?;
        hidden.extend(claimed_rows.into_iter().filter_map(|(cid, _)| cid));
    }
    if hide_converted {
        let converted_rows = contract::Entity::find()
            .select_only()
            .column(contract::Column::CustomerId)
            .column(contract::Column::Id)
            .filter(contract::Column::Deleted.eq(0))
            .filter(contract::Column::Status.is_in(deal_status))
            .filter(contract::Column::CustomerId.is_not_null())
            .into_tuple::<(Option<i64>, i64)>()
            .all(db)
            .await
            .map_err(map_db_err)?;
        hidden.extend(converted_rows.into_iter().filter_map(|(cid, _)| cid));
    }
    if !hidden.is_empty() {
        let hidden_ids: Vec<i64> = hidden.into_iter().collect();
        select = select.filter(customer::Column::Id.is_not_in(hidden_ids));
    }

    let paginator = select
        .order_by_desc(customer::Column::EnteredPoolAt)
        .order_by_desc(customer::Column::Id)
        .paginate(db, page_size as u64);
    let total = paginator.num_items().await.map_err(map_db_err)? as i64;
    let rows = paginator.fetch_page((page - 1) as u64).await.map_err(map_db_err)?;

    // 页级操作权限与配置回显（单池=该池配置；跨池=聚合默认）
    let single_pool_cfg = if scope.len() == 1 {
        config_map.get(&scope[0]).cloned().flatten()
    } else {
        None
    };
    let claim_mode = single_pool_cfg.as_ref().and_then(|c| c.claim_mode);
    let mask_fields = single_pool_cfg
        .as_ref()
        .and_then(|c| c.mask_fields.clone())
        .unwrap_or_default();
    let release_back_to = single_pool_cfg
        .as_ref()
        .and_then(|c| c.release_back_to)
        .unwrap_or(RELEASE_BACK_TO_ORIGIN);
    let allow_detail = single_pool_cfg.as_ref().and_then(|c| c.allow_detail).unwrap_or(0) == 1;
    let can_view_detail = is_super
        || !managed_pool_ids.is_empty()
        || allow_detail
        || scope
            .iter()
            .any(|pid| config_map.get(pid).cloned().flatten().map_or(false, |c| c.allow_detail == Some(1)));
    let can_claim = if let Some(pid) = query.pool_id {
        let mode = single_pool_cfg.as_ref().and_then(|c| c.claim_mode).unwrap_or(1);
        member_pool_ids.contains(&pid) && mode == 1
    } else {
        false
    };

    // 行级脱敏组装
    let items: Vec<CustomerPoolWorkbenchCustomerVO> = rows
        .into_iter()
        .map(|c| {
            let eff_pool = c.pool_id.unwrap_or(customer_pool_service::DEFAULT_CUSTOMER_POOL_ID);
            let is_manager_of_row_pool = managed_pool_ids.contains(&eff_pool);
            let row_cfg = config_map.get(&eff_pool).cloned().flatten();
            let row_allow_detail = row_cfg.as_ref().and_then(|c| c.allow_detail).unwrap_or(0) == 1;
            let mask = !is_super && !is_manager_of_row_pool && !row_allow_detail;
            let fields: Vec<String> = row_cfg
                .as_ref()
                .and_then(|c| c.mask_fields.clone())
                .unwrap_or_default();

            let mask_value = |field: &str, value: &Option<String>| -> Option<String> {
                if !mask || !fields.iter().any(|f| f == field) {
                    return value.clone();
                }
                value.as_ref().map(|v| mask_contact_value(if field == "personal_email" { "email" } else { "mobile" }, v))
            };

            CustomerPoolWorkbenchCustomerVO {
                id: Some(c.id),
                company_name: mask_value("company_name", &c.company_name),
                short_name: c.short_name,
                person_name: c.person_name,
                industry: c.industry,
                level: c.level,
                source: c.source,
                wechat: mask_value("wechat", &c.wechat),
                personal_mobile: mask_value("personal_mobile", &c.personal_mobile),
                personal_email: mask_value("personal_email", &c.personal_email),
                pool_id: Some(eff_pool),
                entered_pool_at: c.entered_pool_at,
                release_count: c.release_count,
            }
        })
        .collect();

    Ok(CustomerPoolWorkbenchPageVO {
        total,
        items,
        claim_mode,
        mask_fields,
        release_back_to,
        allow_detail,
        can_view_detail,
        can_claim,
    })
}

/// 批量领取（§5.4 领取校验链；逐条返回明细，部分成功不阻断）
pub async fn claim(
    db: &DbConn,
    req: &CustomerWorkbenchClaimRequest,
    user_id: i64,
) -> Result<Vec<CustomerPoolOpResultVO>> {
    let ids = req.customer_ids.clone();
    if ids.is_empty() {
        return err("请选择要领取的客户");
    }

    let mut results: Vec<CustomerPoolOpResultVO> = Vec::with_capacity(ids.len());
    let mut claimed_by_pool: BTreeMap<i64, Vec<String>> = BTreeMap::new();

    for cid in &ids {
        match claim_one(db, *cid, user_id, false, ACTION_CLAIM).await {
            Ok((title, pool_id)) => {
                results.push(CustomerPoolOpResultVO {
                    customer_id: Some(*cid),
                    success: true,
                    message: "领取成功".to_string(),
                });
                claimed_by_pool.entry(pool_id).or_default().push(title);
            },
            Err(e) => {
                results.push(CustomerPoolOpResultVO {
                    customer_id: Some(*cid),
                    success: false,
                    message: e.to_string(),
                });
            },
        }
    }

    // 领取成功汇总通知领取人
    let claimed_total: usize = claimed_by_pool.values().map(|v| v.len()).sum();
    if claimed_total > 0 {
        let preview: Vec<String> = claimed_by_pool.values().flatten().take(5).cloned().collect();
        let mut content = format!(
            "<p>您已成功领取 <strong>{}</strong> 条公海客户，负责人责任已生效：</p><ul>{}</ul><p>请及时跟进，避免超期被自动回收。</p>",
            claimed_total,
            preview.iter().map(|t| format!("<li>{}</li>", t)).collect::<String>()
        );
        if claimed_total > 5 {
            content.push_str(&format!("<p>…等共 {} 条客户</p>", claimed_total));
        }
        let _ = send_pool_notice(db, user_id, "公海客户领取成功通知", &content, user_id).await;
        // 池管理员知情（操作人本人跳过）
        for (pid, titles) in &claimed_by_pool {
            let content = format!(
                "<p>池成员领取了 <strong>{}</strong> 条公海客户：</p><ul>{}</ul>",
                titles.len(),
                titles.iter().take(5).map(|t| format!("<li>{}</li>", t)).collect::<String>()
            );
            let _ = notify_pool_admins(db, *pid, "公海客户领取通知", &content, user_id).await;
        }
    }

    Ok(results)
}

/// 单条领取：事务外校验 + 事务内条件 UPDATE 抢占 + 归属历史，成功返回 (客户标题, 池ID)
///
/// * `bypass_claim_mode` - true=跳过领取模式拦截（申领审批通过后执行领取复用）
/// * `history_action` - 归属历史动作类型（自主领取=1）
async fn claim_one(
    db: &DbConn,
    customer_id: i64,
    user_id: i64,
    bypass_claim_mode: bool,
    history_action: i16,
) -> Result<(String, i64)> {
    let customer_model = customer::Entity::find_by_id(customer_id)
        .filter(customer::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| Error::from("客户不存在".to_string()))?;

    if customer_model.release_frozen == Some(1) {
        return err("该客户已被冻结，无法领取");
    }
    if customer_model.assigned_to.is_some() {
        return err("该客户已被他人领取");
    }

    let pool_id = customer_model.pool_id.unwrap_or(customer_pool_service::DEFAULT_CUSTOMER_POOL_ID);
    let pool = customer_pool_service::find_by_id(db, pool_id)
        .await?
        .ok_or_else(|| Error::from("客户公海池不存在".to_string()))?;
    if pool.status != Some(1) {
        return err("该客户所在公海池已停用，无法领取");
    }
    if !customer_pool_member_service::is_member(db, pool_id, user_id).await? {
        return err("您不是该公海池的成员，无法领取");
    }

    let config = customer_pool_config_service::get_vo(db, pool_id).await?;
    if !bypass_claim_mode {
        match config.claim_mode.unwrap_or(1) {
            2 => return err("该池为申领模式，请提交申领申请"),
            3 => return err("该池已停用自主领取，请联系管理员分配"),
            _ => {},
        }
    }

    // 日领取上限（成员级覆盖优先，决策点 #16）
    let my_member = customer_pool_member::Entity::find()
        .filter(customer_pool_member::Column::PoolId.eq(pool_id))
        .filter(customer_pool_member::Column::UserId.eq(user_id))
        .one(db)
        .await
        .map_err(map_db_err)?;
    let daily_limit = my_member
        .as_ref()
        .and_then(|m| m.claim_daily_limit_override)
        .unwrap_or(config.claim_daily_limit.unwrap_or(0));
    if daily_limit > 0 {
        let today_start = chrono::Local::now().date_naive().and_hms_opt(0, 0, 0).unwrap_or_default();
        let claimed_today = customer_assign_history::Entity::find()
            .filter(customer_assign_history::Column::AdminId.eq(user_id))
            .filter(customer_assign_history::Column::ActionType.eq(ACTION_CLAIM))
            .filter(customer_assign_history::Column::PoolId.eq(pool_id))
            .filter(customer_assign_history::Column::StartTime.gte(today_start))
            .count(db)
            .await
            .map_err(map_db_err)?;
        if claimed_today >= daily_limit as u64 {
            return err(format!("已达今日领取上限（{} 条），请明日再试", daily_limit));
        }
    }

    // 私海保有量上限（自建客户是否占额度随池配置）
    let hold_limit = config.hold_limit.unwrap_or(0);
    if hold_limit > 0 {
        let mut holding_cond = Condition::all()
            .add(customer::Column::AssignedTo.eq(user_id))
            .add(customer::Column::Deleted.eq(0));
        if config.include_self_built != Some(1) {
            holding_cond = holding_cond.add(
                Condition::any()
                    .add(customer::Column::FromPool.eq(1))
                    .add(customer::Column::FromPool.is_null()),
            );
        }
        let holding = customer::Entity::find().filter(holding_cond).count(db).await.map_err(map_db_err)?;
        if holding >= hold_limit as u64 {
            return err(format!("已达私海客户保有量上限（{} 条），请先跟进或退回部分客户", hold_limit));
        }
    }

    // 冷却期（原负责人退回后不得立即再领）
    let cool_down_days = config.cool_down_days.unwrap_or(0);
    if cool_down_days > 0 {
        if let Some(last) = customer_assign_history::Entity::find()
            .filter(customer_assign_history::Column::CustomerId.eq(customer_id))
            .filter(customer_assign_history::Column::AdminId.eq(user_id))
            .filter(customer_assign_history::Column::EndTime.is_not_null())
            .order_by_desc(customer_assign_history::Column::EndTime)
            .one(db)
            .await
            .map_err(map_db_err)?
        {
            if let Some(end_time) = last.end_time {
                let deadline = end_time + chrono::Duration::days(cool_down_days as i64);
                let ts = now();
                if deadline > ts {
                    let remain = (deadline - ts).num_days() + 1;
                    return err(format!("该客户处于冷却期，约 {} 天后可再次领取", remain));
                }
            }
        }
    }

    let title = customer_display_name(&customer_model);
    let ts = now();

    let txn: DatabaseTransaction = db.begin().await.map_err(map_db_err)?;
    let fresh = customer::Entity::find_by_id(customer_id)
        .filter(customer::Column::Deleted.eq(0))
        .one(&txn)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| Error::from("客户不存在".to_string()))?;
    if fresh.assigned_to.is_some() || fresh.release_frozen == Some(1) {
        txn.rollback().await.ok();
        return err("该客户已被他人领取");
    }

    let payload = customer::ActiveModel {
        assigned_to: Set(Some(user_id)),
        assigned_at: Set(Some(ts.clone())),
        pool_id: Set(None),
        // 领取即视为公海来源，防止其占用自建客户删除时限等逻辑
        from_pool: Set(Some(1)),
        // 领取=新的跟进周期，清空到期提醒幂等标记
        reminded_at: Set(None),
        recycle_extend_days: Set(Some(0)),
        updated_by: Set(Some(user_id)),
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
        return err("该客户已被他人领取");
    }

    // 级联接管商机/跟进等关联记录的负责人
    customer_service::cascade_update_related_assignees(&txn, customer_id, user_id).await?;

    insert_history(
        &txn,
        customer_id,
        user_id,
        history_action,
        Some(ts.clone()),
        None,
        Some(pool_id),
        if history_action == ACTION_CLAIM { "领取客户".to_string() } else { "领取客户".to_string() },
        None,
        None,
        user_id,
    )
    .await?;

    txn.commit().await.map_err(map_db_err)?;
    Ok((title, pool_id))
}

/// 申领审批通过后执行领取（复用领取校验链，跳过 claim_mode 拦截）
pub async fn claim_for_apply_audit(db: &DbConn, customer_id: i64, user_id: i64) -> Result<()> {
    claim_one(db, customer_id, user_id, true, ACTION_CLAIM).await.map(|_| ())
}

/// 退回公海（§5.4 退回；负责人本人或池管理员/超管，逐条事务）
///
/// 退回去向按池配置：1=原池 2=选择分组 3=指定分组（2/3 需传 target_pool_id）。
/// 决策点 #14：冻结计数仅计被动回收，负责人主动退回不增加 release_count。
pub async fn release(
    db: &DbConn,
    req: &CustomerWorkbenchReleaseRequest,
    user_id: i64,
) -> Result<Vec<CustomerPoolOpResultVO>> {
    let ids = req.customer_ids.clone();
    if ids.is_empty() {
        return err("请选择要退回的客户");
    }
    if req.reason_type.is_none() {
        return err("请选择退回原因");
    }
    if req.reason_type == Some(9) {
        let reason = req.reason.clone().unwrap_or_default();
        if reason.trim().is_empty() {
            return err("请填写退回补充说明");
        }
    }

    let is_super = delete_guard_service::is_super_admin(db, user_id).await?;
    let mut results: Vec<CustomerPoolOpResultVO> = Vec::with_capacity(ids.len());

    for cid in &ids {
        match release_one(db, *cid, req, user_id, is_super).await {
            Ok(()) => {
                results.push(CustomerPoolOpResultVO {
                    customer_id: Some(*cid),
                    success: true,
                    message: "退回成功".to_string(),
                });
            },
            Err(e) => {
                results.push(CustomerPoolOpResultVO {
                    customer_id: Some(*cid),
                    success: false,
                    message: e.to_string(),
                });
            },
        }
    }
    Ok(results)
}

/// 单条退回（事务内条件 UPDATE 抢占；目标池校验通过后写入）
async fn release_one(
    db: &DbConn,
    customer_id: i64,
    req: &CustomerWorkbenchReleaseRequest,
    user_id: i64,
    is_super: bool,
) -> Result<()> {
    let customer_model = customer::Entity::find_by_id(customer_id)
        .filter(customer::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| Error::from("客户不存在".to_string()))?;

    let owner = customer_model.assigned_to.ok_or_else(|| Error::from("该客户已在公海，无需退回".to_string()))?;
    let eff_pool = customer_model.pool_id.unwrap_or(customer_pool_service::DEFAULT_CUSTOMER_POOL_ID);

    // 越权校验：负责人本人 / 池管理员 / 超管
    if owner != user_id && !is_super && !customer_pool_service::is_pool_manager(db, eff_pool, user_id).await? {
        return err("您无权退回该客户");
    }

    let is_manager_op = owner != user_id;
    let action_type = if is_manager_op { ACTION_MANAGER_RETRIEVE } else { ACTION_RELEASE };

    let config = customer_pool_config_service::get_vo(db, eff_pool).await?;
    let back_to = config.release_back_to.unwrap_or(RELEASE_BACK_TO_ORIGIN);
    let target_pool_id = match back_to {
        RELEASE_BACK_TO_PICK | RELEASE_BACK_TO_SPECIFY => req
            .target_pool_id
            .ok_or_else(|| Error::from("请选择退回的目标公海池".to_string()))?,
        _ => customer_model.source_pool_id.unwrap_or(eff_pool),
    };
    let target_pool = customer_pool_service::find_by_id(db, target_pool_id)
        .await?
        .ok_or_else(|| Error::from("目标公海池不存在".to_string()))?;
    if target_pool.status != Some(1) {
        return err("目标公海池已停用");
    }

    let ts = now();
    let txn: DatabaseTransaction = db.begin().await.map_err(map_db_err)?;
    let fresh = customer::Entity::find_by_id(customer_id)
        .filter(customer::Column::Deleted.eq(0))
        .one(&txn)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| Error::from("客户不存在".to_string()))?;
    if fresh.assigned_to != Some(owner) {
        txn.rollback().await.ok();
        return err("客户归属已变化，请刷新后重试");
    }

    let payload = customer::ActiveModel {
        pool_id: Set(Some(target_pool_id)),
        source_pool_id: Set(Some(target_pool_id)),
        assigned_to: Set(None),
        assigned_at: Set(None),
        entered_pool_at: Set(Some(ts.clone())),
        reminded_at: Set(None),
        recycle_extend_days: Set(Some(0)),
        updated_by: Set(Some(user_id)),
        update_time: Set(Some(ts.clone())),
        ..Default::default()
    };
    let res = customer::Entity::update_many()
        .set(payload)
        .filter(customer::Column::Id.eq(customer_id))
        .filter(customer::Column::Deleted.eq(0))
        .filter(customer::Column::AssignedTo.eq(owner))
        .exec(&txn)
        .await
        .map_err(map_db_err)?;
    if res.rows_affected == 0 {
        txn.rollback().await.ok();
        return err("客户归属已变化，请刷新后重试");
    }

    // 冻结计数（决策点 #14）：管理员收回属被动退回计入 release_count，达阈值触发冻结并写留痕(9)
    let mut manager_frozen = false;
    if is_manager_op {
        let threshold = config.freeze_release_count.unwrap_or(0);
        if threshold > 0 {
            let new_count = fresh.release_count.unwrap_or(0) + 1;
            manager_frozen = new_count >= threshold;
            let upd = customer::Entity::update_many()
                .set(customer::ActiveModel {
                    release_count: Set(Some(new_count)),
                    release_frozen: Set(if manager_frozen { Some(1) } else { None }),
                    update_time: Set(Some(ts.clone())),
                    updated_by: Set(Some(user_id)),
                    ..Default::default()
                })
                .filter(customer::Column::Id.eq(customer_id))
                .filter(customer::Column::Deleted.eq(0))
                .exec(&txn)
                .await
                .map_err(map_db_err)?;
            if upd.rows_affected > 0 && manager_frozen {
                insert_history(
                    &txn,
                    customer_id,
                    owner,
                    ACTION_FROZEN,
                    Some(ts.clone()),
                    Some(ts.clone()),
                    Some(target_pool_id),
                    "触发冻结".to_string(),
                    None,
                    Some(format!("连续 {} 次被动收回/回收，已冻结", new_count)),
                    user_id,
                )
                .await?;
            }
        }
    }

    close_open_history(
        &txn,
        customer_id,
        owner,
        action_type,
        req.reason_type,
        &req.reason,
        user_id,
        target_pool_id,
    )
    .await?;
    txn.commit().await.map_err(map_db_err)?;

    // 退回/收回通知原负责人（本人操作不通知）
    if owner != user_id {
        let reason_text = req.reason.clone().unwrap_or_default();
        let rt = match req.reason_type {
            Some(1) => "跟进无回应",
            Some(2) => "客户无意向",
            Some(3) => "客户信息无效",
            Some(4) => "换业务方向",
            _ => "其他",
        };
        let mut content = format!(
            "<p>您的客户<strong>【{}】</strong>已退回公海。</p><p>退回原因：{}{}</p><p>冷却期内您将无法再次领取该客户，请知悉。</p>",
            customer_display_name(&customer_model),
            rt,
            if reason_text.trim().is_empty() { String::new() } else { format!("（{}）", reason_text) }
        );
        if manager_frozen {
            content.push_str("<p>因连续多次被动收回，该客户已<strong>冻结</strong>，需联系管理员解冻后方可再次领取。</p>");
        }
        let _ = send_pool_notice(
            db,
            owner,
            if is_manager_op { "客户收回通知" } else { "客户退回公海通知" },
            &content,
            user_id,
        )
        .await;
    }
    Ok(())
}

/// 提交申领申请（claim_mode=2 池；池管理员审批）
pub async fn apply(db: &DbConn, req: &CustomerWorkbenchApplyRequest, user_id: i64) -> Result<()> {
    let customer_model = customer::Entity::find_by_id(req.customer_id)
        .filter(customer::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| Error::from("客户不存在".to_string()))?;
    if customer_model.release_frozen == Some(1) {
        return err("该客户已被冻结，无法申领");
    }
    if customer_model.assigned_to.is_some() {
        return err("该客户已被他人领取");
    }
    let pool_id = customer_model.pool_id.unwrap_or(customer_pool_service::DEFAULT_CUSTOMER_POOL_ID);

    let pool = customer_pool_service::find_by_id(db, pool_id)
        .await?
        .ok_or_else(|| Error::from("客户公海池不存在".to_string()))?;
    if pool.status != Some(1) {
        return err("该客户所在公海池已停用");
    }
    if !customer_pool_member_service::is_member(db, pool_id, user_id).await? {
        return err("您不是该公海池的成员，无法申领");
    }
    let config = customer_pool_config_service::get_vo(db, pool_id).await?;
    if config.claim_mode.unwrap_or(1) != 2 {
        return err("该池无需申领，可直接领取");
    }

    // 去重：同一客户 + 同一申请人仅允许一条待审批申请
    let dup = customer_pool_apply::Entity::find()
        .filter(customer_pool_apply::Column::CustomerId.eq(req.customer_id))
        .filter(customer_pool_apply::Column::UserId.eq(user_id))
        .filter(customer_pool_apply::Column::Status.eq(APPLY_STATUS_PENDING))
        .count(db)
        .await
        .map_err(map_db_err)?;
    if dup > 0 {
        return err("您已提交该客户的申领申请，请等待审批");
    }

    let ts = now();
    let payload = customer_pool_apply::ActiveModel {
        pool_id: Set(Some(pool_id)),
        customer_id: Set(Some(req.customer_id)),
        user_id: Set(Some(user_id)),
        status: Set(Some(APPLY_STATUS_PENDING)),
        reason: Set(req.reason.clone()),
        create_time: Set(Some(ts)),
        ..Default::default()
    };
    customer_pool_apply::Entity::insert(payload)
        .exec(db)
        .await
        .map_err(map_db_err)?;

    let content = format!(
        "<p>池成员申请领取公海客户<strong>【{}】</strong>，请及时审批。</p>{}",
        customer_display_name(&customer_model),
        req.reason.as_ref().map(|r| format!("<p>申领理由：{}</p>", r)).unwrap_or_default()
    );
    let _ = notify_pool_admins(db, pool_id, "公海客户申领申请", &content, user_id).await;
    Ok(())
}

/// 申领申请分页（默认查待审批；可按池过滤）
pub async fn apply_page(
    db: &DbConn,
    query: &CustomerPoolAuditPageQuery,
) -> Result<ResultPage<Vec<CustomerPoolApplyVO>>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let status = query.status.unwrap_or(APPLY_STATUS_PENDING);

    let mut select = customer_pool_apply::Entity::find()
        .filter(customer_pool_apply::Column::Status.eq(status));
    if let Some(pid) = query.pool_id {
        select = select.filter(customer_pool_apply::Column::PoolId.eq(pid));
    }
    let paginator = select
        .order_by_desc(customer_pool_apply::Column::CreateTime)
        .paginate(db, page_size as u64);
    let total = paginator.num_items().await.map_err(map_db_err)? as i64;
    let rows = paginator.fetch_page((page - 1) as u64).await.map_err(map_db_err)?;

    if rows.is_empty() {
        return Ok(ResultPage::new(Vec::new(), 0, page, page_size));
    }
    let customer_ids: Vec<i64> = rows.iter().filter_map(|r| r.customer_id).collect();
    let pool_ids: Vec<i64> = rows.iter().filter_map(|r| r.pool_id).collect();
    let admin_ids: Vec<i64> = rows
        .iter()
        .flat_map(|r| [r.user_id, r.audit_by].into_iter().flatten())
        .collect();
    let customer_names = customer_name_map(db, &customer_ids).await?;
    let pool_names = pool_name_map(db, &pool_ids).await?;
    let admin_names = build_admin_name_map(db, admin_ids).await;

    let items: Vec<CustomerPoolApplyVO> = rows
        .into_iter()
        .map(|r| {
            let pool_id = r.pool_id;
            CustomerPoolApplyVO {
                id: Option::from(r.id),
                pool_id,
                pool_name: pool_id.and_then(|pid| pool_names.get(&pid).cloned()),
                customer_id: r.customer_id,
                customer_name: r.customer_id.and_then(|cid| customer_names.get(&cid).cloned()),
                user_id: r.user_id,
                user_name: r.user_id.and_then(|uid| admin_names.get(&uid).cloned()),
                reason: r.reason,
                status: r.status,
                audit_by: r.audit_by,
                audit_by_name: r.audit_by.and_then(|aid| admin_names.get(&aid).cloned()),
                audit_remark: r.audit_remark,
                audit_time: r.audit_time,
                create_time: r.create_time,
            }
        })
        .collect();
    Ok(ResultPage::new(items, total, page, page_size))
}

/// 申领审批（对齐 lead 侧惯例：通过=先条件置 APPROVED 再执行领取，领取失败回滚待审）
pub async fn audit_apply(db: &DbConn, req: &CustomerPoolAuditRequest, auditor_id: i64) -> Result<()> {
    let apply_model = customer_pool_apply::Entity::find_by_id(req.id)
        .one(db)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| Error::from("申领申请不存在".to_string()))?;
    if apply_model.status != Some(APPLY_STATUS_PENDING) {
        return err("该申领申请已处理");
    }
    let customer_id = apply_model.customer_id.unwrap_or(0);
    let applicant_id = apply_model.user_id.unwrap_or(0);

    if req.pass {
        // 条件置通过（防并发重复审批）
        let res = customer_pool_apply::Entity::update_many()
            .set(customer_pool_apply::ActiveModel {
                status: Set(Some(APPLY_STATUS_APPROVED)),
                audit_by: Set(Some(auditor_id)),
                audit_time: Set(Some(now())),
                audit_remark: Set(req.remark.clone()),
                ..Default::default()
            })
            .filter(customer_pool_apply::Column::Id.eq(req.id))
            .filter(customer_pool_apply::Column::Status.eq(APPLY_STATUS_PENDING))
            .exec(db)
            .await
            .map_err(map_db_err)?;
        if res.rows_affected == 0 {
            return err("该申领申请已处理");
        }
        match claim_for_apply_audit(db, customer_id, applicant_id).await {
            Ok(()) => {
                let _ = send_pool_notice(
                    db,
                    applicant_id,
                    "公海客户申领审批通过通知",
                    "<p>您的公海客户申领申请已审批通过，客户已领取到您的私海，请及时跟进。</p>",
                    auditor_id,
                )
                .await;
                Ok(())
            },
            Err(e) => {
                // 领取失败回滚至待审，提示申请人状态未变
                let _ = customer_pool_apply::Entity::update_many()
                    .set(customer_pool_apply::ActiveModel {
                        status: Set(Some(APPLY_STATUS_PENDING)),
                        audit_by: Set(None),
                        audit_time: Set(None),
                        audit_remark: Set(None),
                        ..Default::default()
                    })
                    .filter(customer_pool_apply::Column::Id.eq(req.id))
                    .filter(customer_pool_apply::Column::Status.eq(APPLY_STATUS_APPROVED))
                    .exec(db)
                    .await;
                err(format!("审批通过但领取失败，已退回待审：{}", e))
            },
        }
    } else {
        let res = customer_pool_apply::Entity::update_many()
            .set(customer_pool_apply::ActiveModel {
                status: Set(Some(APPLY_STATUS_REJECTED)),
                audit_by: Set(Some(auditor_id)),
                audit_time: Set(Some(now())),
                audit_remark: Set(req.remark.clone()),
                ..Default::default()
            })
            .filter(customer_pool_apply::Column::Id.eq(req.id))
            .filter(customer_pool_apply::Column::Status.eq(APPLY_STATUS_PENDING))
            .exec(db)
            .await
            .map_err(map_db_err)?;
        if res.rows_affected == 0 {
            return err("该申领申请已处理");
        }
        let _ = send_pool_notice(
            db,
            applicant_id,
            "公海客户申领审批驳回通知",
            &format!(
                "<p>您的公海客户申领申请已被驳回。</p><p>审批备注：{}</p>",
                req.remark.clone().unwrap_or_else(|| "无".to_string())
            ),
            auditor_id,
        )
        .await;
        Ok(())
    }
}

/// 提交延期申请（现负责人发起；池管理员审批）
pub async fn retain_apply(db: &DbConn, req: &CustomerWorkbenchRetainRequest, user_id: i64) -> Result<()> {
    let customer_model = customer::Entity::find_by_id(req.customer_id)
        .filter(customer::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| Error::from("客户不存在".to_string()))?;
    if customer_model.assigned_to != Some(user_id) {
        return err("仅客户负责人可提交延期申请");
    }
    let pool_id = customer_model.pool_id.unwrap_or(customer_pool_service::DEFAULT_CUSTOMER_POOL_ID);
    let pool = customer_pool_service::find_by_id(db, pool_id)
        .await?
        .ok_or_else(|| Error::from("客户公海池不存在".to_string()))?;
    if pool.status != Some(1) {
        return err("客户所在公海池已停用");
    }
    let config = customer_pool_config_service::get_vo(db, pool_id).await?;
    let recycle_days = config.recycle_days.unwrap_or(0);
    if recycle_days <= 0 {
        return err("该池未开启未跟进回收规则，无需申请延期");
    }
    if req.extend_days <= 0 || req.extend_days > recycle_days {
        return err(format!("延期天数需在 1 ~ {} 天之间", recycle_days));
    }
    let current_extend = customer_model.recycle_extend_days.unwrap_or(0);
    if let Some(max_days) = config.max_recycle_days {
        if max_days > 0 && current_extend + req.extend_days > max_days {
            return err(format!("累计延期后保护期超过最长限制（{} 天）", max_days));
        }
    }
    // 去重：同一客户 + 同一申请人仅允许一条待审批申请
    let dup = customer_retain_apply::Entity::find()
        .filter(customer_retain_apply::Column::CustomerId.eq(req.customer_id))
        .filter(customer_retain_apply::Column::UserId.eq(user_id))
        .filter(customer_retain_apply::Column::Status.eq(APPLY_STATUS_PENDING))
        .count(db)
        .await
        .map_err(map_db_err)?;
    if dup > 0 {
        return err("您已提交该客户的延期申请，请等待审批");
    }

    let ts = now();
    let payload = customer_retain_apply::ActiveModel {
        customer_id: Set(Some(req.customer_id)),
        user_id: Set(Some(user_id)),
        extend_days: Set(Some(req.extend_days)),
        status: Set(Some(APPLY_STATUS_PENDING)),
        reason: Set(req.reason.clone()),
        create_time: Set(Some(ts)),
        ..Default::default()
    };
    customer_retain_apply::Entity::insert(payload)
        .exec(db)
        .await
        .map_err(map_db_err)?;

    let content = format!(
        "<p>客户<strong>【{}】</strong>负责人申请回收保护期顺延 <strong>{}</strong> 天，请及时审批。</p>{}",
        customer_display_name(&customer_model),
        req.extend_days,
        req.reason.as_ref().map(|r| format!("<p>申请理由：{}</p>", r)).unwrap_or_default()
    );
    let _ = notify_pool_admins(db, pool_id, "客户延期申请", &content, user_id).await;
    Ok(())
}

/// 延期申请分页（默认查待审批；延期表无池维度，池名经客户行联查）
pub async fn retain_page(
    db: &DbConn,
    query: &CustomerPoolAuditPageQuery,
) -> Result<ResultPage<Vec<CustomerRetainApplyVO>>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let status = query.status.unwrap_or(APPLY_STATUS_PENDING);

    let select = customer_retain_apply::Entity::find()
        .filter(customer_retain_apply::Column::Status.eq(status))
        .order_by_desc(customer_retain_apply::Column::CreateTime);
    let paginator = select.paginate(db, page_size as u64);
    let total = paginator.num_items().await.map_err(map_db_err)? as i64;
    let rows = paginator.fetch_page((page - 1) as u64).await.map_err(map_db_err)?;

    if rows.is_empty() {
        return Ok(ResultPage::new(Vec::new(), 0, page, page_size));
    }
    let customer_ids: Vec<i64> = rows.iter().filter_map(|r| r.customer_id).collect();
    let admin_ids: Vec<i64> = rows
        .iter()
        .flat_map(|r| [r.user_id, r.audit_by].into_iter().flatten())
        .collect();
    let customer_names = customer_name_map(db, &customer_ids).await?;
    let admin_names = build_admin_name_map(db, admin_ids).await;
    let customers = customer::Entity::find()
        .filter(customer::Column::Id.is_in(customer_ids))
        .all(db)
        .await
        .map_err(map_db_err)?;
    let customer_pool_map: HashMap<i64, i64> = customers
        .into_iter()
        .map(|c| (c.id, c.pool_id.unwrap_or(customer_pool_service::DEFAULT_CUSTOMER_POOL_ID)))
        .collect();
    let pool_ids: Vec<i64> = customer_pool_map.values().copied().collect();
    let pool_names = pool_name_map(db, &pool_ids).await?;

    let items: Vec<CustomerRetainApplyVO> = rows
        .into_iter()
        .map(|r| {
            let pid = r.customer_id.and_then(|cid| customer_pool_map.get(&cid).copied());
            CustomerRetainApplyVO {
                id: Option::from(r.id),
                customer_id: r.customer_id,
                customer_name: r.customer_id.and_then(|cid| customer_names.get(&cid).cloned()),
                pool_id: pid,
                pool_name: pid.and_then(|p| pool_names.get(&p).cloned()),
                user_id: r.user_id,
                user_name: r.user_id.and_then(|uid| admin_names.get(&uid).cloned()),
                extend_days: r.extend_days,
                reason: r.reason,
                status: r.status,
                audit_by: r.audit_by,
                audit_by_name: r.audit_by.and_then(|aid| admin_names.get(&aid).cloned()),
                audit_remark: r.audit_remark,
                audit_time: r.audit_time,
                create_time: r.create_time,
            }
        })
        .collect();
    Ok(ResultPage::new(items, total, page, page_size))
}

/// 延期审批（通过=事务内重算 recycle_extend_days + 清提醒标记 + 历史留痕）
pub async fn audit_retain(db: &DbConn, req: &CustomerPoolAuditRequest, auditor_id: i64) -> Result<()> {
    let apply_model = customer_retain_apply::Entity::find_by_id(req.id)
        .one(db)
        .await
        .map_err(map_db_err)?
        .ok_or_else(|| Error::from("延期申请不存在".to_string()))?;
    if apply_model.status != Some(APPLY_STATUS_PENDING) {
        return err("该延期申请已处理");
    }
    let customer_id = apply_model.customer_id.unwrap_or(0);
    let applicant_id = apply_model.user_id.unwrap_or(0);
    let extend_days = apply_model.extend_days.unwrap_or(0);

    if req.pass {
        let res = customer_retain_apply::Entity::update_many()
            .set(customer_retain_apply::ActiveModel {
                status: Set(Some(APPLY_STATUS_APPROVED)),
                audit_by: Set(Some(auditor_id)),
                audit_time: Set(Some(now())),
                audit_remark: Set(req.remark.clone()),
                ..Default::default()
            })
            .filter(customer_retain_apply::Column::Id.eq(req.id))
            .filter(customer_retain_apply::Column::Status.eq(APPLY_STATUS_PENDING))
            .exec(db)
            .await
            .map_err(map_db_err)?;
        if res.rows_affected == 0 {
            return err("该延期申请已处理");
        }

        let ts = now();
        let txn: DatabaseTransaction = db.begin().await.map_err(map_db_err)?;
        let fresh = customer::Entity::find_by_id(customer_id)
            .filter(customer::Column::Deleted.eq(0))
            .one(&txn)
            .await
            .map_err(map_db_err)?
            .ok_or_else(|| Error::from("客户不存在".to_string()))?;
        if fresh.assigned_to != Some(applicant_id) {
            txn.rollback().await.ok();
            let _ = rollback_retain_status(db, req.id).await;
            return err("客户负责人已变化，无法批准延期，申请已退回待审");
        }
        let new_extend = fresh.recycle_extend_days.unwrap_or(0) + extend_days;
        let upd = customer::Entity::update_many()
            .set(customer::ActiveModel {
                recycle_extend_days: Set(Some(new_extend)),
                reminded_at: Set(None),
                updated_by: Set(Some(auditor_id)),
                update_time: Set(Some(ts.clone())),
                ..Default::default()
            })
            .filter(customer::Column::Id.eq(customer_id))
            .filter(customer::Column::Deleted.eq(0))
            .filter(customer::Column::AssignedTo.eq(applicant_id))
            .exec(&txn)
            .await
            .map_err(map_db_err)?;
        if upd.rows_affected == 0 {
            txn.rollback().await.ok();
            let _ = rollback_retain_status(db, req.id).await;
            return err("客户负责人已变化，无法批准延期，申请已退回待审");
        }
        let eff_pool = fresh.pool_id.unwrap_or(customer_pool_service::DEFAULT_CUSTOMER_POOL_ID);
        insert_history(
            &txn,
            customer_id,
            applicant_id,
            ACTION_RETAIN_APPROVED,
            Some(ts.clone()),
            Some(ts.clone()),
            Some(eff_pool),
            format!("延期 {} 天获批", extend_days),
            None,
            None,
            auditor_id,
        )
        .await?;
        txn.commit().await.map_err(map_db_err)?;

        let _ = send_pool_notice(
            db,
            applicant_id,
            "客户延期审批通过通知",
            &format!(
                "<p>您的客户延期申请已审批通过，回收保护期顺延 <strong>{}</strong> 天。</p>",
                extend_days
            ),
            auditor_id,
        )
        .await;
        Ok(())
    } else {
        let res = customer_retain_apply::Entity::update_many()
            .set(customer_retain_apply::ActiveModel {
                status: Set(Some(APPLY_STATUS_REJECTED)),
                audit_by: Set(Some(auditor_id)),
                audit_time: Set(Some(now())),
                audit_remark: Set(req.remark.clone()),
                ..Default::default()
            })
            .filter(customer_retain_apply::Column::Id.eq(req.id))
            .filter(customer_retain_apply::Column::Status.eq(APPLY_STATUS_PENDING))
            .exec(db)
            .await
            .map_err(map_db_err)?;
        if res.rows_affected == 0 {
            return err("该延期申请已处理");
        }
        let _ = send_pool_notice(
            db,
            applicant_id,
            "客户延期审批驳回通知",
            &format!(
                "<p>您的客户延期申请已被驳回。</p><p>审批备注：{}</p>",
                req.remark.clone().unwrap_or_else(|| "无".to_string())
            ),
            auditor_id,
        )
        .await;
        Ok(())
    }
}

/// 延期审批失败回滚（状态恢复待审并清空审批字段）
async fn rollback_retain_status(db: &DbConn, apply_id: i64) -> Result<()> {
    customer_retain_apply::Entity::update_many()
        .set(customer_retain_apply::ActiveModel {
            status: Set(Some(APPLY_STATUS_PENDING)),
            audit_by: Set(None),
            audit_time: Set(None),
            audit_remark: Set(None),
            ..Default::default()
        })
        .filter(customer_retain_apply::Column::Id.eq(apply_id))
        .filter(customer_retain_apply::Column::Status.eq(APPLY_STATUS_APPROVED))
        .exec(db)
        .await
        .map_err(map_db_err)?;
    Ok(())
}

/// 客户归属轨迹时间轴（?customer_id=，按开始时间倒序）
pub async fn trace(db: &DbConn, customer_id: i64) -> Result<Vec<CustomerAssignHistoryVO>> {
    let records = customer_assign_history::Entity::find()
        .filter(customer_assign_history::Column::CustomerId.eq(customer_id))
        .order_by_desc(customer_assign_history::Column::StartTime)
        .all(db)
        .await
        .map_err(map_db_err)?;
    if records.is_empty() {
        return Ok(Vec::new());
    }

    let customer_name = customer::Entity::find_by_id(customer_id)
        .one(db)
        .await
        .map_err(map_db_err)?
        .map(|c| customer_display_name(&c));

    let all_ids: Vec<i64> = records
        .iter()
        .flat_map(|r| [r.admin_id, r.operated_by].into_iter().flatten())
        .collect();
    let admin_names = build_admin_name_map(db, all_ids).await;

    let pool_ids: Vec<i64> = records.iter().filter_map(|r| r.pool_id).collect();
    let pool_names = pool_name_map(db, &pool_ids).await?;

    let data: Vec<CustomerAssignHistoryVO> = records
        .into_iter()
        .map(|item| {
            let pool_id = item.pool_id;
            let action_type = item.action_type;
            let mut vo = CustomerAssignHistoryVO {
                id: Option::from(item.id),
                customer_id: item.customer_id,
                customer_name: customer_name.clone(),
                admin_id: item.admin_id,
                admin_name: None,
                pool_id,
                pool_name: pool_id.and_then(|pid| pool_names.get(&pid).cloned()),
                action_type,
                action_label: action_label(action_type.unwrap_or(0)).to_string(),
                start_time: item.start_time,
                end_time: item.end_time,
                remark: item.remark,
                reason_type: item.reason_type,
                reason: item.reason,
                operated_by: item.operated_by,
                operated_by_name: None,
                create_time: item.create_time,
            };
            if let Some(aid) = vo.admin_id {
                vo.admin_name = admin_names.get(&aid).cloned();
            }
            if let Some(oid) = vo.operated_by {
                vo.operated_by_name = admin_names.get(&oid).cloned();
            }
            vo
        })
        .collect();
    Ok(data)
}
