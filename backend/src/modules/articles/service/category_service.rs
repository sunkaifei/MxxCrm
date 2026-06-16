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
use crate::modules::articles::entity::category;
use crate::modules::articles::model::category::{CategoryDetailVO, CategoryModel, CategoryOptionsVO, CategoryPageDTO, CategorySaveDTO, CategoryTreeVO};
use crate::utils::short_code;
use crate::SNOWFLAKE;
use sea_orm::DbConn;

///添加菜单
pub async fn save_category(db: &DbConn, payload: CategorySaveDTO) -> Result<i64> {
    let mut category_data = payload;
    category_data.id = Option::from(SNOWFLAKE.generate() as i64);

    let unique_num = CategoryModel::find_by_name_unique(&db, &category_data.website_id, &category_data.category_name, &None).await?;
    if unique_num > 0 {
        return Err(Error::from("栏目名称已存在".to_string()));
    }

    //获取短网址唯一性
    category_data.short_url = find_short_url_unique(&db).await;

    let result = CategoryModel::insert(&db, &category_data).await?;
    Ok(result)
}

pub async fn update_by_id(db: &DbConn, form_data: CategorySaveDTO) -> Result<i64> {
    let update_result = CategoryModel::update_by_id(&db, &form_data.id, &form_data).await?;
    Ok(update_result)
}

/// * 获取栏目名称唯一性
/// 
pub async fn find_by_name_unique(
    db: &DbConn, 
    website_id: &Option<i64>, 
    category_name: &Option<String>,
    id: &Option<i64>
) -> Result<bool> {
    let result_num = CategoryModel::find_by_name_unique(&db, website_id, category_name, id).await?;
    if result_num > 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}


pub async fn find_short_url_unique(db: &DbConn) -> Option<String> {
    let mut new_short_url: Option<String> = Some(String::new());
    for _ in 0..5 {
        let short_url = short_code::generate_code();
        let unique_num = CategoryModel::find_by_short_url_unique(db, &short_url).await;
        if unique_num.unwrap_or(0) == 0 {
            new_short_url = Some(short_url);
            break;
        }
    }
    new_short_url
}

pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<CategoryDetailVO> {
    let result_data = CategoryModel::find_by_id(&db, id).await?.ok_or_else(|| {
        Error::from(format!(
            "{}",
            "该未获取到分类信息".to_string(),
        ))
    })?;
    let result = CategoryDetailVO::from(result_data);
    Ok(result)
}

///获取所有菜单列表树
pub async fn all_category_tree(db: &DbConn, website_id: i64) -> Result<Vec<CategoryOptionsVO>> {
    let list: Vec<category::Model> = CategoryModel::find_all(&db, website_id).await?;
    let mut router_list = Vec::<CategoryOptionsVO>::new();
    category_arr_to_tree(&mut router_list, list, 0);
    Ok(router_list)
}

// 菜单数组转树
pub fn category_arr_to_tree(re_list: &mut Vec<CategoryOptionsVO>, ori_arr: Vec<category::Model>, pid: i64) {
    for (_, it) in ori_arr.iter().enumerate() {
        if pid == it.parent_id.unwrap_or_default() {
            let mut children = Vec::<CategoryOptionsVO>::new();
            category_arr_to_tree(&mut children, ori_arr.clone(), it.id);
            let temp_router = CategoryOptionsVO {
                children: (|| -> Option<Vec<CategoryOptionsVO>> {
                    if children.len() > 0 {
                        Some(children)
                    } else {
                        None
                    }
                })(),
                value: it.id.clone(),
                label: it.category_name.clone(),
            };
            re_list.push(temp_router)
        }
    }
}

pub fn all_list_arr_to_tree(re_list: &mut Vec<CategoryTreeVO>, ori_arr: Vec<category::Model>, pid: i64) {
    for (_, it) in ori_arr.iter().enumerate() {
        if pid == it.parent_id.unwrap_or_default() {
            let mut children = Vec::<CategoryTreeVO>::new();
            all_list_arr_to_tree(&mut children, ori_arr.clone(), it.id);
            let temp_router = CategoryTreeVO {
                id: it.id.clone(),
                parent_id: it.parent_id.clone(),
                short_url: it.short_url.clone(),
                category_name: it.category_name.clone(),
                sort: it.sort.clone(),
                count_topic: it.count_topic.clone(),
                create_time: it.create_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
                is_show: it.is_show,
                status: Option::from(it.status.clone().unwrap_or_default()),
                children: (|| -> Option<Vec<CategoryTreeVO>> {
                    if children.len() > 0 {
                        Some(children)
                    } else {
                        None
                    }
                })(),
                website_id: None,
            };
            re_list.push(temp_router)
        }
    }
}

// 查询部门所有数据列表
pub async fn select_all_list(db: &DbConn, item: CategoryPageDTO) -> Result<Vec<CategoryTreeVO>> {
    let list: Vec<category::Model> = CategoryModel::find_all(&db, item.website_id.unwrap_or_default()).await?;
    let mut category_list = Vec::<CategoryTreeVO>::new();
    all_list_arr_to_tree(&mut category_list, list, 0);
    Ok(category_list)
}
