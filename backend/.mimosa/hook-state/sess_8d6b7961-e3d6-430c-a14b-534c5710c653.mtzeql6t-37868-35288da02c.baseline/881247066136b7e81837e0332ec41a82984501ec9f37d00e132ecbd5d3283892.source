//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use chrono::{NaiveDate, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DbConn, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
};

use crate::core::errors::error::{Error, Result};
use crate::modules::approval::entity::approval_instance;
use crate::modules::approval::entity::approval_instance::{
    Column as InstanceColumn, Entity as InstanceEntity,
};
use crate::modules::approval::model::approval::ApprovalSubmitRequest;
use crate::modules::approval::service::approval_service::ApprovalService;
use crate::modules::message::service::notification_service::NotificationService;
use crate::modules::system::entity::{
    admin, admin_role_merge, hr_resign_item_template, hr_resign_record, hr_resign_transfer_item,
    role,
};
use crate::modules::system::entity::admin::Column as AdminColumn;

/// 交接单状态：1交接中 2交接完成 3结算完成 4已离职 5已中止
pub const RESIGN_STATUS_TRANSFERRING: i32 = 1;
pub const RESIGN_STATUS_TRANSFERRED: i32 = 2;
pub const RESIGN_STATUS_SETTLED: i32 = 3;
pub const RESIGN_STATUS_DONE: i32 = 4;
pub const RESIGN_STATUS_ABORTED: i32 = 5;

/// 交接项状态：0待确认 1已确认 2不适用
pub const ITEM_STATUS_PENDING: i32 = 0;
pub const ITEM_STATUS_CONFIRMED: i32 = 1;
pub const ITEM_STATUS_NA: i32 = 2;

/// 解析交接项确认人：1=交接人 2=系统管理员 3=指定角色
async fn resolve_assignee(
    db: &impl ConnectionTrait,
    rule: Option<i32>,
    role_id: Option<i64>,
    transfer_to: Option<i64>,
) -> Result<Option<i64>> {
    match rule {
        Some(1) => Ok(transfer_to),
        Some(2) => find_system_admin(db).await,
        Some(3) => find_first_user_by_role(db, role_id).await,
        _ => Ok(None),
    }
}

/// 查找系统管理员（user_type=1）中第一个在职账号
async fn find_system_admin(db: &impl ConnectionTrait) -> Result<Option<i64>> {
    let row = admin::Entity::find()
        .filter(admin::Column::UserType.eq(1))
        .filter(admin::Column::Status.eq(1))
        .filter(admin::Column::Deleted.eq(0))
        .order_by_asc(admin::Column::Id)
        .one(db)
        .await?;
    Ok(row.map(|a| a.id))
}

/// 查找指定角色下第一个在职账号
async fn find_first_user_by_role(
    db: &impl ConnectionTrait,
    role_id: Option<i64>,
) -> Result<Option<i64>> {
    let Some(role_id) = role_id else {
        return Ok(None);
    };
    let row = admin_role_merge::Entity::find()
        .filter(admin_role_merge::Column::RoleId.eq(role_id))
        .order_by_asc(admin_role_merge::Column::AdminId)
        .one(db)
        .await?;
    Ok(row.and_then(|m| m.admin_id))
}

/// B10：按角色 key 列表查找在职账号（去重），用于站内通知人事/财务等角色成员
async fn find_users_by_role_keys(
    db: &impl ConnectionTrait,
    role_keys: &[&str],
) -> Result<Vec<i64>> {
    let roles = role::Entity::find()
        .filter(role::Column::RoleKey.is_in(role_keys.iter().map(|s| s.to_string())))
        .filter(role::Column::Deleted.eq(0))
        .all(db)
        .await?;
    let role_ids: Vec<i64> = roles.into_iter().map(|r| r.id).collect();
    if role_ids.is_empty() {
        return Ok(Vec::new());
    }
    let merges = admin_role_merge::Entity::find()
        .filter(admin_role_merge::Column::RoleId.is_in(role_ids))
        .all(db)
        .await?;
    let mut ids: Vec<i64> = Vec::new();
    for m in merges {
        if let Some(uid) = m.admin_id {
            if !ids.contains(&uid) {
                ids.push(uid);
            }
        }
    }
    // 过滤已删除/停用账号
    let mut valid: Vec<i64> = Vec::new();
    for uid in ids {
        if let Ok(Some(a)) = admin::Entity::find_by_id(uid).one(db).await {
            if a.deleted.unwrap_or(0) == 0 && a.status == Some(1) {
                valid.push(uid);
            }
        }
    }
    Ok(valid)
}

/// B5：离职审批通过（实例 status=3）后，事务内创建交接单（主表+子表+按启用模板初始化交接项）。
/// 需在审批事务内调用：失败会随事务整体回滚，保证不出现"审批通过但无交接单"的中间态。
pub async fn create_handover_after_approval(
    db: &impl ConnectionTrait,
    instance_id: i64,
) -> Result<()> {
    let inst = approval_instance::Entity::find_by_id(instance_id)
        .one(db)
        .await?
        .ok_or_else(|| Error::from("审批实例不存在"))?;

    let admin_id = inst.business_id.unwrap_or_default();
    if admin_id <= 0 {
        return Err(Error::from("离职审批业务ID无效"));
    }

    // 防重复：已有未关闭交接单（1/2/3）则跳过（发起校验已保证，此处为兜底）
    let existing = hr_resign_record::Entity::find()
        .filter(hr_resign_record::Column::AdminId.eq(admin_id))
        .filter(hr_resign_record::Column::Status.is_in([
            RESIGN_STATUS_TRANSFERRING,
            RESIGN_STATUS_TRANSFERRED,
            RESIGN_STATUS_SETTLED,
        ]))
        .one(db)
        .await?;
    if existing.is_some() {
        return Ok(());
    }

    // 申请内容从审批实例 extra_data 读取（B4 发起时写入）
    let extra = inst.extra_data.clone().unwrap_or_else(|| serde_json::json!({}));
    let resign_type = extra
        .get("resign_type")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .unwrap_or(1);
    let resign_date = extra
        .get("resign_date")
        .and_then(|v| v.as_str())
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    let reason = extra.get("reason").and_then(|v| v.as_str()).map(str::to_string);
    let transfer_to_admin_id = extra.get("transfer_to_admin_id").and_then(|v| v.as_i64());

    let now = Some(Utc::now().naive_utc());
    let create_by = inst.submitter_name.clone().unwrap_or_else(|| "system".to_string());

    // 主表
    let record = hr_resign_record::ActiveModel {
        admin_id: Set(admin_id),
        transfer_to_admin_id: Set(transfer_to_admin_id),
        resign_type: Set(Some(resign_type)),
        resign_date: Set(resign_date),
        actual_leave_date: Set(None),
        reason: Set(reason),
        status: Set(Some(RESIGN_STATUS_TRANSFERRING)),
        create_by: Set(Some(create_by.clone())),
        create_time: Set(now),
        update_by: Set(None),
        update_time: Set(now),
        ..Default::default()
    };
    let record_id = record.insert(db).await.map_err(|e| Error::from(e.to_string()))?.id;

    // 交接项：按启用模板生成
    let templates = hr_resign_item_template::Entity::find()
        .filter(hr_resign_item_template::Column::Enabled.eq(1))
        .order_by_asc(hr_resign_item_template::Column::Sort)
        .all(db)
        .await?;
    for tpl in templates {
        let assignee_id =
            resolve_assignee(db, tpl.assignee_rule, tpl.assignee_role_id, transfer_to_admin_id)
                .await?;
        let item = hr_resign_transfer_item::ActiveModel {
            record_id: Set(record_id),
            item_key: Set(tpl.item_key.clone()),
            item_name: Set(tpl.item_name.clone()),
            assignee_id: Set(assignee_id),
            status: Set(Some(ITEM_STATUS_PENDING)),
            confirm_remark: Set(None),
            confirm_time: Set(None),
            create_time: Set(now),
            update_time: Set(now),
            ..Default::default()
        };
        item.insert(db).await.map_err(|e| Error::from(e.to_string()))?;
    }

    Ok(())
}

/// B10：交接任务通知（审批通过生成交接单后 / 超时提醒时复用）
/// 通知离职员工进行中交接单（status=1）各待确认交接项的 assignee（去重），best-effort。
/// admin_id：离职员工ID（resign 审批实例的 business_id）。
pub async fn notify_handover_assignees(db: &DbConn, admin_id: i64) -> Result<()> {
    let Some(record) = hr_resign_record::Entity::find()
        .filter(hr_resign_record::Column::AdminId.eq(admin_id))
        .filter(hr_resign_record::Column::Status.eq(RESIGN_STATUS_TRANSFERRING))
        .one(db)
        .await?
    else {
        return Ok(());
    };
    let items = hr_resign_transfer_item::Entity::find()
        .filter(hr_resign_transfer_item::Column::RecordId.eq(record.id))
        .filter(hr_resign_transfer_item::Column::Status.eq(ITEM_STATUS_PENDING))
        .all(db)
        .await?;
    if items.is_empty() {
        return Ok(());
    }
    let emp_name = admin_name_by_id(db, Some(admin_id)).await;
    let mut notified: Vec<i64> = Vec::new();
    for it in items {
        let Some(aid) = it.assignee_id else { continue };
        if notified.contains(&aid) {
            continue;
        }
        notified.push(aid);
        let _ = NotificationService::send_system_notification(
            db,
            aid,
            format!("【离职交接】{} 的离职交接待您确认", emp_name),
            format!(
                "员工 {} 的离职交接单已生成，您负责的交接项【{}】待确认，请及时处理。",
                emp_name,
                it.item_name.clone().unwrap_or_default()
            ),
            9,
            Some("/system/user".to_string()),
        )
        .await;
    }
    Ok(())
}

/// 校验员工无可进行中的离职审批/交接（防重复发起）
async fn ensure_no_active_resign(db: &DbConn, admin_id: i64) -> Result<()> {
    let inst_cnt = InstanceEntity::find()
        .filter(InstanceColumn::BusinessType.eq("resign"))
        .filter(InstanceColumn::BusinessId.eq(admin_id))
        .filter(InstanceColumn::Status.is_in([1, 2, 6]))
        .count(db)
        .await?;
    if inst_cnt > 0 {
        return Err(Error::from("该员工已有进行中的离职审批，请勿重复提交"));
    }
    let rec_cnt = hr_resign_record::Entity::find()
        .filter(hr_resign_record::Column::AdminId.eq(admin_id))
        .filter(hr_resign_record::Column::Status.is_in([
            RESIGN_STATUS_TRANSFERRING,
            RESIGN_STATUS_TRANSFERRED,
            RESIGN_STATUS_SETTLED,
        ]))
        .count(db)
        .await?;
    if rec_cnt > 0 {
        return Err(Error::from("该员工已有进行中的离职交接，请勿重复提交"));
    }
    Ok(())
}

/// B4：离职申请提交（admin 端 HR/管理员代发起 与 个人中心本人发起 共用）。
/// admin_id=被离职员工；operator_id/operator_name=提交人（JWT，防伪造）。
/// 防重复/交接人离职/在职状态校验通过后，创建 resign_approval 审批实例。
pub async fn submit_resign(
    db: &DbConn,
    admin_id: i64,
    operator_id: i64,
    operator_name: &str,
    resign_type: i32,
    resign_date: Option<String>,
    reason: Option<String>,
    transfer_to_admin_id: Option<i64>,
) -> Result<i64> {
    // 1. 目标员工在职校验
    let target = admin::Entity::find_by_id(admin_id)
        .one(db)
        .await?
        .ok_or_else(|| Error::from("员工不存在"))?;
    if target.deleted.unwrap_or(0) != 0 || target.status != Some(1) {
        return Err(Error::from("员工账号已停用，无法发起离职申请"));
    }
    if target.leave_date.is_some() {
        return Err(Error::from("员工已离职，无需重复申请"));
    }

    // 2. 防重复：无进行中离职实例、无未关闭交接单
    ensure_no_active_resign(db, admin_id).await?;

    // 3. 交接人校验：在职 + 不在离职流程中 + 非本人
    if let Some(tid) = transfer_to_admin_id {
        if tid == admin_id {
            return Err(Error::from("交接人不能是离职员工本人"));
        }
        let t = admin::Entity::find_by_id(tid)
            .one(db)
            .await?
            .ok_or_else(|| Error::from("交接人不存在"))?;
        if t.deleted.unwrap_or(0) != 0 || t.status != Some(1) {
            return Err(Error::from("交接人账号已停用，无法接收交接"));
        }
        ensure_no_active_resign(db, tid).await?;
    }

    // 4. 构建并提交审批实例（提交人由 JWT 身份覆盖，业务内容写入 extra_data）
    let target_name = target
        .nick_name
        .clone()
        .filter(|s| !s.is_empty())
        .or_else(|| target.user_name.clone())
        .unwrap_or_else(|| admin_id.to_string());
    let submit_req = ApprovalSubmitRequest {
        flow_code: "resign_approval".to_string(),
        business_type: "resign".to_string(),
        business_id: admin_id,
        business_title: Some(format!("离职申请（{}）", target_name)),
        submitter_id: operator_id,
        submitter_name: Some(operator_name.to_string()),
        extra_data: Some(serde_json::json!({
            "resign_type": resign_type,
            "resign_date": resign_date,
            "reason": reason,
            "transfer_to_admin_id": transfer_to_admin_id,
        })),
        cc_user_ids: None,
        cc_reason: None,
    };
    ApprovalService::submit(db, &submit_req).await
}

/// 员工显示名（昵称优先，回退登录名）
pub fn admin_display_name(m: &admin::Model) -> String {
    m.nick_name
        .clone()
        .filter(|s| !s.is_empty())
        .or_else(|| m.user_name.clone())
        .unwrap_or_else(|| m.id.to_string())
}

/// 获取员工显示名（按ID，查不到返回空串）
async fn admin_name_by_id(db: &impl ConnectionTrait, id: Option<i64>) -> String {
    let Some(id) = id else {
        return String::new();
    };
    admin::Entity::find_by_id(id)
        .one(db)
        .await
        .ok()
        .flatten()
        .map(|m| admin_display_name(&m))
        .unwrap_or_default()
}

/// 主表状态流转（加载 -> 校验前置 -> 更新）
async fn transition_record_status(
    db: &impl ConnectionTrait,
    record_id: i64,
    from: i32,
    to: i32,
    operator_name: &str,
) -> Result<hr_resign_record::Model> {
    let record = hr_resign_record::Entity::find_by_id(record_id)
        .one(db)
        .await?
        .ok_or_else(|| Error::from("交接单不存在"))?;
    if record.status != Some(from) {
        return Err(Error::from(format!("交接单当前状态不允许该操作（期望状态 {}，实际 {}）", from, record.status.unwrap_or(0))));
    }
    let mut active: hr_resign_record::ActiveModel = record.clone().into();
    active.status = Set(Some(to));
    active.update_by = Set(Some(operator_name.to_string()));
    active.update_time = Set(Some(Utc::now().naive_utc()));
    active.update(db).await.map_err(|e| Error::from(e.to_string()))?;
    Ok(record)
}

/// B6：交接单详情（主表 + 交接项 + 员工信息 + 审批实例列表）
pub async fn get_detail(db: &DbConn, record_id: i64) -> Result<serde_json::Value> {
    let record = hr_resign_record::Entity::find_by_id(record_id)
        .one(db)
        .await?
        .ok_or_else(|| Error::from("交接单不存在"))?;

    let items = hr_resign_transfer_item::Entity::find()
        .filter(hr_resign_transfer_item::Column::RecordId.eq(record_id))
        .order_by_asc(hr_resign_transfer_item::Column::Id)
        .all(db)
        .await?;

    let admin_info = admin::Entity::find_by_id(record.admin_id).one(db).await?.map(|a| {
        serde_json::json!({
            "id": a.id,
            "userName": a.user_name,
            "nickName": a.nick_name,
            "avatar": a.avatar,
        })
    });

    // 交接项（补充确认人名称）
    let mut items_json: Vec<serde_json::Value> = Vec::with_capacity(items.len());
    for it in &items {
        let assignee_name = admin_name_by_id(db, it.assignee_id).await;
        items_json.push(serde_json::json!({
            "id": it.id,
            "itemKey": it.item_key,
            "itemName": it.item_name,
            "assigneeId": it.assignee_id,
            "assigneeName": assignee_name,
            "status": it.status,
            "statusName": item_status_name(it.status),
            "confirmRemark": it.confirm_remark,
            "confirmTime": it.confirm_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        }));
    }

    // 离职审批实例（按提交时间正序，完整历史）
    let instances = InstanceEntity::find()
        .filter(InstanceColumn::BusinessType.eq("resign"))
        .filter(InstanceColumn::BusinessId.eq(record.admin_id))
        .order_by_asc(InstanceColumn::SubmittedAt)
        .all(db)
        .await?;
    let instances_json: Vec<serde_json::Value> = instances
        .iter()
        .map(|i| {
            serde_json::json!({
                "id": i.id,
                "status": i.status,
                "statusName": instance_status_name(i.status),
                "submitterId": i.submitter_id,
                "submitterName": i.submitter_name,
                "businessTitle": i.business_title,
                "submittedAt": i.submitted_at.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
            })
        })
        .collect();

    Ok(serde_json::json!({
        "id": record.id,
        "adminId": record.admin_id,
        "adminInfo": admin_info,
        "transferToAdminId": record.transfer_to_admin_id,
        "transferToName": admin_name_by_id(db, record.transfer_to_admin_id).await,
        "resignType": record.resign_type,
        "resignDate": record.resign_date.map(|d| d.format("%Y-%m-%d").to_string()),
        "actualLeaveDate": record.actual_leave_date.map(|d| d.format("%Y-%m-%d").to_string()),
        "reason": record.reason,
        "status": record.status,
        "statusName": record_status_name(record.status),
        "createBy": record.create_by,
        "createTime": record.create_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        "updateTime": record.update_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        "items": items_json,
        "instances": instances_json,
    }))
}

/// 交接单列表（分页）
pub async fn get_list(
    db: &DbConn,
    keyword: Option<String>,
    status: Option<i32>,
    page: u64,
    page_size: u64,
) -> Result<crate::core::web::response::ResultPage<Vec<serde_json::Value>>> {
    use crate::core::web::response::ResultPage;
    let mut qr = hr_resign_record::Entity::find();

    if let Some(st) = status {
        qr = qr.filter(hr_resign_record::Column::Status.eq(st));
    }
    if let Some(kw) = keyword.filter(|s| !s.trim().is_empty()) {
        // 员工ID/昵称/登录名模糊匹配：先按关键字查员工，再按 admin_id 过滤
        let ids: Vec<i64> = admin::Entity::find()
            .filter(
                sea_orm::Condition::any()
                    .add(AdminColumn::UserName.like(format!("%{}%", kw)))
                    .add(AdminColumn::NickName.like(format!("%{}%", kw))),
            )
            .all(db)
            .await?
            .into_iter()
            .map(|a| a.id)
            .collect();
        if ids.is_empty() {
            return Ok(ResultPage::new(vec![], 0, page as i64, page_size as i64));
        }
        qr = qr.filter(hr_resign_record::Column::AdminId.is_in(ids));
    }

    let paginator = qr.order_by_desc(hr_resign_record::Column::Id).paginate(db, page_size);
    let total = paginator.num_items().await?;
    let records = paginator.fetch_page(page - 1).await?;

    let list: Vec<serde_json::Value> = records
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "adminId": r.admin_id,
                "resignType": r.resign_type,
                "resignDate": r.resign_date.map(|d| d.format("%Y-%m-%d").to_string()),
                "status": r.status,
                "statusName": record_status_name(r.status),
                "createTime": r.create_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
            })
        })
        .collect();

    Ok(ResultPage::new(list, total as i64, page as i64, page_size as i64))
}

/// 我的离职申请列表（个人中心 B9：按本人 admin_id 过滤，身份来自 JWT）
pub async fn get_my_list(db: &DbConn, admin_id: i64) -> Result<Vec<serde_json::Value>> {
    let records = hr_resign_record::Entity::find()
        .filter(hr_resign_record::Column::AdminId.eq(admin_id))
        .order_by_desc(hr_resign_record::Column::Id)
        .all(db)
        .await?;

    let list: Vec<serde_json::Value> = records
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "adminId": r.admin_id,
                "resignType": r.resign_type,
                "resignDate": r.resign_date.map(|d| d.format("%Y-%m-%d").to_string()),
                "status": r.status,
                "statusName": record_status_name(r.status),
                "createTime": r.create_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
            })
        })
        .collect();

    Ok(list)
}

/// 我的交接任务（assignee 视角：本人被指派的交接项 + 所属交接单摘要，身份只信 JWT，无需权限码）
pub async fn get_my_transfer_items(db: &DbConn, admin_id: i64) -> Result<Vec<serde_json::Value>> {
    let items = hr_resign_transfer_item::Entity::find()
        .filter(hr_resign_transfer_item::Column::AssigneeId.eq(admin_id))
        .order_by_desc(hr_resign_transfer_item::Column::Id)
        .all(db)
        .await?;

    let mut list: Vec<serde_json::Value> = Vec::with_capacity(items.len());
    for it in items {
        let Some(record) = hr_resign_record::Entity::find_by_id(it.record_id).one(db).await?
        else {
            continue;
        };
        list.push(serde_json::json!({
            "itemId": it.id,
            "recordId": it.record_id,
            "itemKey": it.item_key,
            "itemName": it.item_name,
            "status": it.status,
            "statusName": item_status_name(it.status),
            "confirmRemark": it.confirm_remark,
            "confirmTime": it.confirm_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
            "recordStatus": record.status,
            "recordStatusName": record_status_name(record.status),
            "resignUserName": admin_name_by_id(db, Some(record.admin_id)).await,
            "resignDate": record.resign_date.map(|d| d.format("%Y-%m-%d").to_string()),
            "createTime": record.create_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        }));
    }
    Ok(list)
}

/// 发起人本人/交接确认人可访问交接单（无 system:resign:view 权限码时的本人数据兜底，方案 3.6.1）
pub async fn can_view_record(db: &DbConn, record_id: i64, operator_id: i64) -> Result<bool> {
    let Some(record) = hr_resign_record::Entity::find_by_id(record_id).one(db).await? else {
        return Ok(false);
    };
    // 发起人（被离职员工）本人
    if record.admin_id == operator_id {
        return Ok(true);
    }
    // 任一交接项的确认人
    let cnt = hr_resign_transfer_item::Entity::find()
        .filter(hr_resign_transfer_item::Column::RecordId.eq(record_id))
        .filter(hr_resign_transfer_item::Column::AssigneeId.eq(operator_id))
        .count(db)
        .await?;
    Ok(cnt > 0)
}

/// 离职原因可见性（方案 3.6.5）：员工本人、人事（system:resign:save/confirm）、超管、审批链当前/候选审批人可见；
/// 财务、系统管理员、交接确认人不可见
async fn reason_visible_for(db: &DbConn, record: &hr_resign_record::Model, operator_id: i64) -> bool {
    if operator_id <= 0 {
        return false;
    }
    // 员工本人
    if record.admin_id == operator_id {
        return true;
    }
    // 超级管理员（全权限走系统逻辑）
    if let Ok(Some(a)) = admin::Entity::find_by_id(operator_id).one(db).await {
        if a.user_type == Some(1) {
            return true;
        }
    }
    // 人事角色（权限码 system:resign:save / system:resign:confirm）
    let perms = crate::modules::system::service::permission_cache_service::get_or_load_permissions(
        db, operator_id,
    )
    .await;
    if perms
        .iter()
        .any(|p| p == "system:resign:save" || p == "system:resign:confirm")
    {
        return true;
    }
    // 审批期间：该员工进行中（status∈{1,2}）离职实例的当前/候选审批人
    if let Ok(instances) = InstanceEntity::find()
        .filter(InstanceColumn::BusinessType.eq("resign"))
        .filter(InstanceColumn::BusinessId.eq(record.admin_id))
        .filter(InstanceColumn::Status.is_in([1, 2]))
        .all(db)
        .await
    {
        for inst in instances {
            if inst.current_approver_id == Some(operator_id) {
                return true;
            }
            let hit = inst
                .candidate_approvers
                .as_ref()
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().any(|x| x.as_i64() == Some(operator_id)))
                .unwrap_or(false);
            if hit {
                return true;
            }
        }
    }
    false
}

/// 交接单详情（按访问者身份过滤敏感字段：离职原因不可见时置空，方案 3.6.5）
pub async fn get_detail_for_operator(
    db: &DbConn,
    record_id: i64,
    operator_id: i64,
) -> Result<serde_json::Value> {
    let record = hr_resign_record::Entity::find_by_id(record_id)
        .one(db)
        .await?
        .ok_or_else(|| Error::from("交接单不存在"))?;
    let mut detail = get_detail(db, record_id).await?;
    if reason_visible_for(db, &record, operator_id).await {
        detail["reasonVisible"] = serde_json::Value::Bool(true);
    } else {
        detail["reason"] = serde_json::Value::Null;
        detail["reasonVisible"] = serde_json::Value::Bool(false);
    }
    Ok(detail)
}

/// 交接项确认（assignee 身份校验；HR 代确认 is_hr_override=true 时跳过）
/// is_na=true 表示"不适用"跳过（status=2）
pub async fn confirm_item(
    db: &DbConn,
    record_id: i64,
    item_id: i64,
    operator_id: i64,
    is_na: bool,
    remark: Option<String>,
    is_hr_override: bool,
) -> Result<()> {
    let record = hr_resign_record::Entity::find_by_id(record_id)
        .one(db)
        .await?
        .ok_or_else(|| Error::from("交接单不存在"))?;
    if record.status != Some(RESIGN_STATUS_TRANSFERRING) {
        return Err(Error::from("仅交接中的交接单可确认交接项"));
    }
    let item = hr_resign_transfer_item::Entity::find_by_id(item_id)
        .one(db)
        .await?
        .ok_or_else(|| Error::from("交接项不存在"))?;
    if item.record_id != record_id {
        return Err(Error::from("交接项不属于该交接单"));
    }
    if item.status != Some(ITEM_STATUS_PENDING) {
        return Err(Error::from("该交接项已确认，不可重复操作"));
    }
    // 身份校验：assignee 本人 或 HR 代确认
    if !is_hr_override && item.assignee_id != Some(operator_id) {
        return Err(Error::from("您不是该交接项的确认人"));
    }

    let mut active: hr_resign_transfer_item::ActiveModel = item.into();
    active.status = Set(Some(if is_na { ITEM_STATUS_NA } else { ITEM_STATUS_CONFIRMED }));
    active.confirm_remark = Set(remark);
    active.confirm_time = Set(Some(Utc::now().naive_utc()));
    active.update_time = Set(Some(Utc::now().naive_utc()));
    active.update(db).await.map_err(|e| Error::from(e.to_string()))?;

    // 全部交接项 ∈ {1,2} -> 主表 1 -> 2
    let pending = hr_resign_transfer_item::Entity::find()
        .filter(hr_resign_transfer_item::Column::RecordId.eq(record_id))
        .filter(hr_resign_transfer_item::Column::Status.eq(ITEM_STATUS_PENDING))
        .count(db)
        .await?;
    if pending == 0 {
        transition_record_status(db, record_id, RESIGN_STATUS_TRANSFERRING, RESIGN_STATUS_TRANSFERRED, "system").await?;
    }
    Ok(())
}

/// 财务结算确认（status=2 -> 3），并在同一事务内自动触发完全离职
/// leave_date：实际离职日，为空时取申请日期（actual_leave_date 优先）
pub async fn settle(db: &DbConn, record_id: i64, leave_date: Option<NaiveDate>) -> Result<()> {
    db.transaction::<_, (), Error>(|txn| {
        Box::pin(async move {
            let record = transition_record_status(txn, record_id, RESIGN_STATUS_TRANSFERRED, RESIGN_STATUS_SETTLED, "system").await?;
            // 写入实际离职日（结算确认时定）
            let actual = leave_date.or(record.actual_leave_date).or(record.resign_date);
            if let Some(d) = actual {
                let mut active: hr_resign_record::ActiveModel = hr_resign_record::Entity::find_by_id(record_id)
                    .one(txn)
                    .await?
                    .ok_or_else(|| Error::from("交接单不存在"))?
                    .into();
                active.actual_leave_date = Set(Some(d));
                active.update_time = Set(Some(Utc::now().naive_utc()));
                active.update(txn).await.map_err(|e| Error::from(e.to_string()))?;
            }
            // 完全离职（同一事务：leave_date + 停用 + 3 -> 4）
            finalize_handover(txn, record_id).await?;
            Ok(())
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    // B10：结算完成 + 完全离职站内通知（事务外 best-effort，失败不影响结算结果）
    if let Ok(Some(rec)) = hr_resign_record::Entity::find_by_id(record_id).one(db).await {
        notify_settled_and_done(db, &rec).await;
    }
    Ok(())
}

/// B10：结算完成 + 完全离职站内通知（best-effort）：
/// 1) 结算完成：通知人事角色（人事经理/人事专员）与离职员工本人
/// 2) 完全离职：通知离职员工本人、人事角色、财务角色（财务/财务经理）
async fn notify_settled_and_done(db: &DbConn, record: &hr_resign_record::Model) {
    let emp_name = admin_name_by_id(db, Some(record.admin_id)).await;
    let hr_ids = find_users_by_role_keys(db, &["hr_manager", "hr_specialist"])
        .await
        .unwrap_or_default();
    let fin_ids = find_users_by_role_keys(db, &["finance", "finance_manager"])
        .await
        .unwrap_or_default();

    // 1. 结算完成通知（人事 + 员工本人）
    let mut recv: Vec<i64> = vec![record.admin_id];
    recv.extend(hr_ids.iter().copied());
    recv.sort_unstable();
    recv.dedup();
    for rid in recv {
        if rid <= 0 {
            continue;
        }
        let _ = NotificationService::send_system_notification(
            db,
            rid,
            format!("【离职结算】{} 的离职结算已完成", emp_name),
            format!(
                "员工 {} 的离职交接已确认结算，账号将于结算确认后停用并完成离职。",
                emp_name
            ),
            9,
            Some("/system/user".to_string()),
        )
        .await;
    }

    // 2. 完全离职通知（员工本人 + 人事 + 财务）
    let mut recv2: Vec<i64> = vec![record.admin_id];
    recv2.extend(hr_ids);
    recv2.extend(fin_ids);
    recv2.sort_unstable();
    recv2.dedup();
    for rid in recv2 {
        if rid <= 0 {
            continue;
        }
        let _ = NotificationService::send_system_notification(
            db,
            rid,
            format!("【离职完成】{} 已完成离职", emp_name),
            format!(
                "员工 {} 已完成全部离职流程（交接、结算、账号停用），请知悉。",
                emp_name
            ),
            9,
            Some("/system/user".to_string()),
        )
        .await;
    }
}

/// 完全离职（事务内自动触发）：写 leave_date + 账号停用(status=0) + 交接单 3->4
async fn finalize_handover(db: &impl ConnectionTrait, record_id: i64) -> Result<()> {
    let record = hr_resign_record::Entity::find_by_id(record_id)
        .one(db)
        .await?
        .ok_or_else(|| Error::from("交接单不存在"))?;
    if record.status != Some(RESIGN_STATUS_SETTLED) {
        return Err(Error::from("仅结算完成的交接单可执行完全离职"));
    }
    // 实际离职日：actual_leave_date 优先，未填取申请日期
    let leave_day = record.actual_leave_date.or(record.resign_date);
    if let Some(d) = leave_day {
        let mut a: admin::ActiveModel = admin::Entity::find_by_id(record.admin_id)
            .one(db)
            .await?
            .ok_or_else(|| Error::from("员工不存在"))?
            .into();
        a.leave_date = Set(Some(d));
        // 账号停用（离职员工不可再登录）
        a.status = Set(Some(0));
        a.update(db).await.map_err(|e| Error::from(e.to_string()))?;
    }
    transition_record_status(db, record_id, RESIGN_STATUS_SETTLED, RESIGN_STATUS_DONE, "system").await?;
    Ok(())
}

/// 离职中止（status ∈ {1,2} -> 5；理由必填；未确认交接项批量置"不适用"留中止备注）
pub async fn abort(db: &DbConn, record_id: i64, reason: String) -> Result<()> {
    let reason = reason.trim().to_string();
    if reason.is_empty() {
        return Err(Error::from("中止离职必须填写理由"));
    }
    db.transaction::<_, (), Error>(|txn| {
        Box::pin(async move {
            let record = hr_resign_record::Entity::find_by_id(record_id)
                .one(txn)
                .await?
                .ok_or_else(|| Error::from("交接单不存在"))?;
            if !matches!(record.status, Some(RESIGN_STATUS_TRANSFERRING) | Some(RESIGN_STATUS_TRANSFERRED)) {
                return Err(Error::from("仅交接中的交接单可中止离职"));
            }
            // 未确认交接项批量置"不适用"，备注中止原因（留痕）
            hr_resign_transfer_item::Entity::update_many()
                .col_expr(
                    hr_resign_transfer_item::Column::Status,
                    sea_orm::sea_query::Expr::value(ITEM_STATUS_NA),
                )
                .col_expr(
                    hr_resign_transfer_item::Column::ConfirmRemark,
                    sea_orm::sea_query::Expr::value(Some(format!("离职中止：{}", reason))),
                )
                .col_expr(
                    hr_resign_transfer_item::Column::ConfirmTime,
                    sea_orm::sea_query::Expr::value(Some(Utc::now().naive_utc())),
                )
                .filter(hr_resign_transfer_item::Column::RecordId.eq(record_id))
                .filter(hr_resign_transfer_item::Column::Status.eq(ITEM_STATUS_PENDING))
                .exec(txn)
                .await?;
            transition_record_status(txn, record_id, record.status.unwrap_or(1), RESIGN_STATUS_ABORTED, "system").await?;
            Ok(())
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))
}

/// assignee 转派（交接人离职/失联等场景；仅待确认项可转派）
pub async fn transfer_assignee(
    db: &DbConn,
    record_id: i64,
    item_id: i64,
    new_assignee_id: i64,
) -> Result<()> {
    let record = hr_resign_record::Entity::find_by_id(record_id)
        .one(db)
        .await?
        .ok_or_else(|| Error::from("交接单不存在"))?;
    if record.status != Some(RESIGN_STATUS_TRANSFERRING) {
        return Err(Error::from("仅交接中的交接单可转派确认人"));
    }
    let item = hr_resign_transfer_item::Entity::find_by_id(item_id)
        .one(db)
        .await?
        .ok_or_else(|| Error::from("交接项不存在"))?;
    if item.record_id != record_id {
        return Err(Error::from("交接项不属于该交接单"));
    }
    if item.status != Some(ITEM_STATUS_PENDING) {
        return Err(Error::from("已确认的交接项不可转派"));
    }
    let new_admin = admin::Entity::find_by_id(new_assignee_id)
        .one(db)
        .await?
        .ok_or_else(|| Error::from("新确认人不存在"))?;
    if new_admin.deleted.unwrap_or(0) != 0 || new_admin.status != Some(1) {
        return Err(Error::from("新确认人账号已停用"));
    }
    let mut active: hr_resign_transfer_item::ActiveModel = item.into();
    active.assignee_id = Set(Some(new_assignee_id));
    active.update_time = Set(Some(Utc::now().naive_utc()));
    active.update(db).await.map_err(|e| Error::from(e.to_string()))?;
    Ok(())
}

/// 交接单状态文案
pub fn record_status_name(status: Option<i32>) -> String {
    match status {
        Some(1) => "交接中".to_string(),
        Some(2) => "交接完成".to_string(),
        Some(3) => "结算完成".to_string(),
        Some(4) => "已离职".to_string(),
        Some(5) => "已中止".to_string(),
        _ => "未知".to_string(),
    }
}

/// 交接项状态文案
pub fn item_status_name(status: Option<i32>) -> String {
    match status {
        Some(0) => "待确认".to_string(),
        Some(1) => "已确认".to_string(),
        Some(2) => "不适用".to_string(),
        _ => "未知".to_string(),
    }
}

/// 审批实例状态文案（对齐审批引擎）
pub fn instance_status_name(status: Option<i32>) -> String {
    match status {
        Some(1) => "待审批".to_string(),
        Some(2) => "审批中".to_string(),
        Some(3) => "已通过".to_string(),
        Some(4) => "已驳回".to_string(),
        Some(5) => "已撤回".to_string(),
        Some(6) => "待修改".to_string(),
        _ => "未知".to_string(),
    }
}

/// B10：每日超时提醒（定时任务 handler 入口，best-effort，单条失败不中断）：
/// 1) user/resign 审批实例 status∈{1,2} 且距最近更新时间超过 1 天 → 提醒当前节点审批人
/// 2) 交接中（status=1）待确认项距最近更新时间超过 1 天 → 提醒 assignee
/// 返回本次提醒人次
pub async fn remind_timeout_tasks(db: &DbConn) -> Result<i64> {
    use chrono::Duration;
    let now = Utc::now().naive_utc();
    let timeout = Duration::days(1);
    let mut count: i64 = 0;

    // 1. 超时未处理的审批待办
    let instances = InstanceEntity::find()
        .filter(
            InstanceColumn::BusinessType
                .is_in(vec!["user".to_string(), "resign".to_string()]),
        )
        .filter(InstanceColumn::Status.is_in([1, 2]))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;
    for inst in instances {
        let last_ts = inst.update_time.or(inst.submitted_at).or(inst.create_time);
        let Some(last_ts) = last_ts else { continue };
        if now.signed_duration_since(last_ts) < timeout {
            continue;
        }
        // 依次审批优先提醒当前审批人，否则提醒全部候选审批人
        let cands: Vec<i64> = inst
            .candidate_approvers
            .as_ref()
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_i64()).collect())
            .unwrap_or_default();
        let targets: Vec<i64> = match inst.current_approver_id {
            Some(id) if id > 0 => vec![id],
            _ => cands,
        };
        let title = format!(
            "【审批超时】{}",
            inst.business_title.clone().unwrap_or_else(|| "审批申请".to_string())
        );
        for tid in targets {
            if tid <= 0 {
                continue;
            }
            let _ = NotificationService::send_system_notification(
                db,
                tid,
                title.clone(),
                "您有审批待办已超时未处理，请及时处理。".to_string(),
                9,
                Some("/system/user".to_string()),
            )
            .await;
            count += 1;
        }
    }

    // 2. 交接项超时未确认
    let records = hr_resign_record::Entity::find()
        .filter(hr_resign_record::Column::Status.eq(RESIGN_STATUS_TRANSFERRING))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;
    for rec in records {
        let emp_name = admin_name_by_id(db, Some(rec.admin_id)).await;
        let items = hr_resign_transfer_item::Entity::find()
            .filter(hr_resign_transfer_item::Column::RecordId.eq(rec.id))
            .filter(hr_resign_transfer_item::Column::Status.eq(ITEM_STATUS_PENDING))
            .all(db)
            .await
            .map_err(|e| e.to_string())?;
        let mut seen: Vec<i64> = Vec::new();
        for it in items {
            let Some(aid) = it.assignee_id else { continue };
            if seen.contains(&aid) {
                continue;
            }
            let last_ts = it.update_time.or(it.create_time);
            let Some(last_ts) = last_ts else { continue };
            if now.signed_duration_since(last_ts) < timeout {
                continue;
            }
            seen.push(aid);
            let _ = NotificationService::send_system_notification(
                db,
                aid,
                format!("【交接超时】{} 的离职交接待确认", emp_name),
                format!(
                    "您负责的交接项【{}】已超时未确认，请及时处理。",
                    it.item_name.clone().unwrap_or_default()
                ),
                9,
                Some("/system/user".to_string()),
            )
            .await;
            count += 1;
        }
    }

    Ok(count)
}
