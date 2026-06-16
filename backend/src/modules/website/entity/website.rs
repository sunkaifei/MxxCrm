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
#[sea_orm(table_name = "mxx_website")]
pub struct Model {
    /// 主键
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 站点名字
    pub site_name: Option<String>,
    /// 用户id
    pub user_id: Option<i64>,
    /// 是否在首页显示Banner  1显示 0不显示
    pub show_banner: Option<i32>,
    /// 模版id
    pub template_id: Option<i64>,
    /// 二级域名
    pub domain: Option<String>,
    /// PC端的LOGO
    pub logo: Option<String>,
    /// 客户端类型，1:PC，  2:WAP，3:CMS
    pub client: Option<i32>,
    /// 搜索的关键词
    pub keywords: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 绑定的域名
    pub bind_domain: Option<String>,
    /// 站点状态，1正常；2冻结
    pub status: Option<i32>,
    /// 是否是默认站点，1是默认，0不是默认,一个用户只能有一个默认的网站
    pub is_default: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 站点备注，代理商给网站的备注，方便代理商记录这个网站干嘛的
    pub remark: Option<String>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 删除状态，1删除，0未删除
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}