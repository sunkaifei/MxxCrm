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
use crate::modules::website::entity::template_category;
use crate::modules::website::entity::template_category::Model;
use crate::modules::website::model::template_category::{CategoryDetailVO, CategoryListVO, CategorySaveDTO, ListQuery, PageWhere, SelectListVO, TemplateCategoryModel};
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;

pub async fn insert(db: &DbConn, form_data: CategorySaveDTO) -> Result<i64> {
    let result = TemplateCategoryModel::insert(&db, &form_data).await?;
    Ok(result)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<Option<String>>) -> Result<i64>{
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let ids = convert_vec_option_string_to_vec_u64(ids_vec.clone());
    let config = TemplateCategoryModel::batch_delete_by_ids(db, ids).await?;
    Ok(config)
}

pub async fn update_by_id(db: &DbConn, form_data: &CategorySaveDTO) -> Result<i64> {
    let result = TemplateCategoryModel::update_by_id(&db, &form_data.id, &form_data).await?;
    Ok(result)
}

pub async fn find_by_name_unique(db: &DbConn, title: &Option<String>, id: &Option<i64>) -> Result<bool> {
    let result = TemplateCategoryModel::find_by_name_unique(&db, title, id).await?;
    if result > 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}


pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<template_category::Model>> {
    let result = TemplateCategoryModel::find_by_id(&db, id).await?.ok_or_else(|| {
        Error::from(format!(
            "{}={}",
            "友情链接信息不存在，id".to_string(),
            &id.unwrap_or_default()
        ))
    })?;
    Ok(Option::from(result))
}

pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<CategoryDetailVO> {
    let result = TemplateCategoryModel::find_by_id(&db, id).await?.ok_or_else(|| {
        Error::from(format!(
            "{}={}",
            "友情链接信息不存在，id".to_string(),
            &id.unwrap_or_default()
        ))
    })?;
    Ok(CategoryDetailVO::from(result))
}

/// 菜单下拉树形结构
pub fn category_all_tree(re_list: &mut Vec<CategoryListVO>, ori_arr: &[Model]) {
    // 构建从 id 到节点的映射
    let mut id_to_node: HashMap<i64, &Model> = HashMap::new();
    // 构建从 parent_id 到子节点列表的映射
    let mut parent_to_children: HashMap<i64, Vec<i64>> = HashMap::new();
    // 用于存储所有已知的 id
    let mut all_ids: HashSet<i64> = HashSet::new();

    for it in ori_arr.iter() {
        id_to_node.insert(it.id, it);
        all_ids.insert(it.id);
        if let Some(parent_id) = it.parent_id {
            parent_to_children.entry(parent_id).or_insert_with(Vec::new).push(it.id);
        }
    }

    // 找到所有根节点（即 parent_id 不存在于 all_ids 中的节点）
    let root_nodes: Vec<i64> = ori_arr.iter()
        .filter_map(|it| {
            if it.parent_id.is_none() || !all_ids.contains(&it.parent_id.unwrap()) {
                Some(it.id)
            } else {
                None
            }
        })
        .collect();

    // 递归构建树
    for root_id in root_nodes {
        if let Some(root_node) = id_to_node.get(&root_id) {
            let mut children = Vec::<CategoryListVO>::new();
            build_tree(&mut children, &id_to_node, &parent_to_children, root_id);

            let temp_router = CategoryListVO {
                id: Option::from(root_node.id),
                parent_id: root_node.parent_id,
                name: root_node.name.clone(),
                sort: root_node.sort,
                is_show: root_node.is_show,
                status: root_node.status,
                create_time: root_node.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
                update_time: root_node.update_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
                children: if children.is_empty() { None } else { Option::from(children) },

            };
            re_list.push(temp_router);
        }
    }
}

#[allow(dead_code)]
fn build_tree(re_list: &mut Vec<CategoryListVO>, id_to_node: &HashMap<i64, &Model>, parent_to_children: &HashMap<i64, Vec<i64>>, pid: i64) {
    if let Some(children_ids) = parent_to_children.get(&pid) {
        for child_id in children_ids {
            if let Some(child_node) = id_to_node.get(child_id) {
                let mut children = Vec::<CategoryListVO>::new();
                build_tree(&mut children, id_to_node, parent_to_children, *child_id);

                let temp_router = CategoryListVO {
                    id: Option::from(child_node.id),
                    parent_id: child_node.parent_id.clone(),
                    name: child_node.name.clone(),
                    sort: child_node.sort.clone(),
                    is_show: child_node.is_show.clone(),
                    status: child_node.status.clone(),
                    create_time: child_node.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
                    update_time: child_node.update_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
                    children: if children.is_empty() { None } else { Some(children) },
                };
                re_list.push(temp_router);
            }
        }
    }
}

pub async fn category_all_tree_list(db: &DbConn, query: &ListQuery) -> Result<Vec<CategoryListVO>> {
    let select_where = PageWhere{
        name: query.name.clone(),
        is_show: query.is_show,
        status: query.status,
    };
    let select_where = select_where.format();
    let list = TemplateCategoryModel::select_all(&db, &select_where).await?;

    let mut router_list = Vec::<CategoryListVO>::new();
    category_all_tree(&mut router_list, &list);
    Ok(router_list)
}

pub async fn category_all_options(db: &DbConn) -> Result<Vec<CategoryListVO>> {
    let list = TemplateCategoryModel::find_all(&db).await?;

    let mut router_list = Vec::<CategoryListVO>::new();
    category_all_tree(&mut router_list, &list);
    Ok(router_list)
}


pub async fn select_by_parent_id(db: &DbConn, parent_id: &Option<i64>) -> Result<Vec<SelectListVO>> {
    let list = TemplateCategoryModel::select_by_parent_id(&db, parent_id).await?;

    let list_data: Vec<SelectListVO> = list.into_iter().map(|item| SelectListVO::from(item)).collect();
    Ok(list_data)
}
