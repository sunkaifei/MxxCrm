//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 定时任务告警日志表（仅管理员在调度管理页查看，禁止再写入公告表 mxx_notice）

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_scheduler_alert")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    /// 关联任务 id；进程级汇总类告警（无对应单一任务）可为空
    pub job_id: Option<i64>,
    pub job_code: Option<String>,
    pub job_name: Option<String>,
    /// 1=重试耗尽失败, 2=执行中断(进程退出/重载), 3=漏跑提醒
    pub alert_type: Option<i32>,
    pub message: Option<String>,
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
