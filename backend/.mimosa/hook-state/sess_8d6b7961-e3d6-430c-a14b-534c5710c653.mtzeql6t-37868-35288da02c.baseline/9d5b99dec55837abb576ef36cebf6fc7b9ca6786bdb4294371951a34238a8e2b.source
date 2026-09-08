//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究。
//!

use sea_orm::{DbConn, DbErr, TransactionTrait};
use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::website::model::website_order::{WebsiteOrderModel, PAY_STATUS_PAID, PAY_STATUS_PARTIAL_REFUND, PAY_STATUS_REFUNDED};
use crate::modules::website::model::website_order_item::WebsiteOrderItemModel;
use crate::modules::website::model::website_refund::{
    RefundApplyRequest, RefundHandleRequest, RefundListQuery, RefundVO,
    WebsiteRefundModel, REFUND_STATUS_APPROVED, REFUND_STATUS_CANCELLED,
    REFUND_STATUS_PENDING, REFUND_STATUS_REFUNDED, REFUND_STATUS_REJECTED,
};

/// 生成退款单号
fn gen_refund_no() -> String {
    let ts = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
    let snowflake_id = crate::SNOWFLAKE.generate();
    format!("R{}", ts.chars().take(8).collect::<String>()).to_string() + &format!("{}", snowflake_id % 100000)
}

/// 用户申请退款
pub async fn apply(db: &DbConn, user_id: i64, req: RefundApplyRequest) -> Result<i64> {
    // 前置校验订单
    let order = WebsiteOrderModel::find_by_id(db, req.order_id)
        .await?
        .ok_or_else(|| Error::from("订单不存在"))?;
    if order.user_id != user_id {
        return Err(Error::from("无权操作该订单"));
    }
    // 必须已支付才能退款
    if order.pay_status.unwrap_or(0) != PAY_STATUS_PAID && order.pay_status.unwrap_or(0) != PAY_STATUS_PARTIAL_REFUND {
        return Err(Error::from("订单支付状态不允许退款"));
    }

    // 计算退款金额
    let refund_amount = if let Some(amount) = req.refund_amount {
        if amount <= sea_orm::prelude::Decimal::from(0) {
            return Err(Error::from("退款金额必须大于0"));
        }
        if amount > order.pay_amount {
            return Err(Error::from("退款金额不能超过订单实付金额"));
        }
        amount
    } else if let Some(item_id) = req.order_item_id {
        // 部分退款：按订单项金额
        let item = WebsiteOrderItemModel::find_by_id(db, item_id)
            .await?
            .ok_or_else(|| Error::from("订单项不存在"))?;
        if item.order_id != req.order_id {
            return Err(Error::from("订单项不属于该订单"));
        }
        item.total_amount
    } else {
        // 整单退款
        order.pay_amount
    };

    let refund_no = gen_refund_no();
    let order_no = Some(order.order_no.clone());
    let order_no_clone = order_no.clone();
    let req_clone = req.clone();
    let refund_amount_clone = refund_amount;

    let refund_id = db
        .transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move {
                WebsiteRefundModel::insert(
                    txn,
                    refund_no,
                    req_clone.order_id,
                    order_no_clone,
                    user_id,
                    req_clone.order_item_id,
                    req_clone.refund_type,
                    req_clone.refund_reason,
                    refund_amount_clone,
                )
                .await
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(refund_id)
}

/// 用户取消退款申请
pub async fn user_cancel(db: &DbConn, user_id: i64, refund_id: i64) -> Result<i64> {
    let refund = WebsiteRefundModel::find_by_id(db, refund_id)
        .await?
        .ok_or_else(|| Error::from("退款单不存在"))?;
    if refund.user_id != user_id {
        return Err(Error::from("无权操作该退款单"));
    }
    if refund.status.unwrap_or(0) != REFUND_STATUS_PENDING {
        return Err(Error::from("当前退款状态不允许取消"));
    }

    db.transaction::<_, i64, DbErr>(|txn| {
        Box::pin(async move {
            WebsiteRefundModel::update_status(
                txn,
                refund_id,
                REFUND_STATUS_CANCELLED,
                Some("用户主动取消".to_string()),
                None,
                user_id,
            )
            .await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;
    Ok(refund_id)
}

/// 用户退款列表
pub async fn user_list(db: &DbConn, user_id: i64, page: i64, page_size: i64, status: Option<i32>) -> Result<ResultPage<Vec<RefundVO>>> {
    let page = page.max(1);
    let page_size = page_size.max(1).min(100);
    let (list, total) = WebsiteRefundModel::select_user_page(db, user_id, page, page_size, status).await?;
    let list_vo: Vec<RefundVO> = list.into_iter().map(|m| m.into()).collect();
    Ok(ResultPage::new(list_vo, total, page, page_size))
}

/// 退款详情
pub async fn get_detail(db: &DbConn, user_id: Option<i64>, refund_id: i64) -> Result<RefundVO> {
    let refund = WebsiteRefundModel::find_by_id(db, refund_id)
        .await?
        .ok_or_else(|| Error::from("退款单不存在"))?;
    if let Some(uid) = user_id {
        if refund.user_id != uid {
            return Err(Error::from("无权访问该退款单"));
        }
    }
    Ok(refund.into())
}

// ==================== 后台管理 ====================

/// 后台退款列表
pub async fn admin_list(db: &DbConn, query: RefundListQuery) -> Result<ResultPage<Vec<RefundVO>>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).max(1).min(100);
    let (list, total) = WebsiteRefundModel::select_admin_page(db, page, page_size, &query).await?;
    let list_vo: Vec<RefundVO> = list.into_iter().map(|m| m.into()).collect();
    Ok(ResultPage::new(list_vo, total, page, page_size))
}

/// 后台退款详情
pub async fn admin_detail(db: &DbConn, refund_id: i64) -> Result<RefundVO> {
    get_detail(db, None, refund_id).await
}

/// 后台审核退款（通过/拒绝）
pub async fn admin_handle(db: &DbConn, refund_id: i64, req: RefundHandleRequest, handle_by: i64) -> Result<i64> {
    let refund = WebsiteRefundModel::find_by_id(db, refund_id)
        .await?
        .ok_or_else(|| Error::from("退款单不存在"))?;
    if refund.status.unwrap_or(0) != REFUND_STATUS_PENDING {
        return Err(Error::from("当前退款状态不允许处理"));
    }

    let new_status = if req.action == 0 {
        REFUND_STATUS_APPROVED
    } else {
        REFUND_STATUS_REJECTED
    };

    let req_clone = req.clone();
    db.transaction::<_, i64, DbErr>(|txn| {
        Box::pin(async move {
            WebsiteRefundModel::update_status(
                txn,
                refund_id,
                new_status,
                req_clone.handle_remark.clone(),
                req_clone.refund_way,
                handle_by,
            )
            .await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;
    Ok(refund_id)
}

/// 后台标记退款完成（实际打款后调用，或退款回调调用）
pub async fn admin_mark_refunded(db: &DbConn, refund_id: i64, transaction_id: Option<String>) -> Result<i64> {
    let refund = WebsiteRefundModel::find_by_id(db, refund_id)
        .await?
        .ok_or_else(|| Error::from("退款单不存在"))?;
    if refund.status.unwrap_or(0) != REFUND_STATUS_APPROVED {
        return Err(Error::from("仅已通过的退款单可标记完成"));
    }

    let order_id = refund.order_id;
    let refund_amount = refund.refund_amount;
    let txn_id = transaction_id.clone();

    db.transaction::<_, i64, DbErr>(|txn| {
        Box::pin(async move {
            // 1. 更新退款单状态为已退款
            WebsiteRefundModel::update_status(
                txn,
                refund_id,
                REFUND_STATUS_REFUNDED,
                None,
                None,
                0,
            )
            .await?;

            // 2. 若传了流水号，更新流水号
            if let Some(tid) = txn_id {
                WebsiteRefundModel::update_transaction_id(txn, refund_id, tid).await?;
            }

            // 3. 更新订单支付状态（整单退款→已退款；部分退款→部分退款）
            use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};
            use crate::modules::website::entity::website_order::{Entity as WebsiteOrder, Column};
            // 检查该订单是否还有未完成的退款单
            let all_refunds = WebsiteRefundModel::find_by_order_id(txn, order_id).await?;
            let all_refunded: bool = all_refunds.iter().all(|r| r.status.unwrap_or(0) == REFUND_STATUS_REFUNDED || r.status.unwrap_or(0) == REFUND_STATUS_CANCELLED);
            let total_refunded: sea_orm::prelude::Decimal = all_refunds.iter()
                .filter(|r| r.status.unwrap_or(0) == REFUND_STATUS_REFUNDED)
                .map(|r| r.refund_amount)
                .sum();

            let new_pay_status = if all_refunded && total_refunded >= refund_amount {
                PAY_STATUS_REFUNDED
            } else {
                PAY_STATUS_PARTIAL_REFUND
            };

            let now = chrono::Local::now().naive_local().to_owned();
            let _ = WebsiteOrder::update_many()
                .col_expr(Column::PayStatus, sea_orm::sea_query::Expr::value(new_pay_status))
                .col_expr(Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
                .filter(Column::Id.eq(order_id))
                .exec(txn)
                .await?;

            Ok(refund_id)
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    Ok(refund_id)
}

/// 后台批量删除退款单
pub async fn admin_batch_delete(db: &DbConn, ids: Vec<i64>) -> Result<i64> {
    db.transaction::<_, i64, DbErr>(|txn| {
        let ids_clone = ids.clone();
        Box::pin(async move {
            WebsiteRefundModel::batch_soft_delete(txn, ids_clone).await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))
}
