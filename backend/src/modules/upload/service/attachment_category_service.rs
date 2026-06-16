//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_http::header::q;
use sea_orm::DbConn;
use crate::core::errors::error::{Error, Result};
use crate::modules::upload::entity::attachment_category;
use crate::modules::upload::entity::attachment_category::Model;
use crate::modules::upload::model::attachment_category::{AttachCategorySaveDTO, AttachCategorySaveRequest, AttachCategoryUpdateRequest, AttachCategoryVO, AttachmentCategoryModel, CategoryTreeVO, ListQuery, PageWhere};

/// # 保存分类
/// * `db` 数据库连接
/// * `form_data` 表单数据
pub async fn save(db: &DbConn, form_data: AttachCategorySaveRequest) -> Result<i64> {
    let dto_data = AttachCategorySaveDTO::from(form_data);
    
    let rows = AttachmentCategoryModel::insert(db,&dto_data).await?;
    if rows > 0 {
        Ok(rows)
    } else {
        Err(Error::from("保存失败"))
    }
}

/// # 更新分类
/// * `db` 数据库连接
/// * `form_data` 表单数据
pub async fn update(db: &DbConn, form_data: AttachCategoryUpdateRequest) -> Result<i64> {
    let dto_data = AttachCategorySaveDTO::from(form_data);
    
    let rows = AttachmentCategoryModel::update(db, &dto_data.id, &dto_data).await?;
    if rows > 0 {
        Ok(rows)
    } else {
        Err(Error::from("更新失败"))
    }
}



/// # 批量删除
/// * `db` 数据库连接
/// * `ids` id集合
pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64> {
    let rows = AttachmentCategoryModel::batch_delete_by_ids(db,ids).await?;
    if rows > 0 {
        Ok(rows)
    } else {
        Err(Error::from("删除失败"))
    }
}


/// 分类下拉树形结构
pub fn category_tree(re_list: &mut Vec<CategoryTreeVO>, ori_arr: Vec<Model>, pid: i64) {
    for (_, it) in ori_arr.iter().enumerate() {
        if pid == it.parent_id.unwrap_or_default() {
            let mut children = Vec::<CategoryTreeVO>::new();
            category_tree(&mut children, ori_arr.clone(), it.id);
            let temp_router = CategoryTreeVO {
                value: it.id.clone(),
                label: it.name.clone(),
                children: (|| -> Option<Vec<CategoryTreeVO>> {
                    if children.len() > 0 {
                        Some(children)
                    } else {
                        None
                    }
                })(),
            };
            re_list.push(temp_router)
        }
    }
}

/// 获取分类树形结构
pub async fn get_category_tree(db: &DbConn, query : ListQuery) -> Result<Vec<CategoryTreeVO>> {
    let select_where = PageWhere {
        name: query.name,
    };
    let search_where = select_where.format();
    let list = AttachmentCategoryModel::find_all(&db,search_where).await?;
    let mut tree_list = Vec::<CategoryTreeVO>::new();
    category_tree(&mut tree_list, list, 0);
    Ok(tree_list)
}

/// # 根据id查询分类
/// * `db` 数据库连接
/// * `id` 分类id
pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<attachment_category::Model>> {
    let result_data = AttachmentCategoryModel::find_by_id(db,id).await?;
    Ok(result_data)
}









