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
use crate::modules::system::entity::mail_config;
use crate::modules::system::model::mail::{
    MailConfigListQuery, MailConfigModel, MailConfigSaveRequest, MailConfigUpdateRequest, MailConfigVO,
};
use sea_orm::{DbConn, DbErr, TransactionTrait};

/// 分页查询邮箱配置列表
pub async fn list(db: &DbConn, query: MailConfigListQuery) -> Result<ResultPage<Vec<MailConfigVO>>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10);
    let name = query.name.clone();
    let status = query.status;
    let (list, _) = MailConfigModel::select_in_page(db, page, page_size, name.clone(), status).await?;
    let count = MailConfigModel::select_count(db, name, status).await.unwrap_or(0);
    let list_data: Vec<MailConfigVO> = list.into_iter().map(|m| m.into()).collect();
    Ok(ResultPage::new(list_data, count, page, page_size))
}

/// 根据 ID 查询邮箱配置（返回 VO）
pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<MailConfigVO>> {
    let m = MailConfigModel::find_by_id(db, id).await?;
    Ok(m.map(|m| m.into()))
}

/// 根据 ID 查询邮箱配置（返回 entity，内部使用）
pub async fn find_by_id_internal(db: &DbConn, id: i64) -> Result<Option<mail_config::Model>> {
    MailConfigModel::find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))
}

/// 获取默认邮箱配置（返回 entity，供发送邮件使用）
pub async fn find_default(db: &DbConn) -> Result<Option<mail_config::Model>> {
    MailConfigModel::find_default(db)
        .await
        .map_err(|e| Error::from(e.to_string()))
}

/// 新增邮箱配置
/// 若 is_default=1，事务内先重置其他默认再插入
pub async fn insert(db: &DbConn, req: MailConfigSaveRequest, user_id: Option<i64>) -> Result<i64> {
    let is_default = req.is_default.unwrap_or(0);
    let req = req.clone();
    let result = db.transaction::<_, i64, DbErr>(|txn| {
        Box::pin(async move {
            if is_default == 1 {
                MailConfigModel::update_reset_default(txn).await?;
            }
            let id = MailConfigModel::insert(txn, &req, user_id).await?;
            Ok(id)
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;
    Ok(result)
}

/// 修改邮箱配置
pub async fn update(db: &DbConn, req: MailConfigUpdateRequest, user_id: Option<i64>) -> Result<i64> {
    let id = req.id.unwrap_or_default();
    if id <= 0 {
        return Err(Error::from("邮箱配置ID不能为空"));
    }
    let is_default = req.is_default.unwrap_or(0);
    let req = req.clone();
    let result = db.transaction::<_, i64, DbErr>(|txn| {
        Box::pin(async move {
            if is_default == 1 {
                MailConfigModel::update_reset_default(txn).await?;
            }
            let r = MailConfigModel::update(txn, id, &req, user_id).await?;
            Ok(r)
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;
    Ok(result)
}

/// 批量软删除邮箱配置
pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64> {
    if ids.is_empty() {
        return Ok(0);
    }
    let ids_clone = ids.clone();
    let result = db.transaction::<_, i64, DbErr>(|txn| {
        Box::pin(async move { MailConfigModel::batch_delete_by_ids(txn, ids_clone).await })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;
    Ok(result)
}

/// 设置默认邮箱配置
/// 事务内先重置其他默认，再将当前设为默认
pub async fn set_default(db: &DbConn, id: i64, user_id: Option<i64>) -> Result<i64> {
    let result = db.transaction::<_, i64, DbErr>(|txn| {
        Box::pin(async move {
            MailConfigModel::update_reset_default(txn).await?;
            let r = MailConfigModel::update_set_default(txn, id, user_id).await?;
            Ok(r)
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;
    Ok(result)
}
