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
#[sea_orm(table_name = "mxx_user")]
pub struct Model {
    /// ID
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 用户名
    pub username: Option<String>,
    /// 昵称
    pub nickname: Option<String>,
    /// 头像
    pub avatar: Option<String>,
    /// 邮箱
    pub email: Option<String>,
    /// 手机
    pub mobile: Option<String>,
    /// 登录失败次数
    pub loginfailure: Option<i32>,
    /// 上次登录时间
    pub lastlogintime: Option<DateTime>,
    /// 上次登录IP
    pub lastloginip: Option<String>,
    /// 密码
    pub password: Option<String>,
    /// 密码盐
    pub salt: Option<String>,
    /// 签名
    pub motto: Option<String>,
    /// 过期时间
    pub overdue_time: Option<DateTime>,
    
    /// 状态:0=禁用,1=启用
    pub status: Option<String>,
    /// 更新时间
    pub create_time: Option<DateTime>,
    /// 创建时间
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
