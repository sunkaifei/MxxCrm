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
use crate::core::web::response::ResultPage;
use crate::core::errors::error::{Error, Result};
use crate::modules::system::entity::region;
use crate::modules::system::model::region::{ListQuery, PageWhere, RegionDetailVO, RegionListVO, RegionModel, RegionSaveDTO, RegionSaveRequest, RegionTreeVO, RegionUpdateRequest, RegionUserTreeVO};
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;

/// 添加地区
pub async fn insert(db: &DbConn, region: &RegionSaveRequest) -> Result<i64> {
    let form_data = RegionSaveDTO::from(region.clone());
    let result = RegionModel::insert(&db, &form_data).await?;
    Ok(result)
}

/// 批量删除区域
pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<Option<String>>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let ids = convert_vec_option_string_to_vec_u64(ids_vec.clone());
    let result = RegionModel::batch_delete_by_ids(&db, ids).await?;
    Ok(result)
}

/// 更新区域
pub async fn update_by_id(db: &DbConn, region: &RegionUpdateRequest) -> Result<i64> {
    let form_data = RegionSaveDTO::from(region.clone());
    let result = RegionModel::update_by_id(&db, form_data.id.unwrap_or_default(), form_data).await?;
    Ok(result)
}

/// 获取区域树
pub async fn get_region_tree(db: &DbConn, ) -> Result<Vec<RegionTreeVO>> {
    let list = RegionModel::find_all(&db).await?;
    let mut router_list = Vec::<RegionTreeVO>::new();
    router_arr_to_tree(&mut router_list, list, Some(0));
    Ok(router_list)
}

// 行政区域数组转树
pub fn router_arr_to_tree(re_list: &mut Vec<RegionTreeVO>, ori_arr: Vec<region::Model>, pid: Option<i64>) {
    for (_, it) in ori_arr.iter().enumerate() {
        if pid == it.parent_id  {
            let mut children = Vec::<RegionTreeVO>::new();
            router_arr_to_tree(&mut children, ori_arr.clone(), Option::from(it.id));
            let temp_router = RegionTreeVO {
                id: Option::from(it.id.clone()),
                parent_id: it.parent_id.clone(),
                title: it.title.clone(),
                region_name: it.region_name.clone(),
                sort: it.sort.clone(),
                status: it.status.clone(),
                children: (|| -> Option<Vec<RegionTreeVO>> {
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


/// 查询区域详情
pub async fn get_by_id(db: &DbConn, id: &Option<i64>) -> Result<RegionDetailVO>{
    let result = RegionModel::find_by_id(db, id).await?.ok_or_else(|| {
        Error::from(format!(
            "{}={:}",
            "地区信息不存在，id".to_string(),
            &id.unwrap_or_default()
        ))
    })?;
    
    let result_data = RegionDetailVO::from(result);
    Ok(result_data)
}


/// 查询地区详情
pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<RegionDetailVO>{
    let result = RegionModel::find_by_id(db, id).await?
        .ok_or_else(|| {
            Error::from(format!(
                "{}={:?}",
                "地区信息不存在，id".to_string(),
                &id.unwrap_or_default().to_string()
            ))
        })?;
    let result_data = RegionDetailVO::from(result);
    Ok(result_data)
}

/// 根据父级id查询地区列表
pub async fn get_by_parent_id(db: &DbConn, parent_ids: &Vec<Option<String>>) -> Result<Vec<RegionListVO>> {
    let ids = convert_vec_option_string_to_vec_u64(parent_ids.clone());
    let list = RegionModel::find_by_parent_ids(db, &ids).await?;
    let list_data: Vec<RegionListVO> = list.into_iter().map(|item| RegionListVO::from(item)).collect();
    Ok(list_data)
}

/// 查询地区列表
pub async fn get_by_page(db: &DbConn, query : ListQuery) -> Result<ResultPage<Vec<RegionListVO>>> {

    let select_where = PageWhere {
        region_name: None,
        status: query.status,
    };
    let search_where = select_where.format();

    let (list, _num_pages) = RegionModel::select_in_page(
        &db,
        query.page_num.unwrap_or(0),
        query.page_size.unwrap_or(10),
        search_where.clone()
    ).await?;

    let list_data: Vec<RegionListVO> = list.into_iter().map(|item| RegionListVO::from(item)).collect();

    let count = RegionModel::select_count(db, select_where.clone()).await.unwrap_or(0);

    let page_data = ResultPage::new_simple(list_data, count);

    Ok(page_data)
}

/// 获取用户端区域树
pub async fn get_region_tree_for_user(db: &DbConn) -> Result<Vec<RegionUserTreeVO>> {
    let list = RegionModel::find_all(&db).await?;
    let mut tree = Vec::<RegionUserTreeVO>::new();
    user_router_arr_to_tree(&mut tree, list, Some(0));
    Ok(tree)
}

pub fn user_router_arr_to_tree(re_list: &mut Vec<RegionUserTreeVO>, ori_arr: Vec<region::Model>, pid: Option<i64>) {
    for (_, it) in ori_arr.iter().enumerate() {
        if pid == it.parent_id {
            let mut children = Vec::<RegionUserTreeVO>::new();
            user_router_arr_to_tree(&mut children, ori_arr.clone(), Option::from(it.id));
            let temp_router = RegionUserTreeVO {
                id: Some(it.id.to_string()),
                label: it.region_name.clone(),
                children: (|| -> Option<Vec<RegionUserTreeVO>> {
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






