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
#[sea_orm(table_name = "mxx_system_scheduler_log")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub job_id: i64,
    pub job_code: Option<String>,
    /// 0=定时触发, 1=手动触发, 2=漏跑补跑
    pub trigger_type: Option<i32>,
    /// 0=失败, 1=成功, 2=运行中, 3=中断
    pub status: Option<i32>,
    pub result_message: Option<String>,
    pub error_message: Option<String>,
    pub elapsed_ms: Option<i64>,
    pub operator_id: Option<i64>,
    pub operator_name: Option<String>,
    pub start_time: Option<DateTime>,
    pub end_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
