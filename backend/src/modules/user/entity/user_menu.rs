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
#[sea_orm(table_name = "mxx_user_menu")]
pub struct Model {
    /// ID
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 上级菜单
    pub pid: Option<i64>,
    /// 类型:menu_dir=菜单目录,menu=菜单项,button=页面按钮
    pub r#type: Option<String>,
    /// 标题
    pub title: Option<String>,
    /// 规则名称
    pub name: Option<String>,
    /// 路由路径
    pub path: Option<String>,
    /// 图标
    pub icon: Option<String>,
    /// 菜单类型:tab=选项卡,link=链接,iframe=Iframe
    pub menu_type: Option<String>,
    /// Url
    pub url: Option<String>,
    /// 组件路径
    pub component: Option<String>,
    /// 缓存:0=关闭,1=开启
    pub keepalive: Option<i32>,
    /// 扩展属性:none=无,add_rules_only=只添加为路由,add_menu_only=只添加为菜单
    pub extend: Option<String>,
    /// 备注
    pub remark: Option<String>,
    /// 权重
    pub weigh: Option<i32>,
    /// 状态:0=禁用,1=启用
    pub status: Option<String>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 创建时间
    pub create_time: Option<DateTime>,
}


#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}