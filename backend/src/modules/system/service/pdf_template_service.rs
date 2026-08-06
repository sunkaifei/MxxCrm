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
use crate::core::web::response::ResultPage;
use crate::modules::system::model::pdf::{
    PdfTemplateListQuery, PdfTemplateModel, PdfTemplateOptionVO, PdfTemplateSaveRequest,
    PdfTemplateUpdateRequest, PdfTemplateVO,
};
use sea_orm::{DbConn, DbErr, TransactionTrait};

/// 分页查询 PDF 模板列表
pub async fn list(db: &DbConn, query: PdfTemplateListQuery) -> Result<ResultPage<Vec<PdfTemplateVO>>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10);
    let name = query.name.clone();
    let doc_type = query.doc_type.clone();
    let status = query.status;
    let (list, _) =
        PdfTemplateModel::select_in_page(db, page, page_size, name.clone(), doc_type.clone(), status)
            .await?;
    let count = PdfTemplateModel::select_count(db, name, doc_type, status)
        .await
        .unwrap_or(0);
    let list_data: Vec<PdfTemplateVO> = list.into_iter().map(|m| m.into()).collect();
    Ok(ResultPage::new(list_data, count, page, page_size))
}

/// 根据 ID 查询 PDF 模板
pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<PdfTemplateVO>> {
    let m = PdfTemplateModel::find_by_id(db, id).await?;
    Ok(m.map(|m| m.into()))
}

/// 查询指定单据类型的默认模板
pub async fn find_default(db: &DbConn, doc_type: &str) -> Result<Option<PdfTemplateVO>> {
    let m = PdfTemplateModel::find_default(db, doc_type).await?;
    Ok(m.map(|m| m.into()))
}

/// 查询指定单据类型的模板选项列表
pub async fn find_options(db: &DbConn, doc_type: &str) -> Result<Vec<PdfTemplateOptionVO>> {
    let list = PdfTemplateModel::find_options(db, doc_type).await?;
    let options: Vec<PdfTemplateOptionVO> = list
        .into_iter()
        .map(|m| PdfTemplateOptionVO {
            id: Option::from(m.id),
            name: m.name,
            template_code: m.template_code,
            is_default: m.is_default,
        })
        .collect();
    Ok(options)
}

/// 新增 PDF 模板
pub async fn insert(
    db: &DbConn,
    req: PdfTemplateSaveRequest,
    user_id: Option<i64>,
) -> Result<i64> {
    let req = req.clone();
    let result = db
        .transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move { PdfTemplateModel::insert(txn, &req, user_id).await })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(result)
}

/// 修改 PDF 模板
pub async fn update(
    db: &DbConn,
    req: PdfTemplateUpdateRequest,
    user_id: Option<i64>,
) -> Result<i64> {
    let id = req.id.unwrap_or_default();
    if id <= 0 {
        return Err(Error::from("PDF模板ID不能为空"));
    }
    let req = req.clone();
    let result = db
        .transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move { PdfTemplateModel::update(txn, id, &req, user_id).await })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(result)
}

/// 批量软删除 PDF 模板
pub async fn bath_delete(db: &DbConn, ids: Vec<i64>) -> Result<i64> {
    if ids.is_empty() {
        return Ok(0);
    }
    let result = db
        .transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move { PdfTemplateModel::bath_delete(txn, &ids).await })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(result)
}

/// 设置默认模板
pub async fn set_default(db: &DbConn, id: i64, doc_type: &str) -> Result<i64> {
    let doc_type = doc_type.to_string();
    let result = db
        .transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move { PdfTemplateModel::set_default(txn, id, &doc_type).await })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(result)
}
