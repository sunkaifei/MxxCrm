//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use chrono::{NaiveDate, NaiveDateTime};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// 离职交接单主表（mxx_hr_resign_record）
#[derive(Clone, Default, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_hr_resign_record")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 离职员工ID
    pub admin_id: i64,
    /// 交接人（接手员工ID，工作交接项 assignee）
    pub transfer_to_admin_id: Option<i64>,
    /// 离职类型：1主动辞职 2协商解除 3辞退
    pub resign_type: Option<i32>,
    /// 期望离职日期
    pub resign_date: Option<NaiveDate>,
    /// 实际离职日（结算确认时填写）
    pub actual_leave_date: Option<NaiveDate>,
    /// 离职原因（敏感：仅审批链与人事可见）
    pub reason: Option<String>,
    /// 1交接中 2交接完成 3结算完成 4已离职 5已中止
    pub status: Option<i32>,
    pub create_by: Option<String>,
    pub create_time: Option<NaiveDateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
