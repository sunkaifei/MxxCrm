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
use crate::modules::articles::model::article_label_merge::ArticleLabelMergeModel;

/// 设置文章标签（先删后插，事务包裹保证原子性）
pub async fn set_labels(db: &DbConn, article_id: i64, label_ids: Vec<i64>) -> Result<i64> {
    let label_ids_clone = label_ids.clone();
    db.transaction::<_, _, DbErr>(|txn| {
        Box::pin(async move {
            ArticleLabelMergeModel::set_article_labels(txn, article_id, label_ids_clone).await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))
}

/// 获取文章的标签ID列表
pub async fn get_labels_by_article(db: &DbConn, article_id: i64) -> Result<Vec<i64>> {
    ArticleLabelMergeModel::get_labels_by_article(db, article_id)
        .await
        .map_err(|e| Error::from(format!("msg={},code=500", e)))
}

/// 按标签分页查询文章ID列表
pub async fn get_articles_by_label(
    db: &DbConn,
    label_id: i64,
    page: i64,
    per_page: i64,
) -> Result<ResultPage<Vec<i64>>> {
    let page = if page < 1 { 1 } else { page };
    let per_page = if per_page < 1 { 10 } else { per_page };

    let (items, total) = ArticleLabelMergeModel::get_articles_by_label(db, label_id, page, per_page)
        .await
        .map_err(|e| Error::from(format!("msg={},code=500", e)))?;

    let total_pages = if per_page == 0 {
        0
    } else {
        (total + per_page - 1) / per_page
    };

    Ok(ResultPage {
        items,
        total,
        current_page: page,
        page_size: per_page,
        total_pages,
    })
}
