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

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_crm_customer_assign_history")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 负责人（销售人员）ID
    pub admin_id: Option<i64>,
    /// 操作类型：1=领取，2=退回公海，3=管理员分配
    pub action_type: Option<i16>,
    /// 开始负责时间
    pub start_time: Option<DateTime>,
    /// 结束负责时间（NULL表示正在负责）
    pub end_time: Option<DateTime>,
    /// 备注
    pub remark: Option<String>,
    /// 操作人ID
    pub operated_by: Option<i64>,
    /// 创建时间
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
