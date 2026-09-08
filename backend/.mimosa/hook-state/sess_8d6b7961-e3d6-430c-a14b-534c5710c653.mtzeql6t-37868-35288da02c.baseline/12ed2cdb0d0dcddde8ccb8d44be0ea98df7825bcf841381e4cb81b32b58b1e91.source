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
use crate::modules::system::model::mail::{MailLogListQuery, MailLogModel, MailLogSaveDTO, MailLogVO};
use sea_orm::{DbConn, DbErr, TransactionTrait};

/// 分页查询邮件日志
pub async fn list(db: &DbConn, query: MailLogListQuery) -> Result<ResultPage<Vec<MailLogVO>>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10);
    let (list, _) = MailLogModel::select_in_page(db, page, page_size, query.customer_id, query.status).await?;
    let count = MailLogModel::select_count(db, query.customer_id, query.status).await.unwrap_or(0);
    let list_data: Vec<MailLogVO> = list.into_iter().map(|m| m.into()).collect();
    Ok(ResultPage::new(list_data, count, page, page_size))
}

/// 按客户查询邮件日志
pub async fn list_by_customer(db: &DbConn, customer_id: i64) -> Result<Vec<MailLogVO>> {
    let list = MailLogModel::select_by_customer(db, customer_id).await?;
    Ok(list.into_iter().map(|m| m.into()).collect())
}

/// 写入邮件日志（事务）
pub async fn insert(db: &DbConn, dto: MailLogSaveDTO) -> Result<i64> {
    let dto = dto.clone();
    let result = db.transaction::<_, i64, DbErr>(|txn| {
        Box::pin(async move { MailLogModel::insert(txn, &dto).await })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;
    Ok(result)
}
