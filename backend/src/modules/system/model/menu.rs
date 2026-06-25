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
use crate::modules::system::entity::{admin_role_merge, menu, menu::Entity as Menu, role_menu_merge};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string, u64_to_string};
use chrono::Local;
use sea_orm::prelude::DateTime;
use sea_orm::*;

/// 菜单类型常量
pub const MENU_TYPE_MENU: &str = "MENU";      // 菜单
pub const MENU_TYPE_FOLDER: &str = "FOLDER";  // 目录
pub const MENU_TYPE_LINK: &str = "LINK";      // 外链
pub const MENU_TYPE_BUTTON: &str = "BUTTON";  // 按钮

/// 菜单元数据请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MenuMetaRequest {
    /// 菜单名称
    pub name: Option<String>,
    /// 固定标签页（1-是 2-否）
    pub affix_tab: Option<i32>,
    /// 子级不展现（1-是 2-否）
    pub hide_children_in_menu: Option<i32>,
    /// 面包屑中不展现（1-是 2-否）
    pub hide_in_breadcrumb: Option<i32>,
    /// 菜单中不展现（1-是 2-否）
    pub hide_in_menu: Option<i32>,
    /// 标签页中不展现（1-是 2-否）
    pub hide_in_tab: Option<i32>,
    /// 【菜单】是否开启页面缓存（1-是 0-否）
    pub keep_alive: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 菜单图标
    pub icon: Option<String>,
}

/// 菜单保存/更新请求（支持嵌套 meta 格式）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MenuSaveRequest {
    /// ID（更新时使用，添加时为 None）
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 父菜单ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub parent_id: Option<i64>,
    /// 菜单类型（MENU-菜单 FOLDER-目录 LINK-外链 BUTTON-按钮）
    pub r#type: Option<String>,
    /// 路由名称（Vue Router 中用于命名路由）
    pub route_name: Option<String>,
    /// 路由路径（Vue Router 中定义的 URL 路径）
    pub path: Option<String>,
    /// 组件路径（组件页面完整路径，相对于 src/views/，缺省后缀 .vue）
    pub component: Option<String>,
    /// 【按钮】权限标识
    pub perm: Option<String>,
    /// 显示状态（1-显示 2-隐藏）
    pub status: Option<i32>,
    /// 跳转路径
    pub redirect: Option<String>,
    /// 路由参数
    pub params: Option<serde_json::Value>,
    /// 菜单元数据（嵌套对象）
    pub meta: Option<MenuMetaRequest>,
}

/// 菜单更新请求（别名，与 MenuSaveRequest 结构相同）
pub type MenuUpdateRequest = MenuSaveRequest;

/// 从包含 meta 的请求转换为 MenuSaveDTO
fn convert_to_dto(id: Option<i64>, parent_id: Option<i64>, r#type: Option<String>, 
                  route_name: Option<String>, path: Option<String>, component: Option<String>,
                  perm: Option<String>, status: Option<i32>, redirect: Option<String>,
                  params: Option<serde_json::Value>, meta: Option<MenuMetaRequest>) -> MenuSaveDTO {
    // 从 meta 中提取字段
    let (name, affix_tab, hide_children_in_menu, hide_in_breadcrumb, 
         hide_in_menu, hide_in_tab, keep_alive, sort, icon) = 
        if let Some(m) = meta {
            (
                m.name, m.affix_tab, m.hide_children_in_menu, m.hide_in_breadcrumb,
                m.hide_in_menu, m.hide_in_tab, m.keep_alive, m.sort, m.icon,
            )
        } else {
            (None, None, None, None, None, None, None, None, None)
        };
    
    MenuSaveDTO {
        id,
        parent_id,
        name,
        r#type,
        route_name,
        path,
        component,
        perm,
        status,
        affix_tab,
        hide_children_in_menu,
        hide_in_breadcrumb,
        hide_in_menu,
        hide_in_tab,
        keep_alive,
        sort,
        icon,
        redirect,
        params,
    }
}

impl From<MenuSaveRequest> for MenuSaveDTO {
    fn from(req: MenuSaveRequest) -> Self {
        convert_to_dto(
            req.id, req.parent_id, req.r#type, req.route_name, req.path,
            req.component, req.perm, req.status, req.redirect, req.params, req.meta
        )
    }
}

#[derive(Debug, Clone)]
pub struct MenuSaveDTO {
    /// ID
    pub id: Option<i64>,
    /// 父菜单ID
    pub parent_id: Option<i64>,
    /// 菜单名称
    pub name: Option<String>,
    /// 菜单类型（MENU-菜单 FOLDER-目录 LINK-外链 BUTTON-按钮）
    pub r#type: Option<String>,  // 使用 `r#` 前缀来避免与关键字冲突
    /// 路由名称（Vue Router 中用于命名路由）
    pub route_name: Option<String>,
    /// 路由路径（Vue Router 中定义的 URL 路径）
    pub path: Option<String>,
    /// 组件路径（组件页面完整路径，相对于 src/views/，缺省后缀 .vue）
    pub component: Option<String>,
    /// 【按钮】权限标识
    pub perm: Option<String>,
    /// 显示状态（1-显示 2-隐藏）
    pub status: Option<i32>,
    /// 固定标签页（1-是 2-否）
    pub affix_tab: Option<i32>,
    /// 子级不展现（1-是 2-否）
    pub hide_children_in_menu: Option<i32>,
    /// 面包屑中不展现（1-是 2-否）
    pub hide_in_breadcrumb: Option<i32>,
    /// 菜单中不展现（1-是 2-否）
    pub hide_in_menu: Option<i32>,
    /// 标签页中不展现（1-是 2-否）
    pub hide_in_tab: Option<i32>,
    /// 【菜单】是否开启页面缓存（1-是 0-否）
    pub keep_alive: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 菜单图标
    pub icon: Option<String>,
    /// 跳转路径
    pub redirect: Option<String>,
    /// 路由参数
    pub params: Option<serde_json::Value>,
}


#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Router {
    pub id: i64,
    /// 路由路径
    pub path: Option<String>,
    /// 组件路径（组件页面完整路径，相对于 src/views/，缺省后缀 .vue）
    pub component: Option<String>,
    /// 菜单名称
    pub name: Option<String>,
    /// 路由参数
    pub meta: Meta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Router>>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    /// 菜单名称
    pub title: Option<String>,
    /// 菜单图标
    pub icon: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 是否在菜单中隐藏
    pub hide_in_menu: Option<bool>,
}

/// 菜单下拉树形结构
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MenuOptionsVO {
    /// 菜单ID
    #[serde(serialize_with = "u64_to_string")]
    pub value: i64,
    /// 菜单名称
    pub label: Option<String>,
    /// 父菜单ID
    pub parent_id: Option<i64>,
    /// 子菜单
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<MenuOptionsVO>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListMeta {
    /// 菜单名称
    pub name: Option<String>,
    /// 固定标签页（1-是 2-否）
    pub affix_tab: Option<i32>,
        /// 子级不展现（1-是 2-否）
    pub hide_children_in_menu: Option<i32>,
    /// 面包屑中不展现（1-是 2-否）
    pub hide_in_breadcrumb: Option<i32>,
    /// 菜单中不展现（1-是 2-否）
    pub hide_in_menu: Option<i32>,
    /// 标签页中不展现（1-是 2-否）
    pub hide_in_tab: Option<i32>,
    /// 【菜单】是否开启页面缓存（1-是 0-否）
    pub keep_alive: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 菜单图标
    pub icon: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MenuAdminVO {
    #[serde(serialize_with = "u64_to_string")]
    pub id: i64,
    /// 父菜单ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub parent_id: Option<i64>,
    /// 父节点ID路径
    #[serde(skip)]
    pub tree_path: Option<String>,
    /// 菜单名称
    pub name: Option<String>,
    pub meta: Option<ListMeta>,
    /// 菜单类型（MENU-菜单 FOLDER-目录 LINK-外链 BUTTON-按钮）
    pub r#type: Option<String>,  // 使用 `r#` 前缀来避免与关键字冲突
    /// 路由名称（Vue Router 中用于命名路由）
    pub route_name: Option<String>,
    /// 路由路径（Vue Router 中定义的 URL 路径）
    pub path: Option<String>,
    /// 组件路径（组件页面完整路径，相对于 src/views/，缺省后缀 .vue）
    pub component: Option<String>,
    /// 【按钮】权限标识
    pub perm: Option<String>,
    /// 显示状态（1-显示 2-隐藏）
    pub status: Option<i32>,
    /// 跳转路径
    pub redirect: Option<String>,
    /// 创建时间
    #[serde(skip)]
    pub create_time: Option<String>,
    /// 更新时间
    pub update_time: Option<String>,
    /// 路由参数
    pub params: Option<serde_json::Value>,
    pub children: Vec<MenuAdminVO>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct MenuDetailVO {
    /// ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 父菜单ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub parent_id: Option<i64>,
    /// 菜单名称
    pub name: Option<String>,
    /// 菜单类型（MENU-菜单 FOLDER-目录 LINK-外链 BUTTON-按钮）
    pub r#type: Option<String>,  // 使用 `r#` 前缀来避免与关键字冲突
    /// 路由名称（Vue Router 中用于命名路由）
    pub route_name: Option<String>,
    /// 路由路径（Vue Router 中定义的 URL 路径）
    pub path: Option<String>,
    /// 组件路径（组件页面完整路径，相对于 src/views/，缺省后缀 .vue）
    pub component: Option<String>,
    /// 【按钮】权限标识
    pub perm: Option<String>,
    /// 显示状态（1-显示 2-隐藏）
    pub status: Option<i32>,
    /// 固定标签页（1-是 2-否）
    pub affix_tab: Option<i32>,
    /// 子级不展现（1-是 2-否）
    pub hide_children_in_menu: Option<i32>,
    /// 面包屑中不展现（1-是 2-否）
    pub hide_in_breadcrumb: Option<i32>,
    /// 菜单中不展现（1-是 2-否）
    pub hide_in_menu: Option<i32>,
    /// 标签页中不展现（1-是 2-否）
    pub hide_in_tab: Option<i32>,
    /// 【菜单】是否开启页面缓存（1-是 0-否）
    pub keep_alive: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 菜单图标
    pub icon: Option<String>,
    /// 跳转路径
    pub redirect: Option<String>,
    /// 路由参数
    pub params: Option<serde_json::Value>,
}

impl From<menu::Model> for MenuDetailVO{
    fn from(model: menu::Model) -> Self {
        MenuDetailVO {
            id: Some(model.id),
            parent_id: Some(model.parent_id),
            name: model.name,
            r#type: model.r#type,
            route_name: model.route_name,
            path: model.path,
            component: model.component,
            perm: model.perm,
            status: Some(model.status),
            affix_tab: Some(model.affix_tab),
            hide_children_in_menu: Some(model.hide_children_in_menu),
            hide_in_breadcrumb: Some(model.hide_in_breadcrumb),
            hide_in_menu: Some(model.hide_in_menu),
            hide_in_tab: Some(model.hide_in_tab),
            keep_alive: Some(model.keep_alive),
            sort: model.sort,
            icon: model.icon,
            redirect: model.redirect,
            params: model.params,
        }
    }
}

#[derive(Iden)]
pub enum MenuEnum {
    Table,
    Id,
    Content,
    CreatedDate,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub keywords: Option<String>,
    pub status: Option<i32>,
}

/// 条件
#[derive(Clone)]
pub struct PageWhere {
    pub name: Option<String>,
    pub status: Option<i32>,
}

impl PageWhere {
    pub fn format(&self) -> Self {
        let mut name = None;
        if self.name != Some("".to_string()) {
            name = self.name.clone();
        }

        Self {
            name,
            status: self.status.clone(),
        }
    }
}


pub struct MenuModel;

impl MenuModel {

    /// ### 新增菜单
    /// * `db` 数据库连接对象
    /// * `form_data`: 表单数据
    pub async fn insert(db: &DbConn, form_data: &MenuSaveDTO) -> Result<i64, DbErr> {
        let payload = menu::ActiveModel {
            parent_id:              Set(form_data.parent_id.to_owned().unwrap_or_default()),
            name:                   Set(form_data.name.to_owned()),
            r#type:                 Set(form_data.r#type.to_owned()),
            route_name:             Set(form_data.route_name.to_owned()),
            path:                   Set(form_data.path.to_owned()),
            component:              Set(form_data.component.to_owned()),
            perm:                   Set(form_data.perm.to_owned()),
            status:                 Set(form_data.status.to_owned().unwrap_or(1)),
            affix_tab:              Set(form_data.affix_tab.to_owned().unwrap_or(0)),
            hide_children_in_menu:  Set(form_data.hide_children_in_menu.to_owned().unwrap_or(0)),
            hide_in_breadcrumb:     Set(form_data.hide_in_breadcrumb.to_owned().unwrap_or(0)),
            hide_in_menu:           Set(form_data.hide_in_menu.to_owned().unwrap_or(0)),
            hide_in_tab:            Set(form_data.hide_in_tab.to_owned().unwrap_or(0)),
            keep_alive:             Set(form_data.keep_alive.to_owned().unwrap_or(0)),
            sort:                   Set(form_data.sort.to_owned()),
            icon:                   Set(form_data.icon.to_owned()),
            redirect:               Set(form_data.redirect.to_owned()),
            params:                 Set(form_data.params.to_owned()),
            create_time:            Set(Option::from(Local::now().naive_utc())),
            update_time:           Set(Option::from(Local::now().naive_utc())),
            ..Default::default()
        };

        Menu::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// ### 按id更新菜单
    /// * `db` 数据库连接对象
    /// * `id` 需要更新的菜单id
    /// * `menu` 菜单数据
    /// 
    /// 返回值：更新的条数
    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, form_data: &MenuSaveDTO) -> Result<i64, DbErr> {
        let menu_data = menu::ActiveModel{
            parent_id:              Set(form_data.parent_id.to_owned().unwrap_or_default()),
            name:                   Set(form_data.name.to_owned()),
            r#type:                 Set(form_data.r#type.to_owned()),
            route_name:             Set(form_data.route_name.to_owned()),
            path:                   Set(form_data.path.to_owned()),
            component:              Set(form_data.component.to_owned()),
            perm:                   Set(form_data.perm.to_owned()),
            status:                 Set(form_data.status.to_owned().unwrap_or(1)),
            affix_tab:              Set(form_data.affix_tab.to_owned().unwrap_or(0)),
            hide_children_in_menu:  Set(form_data.hide_children_in_menu.to_owned().unwrap_or(0)),
            hide_in_breadcrumb:     Set(form_data.hide_in_breadcrumb.to_owned().unwrap_or(0)),
            hide_in_menu:           Set(form_data.hide_in_menu.to_owned().unwrap_or(0)),
            hide_in_tab:            Set(form_data.hide_in_tab.to_owned().unwrap_or(0)),
            keep_alive:             Set(form_data.keep_alive.to_owned().unwrap_or(0)),
            sort:                   Set(form_data.sort.to_owned()),
            icon:                   Set(form_data.icon.to_owned()),
            redirect:               Set(form_data.redirect.to_owned()),
            params:                 Set(form_data.params.to_owned()),
            update_time:           Set(Option::from(Local::now().naive_utc())),
            ..Default::default()
        };

        let update_result: UpdateResult = Menu::update_many()
            .set(menu_data)
            .filter(menu::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }

    /// ### 批量删除菜单
    /// * `db` 数据库连接对象
    /// * `ids` 菜单ID集合
    /// 
    /// 返回值：受影响的行数
    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        let delete_result = Menu::delete_many()
            .filter(menu::Column::Id.is_in(ids))
            .exec(db)
            .await?;

        Ok(delete_result.rows_affected as i64)
    }

    
    /// ### 根据ID查询菜单
    /// * `db` 数据库连接对象
    /// * `id` 菜单ID
    /// 
    /// 返回值：菜单对象
    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<menu::Model>, DbErr> {
        Menu::find_by_id(id.clone().unwrap_or_default())
            .one(db)
            .await
    }

    
    /// ### 校验菜单名称是否唯一
    /// * `db` 数据库连接对象
    /// * `name` 菜单名称
    /// * `parent_id` 父菜单ID
    /// * `id` 是否排除菜单ID
    /// 
    /// 返回值：查询到的条数
    pub async fn find_by_name_unique(db: &DbConn, name: &Option<String>, parent_id: &Option<i64>, id: &Option<i64>) -> Result<i64, DbErr> {
        let name_val = name.clone().unwrap_or_default();
        let mut query = Menu::find()
            .filter(menu::Column::Name.eq(name_val));
        if let Some(pid) = parent_id.clone() {
            query = query.filter(menu::Column::ParentId.eq(pid));
        }
        if let Some(exclude_id) = id.clone() {
            query = query.filter(menu::Column::Id.ne(exclude_id));
        }
        query.count(db).await.map(|c| c as i64)
    }
    
    /// ### 校验路由路径是否唯一
    /// * `db` 数据库连接对象
    /// * `path` 路由路径
    /// * `parent_id` 父菜单ID
    /// * `id` 是否排除菜单ID
    /// 
    /// 返回值：查询到的条数
    pub async fn find_by_path_unique(db: &DbConn, path: &Option<String>, parent_id: &Option<i64>,id: &Option<i64>) -> Result<i64, DbErr> {
        let path_val = path.clone().unwrap_or_default();
        let pid_val = parent_id.clone().unwrap_or_default();
        let mut query = Menu::find()
            .filter(menu::Column::Path.eq(path_val))
            .filter(menu::Column::ParentId.eq(pid_val));
        if let Some(exclude_id) = id.clone() {
            query = query.filter(menu::Column::Id.ne(exclude_id));
        }
        query.count(db).await.map(|c| c as i64)
    }
    
    /// ### 校验权限标识是否唯一
    /// * `db` 数据库连接对象
    /// * `perm` 权限标识
    /// * `id` 是否排除菜单ID
    /// 
    /// 返回值：查询到的条数
    pub async fn find_by_perms_unique(db: &DbConn, perm: &Option<String>, id: &Option<i64>) -> Result<i64, DbErr> {
        let perm_val = perm.clone().unwrap_or_default();
        let mut query = Menu::find()
            .filter(menu::Column::Perm.eq(perm_val));
        if let Some(exclude_id) = id.clone() {
            query = query.filter(menu::Column::Id.ne(exclude_id));
        }
        query.count(db).await.map(|c| c as i64)
    }

    /// ### 校验路由名称是否唯一
    /// * `db` 数据库连接对象
    /// * `route_name` 路由名称
    /// * `id` 是否排除菜单ID
    /// 
    /// 返回值：查询到的条数
    pub async fn find_by_route_name_unique(db: &DbConn, route_name: &Option<String>, id: &Option<i64>) -> Result<i64, DbErr> {
        let route_val = route_name.clone().unwrap_or_default();
        let mut query = Menu::find()
            .filter(menu::Column::RouteName.eq(route_val));
        if let Some(exclude_id) = id.clone() {
            query = query.filter(menu::Column::Id.ne(exclude_id));
        }
        query.count(db).await.map(|c| c as i64)
    }

    /// ### 查询所有菜单
    /// * `db` 数据库连接对象
    /// 
    /// 返回值：菜单列表`Vec<menu::Model>`，如果查询失败，则返回错误信息。
    pub async fn find_all(db: &DbConn) -> Result<Vec<menu::Model>, DbErr> {
        Menu::find()
            .filter(menu::Column::Deleted.eq(0))
            .filter(menu::Column::Status.eq(1))
            .order_by_asc(menu::Column::Sort)
            .all(db)
            .await
    }

    pub async fn select_list_by_admin_id(
        db: &DbConn,
        admin_id: &Option<i64>
    ) -> Result<Vec<menu::Model>, DbErr> {
        Menu::find()
            .distinct()
            .join_rev(
                JoinType::LeftJoin,
                role_menu_merge::Relation::Menu.def(),
            )
            .join_rev(
                JoinType::LeftJoin,
                admin_role_merge::Relation::AdminRoleMerge.def(),
            )
            .filter(admin_role_merge::Column::AdminId.eq(admin_id.clone().unwrap_or_default()))
            .filter(menu::Column::Deleted.eq(0))
            .filter(menu::Column::Status.eq(1))
            .order_by_asc(menu::Column::Sort)
            .all(db)
            .await
    }
    
    /// 查询所有菜单
    pub async fn select_all_list(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<Vec<menu::Model>, DbErr> {
        Menu::find()
            .apply_if(wheres.name, |query, v| {
                query.filter(menu::Column::Name.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(menu::Column::Status.eq(v))
            })
            .order_by_asc(menu::Column::Sort)
            .all(db)
            .await
    }

    pub async fn find_by_parent_id(db: &DbConn, parent_id: i64) -> Result<Vec<menu::Model>, DbErr> {
        Menu::find()
            .filter(menu::Column::ParentId.eq(parent_id))
            .order_by_asc(menu::Column::Sort)
            .all(db)
            .await
    }
    
    /// 查询菜单下级数量
    pub async fn select_in_column(db: &DbConn, ids: &Vec<i64>) -> Result<i64, DbErr> {
        let result = Menu::find()
            .filter(menu::Column::ParentId.is_in(ids.clone()))
            .all(db)
            .await;
        Ok(result?.len() as i64)
    }
}