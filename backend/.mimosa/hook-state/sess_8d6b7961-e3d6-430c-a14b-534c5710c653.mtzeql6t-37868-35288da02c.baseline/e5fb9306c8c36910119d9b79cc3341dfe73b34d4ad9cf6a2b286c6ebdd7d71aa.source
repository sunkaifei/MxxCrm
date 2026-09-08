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
use crate::modules::crm::entity::lead_pool_config;
use crate::modules::crm::model::lead_pool::{PoolConfigSaveRequest, PoolConfigVO};
use crate::modules::crm::service::pool_service;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, QueryFilter, Set,
};
use sea_orm::sea_query::OnConflict;

/// 脱敏字段白名单（手机/座机/邮箱/微信）
const MASK_FIELD_WHITELIST: [&str; 4] = ["mobile", "phone", "email", "wechat"];

/// 确保池配置行存在（建池时调用；已存在则跳过）
pub async fn ensure_config(db: &DbConn, pool_id: i64) -> Result<()> {
    let active = lead_pool_config::ActiveModel {
        pool_id: Set(pool_id),
        assign_cursor: Set(Some(0)),
        ..Default::default()
    };
    let result = lead_pool_config::Entity::insert(active)
        .on_conflict(OnConflict::column(lead_pool_config::Column::PoolId).do_nothing().to_owned())
        .exec(db)
        .await;
    match result {
        Ok(_) | Err(DbErr::RecordNotInserted) => Ok(()),
        Err(e) => Err(Error::from(e.to_string())),
    }
}

/// 查询原始配置行
pub async fn get_raw(db: &DbConn, pool_id: i64) -> Result<Option<lead_pool_config::Model>> {
    lead_pool_config::Entity::find()
        .filter(lead_pool_config::Column::PoolId.eq(pool_id))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))
}

/// 查询配置 VO（不存在时返回 DB 默认参数的虚拟配置）
pub async fn get_vo(db: &DbConn, pool_id: i64) -> Result<PoolConfigVO> {
    pool_service::find_by_id(db, pool_id)
        .await?
        .ok_or_else(|| Error::from("公海池不存在".to_string()))?;

    match get_raw(db, pool_id).await? {
        Some(model) => Ok(model_to_vo(&model)),
        None => Ok(PoolConfigVO {
            pool_id: Some(pool_id),
            recycle_days: Some(30),
            reminder_days: Some(3),
            cool_down_days: Some(7),
            claim_daily_limit: Some(0),
            hold_limit: Some(0),
            claim_mode: Some(1),
            auto_assign_enabled: Some(0),
            auto_assign_mode: Some(1),
            mask_fields: None,
            update_time: None,
        }),
    }
}

/// 保存池配置（参数边界校验 + upsert）
pub async fn save(db: &DbConn, req: &PoolConfigSaveRequest, updated_by: i64) -> Result<()> {
    let pool_id = req.pool_id.ok_or_else(|| Error::from("池ID不能为空".to_string()))?;
    pool_service::find_by_id(db, pool_id)
        .await?
        .ok_or_else(|| Error::from("公海池不存在".to_string()))?;

    validate(req)?;

    let now = chrono::Local::now().naive_local().to_owned();
    let mask_json: Option<serde_json::Value> = req.mask_fields.as_ref().map(|fields| {
        serde_json::Value::Array(fields.iter().map(|f| serde_json::Value::String(f.clone())).collect())
    });

    let active = lead_pool_config::ActiveModel {
        pool_id: Set(pool_id),
        assign_cursor: Default::default(),
        recycle_days: Set(req.recycle_days),
        reminder_days: Set(req.reminder_days),
        cool_down_days: Set(req.cool_down_days),
        claim_daily_limit: Set(req.claim_daily_limit),
        hold_limit: Set(req.hold_limit),
        claim_mode: Set(req.claim_mode),
        auto_assign_enabled: Set(req.auto_assign_enabled),
        auto_assign_mode: Set(req.auto_assign_mode),
        mask_fields: Set(mask_json),
        update_time: Set(Some(now)),
        updated_by: Set(Some(updated_by)),
    };

    let result = lead_pool_config::Entity::insert(active)
        .on_conflict(
            OnConflict::column(lead_pool_config::Column::PoolId)
                .update_columns([
                    lead_pool_config::Column::RecycleDays,
                    lead_pool_config::Column::ReminderDays,
                    lead_pool_config::Column::CoolDownDays,
                    lead_pool_config::Column::ClaimDailyLimit,
                    lead_pool_config::Column::HoldLimit,
                    lead_pool_config::Column::ClaimMode,
                    lead_pool_config::Column::AutoAssignEnabled,
                    lead_pool_config::Column::AutoAssignMode,
                    lead_pool_config::Column::MaskFields,
                    lead_pool_config::Column::UpdateTime,
                    lead_pool_config::Column::UpdatedBy,
                ])
                .to_owned(),
        )
        .exec(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    let _ = result;
    Ok(())
}

/// 参数边界校验（方案 §5.2：0 ≤ 值 ≤ 365 等）
fn validate(req: &PoolConfigSaveRequest) -> Result<()> {
    check_range("未跟进自动回收天数", req.recycle_days, 0, 365)?;
    check_range("回收前提醒天数", req.reminder_days, 0, 365)?;
    check_range("退回冷却期天数", req.cool_down_days, 0, 365)?;
    check_range("每人每日领取上限", req.claim_daily_limit, 0, 999)?;
    check_range("私海保有量上限", req.hold_limit, 0, 9999)?;

    // 提醒天数必须小于回收天数（否则提醒无意义）
    if let (Some(recycle), Some(reminder)) = (req.recycle_days, req.reminder_days) {
        if recycle > 0 && reminder >= recycle {
            return Err(Error::from("回收前提醒天数必须小于未跟进自动回收天数".to_string()));
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
    if let Some(fields) = &req.mask_fields {
        for f in fields {
            if !MASK_FIELD_WHITELIST.contains(&f.as_str()) {
                return Err(Error::from(format!("不支持的脱敏字段：{}", f)));
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

/// 配置实体转 VO（JSONB mask_fields → 字符串数组）
pub fn model_to_vo(model: &lead_pool_config::Model) -> PoolConfigVO {
    let mask_fields = model.mask_fields.as_ref().and_then(|v| v.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|item| item.as_str().map(|s| s.to_string()))
            .collect::<Vec<String>>()
    });

    PoolConfigVO {
        pool_id: Some(model.pool_id),
        recycle_days: model.recycle_days,
        reminder_days: model.reminder_days,
        cool_down_days: model.cool_down_days,
        claim_daily_limit: model.claim_daily_limit,
        hold_limit: model.hold_limit,
        claim_mode: model.claim_mode,
        auto_assign_enabled: model.auto_assign_enabled,
        auto_assign_mode: model.auto_assign_mode,
        mask_fields,
        update_time: model.update_time,
    }
}
