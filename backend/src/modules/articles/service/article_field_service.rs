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
use crate::modules::articles::model::article_field::{
    ArticleFieldListQuery, ArticleFieldModel, ArticleFieldSaveDTO, ArticleFieldVO,
    ArticleFieldValueBatchDTO, ArticleFieldValueModel, ArticleFieldValueVO,
};

/// 分页查询字段定义
pub async fn get_by_page(
    db: &DbConn,
    query: ArticleFieldListQuery,
) -> Result<ResultPage<Vec<ArticleFieldVO>>> {
    ArticleFieldModel::find_by_page(db, &query)
        .await
        .map_err(|e| Error::from(e.to_string()))
}

/// 根据ID查询字段定义
pub async fn get_by_id(db: &DbConn, id: i64) -> Result<ArticleFieldVO> {
    ArticleFieldModel::find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("字段定义不存在"))
}

/// 按栏目查询全部字段（供文章编辑页动态表单用）
pub async fn get_by_category(db: &DbConn, category_id: i64) -> Result<Vec<ArticleFieldVO>> {
    ArticleFieldModel::find_by_category(db, category_id)
        .await
        .map_err(|e| Error::from(e.to_string()))
}

/// 新增字段定义
pub async fn create(db: &DbConn, req: ArticleFieldSaveDTO) -> Result<i64> {
    if req.field_name.is_empty() {
        return Err(Error::from("字段名不能为空"));
    }
    let req_clone = req.clone();
    let id = db
        .transaction::<_, i64, DbErr>(|txn| {
            let req_clone2 = req_clone.clone();
            Box::pin(async move { ArticleFieldModel::insert(txn, &req_clone2).await })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(id)
}

/// 更新字段定义
pub async fn update(db: &DbConn, id: i64, req: ArticleFieldSaveDTO) -> Result<i64> {
    if req.field_name.is_empty() {
        return Err(Error::from("字段名不能为空"));
    }
    let req_clone = req.clone();
    db.transaction::<_, i64, DbErr>(|txn| {
        let req_clone2 = req_clone.clone();
        Box::pin(async move { ArticleFieldModel::update(txn, id, &req_clone2).await })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;
    Ok(id)
}

/// 批量删除字段定义
pub async fn batch_delete(db: &DbConn, ids: Vec<i64>) -> Result<i64> {
    db.transaction::<_, i64, DbErr>(|txn| {
        let ids_clone = ids.clone();
        Box::pin(async move { ArticleFieldModel::batch_delete(txn, ids_clone).await })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))
}

/// 查询文章的自定义字段值（含字段定义信息）
pub async fn get_article_values(db: &DbConn, article_id: i64) -> Result<Vec<ArticleFieldValueVO>> {
    ArticleFieldValueModel::find_by_article(db, article_id)
        .await
        .map_err(|e| Error::from(e.to_string()))
}

/// 批量保存文章字段值（先删后插，事务保证原子性）
pub async fn save_article_values(db: &DbConn, req: ArticleFieldValueBatchDTO) -> Result<i64> {
    let article_id = req.article_id;
    let values = req.values.clone();
    db.transaction::<_, i64, DbErr>(|txn| {
        let values_clone = values.clone();
        Box::pin(async move {
            ArticleFieldValueModel::save_article_values(txn, article_id, &values_clone).await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))
}

/// 删除文章的所有字段值（文章删除时联动清理）
pub async fn delete_article_values(db: &DbConn, article_id: i64) -> Result<i64> {
    ArticleFieldValueModel::delete_by_article(db, article_id)
        .await
        .map_err(|e| Error::from(e.to_string()))
}
