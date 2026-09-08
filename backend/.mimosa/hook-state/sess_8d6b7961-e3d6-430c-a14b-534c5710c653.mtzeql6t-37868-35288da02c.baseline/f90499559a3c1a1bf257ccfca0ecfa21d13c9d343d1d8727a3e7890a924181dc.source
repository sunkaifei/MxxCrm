//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! CRM 数据出口统一前置校验（删除 / 作废 / 退回原因）。
//! 规则来源：docs/CRM数据删除与作废策略-规划方案.md（5.1-5.5、6.2）。
//! 错误文案与验收标准（第七章）保持一致，供各 service 复用。

use crate::core::errors::error::{Error, Result};
use crate::modules::crm::entity::{contact, contract, customer, followup, opportunity};
use crate::modules::sale::entity::{invoice, order, payment, quotation};
use crate::modules::system::entity::admin::Entity as Admin;
use crate::modules::system::service::menu_service;
use chrono::{Duration, NaiveDateTime};
use sea_orm::{ColumnTrait, ConnectionTrait, DbConn, EntityTrait, PaginatorTrait, QueryFilter};

/// 客户删除时间窗（小时）：自建客户若 1 小时内无人跟进，大概率是错录（第一版为模块常量，改值需重新编译）
pub const CUSTOMER_DELETE_WINDOW_HOURS: i64 = 1;

/// 商机/跟进/联系人/线索删除时间窗（小时）：给一个跨天修正窗口
pub const DEFAULT_DELETE_WINDOW_HOURS: i64 = 24;

/// 回收站保留期（天）：主流 15/60/90 天，取中
pub const RECYCLE_RETENTION_DAYS: i64 = 30;

/// 退回原因类型合法枚举：1=跟进无回应 2=客户无意向 3=客户信息无效 4=换业务方向 9=其他
pub const RELEASE_REASON_TYPES: &[i16] = &[1, 2, 3, 4, 9];

/// 退回原因类型"其他"（选此值时补充说明必填）
pub const REASON_TYPE_OTHER: i16 = 9;

/// 商机终态：成交/丢单（业务结果，禁删禁作废）
pub const OPPORTUNITY_STAGE_CLOSED: i32 = 5;

/// 商机作废态：已作废（数据治理动作，可恢复）
pub const OPPORTUNITY_STAGE_VOIDED: i32 = 6;

/// 是否超级管理员（user_type=1，规划方案 4.1 #6 口径；接收 ConnectionTrait 以便事务内调用）
pub async fn is_super_admin(db: &impl ConnectionTrait, user_id: i64) -> Result<bool> {
    let admin = Admin::find_by_id(user_id)
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(admin.is_some_and(|a| a.user_type == Some(1)))
}

/// 是否"管理员"：超管，或拥有指定权限码（如 crm:followup:delete）的角色（规划方案 4.1 #6 口径）
pub async fn is_manager(db: &DbConn, user_id: i64, perm_code: &str) -> Result<bool> {
    if is_super_admin(db, user_id).await? {
        return Ok(true);
    }
    let perms = menu_service::find_user_role_keys(db, &false, &Some(user_id)).await?;
    Ok(perms.iter().any(|p| p == perm_code))
}

/// 删除时间窗校验：create_time 距今须在 window_hours 内
/// entity_label 用于缺创建时间的报错；hint 追加到超窗文案末尾（如"，请走作废"）
pub fn check_delete_window(
    create_time: Option<NaiveDateTime>,
    window_hours: i64,
    entity_label: &str,
    hint: &str,
) -> Result<()> {
    let create_time = create_time.ok_or_else(|| {
        Error::from(format!("{}缺少创建时间，无法校验删除时限", entity_label))
    })?;
    let elapsed = chrono::Local::now().naive_local() - create_time;
    if elapsed > Duration::hours(window_hours) {
        return Err(Error::from(format!(
            "已超过可删除时限（创建后 {} 小时内），无法删除{}",
            window_hours, hint
        )));
    }
    Ok(())
}

/// 客户删除前置校验（规划方案 5.1 四条件）：
/// 1. 创建人或超管；2. 自建（from_pool=0）；3. 无任何关联业务数据；4. 创建后 1 小时内。
/// 超管兜底仅豁免"自建判别 + 时间窗"，关联校验不可豁免（防删掉挂单据的客户制造孤儿数据）。
pub async fn check_customer_deletable(
    db: &impl ConnectionTrait,
    customer: &customer::Model,
    current_user_id: i64,
) -> Result<()> {
    let is_super = is_super_admin(db, current_user_id).await?;
    if customer.created_by != Some(current_user_id) && !is_super {
        return Err(Error::from("仅客户创建人可删除该客户"));
    }
    if !is_super {
        // 条件2：公海来源客户只能退回公海，不允许删除
        if customer.from_pool.unwrap_or(0) != 0 {
            return Err(Error::from("该客户来自公海，无法删除，请退回公海"));
        }
        // 条件4：时间窗
        check_delete_window(
            customer.create_time,
            CUSTOMER_DELETE_WINDOW_HOURS,
            "客户",
            "",
        )?;
    }
    // 条件3：关联计数（超管不豁免）
    let linked = count_customer_relations(db, customer.id).await?;
    if linked > 0 {
        return Err(Error::from(
            "该客户已存在商机/联系人/跟进/合同等关联数据，无法删除",
        ));
    }
    Ok(())
}

/// 统计客户名下有效关联业务数据条数（商机/联系人/跟进/合同/报价单/订单/发票/回款，逐项早退）
pub async fn count_customer_relations(db: &impl ConnectionTrait, customer_id: i64) -> Result<u64> {
    let count = opportunity::Entity::find()
        .filter(opportunity::Column::CustomerId.eq(customer_id))
        .filter(opportunity::Column::Deleted.eq(0))
        .count(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    if count > 0 {
        return Ok(count);
    }
    let count = contact::Entity::find()
        .filter(contact::Column::CustomerId.eq(customer_id))
        .filter(contact::Column::Deleted.eq(0))
        .count(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    if count > 0 {
        return Ok(count);
    }
    let count = followup::Entity::find()
        .filter(followup::Column::CustomerId.eq(customer_id))
        .filter(followup::Column::Deleted.eq(0))
        .count(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    if count > 0 {
        return Ok(count);
    }
    let count = contract::Entity::find()
        .filter(contract::Column::CustomerId.eq(customer_id))
        .filter(contract::Column::Deleted.eq(0))
        .count(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    if count > 0 {
        return Ok(count);
    }
    let count = quotation::Entity::find()
        .filter(quotation::Column::CustomerId.eq(customer_id))
        .filter(quotation::Column::Deleted.eq(0))
        .count(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    if count > 0 {
        return Ok(count);
    }
    let count = order::Entity::find()
        .filter(order::Column::CustomerId.eq(customer_id))
        .filter(order::Column::Deleted.eq(0))
        .count(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    if count > 0 {
        return Ok(count);
    }
    let count = invoice::Entity::find()
        .filter(invoice::Column::CustomerId.eq(customer_id))
        .filter(invoice::Column::Deleted.eq(0))
        .count(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    if count > 0 {
        return Ok(count);
    }
    let count = payment::Entity::find()
        .filter(payment::Column::CustomerId.eq(customer_id))
        .filter(payment::Column::Deleted.eq(0))
        .count(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(count)
}

/// 商机删除前置校验（规划方案 5.2 四条件）：
/// 1. 创建人或超管；2. 6 个状态字段任一 > 0 视为有关联单据；3. 创建后 24 小时内；4. 非终态。
pub async fn check_opportunity_deletable(
    db: &impl ConnectionTrait,
    opp: &opportunity::Model,
    current_user_id: i64,
) -> Result<()> {
    // 条件1：仅商机创建人或超管可删除
    if opp.created_by != Some(current_user_id) {
        let is_super = is_super_admin(db, current_user_id).await?;
        if !is_super {
            return Err(Error::from("仅商机创建人可删除该商机"));
        }
    }
    // 条件2：关联单据以商机自身 6 个状态字段判断（G1，避免跨单据表查询）
    let has_doc = [
        opp.quote_status,
        opp.order_status,
        opp.contract_status,
        opp.shipment_status,
        opp.payment_status,
        opp.invoice_status,
    ]
    .iter()
    .any(|s| s.unwrap_or(0) > 0);
    if has_doc {
        return Err(Error::from("该商机关联了报价/合同/订单，无法删除，请走作废"));
    }
    // 条件3：时间窗（24 小时）
    check_delete_window(
        opp.create_time,
        DEFAULT_DELETE_WINDOW_HOURS,
        "商机",
        "，请走作废",
    )?;
    // 条件4：终态（成交/丢单/已作废）禁删，数据须保留
    if opp.stage == Some(OPPORTUNITY_STAGE_CLOSED) {
        return Err(Error::from(
            "该商机已是终态（成交/丢单），不允许删除，数据须保留",
        ));
    }
    if opp.stage == Some(OPPORTUNITY_STAGE_VOIDED) {
        return Err(Error::from(
            "该商机已作废，不允许删除，数据须保留",
        ));
    }
    Ok(())
}

/// 商机作废前置校验（规划方案 5.2 作废规则 3）：
/// 终态禁止（stage=5 成交/丢单、stage=6 已作废）；存在关联合同或订单须先处理下游。
/// 报价单不拦（仅关联报价的允许作废，报价单随作废动作同步失效，见 opportunity_service）。
pub async fn check_opportunity_voidable(db: &impl ConnectionTrait, opp: &opportunity::Model) -> Result<()> {
    if opp.stage == Some(OPPORTUNITY_STAGE_CLOSED) {
        return Err(Error::from("该商机已成交/丢单（终态），不允许作废"));
    }
    if opp.stage == Some(OPPORTUNITY_STAGE_VOIDED) {
        return Err(Error::from("该商机已作废，无需重复作废"));
    }
    let count = contract::Entity::find()
        .filter(contract::Column::OpportunityId.eq(opp.id))
        .filter(contract::Column::Deleted.eq(0))
        .count(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    if count > 0 {
        return Err(Error::from("该商机关联了合同，无法作废，请先处理下游合同"));
    }
    let count = order::Entity::find()
        .filter(order::Column::OpportunityId.eq(opp.id))
        .filter(order::Column::Deleted.eq(0))
        .count(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    if count > 0 {
        return Err(Error::from("该商机关联了订单，无法作废，请先处理下游订单"));
    }
    // 报价单审批中（approval_status=2）禁止作废（规划方案 5.2 作废规则3：审批进行中）
    let count = quotation::Entity::find()
        .filter(quotation::Column::OpportunityId.eq(opp.id))
        .filter(quotation::Column::Deleted.eq(0))
        .filter(quotation::Column::ApprovalStatus.eq(2))
        .count(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    if count > 0 {
        return Err(Error::from("该商机关联的报价单正在审批中，无法作废"));
    }
    Ok(())
}

/// 退回原因校验（客户退回公海 / 线索退回线索池共用，后端兜底）：
/// 原因类型必填且在合法枚举内；类型为"其他"（9）时补充说明必填。
pub fn validate_release_reason(reason_type: Option<i16>, reason: &Option<String>) -> Result<()> {
    let rt = reason_type.ok_or_else(|| Error::from("请选择退回原因类型"))?;
    if !RELEASE_REASON_TYPES.contains(&rt) {
        return Err(Error::from("无效的退回原因类型"));
    }
    if rt == REASON_TYPE_OTHER {
        let note = reason.as_deref().map(str::trim).unwrap_or("");
        if note.is_empty() {
            return Err(Error::from("退回原因为【其他】时，必须填写补充说明"));
        }
    }
    Ok(())
}
