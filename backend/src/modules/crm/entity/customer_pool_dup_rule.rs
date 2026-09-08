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

/// 客户查重规则（新建/导入/线索转客户时依 fields 组合查重）
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_crm_customer_pool_dup_rule")]
pub struct Model {
    /// 主键
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 规则名称
    pub name: Option<String>,
    /// 查重字段（JSON 数组，如 ["company_name","personal_mobile"]）
    pub fields: Option<serde_json::Value>,
    /// 控制强度（1=提示 2=阻断 3=需审批）
    pub strength: Option<i16>,
    /// 最近联系日期有效天数（0=不过期）
    pub valid_days: Option<i32>,
    /// 名称连续匹配度阈值（0-100）
    pub match_ratio: Option<i32>,
    /// 启用状态（1=启用 0=停用）
    pub enabled: Option<i16>,
    /// 创建人ID
    pub created_by: Option<i64>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新人ID
    pub updated_by: Option<i64>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 软删除标识
    pub deleted: Option<i32>,
    /// 删除人ID
    pub delete_by: Option<i64>,
    /// 删除时间
    pub delete_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}