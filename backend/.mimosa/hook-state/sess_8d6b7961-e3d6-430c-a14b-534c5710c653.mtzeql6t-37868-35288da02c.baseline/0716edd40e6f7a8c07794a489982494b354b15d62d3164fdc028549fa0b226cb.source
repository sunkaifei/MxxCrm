//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use rust_decimal::Decimal;
use sea_orm::*;
use sea_orm::sea_query::Expr;
use serde::{Deserialize, Serialize};
use crate::modules::inventory::entity::quality_check;

/// 质检单保存请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QualityCheckSaveRequest {
    pub id: Option<i64>,
    /// 关联入库单ID
    pub inbound_id: Option<i64>,
    /// 仓库ID
    pub warehouse_id: Option<i64>,
    /// 产品ID
    pub product_id: Option<i64>,
    /// 产品名称
    pub product_name: Option<String>,
    /// 产品SKU
    pub product_sku: Option<String>,
    /// 质检数量
    pub quantity: Option<Decimal>,
    /// 备注
    pub remark: Option<String>,
}

/// 质检结果录入请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QualityCheckResultRequest {
    /// 合格数量
    pub qualified_quantity: Option<Decimal>,
    /// 不合格数量
    pub unqualified_quantity: Option<Decimal>,
    /// 质检结果：1=合格 2=不合格 3=部分合格
    pub check_result: Option<i32>,
    /// 备注
    pub remark: Option<String>,
}

/// 质检单列表查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QualityCheckListQuery {
    #[serde(rename = "page")]
    pub page_num: u64,
    pub page_size: u64,
    pub check_no: Option<String>,
    pub warehouse_id: Option<i64>,
    pub product_id: Option<i64>,
    pub check_result: Option<i32>,
    pub status: Option<i32>,
}

/// 质检单列表分页响应
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QualityCheckListVO {
    pub list: Vec<QualityCheckListItem>,
    pub total: u64,
}

/// 质检单列表项
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QualityCheckListItem {
    pub id: i64,
    pub check_no: Option<String>,
    pub inbound_id: Option<i64>,
    pub warehouse_id: Option<i64>,
    pub warehouse_name: Option<String>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_sku: Option<String>,
    pub quantity: Option<Decimal>,
    pub qualified_quantity: Option<Decimal>,
    pub unqualified_quantity: Option<Decimal>,
    pub check_result: Option<i32>,
    pub checker: Option<i64>,
    pub checker_name: Option<String>,
    pub check_time: Option<chrono::NaiveDateTime>,
    pub remark: Option<String>,
    pub status: Option<i32>,
    pub created_by: Option<i64>,
    pub created_by_name: Option<String>,
    pub create_time: Option<chrono::NaiveDateTime>,
    pub update_time: Option<chrono::NaiveDateTime>,
}

impl From<quality_check::Model> for QualityCheckListItem {
    fn from(m: quality_check::Model) -> Self {
        Self {
            id: m.id,
            check_no: m.check_no,
            inbound_id: m.inbound_id,
            warehouse_id: m.warehouse_id,
            warehouse_name: None,
            product_id: m.product_id,
            product_name: m.product_name,
            product_sku: m.product_sku,
            quantity: m.quantity,
            qualified_quantity: m.qualified_quantity,
            unqualified_quantity: m.unqualified_quantity,
            check_result: m.check_result,
            checker: m.checker,
            checker_name: None,
            check_time: m.check_time,
            remark: m.remark,
            status: m.status,
            created_by: m.created_by,
            created_by_name: None,
            create_time: m.create_time,
            update_time: m.update_time,
        }
    }
}

/// 生成质检单号：QC + yyyyMMdd + 4位流水号
pub async fn generate_check_no<C: ConnectionTrait>(db: &C) -> Result<String, DbErr> {
    let today = chrono::Local::now().format("%Y%m%d").to_string();
    let prefix = format!("QC{}", today);

    let max_no = quality_check::Entity::find()
        .filter(quality_check::Column::CheckNo.starts_with(&prefix))
        .order_by_desc(quality_check::Column::CheckNo)
        .one(db)
        .await?;

    let seq = match max_no {
        Some(m) => {
            let no = m.check_no.unwrap_or_default();
            let seq_str = no.trim_start_matches(&prefix);
            seq_str.parse::<i32>().unwrap_or(0) + 1
        }
        None => 1,
    };

    Ok(format!("{}{:04}", prefix, seq))
}

/// 分页查询
pub async fn select_page<C: ConnectionTrait>(
    db: &C,
    query: &QualityCheckListQuery,
) -> Result<(Vec<quality_check::Model>, u64), DbErr> {
    let mut q = quality_check::Entity::find()
        .filter(quality_check::Column::Deleted.eq(0));

    if let Some(ref no) = query.check_no {
        q = q.filter(quality_check::Column::CheckNo.contains(no));
    }
    if let Some(wid) = query.warehouse_id {
        q = q.filter(quality_check::Column::WarehouseId.eq(wid));
    }
    if let Some(pid) = query.product_id {
        q = q.filter(quality_check::Column::ProductId.eq(pid));
    }
    if let Some(cr) = query.check_result {
        q = q.filter(quality_check::Column::CheckResult.eq(cr));
    }
    if let Some(s) = query.status {
        q = q.filter(quality_check::Column::Status.eq(s));
    }

    let total = q.clone().count(db).await?;
    let rows = q
        .order_by_desc(quality_check::Column::CreateTime)
        .offset((query.page_num - 1) * query.page_size)
        .limit(query.page_size)
        .all(db)
        .await?;

    Ok((rows, total))
}

/// 按ID查询
pub async fn find_by_id<C: ConnectionTrait>(
    db: &C,
    id: i64,
) -> Result<Option<quality_check::Model>, DbErr> {
    quality_check::Entity::find_by_id(id)
        .filter(quality_check::Column::Deleted.eq(0))
        .one(db)
        .await
}

/// 新增
pub async fn insert<C: ConnectionTrait>(
    db: &C,
    req: &QualityCheckSaveRequest,
    check_no: &str,
    created_by: i64,
) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let active = quality_check::ActiveModel {
        check_no: Set(Some(check_no.to_string())),
        inbound_id: Set(req.inbound_id),
        warehouse_id: Set(req.warehouse_id),
        product_id: Set(req.product_id),
        product_name: Set(req.product_name.clone()),
        product_sku: Set(req.product_sku.clone()),
        quantity: Set(req.quantity),
        qualified_quantity: Set(Some(Decimal::ZERO)),
        unqualified_quantity: Set(Some(Decimal::ZERO)),
        check_result: Set(Some(0)), // 待检
        checker: Set(None),
        check_time: Set(None),
        remark: Set(req.remark.clone()),
        status: Set(Some(0)), // 草稿
        deleted: Set(Some(0)),
        created_by: Set(Some(created_by)),
        updated_by: Set(Some(created_by)),
        create_time: Set(Some(now)),
        update_time: Set(Some(now)),
        ..Default::default()
    };
    let result = active.insert(db).await?;
    Ok(result.id)
}

/// 更新基本信息（仅草稿状态可编辑）
pub async fn update_by_id<C: ConnectionTrait>(
    db: &C,
    id: i64,
    req: &QualityCheckSaveRequest,
    updated_by: i64,
) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let result = quality_check::Entity::update_many()
        .col_expr(quality_check::Column::InboundId, Expr::value(req.inbound_id))
        .col_expr(quality_check::Column::WarehouseId, Expr::value(req.warehouse_id))
        .col_expr(quality_check::Column::ProductId, Expr::value(req.product_id))
        .col_expr(quality_check::Column::ProductName, Expr::value(req.product_name.clone()))
        .col_expr(quality_check::Column::ProductSku, Expr::value(req.product_sku.clone()))
        .col_expr(quality_check::Column::Quantity, Expr::value(req.quantity))
        .col_expr(quality_check::Column::Remark, Expr::value(req.remark.clone()))
        .col_expr(quality_check::Column::UpdatedBy, Expr::value(updated_by))
        .col_expr(quality_check::Column::UpdateTime, Expr::value(now))
        .filter(quality_check::Column::Id.eq(id))
        .filter(quality_check::Column::Deleted.eq(0))
        .filter(quality_check::Column::Status.eq(0))
        .exec(db)
        .await?;
    Ok(result.rows_affected as i64)
}

/// 录入质检结果
pub async fn update_check_result<C: ConnectionTrait>(
    db: &C,
    id: i64,
    req: &QualityCheckResultRequest,
    checker: i64,
) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let result = quality_check::Entity::update_many()
        .col_expr(quality_check::Column::QualifiedQuantity, Expr::value(req.qualified_quantity))
        .col_expr(quality_check::Column::UnqualifiedQuantity, Expr::value(req.unqualified_quantity))
        .col_expr(quality_check::Column::CheckResult, Expr::value(req.check_result))
        .col_expr(quality_check::Column::Remark, Expr::value(req.remark.clone()))
        .col_expr(quality_check::Column::Checker, Expr::value(checker))
        .col_expr(quality_check::Column::CheckTime, Expr::value(now))
        .col_expr(quality_check::Column::Status, Expr::value(1)) // 已质检
        .col_expr(quality_check::Column::UpdatedBy, Expr::value(checker))
        .col_expr(quality_check::Column::UpdateTime, Expr::value(now))
        .filter(quality_check::Column::Id.eq(id))
        .filter(quality_check::Column::Deleted.eq(0))
        .filter(quality_check::Column::Status.eq(0)) // 仅草稿可录入结果
        .exec(db)
        .await?;
    Ok(result.rows_affected as i64)
}

/// 批量软删除（仅草稿可删除）
pub async fn batch_delete<C: ConnectionTrait>(
    db: &C,
    ids: &[i64],
) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local();
    let result = quality_check::Entity::update_many()
        .col_expr(quality_check::Column::Deleted, Expr::value(1))
        .col_expr(quality_check::Column::UpdateTime, Expr::value(now))
        .filter(quality_check::Column::Id.is_in(ids.iter().map(|&id| id).collect::<Vec<_>>()))
        .filter(quality_check::Column::Deleted.eq(0))
        .filter(quality_check::Column::Status.eq(0))
        .exec(db)
        .await?;
    Ok(result.rows_affected as i64)
}
