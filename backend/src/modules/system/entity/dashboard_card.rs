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

/// 工作台卡片定义（集中控制各页面统计/概览卡片对哪些角色可见）
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_dashboard_card")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 卡片编码（全局唯一，如 payslip_stat）
    pub card_code: Option<String>,
    /// 卡片名称
    pub card_name: Option<String>,
    /// 所属页面标识（如 finance/payslip，用于前端按页过滤）
    pub page_key: Option<String>,
    /// 显示顺序
    pub sort_order: Option<i32>,
    /// 状态（1启用 0停用）
    pub status: Option<i32>,
    /// 备注
    pub remark: Option<String>,
    /// 删除标志（0存在 1删除）
    pub deleted: Option<i32>,
    /// 创建者
    pub create_by: Option<String>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新者
    pub update_by: Option<String>,
    /// 更新时间
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        has_many = "super::dashboard_card_role_merge::Entity",
        from = "Column::Id",
        to = "super::dashboard_card_role_merge::Column::CardId"
    )]
    DashboardCardRoleMerge,
}

impl Related<super::dashboard_card_role_merge::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DashboardCardRoleMerge.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
