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
#[sea_orm(table_name = "mxx_user_platform")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user_id: i64,
    pub system_type: Option<String>,
    pub platform: Option<String>,
    pub token: Option<String>,
    pub alipay_openid: Option<String>,
    pub weixin_openid: Option<String>,
    pub weixin_unionid: Option<String>,
    pub weixin_web_openid: Option<String>,
    pub baidu_openid: Option<String>,
    pub toutiao_openid: Option<String>,
    pub toutiao_unionid: Option<String>,
    pub qq_openid: Option<String>,
    pub qq_unionid: Option<String>,
    pub kuaishou_openid: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}