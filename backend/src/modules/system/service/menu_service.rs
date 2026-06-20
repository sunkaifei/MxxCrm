//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use std::collections::{HashMap, HashSet};
use sea_orm::DbConn;
use crate::core::errors::error::{Error, Result};
use crate::modules::system::entity::menu::Model;
use crate::modules::system::model::admin::AdminModel;
use crate::modules::system::model::admin_role_merge::AdminRoleMergeModel;
use crate::modules::system::model::menu;
use crate::modules::system::model::menu::{ListQuery, ListMeta, MenuAdminVO, MenuDetailVO, MenuModel, MenuOptionsVO, MenuSaveDTO, MenuSaveRequest, MenuUpdateRequest, PageWhere, Router, MENU_TYPE_BUTTON};
use crate::modules::system::model::role::RoleModel;
use crate::modules::system::model::role_menu_merge::RoleMenuMergeModel;
use crate::utils::string_utils::{convert_vec_option_string_to_vec_u64};

pub async fn insert(db: &DbConn, payload: MenuSaveRequest) -> Result<i64> {
    let form_data = MenuSaveDTO::from(payload);
    let result = MenuModel::insert(&db, &form_data).await?;
    Ok(result)
}


pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<Option<String>>) -> Result<i64> {
    //有下级的时候 不能直接删除
    let vec_ids = convert_vec_option_string_to_vec_u64(ids);
    let menus = MenuModel::select_in_column(&db, &vec_ids).await?;

    if menus > 0 {
        return Err(Error::from("有下级的菜单不能删除"));
    };

    Ok(MenuModel::batch_delete_by_ids(db, vec_ids).await?)
}

/// 按id更新菜单
pub async fn update_by_id(db: &DbConn, menu: &MenuUpdateRequest) -> Result<i64> {
    let form_data = MenuSaveDTO::from(menu.clone());
    let result = MenuModel::update_by_id(&db, &form_data.id, &form_data).await?;
    Ok(result)
}




/// 查询用户路由树
pub async fn get_user_router_tree(db: &DbConn, is_admin: &bool, user_id: &Option<i64>) -> Result<Vec<menu::Router>> {
    let mut list: Vec<Model> = Vec::<Model>::new();
    let mut seen_ids = HashSet::new(); // 用于跟踪已经添加过的菜单 ID

    if is_admin.clone() {
        list = MenuModel::find_all(db).await?;
    } else {
        let result_merge = AdminRoleMergeModel::find_by_admin_id(db, user_id).await?;
        for merge in result_merge {
            let result_role = RoleModel::find_by_id(db, merge.role_id.unwrap_or_default()).await?;

            if let Some(role) = result_role {
                let role_menu_merge = RoleMenuMergeModel::find_by_role_id(db, &Some(role.id)).await?;
                for role_menu in role_menu_merge {
                    let result_menu = MenuModel::find_by_id(db, &role_menu.menu_id).await?;
                    if let Some(menu) = result_menu {
                        if seen_ids.insert(menu.id) { // 只有当菜单 ID 还未被添加过时，才添加到 list
                            list.push(menu);
                        }
                    }
                }
            }
        }
    }
    let mut router_list = Vec::<menu::Router>::new();
    // 按 sort 升序排序
    list.sort_by(|a, b| a.sort.unwrap_or(0).cmp(&b.sort.unwrap_or(0)));
    router_arr_to_tree(&mut router_list, list, Some(0));
    Ok(router_list)
}

// 路由数组转树
pub fn router_arr_to_tree(re_list: &mut Vec<menu::Router>, ori_arr: Vec<Model>, pid: Option<i64>) {
    for (_, it) in ori_arr.iter().enumerate() {
        if pid == Some(it.parent_id) && it.r#type != Some(MENU_TYPE_BUTTON.to_string()) {
            let temp_meta =  menu::Meta {
                title: it.name.clone(),
                icon: it.icon.clone(),
                sort: it.sort.clone(),
            };

            let mut children = Vec::<menu::Router>::new();
            router_arr_to_tree(&mut children, ori_arr.clone(), Some(it.id));

            let temp_router = Router {
                id: it.id.clone(),
                component: it.component.clone(),
                name: it.route_name.clone(),
                path: it.path.clone(),
                children: (|| -> Option<Vec<menu::Router>> {
                    if children.len() > 0 {
                        Some(children)
                    } else {
                        None
                    }
                })(),
                meta: temp_meta,
            };
            re_list.push(temp_router)
        }
    }
}



/// 查询按用户关联的按钮权限
pub async fn find_user_role_keys(db: &DbConn, is_admin: &bool, id: &Option<i64>) -> Result<Vec<String>> {
    let mut btn_menu: Vec<String> = Vec::new();
    let mut unique_set: HashSet<String> = HashSet::new();
    if is_admin.clone() {
        let menu_list = MenuModel::find_all(db).await?;
        for menu in menu_list {
            let perm = menu.clone().perm.unwrap_or_default(); // 存储 perm 的值
            if !perm.is_empty() && unique_set.insert(perm.clone()) {
                btn_menu.push(perm);
            }
        }
    } else {
        let result_merge = AdminRoleMergeModel::find_by_admin_id(db, &id.clone()).await?;
        for merge in result_merge {
            let role_menu_merge = RoleMenuMergeModel::find_by_role_id(db,&merge.role_id).await?;
            for role_menu in role_menu_merge {
                let result_menu = MenuModel::find_by_id(db, &role_menu.menu_id).await?;
                if unique_set.insert(result_menu.clone().unwrap_or_default().perm.unwrap_or_default().to_string()) {
                    btn_menu.push(result_menu.unwrap_or_default().perm.unwrap_or_default().to_string());
                }
            }
        }
    }
    Ok(btn_menu)
}

pub async fn all_menu_list(db: &DbConn, query : ListQuery) -> Result<Vec<MenuAdminVO>> {
    let select_where = PageWhere {
        name: query.keywords,
        status: query.status,
    };
    let search_where = select_where.format();
    let list: Vec<Model> = MenuModel::select_all_list(db,search_where).await?;
    
    let mut vo_list: Vec<MenuAdminVO> = Vec::new();
    menu_list_tree(&mut vo_list, list, Some(0));
    
    Ok(vo_list)
}

// 菜单数组转树
pub fn menu_list_tree(re_list: &mut Vec<MenuAdminVO>, ori_arr: Vec<Model>, pid: Option<i64>) {
    for (_, it) in ori_arr.iter().enumerate() {
        if pid == Some(it.parent_id) {
            let mut children = Vec::<MenuAdminVO>::new();
            menu_list_tree(&mut children, ori_arr.clone(), Some(it.id));

            let meta = ListMeta {
                name: it.name.clone(),
                affix_tab: Some(it.affix_tab),
                hide_children_in_menu: Some(it.hide_children_in_menu),
                hide_in_breadcrumb: Some(it.hide_in_breadcrumb),
                hide_in_menu: Some(it.hide_in_menu),
                hide_in_tab: Some(it.hide_in_tab),
                keep_alive: Some(it.keep_alive),
                sort: it.sort,
                icon: it.icon.clone(),
            };

            let temp_router = MenuAdminVO {
                id: it.id.clone(),
                parent_id: Some(it.parent_id),
                tree_path: it.tree_path.clone(),
                name: it.name.clone(),
                meta: Some(meta),
                r#type: it.r#type.clone(),
                route_name: it.route_name.clone(),
                path: it.path.clone(),
                component: it.component.clone(),
                perm: it.perm.clone(),
                status: Some(it.status),
                redirect: it.redirect.clone(),
                create_time: it.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
                update_time: it.updated_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
                params: it.params.clone(),
                children,
            };
            re_list.push(temp_router)
        }
    }
}

/// 菜单下拉树形结构
pub fn menu_all_tree(re_list: &mut Vec<MenuOptionsVO>, ori_arr: &[Model]) {
    // 构建从 id 到节点的映射
    let mut id_to_node: HashMap<i64, &Model> = HashMap::new();
    // 构建从 parent_id 到子节点列表的映射
    let mut parent_to_children: HashMap<i64, Vec<i64>> = HashMap::new();
    // 用于存储所有已知的 id
    let mut all_ids: HashSet<i64> = HashSet::new();

    for it in ori_arr.iter() {
        id_to_node.insert(it.id, it);
        all_ids.insert(it.id);
        if it.parent_id != 0 {
            parent_to_children.entry(it.parent_id).or_insert_with(Vec::new).push(it.id);
        }
    }

    // 找到所有根节点（即 parent_id 不存在于 all_ids 中的节点）
    let root_nodes: Vec<i64> = ori_arr.iter()
        .filter_map(|it| {
            if it.parent_id == 0 || !all_ids.contains(&it.parent_id) {
                Some(it.id)
            } else {
                None
            }
        })
        .collect();

    // 递归构建树
    for root_id in root_nodes {
        if let Some(root_node) = id_to_node.get(&root_id) {
            let mut children = Vec::<MenuOptionsVO>::new();
            build_tree(&mut children, &id_to_node, &parent_to_children, root_id);

            let temp_router = MenuOptionsVO {
                value: root_node.id,
                label: root_node.name.clone(),
                parent_id: Some(root_node.parent_id),
                children: if children.is_empty() { None } else { Some(children) },
            };
            re_list.push(temp_router);
        }
    }
}

#[allow(dead_code)]
fn build_tree(re_list: &mut Vec<MenuOptionsVO>, id_to_node: &HashMap<i64, &Model>, parent_to_children: &HashMap<i64, Vec<i64>>, pid: i64) {
    if let Some(children_ids) = parent_to_children.get(&pid) {
        for child_id in children_ids {
            if let Some(child_node) = id_to_node.get(child_id) {
                let mut children = Vec::<MenuOptionsVO>::new();
                build_tree(&mut children, id_to_node, parent_to_children, *child_id);

                let temp_router = MenuOptionsVO {
                    value: child_node.id,
                    parent_id: Some(child_node.parent_id),
                    children: if children.is_empty() { None } else { Some(children) },
                    label: child_node.name.clone(),
                };
                re_list.push(temp_router);
            }
        }
    }
}

pub async fn get_menu_options(db: &DbConn, admin_id: &Option<i64>) -> Result<Vec<MenuOptionsVO>> {
    let user_info = AdminModel::find_by_id(&db,admin_id).await?.ok_or_else(|| { Error::from(format!("msg={},code={}", "未获取到用户信息".to_string(), 404))})?;

    //判断是否是管理员
    let is_admin = user_info.user_type == Option::from(1);

    let list = if is_admin {
        MenuModel::find_all(db).await?
    } else {
        MenuModel::select_list_by_admin_id(db, admin_id).await?
    };
    
    let mut router_list = Vec::<MenuOptionsVO>::new();
    menu_all_tree(&mut router_list, &list);
    Ok(router_list)
}


pub async fn get_menu_vec_ids(db: &DbConn, admin_id: &Option<i64>) -> Result<Vec<i64>> {
    let user_info = AdminModel::find_by_id(&db,admin_id).await?.ok_or_else(|| { Error::from(format!("msg={},code={}", "未获取到用户信息".to_string(), 404))})?;
    //判断是否是管理员
    let is_admin = user_info.user_type == Option::from(1);
    let list = if is_admin {
        MenuModel::find_all(db).await?
    } else {
        MenuModel::select_list_by_admin_id(db, admin_id).await?
    };

    let ids: Vec<i64> = list.into_iter().map(|model| model.id).collect();
    Ok(ids)
}

/// 判断主向量是否包含子向量
pub fn contains_all_elements(main_vec: &Vec<i64>, sub_vec: &Vec<i64>) -> bool {
    let main_set: HashSet<i64> = main_vec.iter().cloned().collect();
    let sub_set: HashSet<i64> = sub_vec.iter().cloned().collect();
    main_set.is_superset(&sub_set)
}

/// 查询同一个父级下菜单名称是否唯一
pub async fn find_by_name_unique(db: &DbConn, menu_name: &Option<String>, parent_id: &Option<i64>, id: &Option<i64>) -> Result<bool>{
    let menus_num= MenuModel::find_by_name_unique(db, menu_name, parent_id, id).await?;
    if menus_num > 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// 查询菜单路由是否唯一
pub async fn find_by_path_unique(db: &DbConn, path: &Option<String>, parent_id: &Option<i64>, id: &Option<i64>) -> Result<bool>{
    let menus_num= MenuModel::find_by_path_unique(&db, path,parent_id, id).await?;
    if menus_num > 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// 查询菜单权限标识是否唯一
pub async fn find_by_perms_unique(db: &DbConn, perms: &Option<String>, id: &Option<i64>) -> Result<bool>{
    let menus_num= MenuModel::find_by_perms_unique(&db, perms, id).await?;
    if menus_num > 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// 查询路由名称是否唯一
pub async fn find_by_route_name_unique(db: &DbConn, route_name: &Option<String>, id: &Option<i64>) -> Result<bool>{
    let menus_num= MenuModel::find_by_route_name_unique(&db, route_name, id).await?;
    if menus_num > 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}


pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<MenuDetailVO> {
    let result_data = MenuModel::find_by_id(db, id).await?.ok_or_else(|| {
        Error::from(format!(
            "{}={}",
            "用户信息不存在，id".to_string(),
            &id.unwrap_or_default()
        ))
    })?;

    let result = MenuDetailVO::from(result_data);
    Ok(result)
}