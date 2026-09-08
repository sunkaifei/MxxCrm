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
use crate::modules::crm::entity::customer_pool_config;
use crate::modules::crm::model::customer_pool::{CustomerPoolConfigSaveRequest, CustomerPoolConfigVO};
use crate::modules::crm::service::customer_pool_service;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, QueryFilter, Set,
};
use sea_orm::sea_query::OnConflict;

/// 脱敏字段白名单（对齐客户列表 VO 字段：公司名/个人手机/个人邮箱/微信）
const MASK_FIELD_WHITELIST: [&str; 4] = ["company_name", "personal_mobile", "personal_email", "wechat"];

/// 有效合同状态全集（1=草拟 2=履行中 3=已完成 4=已收款？对齐合同状态枚举）
const DEAL_STATUS_ALL: [i16; 4] = [1, 2, 3, 4];

/// 确保池配置行存在（建池时调用；已存在则跳过）
pub async fn ensure_config(db: &DbConn, pool_id: i64) -> Result<()> {
    let active = customer_pool_config::ActiveModel {
        pool_id: Set(pool_id),
        assign_cursor: Set(Some(0)),
        ..Default::default()
    };
    let result = customer_pool_config::Entity::insert(active)
        .on_conflict(OnConflict::column(customer_pool_config::Column::PoolId).do_nothing().to_owned())
        .exec(db)
        .await;
    match result {
        Ok(_) | Err(DbErr::RecordNotInserted) => Ok(()),
        Err(e) => Err(Error::from(e.to_string())),
    }
}

/// 查询原始配置行
pub async fn get_raw(db: &DbConn, pool_id: i64) -> Result<Option<customer_pool_config::Model>> {
    customer_pool_config::Entity::find()
        .filter(customer_pool_config::Column::PoolId.eq(pool_id))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))
}

/// 查询配置 VO（不存在时返回 DB 默认参数的虚拟配置）
pub async fn get_vo(db: &DbConn, pool_id: i64) -> Result<CustomerPoolConfigVO> {
    customer_pool_service::find_by_id(db, pool_id)
        .await?
        .ok_or_else(|| Error::from("客户公海池不存在".to_string()))?;

    match get_raw(db, pool_id).await? {
        Some(model) => Ok(model_to_vo(&model)),
        None => Ok(default_vo(pool_id)),
    }
}

/// 保存池配置（参数边界校验 + upsert；轮询游标为运行时状态不在此保存）
pub async fn save(db: &DbConn, req: &CustomerPoolConfigSaveRequest, updated_by: i64) -> Result<()> {
    let pool_id = req.pool_id.ok_or_else(|| Error::from("池ID不能为空".to_string()))?;
    customer_pool_service::find_by_id(db, pool_id)
        .await?
        .ok_or_else(|| Error::from("客户公海池不存在".to_string()))?;

    validate(req)?;

    let now = chrono::Local::now().naive_local().to_owned();
    let mask_json: Option<serde_json::Value> = req.mask_fields.as_ref().map(|fields| {
        serde_json::Value::Array(fields.iter().map(|f| serde_json::Value::String(f.clone())).collect())
    });
    let deal_status_json: Option<serde_json::Value> = req.deal_status.as_ref().map(|list| {
        serde_json::Value::Array(list.iter().map(|s| serde_json::Value::Number((*s as i64).into())).collect())
    });

    let active = customer_pool_config::ActiveModel {
        pool_id: Set(pool_id),
        assign_cursor: Default::default(),
        recycle_days: Set(req.recycle_days),
        max_recycle_days: Set(req.max_recycle_days),
        no_opportunity_days: Set(req.no_opportunity_days),
        no_contract_days: Set(req.no_contract_days),
        reminder_days: Set(req.reminder_days),
        cool_down_days: Set(req.cool_down_days),
        claim_daily_limit: Set(req.claim_daily_limit),
        hold_limit: Set(req.hold_limit),
        claim_mode: Set(req.claim_mode),
        auto_assign_enabled: Set(req.auto_assign_enabled),
        auto_assign_mode: Set(req.auto_assign_mode),
        mask_fields: Set(mask_json),
        deal_protect_enabled: Set(req.deal_protect_enabled),
        deal_status: Set(deal_status_json),
        opportunity_protect_enabled: Set(req.opportunity_protect_enabled),
        include_self_built: Set(req.include_self_built),
        self_built_recycle_enabled: Set(req.self_built_recycle_enabled),
        hide_claimed: Set(req.hide_claimed),
        hide_converted: Set(req.hide_converted),
        allow_detail: Set(req.allow_detail),
        release_back_to: Set(req.release_back_to),
        freeze_release_count: Set(req.freeze_release_count),
        first_touch_hours: Set(req.first_touch_hours),
        update_time: Set(Some(now)),
        updated_by: Set(Some(updated_by)),
    };

    let result = customer_pool_config::Entity::insert(active)
        .on_conflict(
            OnConflict::column(customer_pool_config::Column::PoolId)
                .update_columns([
                    customer_pool_config::Column::RecycleDays,
                    customer_pool_config::Column::MaxRecycleDays,
                    customer_pool_config::Column::NoOpportunityDays,
                    customer_pool_config::Column::NoContractDays,
                    customer_pool_config::Column::ReminderDays,
                    customer_pool_config::Column::CoolDownDays,
                    customer_pool_config::Column::ClaimDailyLimit,
                    customer_pool_config::Column::HoldLimit,
                    customer_pool_config::Column::ClaimMode,
                    customer_pool_config::Column::AutoAssignEnabled,
                    customer_pool_config::Column::AutoAssignMode,
                    customer_pool_config::Column::MaskFields,
                    customer_pool_config::Column::DealProtectEnabled,
                    customer_pool_config::Column::DealStatus,
                    customer_pool_config::Column::OpportunityProtectEnabled,
                    customer_pool_config::Column::IncludeSelfBuilt,
                    customer_pool_config::Column::SelfBuiltRecycleEnabled,
                    customer_pool_config::Column::HideClaimed,
                    customer_pool_config::Column::HideConverted,
                    customer_pool_config::Column::AllowDetail,
                    customer_pool_config::Column::ReleaseBackTo,
                    customer_pool_config::Column::FreezeReleaseCount,
                    customer_pool_config::Column::FirstTouchHours,
                    customer_pool_config::Column::UpdateTime,
                    customer_pool_config::Column::UpdatedBy,
                ])
                .to_owned(),
        )
        .exec(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    let _ = result;
    Ok(())
}

/// 参数边界校验（方案 §5.2：多规则回收 + 三口径保护期 + 豁免 + 可见性 + 冻结 + SLA）
fn validate(req: &CustomerPoolConfigSaveRequest) -> Result<()> {
    // 默认保护期（0=关闭未跟进回收规则）
    check_range("未跟进自动回收天数", req.recycle_days, 0, 365)?;
    // 最长保护期（NULL=无硬上限）
    check_range("最长保护期天数", req.max_recycle_days, 1, 3650)?;
    // 多规则回收（NULL=关闭该规则）
    check_range("未新增商机回收天数", req.no_opportunity_days, 1, 365)?;
    check_range("未签合同回收天数", req.no_contract_days, 1, 365)?;
    check_range("回收前提醒天数", req.reminder_days, 0, 365)?;
    check_range("退回冷却期天数", req.cool_down_days, 0, 365)?;
    check_range("每人每日领取上限", req.claim_daily_limit, 0, 999)?;
    check_range("私海保有量上限", req.hold_limit, 0, 9999)?;
    check_range("冻结触发退回次数", req.freeze_release_count, 0, 99)?;
    check_range("首次触达 SLA 小时数", req.first_touch_hours, 0, 720)?;

    // 提醒天数必须小于默认保护期（否则提醒无意义）
    if let (Some(recycle), Some(reminder)) = (req.recycle_days, req.reminder_days) {
        if recycle > 0 && reminder >= recycle {
            return Err(Error::from("回收前提醒天数必须小于未跟进自动回收天数".to_string()));
        }
    }
    // 最长保护期不得小于默认保护期（否则默认保护期无意义）
    if let (Some(recycle), Some(max)) = (req.recycle_days, req.max_recycle_days) {
        if max < recycle {
            return Err(Error::from("最长保护期天数不得小于未跟进自动回收天数".to_string()));
        }
    }

    if let Some(mode) = req.claim_mode {
        if mode != 1 && mode != 2 && mode != 3 {
            return Err(Error::from("非法的领取模式".to_string()));
        }
    }
    if let Some(enabled) = req.auto_assign_enabled {
        if enabled != 0 && enabled != 1 {
            return Err(Error::from("非法的自动分配开关值".to_string()));
        }
    }
    if let Some(mode) = req.auto_assign_mode {
        if mode != 1 && mode != 2 {
            return Err(Error::from("非法的自动分配模式".to_string()));
        }
    }
    if let Some(mode) = req.release_back_to {
        if mode != 1 && mode != 2 && mode != 3 {
            return Err(Error::from("非法的退回去向".to_string()));
        }
    }
    for (name, value) in [
        ("成单保护开关", req.deal_protect_enabled),
        ("商机豁免开关", req.opportunity_protect_enabled),
        ("自建客户占保有量开关", req.include_self_built),
        ("自建客户回收开关", req.self_built_recycle_enabled),
        ("隐藏已领取开关", req.hide_claimed),
        ("隐藏已转化开关", req.hide_converted),
        ("普通成员查看详情开关", req.allow_detail),
    ] {
        if let Some(v) = value {
            if v != 0 && v != 1 {
                return Err(Error::from(format!("非法的{}值", name)));
            }
        }
    }
    if let Some(fields) = &req.mask_fields {
        for f in fields {
            if !MASK_FIELD_WHITELIST.contains(&f.as_str()) {
                return Err(Error::from(format!("不支持的脱敏字段：{}", f)));
            }
        }
    }
    if let Some(list) = &req.deal_status {
        if list.is_empty() {
            return Err(Error::from("有效合同状态集合不能为空".to_string()));
        }
        for s in list {
            if !DEAL_STATUS_ALL.contains(s) {
                return Err(Error::from(format!("非法的合同状态值：{}", s)));
            }
        }
    }
    Ok(())
}

fn check_range(field: &str, value: Option<i32>, min: i32, max: i32) -> Result<()> {
    if let Some(v) = value {
        if v < min || v > max {
            return Err(Error::from(format!("{} 取值范围为 {} ~ {}", field, min, max)));
        }
    }
    Ok(())
}

/// JSON 数组 → 字符串数组
fn json_to_string_vec(value: &Option<serde_json::Value>) -> Option<Vec<String>> {
    value
        .as_ref()
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str().map(|s| s.to_string()))
                .collect::<Vec<String>>()
        })
}

/// JSON 数组 → i16 数组
fn json_to_i16_vec(value: &Option<serde_json::Value>) -> Option<Vec<i16>> {
    value
        .as_ref()
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_i64().map(|n| n as i16))
                .collect::<Vec<i16>>()
        })
}

/// 配置实体转 VO（JSONB mask_fields/deal_status → 数组）
pub fn model_to_vo(model: &customer_pool_config::Model) -> CustomerPoolConfigVO {
    CustomerPoolConfigVO {
        pool_id: Some(model.pool_id),
        recycle_days: model.recycle_days,
        max_recycle_days: model.max_recycle_days,
        no_opportunity_days: model.no_opportunity_days,
        no_contract_days: model.no_contract_days,
        reminder_days: model.reminder_days,
        cool_down_days: model.cool_down_days,
        claim_daily_limit: model.claim_daily_limit,
        hold_limit: model.hold_limit,
        claim_mode: model.claim_mode,
        auto_assign_enabled: model.auto_assign_enabled,
        auto_assign_mode: model.auto_assign_mode,
        mask_fields: json_to_string_vec(&model.mask_fields),
        deal_protect_enabled: model.deal_protect_enabled,
        deal_status: json_to_i16_vec(&model.deal_status),
        opportunity_protect_enabled: model.opportunity_protect_enabled,
        include_self_built: model.include_self_built,
        self_built_recycle_enabled: model.self_built_recycle_enabled,
        hide_claimed: model.hide_claimed,
        hide_converted: model.hide_converted,
        allow_detail: model.allow_detail,
        release_back_to: model.release_back_to,
        freeze_release_count: model.freeze_release_count,
        first_touch_hours: model.first_touch_hours,
        update_time: model.update_time,
    }
}

/// 无配置行时的虚拟默认值（对齐 v57 DDL 默认值）
fn default_vo(pool_id: i64) -> CustomerPoolConfigVO {
    CustomerPoolConfigVO {
        pool_id: Some(pool_id),
        recycle_days: Some(30),
        max_recycle_days: None,
        no_opportunity_days: None,
        no_contract_days: None,
        reminder_days: Some(3),
        cool_down_days: Some(7),
        claim_daily_limit: Some(0),
        hold_limit: Some(0),
        claim_mode: Some(1),
        auto_assign_enabled: Some(0),
        auto_assign_mode: Some(1),
        mask_fields: None,
        deal_protect_enabled: Some(1),
        deal_status: Some(vec![2, 3, 4]),
        opportunity_protect_enabled: Some(1),
        include_self_built: Some(0),
        self_built_recycle_enabled: Some(0),
        hide_claimed: Some(0),
        hide_converted: Some(0),
        allow_detail: Some(0),
        release_back_to: Some(1),
        freeze_release_count: Some(3),
        first_touch_hours: Some(48),
        update_time: None,
    }
}
