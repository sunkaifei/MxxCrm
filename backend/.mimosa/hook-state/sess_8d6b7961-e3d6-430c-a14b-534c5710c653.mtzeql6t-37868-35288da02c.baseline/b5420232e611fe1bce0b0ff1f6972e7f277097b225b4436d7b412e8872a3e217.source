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
use crate::modules::system::model::mail::{
    MailTemplateListQuery, MailTemplateModel, MailTemplateOption, MailTemplateSaveRequest,
    MailTemplateUpdateRequest, MailTemplateVO,
};
use sea_orm::{DbConn, DbErr, TransactionTrait};

/// 分页查询邮件模板列表
pub async fn list(db: &DbConn, query: MailTemplateListQuery) -> Result<ResultPage<Vec<MailTemplateVO>>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10);
    let name = query.name.clone();
    let (list, _) = MailTemplateModel::select_in_page(db, page, page_size, name.clone()).await?;
    let count = MailTemplateModel::select_count(db, name).await.unwrap_or(0);
    let list_data: Vec<MailTemplateVO> = list.into_iter().map(|m| m.into()).collect();
    Ok(ResultPage::new(list_data, count, page, page_size))
}

/// 根据 ID 查询邮件模板
pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<MailTemplateVO>> {
    let m = MailTemplateModel::find_by_id(db, id).await?;
    Ok(m.map(|m| m.into()))
}

/// 获取全部模板选项（id + name，不分页）
pub async fn options(db: &DbConn) -> Result<Vec<MailTemplateOption>> {
    MailTemplateModel::find_all_options(db)
        .await
        .map_err(|e| Error::from(e.to_string()))
}

/// 新增邮件模板
pub async fn insert(db: &DbConn, req: MailTemplateSaveRequest, user_id: Option<i64>) -> Result<i64> {
    let req = req.clone();
    let result = db.transaction::<_, i64, DbErr>(|txn| {
        Box::pin(async move { MailTemplateModel::insert(txn, &req, user_id).await })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;
    Ok(result)
}

/// 修改邮件模板
pub async fn update(db: &DbConn, req: MailTemplateUpdateRequest, user_id: Option<i64>) -> Result<i64> {
    let id = req.id.unwrap_or_default();
    if id <= 0 {
        return Err(Error::from("邮件模板ID不能为空"));
    }
    let req = req.clone();
    let result = db.transaction::<_, i64, DbErr>(|txn| {
        Box::pin(async move { MailTemplateModel::update(txn, id, &req, user_id).await })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;
    Ok(result)
}

/// 批量软删除邮件模板
pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64> {
    if ids.is_empty() {
        return Ok(0);
    }
    let ids_clone = ids.clone();
    let result = db.transaction::<_, i64, DbErr>(|txn| {
        Box::pin(async move { MailTemplateModel::batch_delete_by_ids(txn, ids_clone).await })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;
    Ok(result)
}
