//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 销售计划审批链节点实体
//! 存储提交时确定的审批人快照，防止后续组织架构变动导致审批人错乱

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_statistics_plan_approval_node")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 计划 ID
    pub plan_id: i64,
    /// 审批层级（1=一级，2=二级...）
    pub level: i32,
    /// 审批人 ID（提交时从 direct_manager_id 链快照）
    pub approver_id: i64,
    /// 审批人姓名
    pub approver_name: Option<String>,
    /// 节点状态：0=待审批，1=已通过，2=已驳回，3=已跳过
    pub status: Option<i32>,
    /// 审批意见
    pub comment: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
