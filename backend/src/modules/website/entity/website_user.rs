//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究。
//!

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// 网站前台用户（mxx_website_user）
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_website_user")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,

    /// 用户名（唯一）
    pub username: String,

    /// 密码（bcrypt 加密）
    #[serde(skip_serializing)]
    pub password: String,

    /// 真实姓名
    pub real_name: Option<String>,

    /// 手机号
    pub phone: Option<String>,

    /// 邮箱
    pub email: Option<String>,

    /// 头像
    pub avatar: Option<String>,

    /// 性别：0未知 1男 2女
    #[serde(default)]
    pub gender: Option<i16>,

    /// 状态：0正常 1停用
    #[serde(default)]
    pub status: Option<i32>,

    /// 会员等级
    #[serde(default)]
    pub member_level: Option<i32>,

    /// 总积分
    #[serde(default)]
    pub total_points: Option<i32>,

    /// 累计消费金额
    #[serde(default)]
    pub total_spent: Option<Decimal>,

    /// 订单数量
    #[serde(default)]
    pub order_count: Option<i32>,

    /// 最后登录时间
    pub last_login_time: Option<DateTime>,

    /// 最后登录IP
    pub last_login_ip: Option<String>,

    /// 注册IP
    pub register_ip: Option<String>,

    /// 注册来源：website / wechat / admin
    #[serde(default = "default_register_source")]
    pub register_source: Option<String>,

    /// 微信 open_id
    pub open_id: Option<String>,

    /// 微信 union_id
    pub union_id: Option<String>,

    /// 备注
    pub remark: Option<String>,

    /// 创建时间
    pub create_time: Option<DateTime>,

    /// 更新时间
    pub update_time: Option<DateTime>,

    /// 软删除：0未删除 1已删除
    #[serde(default)]
    pub deleted: Option<i32>,
}

fn default_register_source() -> Option<String> {
    Some("website".to_string())
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
