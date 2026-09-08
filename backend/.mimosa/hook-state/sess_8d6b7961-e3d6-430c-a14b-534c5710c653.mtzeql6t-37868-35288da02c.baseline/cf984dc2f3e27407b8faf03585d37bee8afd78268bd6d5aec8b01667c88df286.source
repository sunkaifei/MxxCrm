//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 验证码存储实体（文字点选方案 d49：Db 存储态使用；Cache 态不走本表）

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_captcha")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 验证码会话 key（uuid）
    pub captcha_key: Option<String>,
    /// 类型：click_verify（点选会话）/ click_ticket（登录二次凭证）
    pub captcha_type: Option<String>,
    /// 绑定维度：登录账号或 IP
    pub user_key: Option<String>,
    /// 业务数据 JSONB（点选: 字符与坐标数组；ticket: 上下文）
    pub data: Option<serde_json::Value>,
    /// 已错误尝试次数
    pub tries: Option<i32>,
    /// 过期时间
    pub expire_time: Option<DateTime>,
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
