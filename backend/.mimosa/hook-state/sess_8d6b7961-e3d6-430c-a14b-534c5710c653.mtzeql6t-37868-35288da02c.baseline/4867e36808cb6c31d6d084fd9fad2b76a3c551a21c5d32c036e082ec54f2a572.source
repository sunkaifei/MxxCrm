//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::{DbConn, DbErr, TransactionTrait};
use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::system::model::salary_band::{
    ListQuery, PageWhere, SalaryBandDetailVO, SalaryBandListVO, SalaryBandModel,
    SalaryBandSaveDTO, SalaryBandSaveRequest, SalaryBandUpdateRequest,
};
use crate::modules::system::model::post::PostModel;
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;

/// ### 添加岗位薪资带宽
pub async fn save(db: &DbConn, band: &SalaryBandSaveRequest) -> Result<i64> {
    let form_data = SalaryBandSaveDTO::from(band.clone());
    let result = SalaryBandModel::insert(&db, form_data).await?;
    Ok(result)
}

/// ### 批量删除岗位薪资带宽
pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<Option<String>>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let ids = convert_vec_option_string_to_vec_u64(ids_vec.clone());
    let result = SalaryBandModel::batch_delete_by_ids(&db, ids).await?;
    Ok(result)
}

/// ### 更新岗位薪资带宽
pub async fn update_by_id(db: &DbConn, band: &SalaryBandUpdateRequest) -> Result<i64> {
    let form_data = SalaryBandSaveDTO::from(band.clone());
    let result = SalaryBandModel::update_by_id(&db, form_data.id.unwrap_or_default(), form_data).await?;
    Ok(result)
}

/// ### 查询带宽详情（补岗位名称）
pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<SalaryBandDetailVO> {
    let data = SalaryBandModel::find_by_id(&db, id).await?;
    match data {
        None => Err(Error::from(format!(
            "{}={}",
            "薪资带宽不存在，id".to_string(),
            &id.unwrap_or_default()
        ))),
        Some(band) => {
            let post_name = PostModel::find_by_id(&db, &Some(band.post_id))
                .await
                .ok()
                .flatten()
                .and_then(|p| p.post_name);
            Ok(SalaryBandDetailVO {
                id: Some(band.id),
                post_id: Some(band.post_id),
                post_name,
                min_salary: Some(band.min_salary),
                max_salary: Some(band.max_salary),
                status: band.status,
                remark: band.remark,
                create_time: band
                    .create_time
                    .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
                update_time: band
                    .update_time
                    .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
            })
        }
    }
}

/// ### 按岗位ID查询启用的带宽（供入职定薪参照）
pub async fn get_band_by_post(db: &DbConn, post_id: i64) -> Result<Option<SalaryBandDetailVO>> {
    let data = SalaryBandModel::find_by_post_id(&db, post_id).await?;
    match data {
        None => Ok(None),
        Some(band) => {
            if band.status != Some(1) {
                return Ok(None);
            }
            let post_name = PostModel::find_by_id(&db, &Some(band.post_id))
                .await
                .ok()
                .flatten()
                .and_then(|p| p.post_name);
            Ok(Some(SalaryBandDetailVO {
                id: Some(band.id),
                post_id: Some(band.post_id),
                post_name,
                min_salary: Some(band.min_salary),
                max_salary: Some(band.max_salary),
                status: band.status,
                remark: band.remark,
                create_time: band
                    .create_time
                    .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
                update_time: band
                    .update_time
                    .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
            }))
        }
    }
}

/// ### 查询岗位薪资带宽列表（分页，补岗位名称）
pub async fn get_by_page(db: &DbConn, query: ListQuery) -> Result<ResultPage<Vec<SalaryBandListVO>>> {
    let select_where = PageWhere {
        post_id: query.post_id,
        status: query.status,
    };
    let search_where = select_where.format();

    let (list, _num_pages) = SalaryBandModel::select_in_page(
        &db,
        query.page_num.unwrap_or(1),
        query.page_size.unwrap_or(10),
        search_where.clone(),
    )
    .await?;

    // 批量补岗位名称
    let post_ids: Vec<i64> = list
        .iter()
        .map(|b| b.post_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let posts = PostModel::find_by_ids(&db, post_ids).await.unwrap_or_default();
    let post_map: std::collections::HashMap<i64, Option<String>> = posts
        .into_iter()
        .map(|p| (p.id, p.post_name))
        .collect();

    let list_data: Vec<SalaryBandListVO> = list
        .into_iter()
        .map(|item| {
            let mut vo = SalaryBandListVO::from(item);
            vo.post_name = post_map.get(&vo.post_id.unwrap_or_default()).cloned().flatten();
            vo
        })
        .collect();

    let count = SalaryBandModel::select_count(db, select_where.clone()).await.unwrap_or(0);
    let page_data = ResultPage::new_simple(list_data, count);
    Ok(page_data)
}
