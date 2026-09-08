//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// 公海池级规则配置（每池一行，pool_id 为主键）
#[derive(Clone, Default, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_crm_lead_pool_config")]
pub struct Model {
    /// 池ID（主键）
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub pool_id: i64,
    /// 未跟进自动回收天数
    pub recycle_days: Option<i32>,
    /// 回收前 N 天提醒（0=不提醒）
    pub reminder_days: Option<i32>,
    /// 退回后 N 天内原负责人不得再领（0=不启用）
    pub cool_down_days: Option<i32>,
    /// 每人每日领取上限（0=不限制）
    pub claim_daily_limit: Option<i32>,
    /// 成员私海线索保有量上限（0=不限制）
    pub hold_limit: Option<i32>,
    /// 领取模式（1=自主领取 2=申领需审批 3=停用领取仅分配）
    pub claim_mode: Option<i16>,
    /// 入池自动分配开关（0=关 1=开）
    pub auto_assign_enabled: Option<i16>,
    /// 自动分配模式（1=轮询 2=负载均衡）
    pub auto_assign_mode: Option<i16>,
    /// 轮询游标（乐观锁）
    pub assign_cursor: Option<i32>,
    /// 脱敏字段（JSON 数组，如 ["mobile","phone","email"]；NULL=不脱敏）
    pub mask_fields: Option<serde_json::Value>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 更新人ID
    pub updated_by: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
