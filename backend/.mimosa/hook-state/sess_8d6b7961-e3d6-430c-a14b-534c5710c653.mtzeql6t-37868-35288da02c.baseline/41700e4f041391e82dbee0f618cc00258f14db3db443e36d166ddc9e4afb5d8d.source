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

/// 公海客户申领申请（claim_mode=2 的池）
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_crm_customer_pool_apply")]
pub struct Model {
    /// 主键
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 池ID
    pub pool_id: Option<i64>,
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 申请人ID
    pub user_id: Option<i64>,
    /// 状态（1=待审批 2=通过 3=拒绝）
    pub status: Option<i16>,
    /// 申领理由
    pub reason: Option<String>,
    /// 审批人ID
    pub audit_by: Option<i64>,
    /// 审批时间
    pub audit_time: Option<DateTime>,
    /// 审批备注
    pub audit_remark: Option<String>,
    /// 创建时间
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}