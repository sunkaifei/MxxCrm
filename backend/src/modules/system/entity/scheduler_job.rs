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
#[sea_orm(table_name = "mxx_system_scheduler_job")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub job_code: String,
    pub job_name: String,
    pub cron_expression: String,
    pub handler: String,
    pub handler_params: Option<Json>,
    pub description: Option<String>,
    /// 0=系统内置, 1=用户自定义
    pub job_type: Option<i32>,
    pub enabled: Option<i32>,
    pub last_run_time: Option<DateTime>,
    /// 0=失败, 1=成功, null=未执行
    pub last_run_status: Option<i32>,
    pub last_run_result: Option<String>,
    pub next_run_time: Option<DateTime>,
    /// P2-6: 最大重试次数（0=不重试，默认3）
    pub max_retries: Option<i32>,
    /// P2-6: 重试间隔基数（秒，指数退避 base * 2^attempt），默认60
    pub retry_interval_base: Option<i32>,
    /// P2-6: 最近一次执行的重试次数（0=未重试）
    pub last_retry_count: Option<i32>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
