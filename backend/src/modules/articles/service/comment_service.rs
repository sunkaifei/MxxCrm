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
use crate::modules::articles::model::comment::{CommentDetailVO, CommentListVO, CommentModel, CommentSaveRequest, ListQuery, PageWhere};
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;

/// 新增评论
pub async fn insert(db: &DbConn, payload: &CommentSaveRequest) -> Result<i64> {
    let result = CommentModel::insert(db, payload).await?;
    Ok(result)
}

/// 批量删除评论（软删除 deleted=1）
pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<Option<String>>) -> Result<i64> {
    let ids = convert_vec_option_string_to_vec_u64(ids_vec.clone());
    let result = CommentModel::batch_delete_by_ids(db, ids).await?;
    Ok(result)
}

/// 审核评论
pub async fn update_status(db: &DbConn, id: i64, status: i32) -> Result<i64> {
    let result = CommentModel::update_status(db, id, status).await?;
    Ok(result)
}

/// 根据id查询评论详情
pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<Option<CommentDetailVO>> {
    let comment_data = CommentModel::find_by_id(db, id).await?;
    match comment_data {
        None => {
            Err(Error::from(format!(
                "{}={}",
                "评论信息不存在，id".to_string(),
                &id.unwrap_or_default()
            )))
        }
        Some(comment) => {
            Ok(Some(CommentDetailVO::from(comment)))
        }
    }
}

/// 根据分页查询评论列表（后台）
pub async fn get_by_page(db: &DbConn, query: ListQuery) -> Result<ResultPage<Vec<CommentListVO>>> {
    let select_where = PageWhere {
        article_id: query.article_id,
        status: query.status,
        keywords: query.keywords,
    };
    let search_where = select_where.format();

    let (list, _num_pages) = CommentModel::select_in_page(
        db,
        query.page_num.unwrap_or(0),
        query.page_size.unwrap_or(10),
        search_where.clone(),
    ).await?;

    let list_data: Vec<CommentListVO> = list.into_iter().map(CommentListVO::from).collect();

    let count = CommentModel::select_count(db, select_where.clone()).await.unwrap_or(0);

    let page_data = ResultPage::new_simple(list_data, count);

    Ok(page_data)
}

/// 前台按文章查询评论（仅返回已审核 status=1 且 deleted=0）
pub async fn get_by_article(db: &DbConn, article_id: i64, page: i64, per_page: i64) -> Result<ResultPage<Vec<CommentListVO>>> {
    let (list, count) = CommentModel::find_by_article(db, article_id, page, per_page).await?;
    let list_data: Vec<CommentListVO> = list.into_iter().map(CommentListVO::from).collect();
    Ok(ResultPage::new_simple(list_data, count))
}
