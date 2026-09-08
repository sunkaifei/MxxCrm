//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::*;
use crate::core::errors::error::{Error, Result};
use crate::modules::inventory::entity::warehouse;
use crate::modules::inventory::model::quality_check as qc_model;
use crate::modules::inventory::model::quality_check::{
    QualityCheckListQuery, QualityCheckListVO, QualityCheckListItem, QualityCheckResultRequest,
    QualityCheckSaveRequest,
};
use crate::modules::system::entity::admin;

/// 创建质检单
pub async fn create(
    db: &DatabaseConnection,
    req: &QualityCheckSaveRequest,
    created_by: i64,
) -> Result<i64> {
    let check_no = qc_model::generate_check_no(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    db.transaction::<_, _, DbErr>(|txn| {
        let req_clone = req.clone();
        let check_no = check_no.clone();
        Box::pin(async move {
            let id = qc_model::insert(txn, &req_clone, &check_no, created_by).await?;
            Ok(id)
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))
}

/// 更新质检单（仅草稿状态可编辑）
pub async fn update(
    db: &DatabaseConnection,
    id: i64,
    req: &QualityCheckSaveRequest,
    updated_by: i64,
) -> Result<i64> {
    let existing = qc_model::find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("质检单不存在"))?;

    if existing.status.unwrap_or(0) != 0 {
        return Err(Error::from("仅草稿状态的质检单可编辑"));
    }

    db.transaction::<_, _, DbErr>(|txn| {
        let req_clone = req.clone();
        Box::pin(async move {
            let rows = qc_model::update_by_id(txn, id, &req_clone, updated_by).await?;
            Ok(rows)
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))
}

/// 录入质检结果
pub async fn check(
    db: &DatabaseConnection,
    id: i64,
    req: &QualityCheckResultRequest,
    checker: i64,
) -> Result<i64> {
    let existing = qc_model::find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("质检单不存在"))?;

    if existing.status.unwrap_or(0) != 0 {
        return Err(Error::from("仅草稿状态的质检单可录入结果"));
    }

    db.transaction::<_, _, DbErr>(|txn| {
        let req_clone = req.clone();
        Box::pin(async move {
            let rows = qc_model::update_check_result(txn, id, &req_clone, checker).await?;
            Ok(rows)
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))
}

/// 批量删除质检单
pub async fn batch_delete(db: &DatabaseConnection, ids: &[i64]) -> Result<i64> {
    db.transaction::<_, _, DbErr>(|txn| {
        let ids = ids.to_vec();
        Box::pin(async move {
            let rows = qc_model::batch_delete(txn, &ids).await?;
            Ok(rows)
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))
}

/// 质检单详情
pub async fn get_detail(
    db: &DatabaseConnection,
    id: i64,
) -> Result<serde_json::Value> {
    let model = qc_model::find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("质检单不存在"))?;

    Ok(serde_json::json!(model))
}

/// 质检单列表
pub async fn get_list(
    db: &DatabaseConnection,
    query: &QualityCheckListQuery,
) -> Result<QualityCheckListVO> {
    let (models, total) = qc_model::select_page(db, query)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let mut list: Vec<QualityCheckListItem> = models.into_iter().map(|m| m.into()).collect();

    // 补充仓库名称、创建人/质检人姓名
    for item in &mut list {
        if let Some(wid) = item.warehouse_id {
            if let Ok(Some(wh)) = warehouse::Entity::find_by_id(wid)
                .filter(warehouse::Column::Deleted.eq(0))
                .one(db)
                .await
            {
                item.warehouse_name = wh.name;
            }
        }
        if let Some(cb) = item.created_by {
            if let Ok(Some(admin)) = admin::Entity::find_by_id(cb).one(db).await {
                item.created_by_name = admin.nick_name.or(admin.user_name);
            }
        }
        if let Some(ck) = item.checker {
            if let Ok(Some(admin)) = admin::Entity::find_by_id(ck).one(db).await {
                item.checker_name = admin.nick_name.or(admin.user_name);
            }
        }
    }

    Ok(QualityCheckListVO { list, total })
}
