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
use crate::modules::website::entity::website_links;
use crate::modules::website::model::website_links::{ListQuery, PageWhere, LinkListVO, WebsiteLinksModel, LinkSaveDTO, LinkDetailVO};
use crate::SNOWFLAKE;
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;

pub async fn insert(db: &DbConn, form_data: LinkSaveDTO) -> Result<i64> {
    let mut link_data= form_data;
    link_data.id = Option::from(SNOWFLAKE.generate() as i64);
    let result = WebsiteLinksModel::insert(&db, &link_data).await?;
    Ok(result)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<Option<String>>) -> Result<i64>{
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let ids = convert_vec_option_string_to_vec_u64(ids_vec.clone());
    let config = WebsiteLinksModel::batch_delete_by_ids(db, ids).await?;
    Ok(config)
}


pub async fn update_by_id(db: &DbConn, form_data: &LinkSaveDTO) -> Result<i64> {
    let result = WebsiteLinksModel::update_by_id(&db, &form_data.id, &form_data).await?;
    Ok(result)
}

/// 根据link_url查询友情链接信息是否存在
pub async fn find_by_link_url_unique(db: &DbConn, link_url: &Option<String>, id: &Option<i64>) -> Result<bool>{
    let result_count = WebsiteLinksModel::find_by_link_url_unique(&db, link_url, id).await?;
    if result_count > 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// 根据id查询友情链接信息
pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<website_links::Model>> {
    let result = WebsiteLinksModel::find_by_id(&db, id).await?.ok_or_else(|| {
        Error::from(format!(
            "{}={}",
            "友情链接信息不存在，id".to_string(),
            &id.unwrap_or_default()
        ))
    })?;
    Ok(Option::from(result))
}

/// 根据id查询友情链接信息
pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<LinkDetailVO> {
    let result = WebsiteLinksModel::find_by_id(&db, id).await?.ok_or_else(|| {
        Error::from(format!(
            "{}={}",
            "友情链接信息不存在，id".to_string(),
            &id.unwrap_or_default()
        ))
    })?;
    Ok(LinkDetailVO::from(result))
}

/// 根据条件查询友情链接翻页列表
pub async fn get_by_page(db: &DbConn, query: ListQuery) -> Result<ResultPage<Vec<LinkListVO>>> {
    let select_where = PageWhere{
        website_id: query.website_id.clone(),
        link_name: query.link_name.clone(),
        link_type: query.link_type.clone(),
        link_url: query.link_url.clone(),
        status: query.status.clone(),
    };
    let select_where = select_where.format();

    let (list, _num_pages) = WebsiteLinksModel::select_in_page(
        &db,
        query.page_num.unwrap_or(0),
        query.page_size.unwrap_or(10),
        select_where.clone()
    ).await?;

    let list_data: Vec<LinkListVO> = list.into_iter().map(|item| LinkListVO::from(item)).collect();

    let count = WebsiteLinksModel::select_count(db, select_where.clone()).await.unwrap_or(0);
    let page_data = ResultPage::new_simple(list_data, count);

    Ok(page_data)
}