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

/// 客户公海自动分配规则条目（一条规则下可多条，按 sort 升序匹配，首个匹配生效）
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_crm_customer_pool_assign_rule_entry")]
pub struct Model {
    /// 主键
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 规则ID
    pub rule_id: Option<i64>,
    /// 筛选条件（字段+运算符+值 AND 组合，JSON）
    pub conditions: Option<serde_json::Value>,
    /// 分配目标类型（1=用户 2=职位 3=群组 4=池成员）
    pub target_type: Option<i16>,
    /// 分配目标 ID 数组（JSON）
    pub target_ids: Option<serde_json::Value>,
    /// 分配方式（1=指定 2=轮询 3=权重）
    pub mode: Option<i16>,
    /// 权重分配（JSON，如 {"userId": weight}）
    pub weight: Option<serde_json::Value>,
    /// 条目排序（升序，首个匹配生效）
    pub sort: Option<i32>,
    /// 创建时间
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}