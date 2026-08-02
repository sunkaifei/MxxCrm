//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!


use sea_orm::DbConn;
use crate::core::errors::error::{Error, Result};
use crate::modules::website::model::website_media_category::{MediaCategoryDetailVO, MediaCategoryListVO, MediaCategorySaveDTO, WebsiteMediaCategoryModel};
use std::collections::{HashMap, HashSet};
use crate::modules::website::entity::website_media_category::Model;

/// 新增媒体分类
pub async fn insert(db: &DbConn, form_data: MediaCategorySaveDTO) -> Result<i64> {
    let result = WebsiteMediaCategoryModel::insert(db, &form_data).await?;
    Ok(result)
}

/// 软删除媒体分类
pub async fn delete_by_id(db: &DbConn, id: i64) -> Result<i64> {
    let result = WebsiteMediaCategoryModel::delete_by_id(db, id).await?;
    Ok(result)
}

/// 更新媒体分类
pub async fn update_by_id(db: &DbConn, form_data: &MediaCategorySaveDTO) -> Result<i64> {
    let result = WebsiteMediaCategoryModel::update_by_id(db, &form_data.id, form_data).await?;
    Ok(result)
}

/// 根据id查询详情
pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<MediaCategoryDetailVO> {
    let result = WebsiteMediaCategoryModel::find_by_id(db, id).await?.ok_or_else(|| {
        Error::from(format!("{}={}", "媒体分类不存在，id".to_string(), &id.unwrap_or_default()))
    })?;
    Ok(MediaCategoryDetailVO::from(result))
}

/// 查询所有分类（树形结构）
pub async fn select_all(db: &DbConn) -> Result<Vec<MediaCategoryListVO>> {
    let list = WebsiteMediaCategoryModel::select_all(db).await?;
    let mut router_list = Vec::<MediaCategoryListVO>::new();
    category_tree(&mut router_list, &list);
    Ok(router_list)
}

/// 查询所有分类（下拉选项树形结构）
pub async fn select_all_options(db: &DbConn) -> Result<Vec<crate::modules::website::model::website_media_category::MediaCategorySelectVO>> {
    let list = WebsiteMediaCategoryModel::select_all(db).await?;
    let mut router_list = Vec::<crate::modules::website::model::website_media_category::MediaCategorySelectVO>::new();
    category_select_tree(&mut router_list, &list);
    Ok(router_list)
}

/// 构建列表树形结构
fn category_tree(re_list: &mut Vec<MediaCategoryListVO>, ori_arr: &[Model]) {
    let mut id_to_node: HashMap<i64, &Model> = HashMap::new();
    let mut parent_to_children: HashMap<i64, Vec<i64>> = HashMap::new();
    let mut all_ids: HashSet<i64> = HashSet::new();

    for it in ori_arr.iter() {
        id_to_node.insert(it.id, it);
        all_ids.insert(it.id);
        if let Some(parent_id) = it.parent_id {
            parent_to_children.entry(parent_id).or_insert_with(Vec::new).push(it.id);
        }
    }

    let root_nodes: Vec<i64> = ori_arr.iter()
        .filter_map(|it| {
            if it.parent_id.is_none() || it.parent_id == Some(0) || !all_ids.contains(&it.parent_id.unwrap()) {
                Some(it.id)
            } else {
                None
            }
        })
        .collect();

    for root_id in root_nodes {
        if let Some(root_node) = id_to_node.get(&root_id) {
            let mut children = Vec::<MediaCategoryListVO>::new();
            build_tree(&mut children, &id_to_node, &parent_to_children, root_id);

            let temp_node = MediaCategoryListVO {
                id: Option::from(root_node.id),
                category_name: root_node.category_name.clone(),
                parent_id: root_node.parent_id,
                sort: root_node.sort,
                create_time: root_node.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
                children: if children.is_empty() { None } else { Option::from(children) },
            };
            re_list.push(temp_node);
        }
    }
}

#[allow(dead_code)]
fn build_tree(re_list: &mut Vec<MediaCategoryListVO>, id_to_node: &HashMap<i64, &Model>, parent_to_children: &HashMap<i64, Vec<i64>>, pid: i64) {
    if let Some(children_ids) = parent_to_children.get(&pid) {
        for child_id in children_ids {
            if let Some(child_node) = id_to_node.get(child_id) {
                let mut children = Vec::<MediaCategoryListVO>::new();
                build_tree(&mut children, id_to_node, parent_to_children, *child_id);

                let temp_node = MediaCategoryListVO {
                    id: Option::from(child_node.id),
                    category_name: child_node.category_name.clone(),
                    parent_id: child_node.parent_id,
                    sort: child_node.sort,
                    create_time: child_node.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
                    children: if children.is_empty() { None } else { Some(children) },
                };
                re_list.push(temp_node);
            }
        }
    }
}

/// 构建下拉选项树形结构
fn category_select_tree(re_list: &mut Vec<crate::modules::website::model::website_media_category::MediaCategorySelectVO>, ori_arr: &[Model]) {
    let mut id_to_node: HashMap<i64, &Model> = HashMap::new();
    let mut parent_to_children: HashMap<i64, Vec<i64>> = HashMap::new();
    let mut all_ids: HashSet<i64> = HashSet::new();

    for it in ori_arr.iter() {
        id_to_node.insert(it.id, it);
        all_ids.insert(it.id);
        if let Some(parent_id) = it.parent_id {
            parent_to_children.entry(parent_id).or_insert_with(Vec::new).push(it.id);
        }
    }

    let root_nodes: Vec<i64> = ori_arr.iter()
        .filter_map(|it| {
            if it.parent_id.is_none() || it.parent_id == Some(0) || !all_ids.contains(&it.parent_id.unwrap()) {
                Some(it.id)
            } else {
                None
            }
        })
        .collect();

    for root_id in root_nodes {
        if let Some(root_node) = id_to_node.get(&root_id) {
            let mut children = Vec::<crate::modules::website::model::website_media_category::MediaCategorySelectVO>::new();
            build_select_tree(&mut children, &id_to_node, &parent_to_children, root_id);

            let temp_node = crate::modules::website::model::website_media_category::MediaCategorySelectVO {
                id: Option::from(root_node.id),
                category_name: root_node.category_name.clone(),
                parent_id: root_node.parent_id,
                children: if children.is_empty() { None } else { Option::from(children) },
            };
            re_list.push(temp_node);
        }
    }
}

#[allow(dead_code)]
fn build_select_tree(re_list: &mut Vec<crate::modules::website::model::website_media_category::MediaCategorySelectVO>, id_to_node: &HashMap<i64, &Model>, parent_to_children: &HashMap<i64, Vec<i64>>, pid: i64) {
    if let Some(children_ids) = parent_to_children.get(&pid) {
        for child_id in children_ids {
            if let Some(child_node) = id_to_node.get(child_id) {
                let mut children = Vec::<crate::modules::website::model::website_media_category::MediaCategorySelectVO>::new();
                build_select_tree(&mut children, id_to_node, parent_to_children, *child_id);

                let temp_node = crate::modules::website::model::website_media_category::MediaCategorySelectVO {
                    id: Option::from(child_node.id),
                    category_name: child_node.category_name.clone(),
                    parent_id: child_node.parent_id,
                    children: if children.is_empty() { None } else { Some(children) },
                };
                re_list.push(temp_node);
            }
        }
    }
}
