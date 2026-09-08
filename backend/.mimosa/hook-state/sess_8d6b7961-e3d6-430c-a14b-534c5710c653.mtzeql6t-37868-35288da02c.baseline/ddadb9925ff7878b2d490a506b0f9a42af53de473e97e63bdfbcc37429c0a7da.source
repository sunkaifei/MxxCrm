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

/// 工作台卡片-角色关联表（决定哪些角色能看到哪些卡片）
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_dashboard_card_role_merge")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 卡片ID
    pub card_id: Option<i64>,
    /// 角色ID
    pub role_id: Option<i64>,
    /// 创建时间
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::dashboard_card::Entity",
        from = "Column::CardId",
        to = "super::dashboard_card::Column::Id"
    )]
    DashboardCard,
}

impl Related<super::dashboard_card::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DashboardCard.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
