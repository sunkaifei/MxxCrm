//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::{Error, Result};
use crate::modules::shop::entity::shop_category;
use crate::modules::shop::model::category::*;
use sea_orm::DbConn;

pub async fn create_category(db: &DbConn, req: CategoryRequest) -> Result<i64> {
    let dto: CategoryDTO = req.into();
    CategoryModel::insert(db, &dto).await
        .map_err(|e| Error::from(format!("创建分类失败: {:?}", e)))
}

pub async fn update_category(db: &DbConn, id: i64, req: CategoryRequest) -> Result<i64> {
    let dto: CategoryDTO = req.into();
    CategoryModel::update_by_id(db, &dto, id).await
        .map_err(|e| Error::from(format!("更新分类失败: {:?}", e)))
}

pub async fn delete_category(db: &DbConn, id: i64) -> Result<i64> {
    let children = CategoryModel::find_by_parent_id(db, id).await
        .map_err(|e| Error::from(format!("查询子分类失败: {:?}", e)))?;
    if !children.is_empty() {
        return Err(Error::from("该分类下有子分类，无法删除"));
    }
    CategoryModel::delete_by_id(db, id).await
        .map_err(|e| Error::from(format!("删除分类失败: {:?}", e)))
}

pub async fn get_category_by_id(db: &DbConn, id: i64) -> Result<Option<CategoryVO>> {
    let model = CategoryModel::find_by_id(db, id).await
        .map_err(|e| Error::from(format!("查询分类失败: {:?}", e)))?;
    Ok(model.map(|m| m.into()))
}

pub async fn get_category_list(db: &DbConn) -> Result<Vec<CategoryTreeVO>> {
    let all = CategoryModel::find_all(db).await
        .map_err(|e| Error::from(format!("查询分类列表失败: {:?}", e)))?;

    build_category_tree(all, 0)
}

fn build_category_tree(all: Vec<shop_category::Model>, parent_id: i64) -> Result<Vec<CategoryTreeVO>> {
    let children: Vec<shop_category::Model> = all.iter()
        .filter(|c| c.parent_id == parent_id)
        .cloned()
        .collect();

    let mut tree = Vec::new();
    for child in children {
        let child_id = child.id;
        let vo = CategoryTreeVO {
            id: Some(child.id),
            name: Some(child.name),
            icon: child.icon,
            sort_order: Some(child.sort_order),
            is_show: Some(child.is_show),
            level: Some(child.level),
            children: Some(build_category_tree(all.clone(), child_id)?),
        };
        tree.push(vo);
    }
    Ok(tree)
}

pub async fn get_category_page(db: &DbConn, page_num: i64, page_size: i64) -> Result<(Vec<CategoryVO>, i64)> {
    let (list, total) = CategoryModel::find_page(db, page_num, page_size).await
        .map_err(|e| Error::from(format!("查询分类分页失败: {:?}", e)))?;

    let vo_list: Vec<CategoryVO> = list.into_iter().map(|m| m.into()).collect();
    Ok((vo_list, total))
}
