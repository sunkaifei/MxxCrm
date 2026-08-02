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
use crate::core::web::response::ResultPage;
use crate::modules::website::model::website_media::{ListQuery, PageWhere, MediaDetailVO, MediaListVO, WebsiteMediaModel, MediaSaveDTO};
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;

/// 新增媒体
pub async fn insert(db: &DbConn, form_data: &MediaSaveDTO) -> Result<i64> {
    let result = WebsiteMediaModel::insert(db, form_data).await?;
    Ok(result)
}

/// 批量软删除
pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<Option<String>>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let ids = convert_vec_option_string_to_vec_u64(ids_vec.clone());
    let result = WebsiteMediaModel::batch_delete_by_ids(db, ids).await?;
    Ok(result)
}

/// 更新媒体元数据
pub async fn update_by_id(db: &DbConn, form_data: &MediaSaveDTO) -> Result<i64> {
    let result = WebsiteMediaModel::update_by_id(db, &form_data.id, form_data).await?;
    Ok(result)
}

/// 根据id查询详情
pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<MediaDetailVO> {
    let result = WebsiteMediaModel::find_by_id(db, id).await?.ok_or_else(|| {
        Error::from(format!("{}={}", "媒体文件不存在，id".to_string(), &id.unwrap_or_default()))
    })?;
    Ok(MediaDetailVO::from(result))
}

/// 分页查询
pub async fn get_by_page(db: &DbConn, query: ListQuery) -> Result<ResultPage<Vec<MediaListVO>>> {
    let select_where = PageWhere {
        keywords: query.keywords,
        file_type: query.file_type,
        category_id: query.category_id,
        status: query.status,
    };
    let select_where = select_where.format();
    let (list, _total) = WebsiteMediaModel::select_in_page(db, query.page_num.unwrap_or(0), query.page_size.unwrap_or(10), select_where.clone()).await?;
    let list_data: Vec<MediaListVO> = list.into_iter().map(|item| MediaListVO::from(item)).collect();
    let count = WebsiteMediaModel::select_count(db, select_where).await.unwrap_or(0);
    let page_data = ResultPage::new_simple(list_data, count);
    Ok(page_data)
}

/// 引用计数增减
pub async fn update_ref_count(db: &DbConn, id: i64, delta: i32) -> Result<i64> {
    let result = WebsiteMediaModel::update_ref_count(db, id, delta).await?;
    Ok(result)
}
