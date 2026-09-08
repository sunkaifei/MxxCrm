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

/// 客户公海自动分配规则主表
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_crm_customer_pool_assign_rule")]
pub struct Model {
    /// 主键
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 池ID
    pub pool_id: Option<i64>,
    /// 规则名称
    pub name: Option<String>,
    /// 触发事件（1=进池 2=新建 3=编辑）
    pub trigger_event: Option<i16>,
    /// 优先级（越小越先评估）
    pub priority: Option<i32>,
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