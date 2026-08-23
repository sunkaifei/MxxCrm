//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// 离职交接项子表（mxx_hr_resign_transfer_item）
#[derive(Clone, Default, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_hr_resign_transfer_item")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 所属交接单ID
    pub record_id: i64,
    /// 项标识：work/customer/account/asset/自定义
    pub item_key: Option<String>,
    /// 项名称（快照模板名）
    pub item_name: Option<String>,
    /// 确认人ID
    pub assignee_id: Option<i64>,
    /// 0待确认 1已确认 2不适用
    pub status: Option<i32>,
    /// 确认备注
    pub confirm_remark: Option<String>,
    pub confirm_time: Option<NaiveDateTime>,
    pub create_time: Option<NaiveDateTime>,
    pub update_time: Option<NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
