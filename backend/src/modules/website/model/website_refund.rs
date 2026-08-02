//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究。
//!

use sea_orm::*;
use sea_orm::prelude::{DateTime, Decimal};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::website::entity::{website_refund, website_refund::Entity as WebsiteRefund};

// ==================== 常量 ====================

/// 退款状态：0待审核 1已通过 2已拒绝 3已退款 4已取消
pub const REFUND_STATUS_PENDING: i32 = 0;
pub const REFUND_STATUS_APPROVED: i32 = 1;
pub const REFUND_STATUS_REJECTED: i32 = 2;
pub const REFUND_STATUS_REFUNDED: i32 = 3;
pub const REFUND_STATUS_CANCELLED: i32 = 4;

// ==================== DTO ====================

/// 用户申请退款请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RefundApplyRequest {
    pub order_id: i64,
    /// 订单项ID（部分退款时必填）
    pub order_item_id: Option<i64>,
    /// 退款类型：1仅退款 2退货退款
    #[serde(default = "default_refund_type")]
    pub refund_type: i32,
    pub refund_reason: String,
    /// 退款金额（不传则自动计算）
    pub refund_amount: Option<Decimal>,
}

fn default_refund_type() -> i32 {
    1
}

/// 后台处理退款请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RefundHandleRequest {
    /// 0通过 1拒绝
    pub action: i32,
    pub handle_remark: Option<String>,
    /// 退款方式：1原路退回 2余额
    pub refund_way: Option<i32>,
}

/// 退款列表查询
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RefundListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub refund_no: Option<String>,
    pub order_no: Option<String>,
    pub status: Option<i32>,
    pub user_id: Option<i64>,
}

// ==================== VO ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct RefundVO {
    pub id: Option<i64>,
    pub refund_no: Option<String>,
    pub order_id: Option<i64>,
    pub order_no: Option<String>,
    pub user_id: Option<i64>,
    pub order_item_id: Option<i64>,
    pub refund_type: Option<i32>,
    pub refund_reason: Option<String>,
    pub refund_amount: Option<Decimal>,
    pub status: Option<i32>,
    pub refund_way: Option<i32>,
    pub transaction_id: Option<String>,
    pub handle_remark: Option<String>,
    pub handle_by: Option<i64>,
    pub handle_time: Option<DateTime>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

impl From<website_refund::Model> for RefundVO {
    fn from(item: website_refund::Model) -> Self {
        RefundVO {
            id: Option::from(item.id),
            refund_no: Some(item.refund_no),
            order_id: Some(item.order_id),
            order_no: item.order_no,
            user_id: Some(item.user_id),
            order_item_id: item.order_item_id,
            refund_type: item.refund_type,
            refund_reason: item.refund_reason,
            refund_amount: Some(item.refund_amount),
            status: item.status,
            refund_way: item.refund_way,
            transaction_id: item.transaction_id,
            handle_remark: item.handle_remark,
            handle_by: item.handle_by,
            handle_time: item.handle_time,
            create_time: item.create_time,
            update_time: item.update_time,
        }
    }
}

// ==================== Model ====================

pub struct WebsiteRefundModel;

impl WebsiteRefundModel {
    /// 新增退款单
    pub async fn insert<C: ConnectionTrait>(
        db: &C,
        refund_no: String,
        order_id: i64,
        order_no: Option<String>,
        user_id: i64,
        order_item_id: Option<i64>,
        refund_type: i32,
        refund_reason: String,
        refund_amount: Decimal,
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = website_refund::ActiveModel {
            refund_no: Set(refund_no),
            order_id: Set(order_id),
            order_no: Set(order_no),
            user_id: Set(user_id),
            order_item_id: Set(order_item_id),
            refund_type: Set(Some(refund_type)),
            refund_reason: Set(Some(refund_reason)),
            refund_amount: Set(refund_amount),
            status: Set(Some(REFUND_STATUS_PENDING)),
            handle_remark: Set(None),
            create_time: Set(Some(now.clone())),
            update_time: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        WebsiteRefund::insert(payload).exec(db).await.map(|r| r.last_insert_id)
    }

    /// 根据ID查询
    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<website_refund::Model>, DbErr> {
        WebsiteRefund::find_by_id(id)
            .filter(website_refund::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 根据退款单号查询
    pub async fn find_by_refund_no<C: ConnectionTrait>(db: &C, refund_no: &str) -> Result<Option<website_refund::Model>, DbErr> {
        WebsiteRefund::find()
            .filter(website_refund::Column::RefundNo.eq(refund_no))
            .filter(website_refund::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 按订单ID查询退款单
    pub async fn find_by_order_id<C: ConnectionTrait>(db: &C, order_id: i64) -> Result<Vec<website_refund::Model>, DbErr> {
        WebsiteRefund::find()
            .filter(website_refund::Column::OrderId.eq(order_id))
            .filter(website_refund::Column::Deleted.eq(0))
            .order_by_desc(website_refund::Column::CreateTime)
            .all(db)
            .await
    }

    /// 用户退款列表
    pub async fn select_user_page(
        db: &DbConn,
        user_id: i64,
        page: i64,
        per_page: i64,
        status: Option<i32>,
    ) -> Result<(Vec<website_refund::Model>, i64), DbErr> {
        let mut q = WebsiteRefund::find()
            .filter(website_refund::Column::Deleted.eq(0))
            .filter(website_refund::Column::UserId.eq(user_id));
        if let Some(s) = status { q = q.filter(website_refund::Column::Status.eq(s)); }
        let paginator = q
            .order_by_desc(website_refund::Column::CreateTime)
            .paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;
        let rows = paginator.fetch_page((page - 1) as u64).await?;
        Ok((rows, total))
    }

    /// 后台退款列表
    pub async fn select_admin_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        query: &RefundListQuery,
    ) -> Result<(Vec<website_refund::Model>, i64), DbErr> {
        let mut q = WebsiteRefund::find()
            .filter(website_refund::Column::Deleted.eq(0));
        if let Some(no) = &query.refund_no { q = q.filter(website_refund::Column::RefundNo.like(format!("%{}%", no))); }
        if let Some(no) = &query.order_no { q = q.filter(website_refund::Column::OrderNo.like(format!("%{}%", no))); }
        if let Some(s) = query.status { q = q.filter(website_refund::Column::Status.eq(s)); }
        if let Some(u) = query.user_id { q = q.filter(website_refund::Column::UserId.eq(u)); }
        let paginator = q
            .order_by_desc(website_refund::Column::CreateTime)
            .paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;
        let rows = paginator.fetch_page((page - 1) as u64).await?;
        Ok((rows, total))
    }

    /// 更新状态
    pub async fn update_status<C: ConnectionTrait>(
        db: &C,
        id: i64,
        status: i32,
        handle_remark: Option<String>,
        refund_way: Option<i32>,
        handle_by: i64,
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let result: UpdateResult = WebsiteRefund::update_many()
            .col_expr(website_refund::Column::Status, sea_orm::sea_query::Expr::value(status))
            .col_expr(website_refund::Column::HandleRemark, sea_orm::sea_query::Expr::value(handle_remark))
            .col_expr(website_refund::Column::RefundWay, sea_orm::sea_query::Expr::value(refund_way))
            .col_expr(website_refund::Column::HandleBy, sea_orm::sea_query::Expr::value(handle_by))
            .col_expr(website_refund::Column::HandleTime, sea_orm::sea_query::Expr::value(now.clone()))
            .col_expr(website_refund::Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
            .filter(website_refund::Column::Id.eq(id))
            .filter(website_refund::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 更新第三方退款流水号
    pub async fn update_transaction_id<C: ConnectionTrait>(
        db: &C,
        id: i64,
        transaction_id: String,
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let result: UpdateResult = WebsiteRefund::update_many()
            .col_expr(website_refund::Column::TransactionId, sea_orm::sea_query::Expr::value(transaction_id))
            .col_expr(website_refund::Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
            .filter(website_refund::Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 软删除
    pub async fn batch_soft_delete<C: ConnectionTrait>(db: &C, ids: Vec<i64>) -> Result<i64, DbErr> {
        if ids.is_empty() { return Ok(0); }
        let now = chrono::Local::now().naive_local().to_owned();
        let result: UpdateResult = WebsiteRefund::update_many()
            .col_expr(website_refund::Column::Deleted, sea_orm::sea_query::Expr::value(1))
            .col_expr(website_refund::Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
            .filter(website_refund::Column::Id.is_in(ids))
            .filter(website_refund::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }
}
