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

/// 线索归属历史（列结构对齐 mxx_crm_customer_assign_history，另加 pool_id 留痕涉池动作）
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_crm_lead_assign_history")]
pub struct Model {
    /// 主键
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 线索ID
    pub lead_id: Option<i64>,
    /// 归属人ID（自动回收产生的关闭记录为 NULL）
    pub admin_id: Option<i64>,
    /// 操作类型：1=领取 2=退回 3=管理员分配 4=转移 5=自动回收 6=管理员收回 7=申领通过
    pub action_type: Option<i16>,
    /// 本次归属开始
    pub start_time: Option<DateTime>,
    /// 本次归属结束（NULL=正在负责）
    pub end_time: Option<DateTime>,
    /// 备注
    pub remark: Option<String>,
    /// 退回原因类型（复用 1/2/3/4/9）
    pub reason_type: Option<i16>,
    /// 退回补充说明
    pub reason: Option<String>,
    /// 涉及池
    pub pool_id: Option<i64>,
    /// 操作人ID
    pub operated_by: Option<i64>,
    /// 创建时间
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
