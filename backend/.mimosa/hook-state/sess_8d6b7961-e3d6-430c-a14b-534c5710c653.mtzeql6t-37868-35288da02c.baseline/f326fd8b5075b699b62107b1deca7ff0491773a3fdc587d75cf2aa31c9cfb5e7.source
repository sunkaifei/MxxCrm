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

/// 公海池（多池体系主表）
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_crm_lead_pool")]
pub struct Model {
    /// 池ID（主键）
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 池名称（唯一）
    pub name: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 状态（1=启用 2=停用）
    pub status: Option<i16>,
    /// 排序
    pub sort: Option<i32>,
    /// 默认池标识（1=默认池，全局唯一，不可删）
    pub is_default: Option<i16>,
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
