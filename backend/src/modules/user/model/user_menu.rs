//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::user::entity::{user_menu, user_menu::Entity as UserMenuEntity};
use crate::utils::string_utils::serialize_option_u64_to_string;
use sea_orm::prelude::DateTime;
use sea_orm::*;


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct MenuSaveRequest {
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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct MenuUpdateRequest {
    /// ID
    pub id: Option<i64>,
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

pub struct MenuSaveDTO {
    /// ID
    pub id: Option<i64>,
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
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct MenuListVO {
    /// ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 上级菜单
    #[serde(serialize_with = "serialize_option_u64_to_string")]
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
}

impl From<user_menu::Model> for MenuListVO {
    fn from(model: user_menu::Model) -> Self {
        Self {
            id: Option::from(model.id),
            pid: model.pid,
            r#type: model.r#type,
            title: model.title,
            name: model.name,
            path: model.path,
            icon: model.icon,
            menu_type: model.menu_type,
            url: model.url,
            component: model.component,
            keepalive: model.keepalive,
            extend: None,
            remark: None,
            weigh: None,
            status: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct MenuDetailVO {
    /// ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 上级菜单
    #[serde(serialize_with = "serialize_option_u64_to_string")]
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
}

impl From<user_menu::Model> for MenuDetailVO {
    fn from(model: user_menu::Model) -> Self {
        Self {
            id: Option::from(model.id),
            pid: model.pid,
            r#type: model.r#type,
            title: model.title,
            name: model.name,
            path: model.path,
            icon: model.icon,
            menu_type: model.menu_type,
            url: model.url,
            component: model.component,
            keepalive: model.keepalive,
            extend: None,
            remark: None,
            weigh: None,
            status: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery{
    pub keywords: Option<String>,
    pub status: Option<i32>,
}

#[derive(Clone)]
pub struct PageWhere {
    pub name: Option<String>,
    pub status: Option<i32>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut name = None;
        if self.name != Some("".to_string()) {
            name = self.name.clone();
        }

        let mut status = None;
        if self.status == Some(1) || self.status == Some(0) {
            status = self.status;
        }

        Self {
            name,
            status,
        }
    }
}



pub struct UserMenuModel;

impl UserMenuModel {
    pub async fn insert(db: &DbConn, form_data: &MenuSaveDTO) -> Result<i64, DbErr> {
        let payload = user_menu::ActiveModel {
            pid:              Set(form_data.pid.to_owned()),
            r#type:           Set(form_data.r#type.to_owned()),
            title:            Set(form_data.title.to_owned()),
            name:             Set(form_data.name.to_owned()),
            path:             Set(form_data.path.to_owned()),
            icon:             Set(form_data.icon.to_owned()),
            menu_type:        Set(form_data.menu_type.to_owned()),
            url:              Set(form_data.url.to_owned()),
            component:        Set(form_data.component.to_owned()),
            keepalive:        Set(form_data.keepalive.to_owned()),
            extend:           Set(form_data.extend.to_owned()),
            remark:           Set(form_data.remark.to_owned()),
            weigh:            Set(form_data.weigh.to_owned()),
            status:           Set(form_data.status.to_owned()),
            create_time:      Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            update_time:      Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        UserMenuEntity::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
     }

    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        UserMenuEntity::delete_many()
            .filter(user_menu::Column::Id.is_in(ids))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }
    
    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, form_data: &MenuSaveDTO) -> Result<i64, DbErr> {
        let payload = user_menu::ActiveModel {
            pid:              Set(form_data.pid.to_owned()),
            r#type:           Set(form_data.r#type.to_owned()),
            title:            Set(form_data.title.to_owned()),
            name:             Set(form_data.name.to_owned()),
            path:             Set(form_data.path.to_owned()),
            icon:             Set(form_data.icon.to_owned()),
            menu_type:        Set(form_data.menu_type.to_owned()),
            url:              Set(form_data.url.to_owned()),
            component:        Set(form_data.component.to_owned()),
            keepalive:        Set(form_data.keepalive.to_owned()),
            extend:           Set(form_data.extend.to_owned()),
            remark:           Set(form_data.remark.to_owned()),
            weigh:            Set(form_data.weigh.to_owned()),
            status:           Set(form_data.status.to_owned()),
            update_time:      Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let update_result: UpdateResult = UserMenuEntity::update_many()
            .set(payload)
            .filter(user_menu::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }
    
    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<user_menu::Model>, DbErr> {
        let entity = UserMenuEntity::find_by_id(id.clone().unwrap_or_default())
            .one(db)
            .await?;
        Ok(entity)
    }
    
    pub async fn find_list(db: &DbConn, wheres: PageWhere) -> Result<Vec<user_menu::Model>, DbErr> {
        let result = UserMenuEntity::find()
            .apply_if(wheres.name, |query, v| {
                query.filter(user_menu::Column::Name.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(user_menu::Column::Status.eq(v))
            })
            .order_by_asc(user_menu::Column::Id)
            .all(db)
            .await;
        Ok(result?)
    }
    
    
}