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

/// 客户公海池级规则配置（每池一行，pool_id 为主键）
#[derive(Clone, Default, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_crm_customer_pool_config")]
pub struct Model {
    /// 池ID（主键）
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub pool_id: i64,
    /// 默认保护期：未跟进自动回收天数
    pub recycle_days: Option<i32>,
    /// 最长保护期（绝对到期，NULL=无硬上限）
    pub max_recycle_days: Option<i32>,
    /// 未新增商机回收天数（NULL=关闭该规则）
    pub no_opportunity_days: Option<i32>,
    /// 未签合同回收天数（NULL=关闭该规则）
    pub no_contract_days: Option<i32>,
    /// 回收前 N 天提醒（0=不提醒）
    pub reminder_days: Option<i32>,
    /// 退回后 N 天内原负责人不得再领（0=不启用）
    pub cool_down_days: Option<i32>,
    /// 每人每日领取上限（0=不限制）
    pub claim_daily_limit: Option<i32>,
    /// 成员私海客户保有量上限（0=不限制）
    pub hold_limit: Option<i32>,
    /// 领取模式（1=自主领取 2=申领需审批 3=停用领取仅分配）
    pub claim_mode: Option<i16>,
    /// 入池自动分配开关（0=关 1=开）
    pub auto_assign_enabled: Option<i16>,
    /// 自动分配模式（1=轮询 2=权重）
    pub auto_assign_mode: Option<i16>,
    /// 轮询游标
    pub assign_cursor: Option<i32>,
    /// 脱敏字段（JSON 数组，如 ["personal_mobile","personal_email"]；NULL=不脱敏）
    pub mask_fields: Option<serde_json::Value>,
    /// 成单保护开关（0=关 1=开）
    pub deal_protect_enabled: Option<i16>,
    /// 有效合同状态集合（JSON 数组，默认 [2,3,4]）
    pub deal_status: Option<serde_json::Value>,
    /// 商机豁免开关（0=关 1=开，有进行中商机不回收）
    pub opportunity_protect_enabled: Option<i16>,
    /// 自建客户占保有量（0=不占 1=占）
    pub include_self_built: Option<i16>,
    /// 自建客户按池规则回收（0=仅统计不强制 1=按池规则）
    pub self_built_recycle_enabled: Option<i16>,
    /// 工作台隐藏已领取客户（0=否 1=是）
    pub hide_claimed: Option<i16>,
    /// 工作台隐藏已转化（有有效合同）客户（0=否 1=是）
    pub hide_converted: Option<i16>,
    /// 普通成员可查看详情（0=不可，防挑单 1=可）
    pub allow_detail: Option<i16>,
    /// 退回去向（1=原池 2=选择分组 3=指定分组）
    pub release_back_to: Option<i16>,
    /// 连续被动退回/回收达该值触发冻结（0=不冻结）
    pub freeze_release_count: Option<i32>,
    /// 首次触达 SLA（领取后 N 小时内需首跟）
    pub first_touch_hours: Option<i32>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 更新人ID
    pub updated_by: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}