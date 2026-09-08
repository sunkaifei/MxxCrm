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
use sea_orm::DbConn;
use crate::core::web::response::ResultPage;
use crate::modules::articles::model::article_tag::{ArticleTagDetailVO, ArticleTagListVO, ArticleTagModel, ArticleTagSaveDTO, ListQuery, PageWhere};
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;

pub async fn insert(db: &DbConn, form_data: &ArticleTagSaveDTO) -> Result<i64> {
    // 检查名称是否重复
    let existing = ArticleTagModel::find_by_name_unique(db, &form_data.name, &None).await?;
    if existing > 0 {
        return Err(Error::from("标签名称已存在"));
    }
    let result = ArticleTagModel::insert(db, form_data).await?;
    Ok(result)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<Option<String>>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let ids = convert_vec_option_string_to_vec_u64(ids_vec.clone());
    let result = ArticleTagModel::batch_delete_by_ids(db, ids).await?;
    Ok(result)
}

pub async fn update_by_id(db: &DbConn, form_data: &ArticleTagSaveDTO) -> Result<i64> {
    // 检查名称是否重复（排除自身）
    let existing = ArticleTagModel::find_by_name_unique(db, &form_data.name, &form_data.id).await?;
    if existing > 0 {
        return Err(Error::from("标签名称已存在"));
    }
    let result = ArticleTagModel::update_by_id(db, &form_data.id, form_data).await?;
    Ok(result)
}

pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<ArticleTagDetailVO> {
    let result = ArticleTagModel::find_by_id(db, id).await?.ok_or_else(|| {
        Error::from(format!(
            "{}={}",
            "文章标签不存在，id".to_string(),
            &id.unwrap_or_default()
        ))
    })?;
    let result = ArticleTagDetailVO::from(result);
    Ok(result)
}

pub async fn get_by_page(db: &DbConn, query: ListQuery) -> Result<ResultPage<Vec<ArticleTagListVO>>> {
    let select_where = PageWhere {
        keywords: query.keywords,
        status: query.status,
    };
    let select_where = select_where.format();
    let (list, _total) = ArticleTagModel::select_in_page(
        db,
        query.page_num.unwrap_or(0),
        query.page_size.unwrap_or(10),
        select_where.clone(),
    ).await?;

    // 转换为VO并填充文章数量
    let mut list_data: Vec<ArticleTagListVO> = Vec::new();
    for item in list {
        let mut vo = ArticleTagListVO::from(item);
        vo.article_count = Some(ArticleTagModel::count_articles(db, &vo.id).await.unwrap_or(0));
        list_data.push(vo);
    }

    let count = ArticleTagModel::select_count(db, select_where.clone()).await.unwrap_or(0);
    let page_data = ResultPage::new_simple(list_data, count);
    Ok(page_data)
}

pub async fn get_all(db: &DbConn) -> Result<Vec<ArticleTagListVO>> {
    let list = ArticleTagModel::select_all(db).await?;
    let mut list_data: Vec<ArticleTagListVO> = Vec::new();
    for item in list {
        let mut vo = ArticleTagListVO::from(item);
        vo.article_count = Some(ArticleTagModel::count_articles(db, &vo.id).await.unwrap_or(0));
        list_data.push(vo);
    }
    Ok(list_data)
}