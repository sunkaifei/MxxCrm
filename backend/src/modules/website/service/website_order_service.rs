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
use crate::modules::website::model::website_cart::WebsiteCartModel;
use crate::modules::website::model::website_order::{
    OrderCreateRequest, OrderItemDTO, OrderListQuery, OrderUpdateRequest, OrderVO,
    WebsiteOrderModel, STATUS_PENDING_PAY, STATUS_PENDING_RECEIVE,
};
use crate::modules::website::model::website_order_item::WebsiteOrderItemModel;
use crate::modules::website::service::website_user_service;

// ==================== 订单 Service ====================

/// 生成订单号
fn gen_order_no() -> String {
    let ts = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
    let snowflake_id = crate::SNOWFLAKE.generate();
    format!("W{}", ts.chars().take(8).collect::<String>()).to_string() + &format!("{}", snowflake_id % 100000)
}

/// 创建订单
pub async fn create_order(db: &DbConn, user_id: i64, req: OrderCreateRequest) -> Result<i64> {
    // 收集订单项
    let mut items: Vec<OrderItemDTO> = Vec::new();
    let mut cart_ids_to_delete: Vec<i64> = Vec::new();

    if let Some(cart_ids) = &req.cart_ids {
        if !cart_ids.is_empty() {
            // 从购物车下单
            let cart_items = WebsiteCartModel::find_by_user(db, user_id).await?;
            for id in cart_ids {
                if let Some(item) = cart_items.iter().find(|c| c.id == *id) {
                    items.push(OrderItemDTO {
                        product_id: item.product_id,
                        sku_id: item.sku_id,
                        product_name: item.product_name.clone(),
                        product_image: item.product_image.clone(),
                        sku_code: item.sku_code.clone(),
                        sku_specs: item.sku_specs.clone(),
                        price: item.price,
                        quantity: item.quantity,
                    });
                    cart_ids_to_delete.push(item.id);
                }
            }
        }
    } else if let Some(direct_items) = &req.items {
        items = direct_items.clone();
    }

    if items.is_empty() {
        return Err(Error::from("订单商品不能为空"));
    }

    // 校验数量
    for item in &items {
        if item.quantity <= 0 {
            return Err(Error::from("商品数量必须大于0"));
        }
    }

    // 计算金额
    let total_amount: sea_orm::prelude::Decimal = items
        .iter()
        .map(|i| i.price * sea_orm::prelude::Decimal::from(i.quantity))
        .sum();
    let pay_amount = total_amount;

    let order_no = gen_order_no();
    let order_no_clone = order_no.clone();
    let req_clone = req.clone();
    let items_clone = items.clone();
    let cart_ids_clone = cart_ids_to_delete.clone();

    let order_id = db
        .transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move {
                // 1. 创建订单
                let order_id = WebsiteOrderModel::insert(
                    txn,
                    order_no_clone,
                    user_id,
                    &req_clone,
                    total_amount,
                    pay_amount,
                )
                .await?;

                // 2. 创建订单项
                WebsiteOrderItemModel::batch_insert(txn, order_id, &items_clone).await?;

                // 3. 清理已下单的购物车项
                if !cart_ids_clone.is_empty() {
                    WebsiteCartModel::delete_by_ids(txn, cart_ids_clone).await?;
                }

                Ok(order_id)
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    // G-2.8: 触发"新订单"邮件通知（失败不影响主流程）
    let notify_ctx = serde_json::json!({
        "order_id": order_id,
        "order_no": order_no,
        "user_id": user_id,
        "total_amount": total_amount.to_string(),
        "item_count": items.len(),
    });
    if let Ok(site) = crate::modules::website::service::website_service::find_default(db).await {
        let site_id = site.id.unwrap_or_default();
        if site_id > 0 {
            if let Err(e) = crate::modules::website::service::website_notification_config_service::send_notification(
                db,
                site_id,
                "new_order",
                notify_ctx,
            ).await {
                log::warn!("[通知触发失败] new_order, error={}", e);
            }
        }
    }

    Ok(order_id)
}

/// 订单详情（含订单项）
pub async fn get_order_detail(db: &DbConn, user_id: i64, order_id: i64) -> Result<OrderVO> {
    let order = WebsiteOrderModel::find_by_id(db, order_id)
        .await?
        .ok_or_else(|| Error::from("订单不存在"))?;
    if order.user_id != user_id {
        return Err(Error::from("无权访问该订单"));
    }
    let mut vo: OrderVO = order.into();
    let items = WebsiteOrderItemModel::find_by_order_id(db, order_id).await?;
    vo.items = items.into_iter().map(|m| m.into()).collect();
    Ok(vo)
}

/// 用户订单列表
pub async fn user_order_list(
    db: &DbConn,
    user_id: i64,
    page: i64,
    page_size: i64,
    status: Option<i32>,
) -> Result<ResultPage<Vec<OrderVO>>> {
    let page = page.max(1);
    let page_size = page_size.max(1).min(100);
    let (orders, total) = WebsiteOrderModel::select_user_page(db, user_id, page, page_size, status).await?;

    // 批量加载订单项
    let order_ids: Vec<i64> = orders.iter().map(|o| o.id).collect();
    let all_items = WebsiteOrderItemModel::find_by_order_ids(db, order_ids).await?;
    let mut vo_list: Vec<OrderVO> = Vec::with_capacity(orders.len());
    for order in orders {
        let oid = order.id;
        let mut vo: OrderVO = order.into();
        vo.items = all_items
            .iter()
            .filter(|i| i.order_id == oid)
            .cloned()
            .map(|m| m.into())
            .collect();
        vo_list.push(vo);
    }
    Ok(ResultPage::new(vo_list, total, page, page_size))
}

/// 用户取消订单（仅待付款状态可取消）
pub async fn user_cancel_order(db: &DbConn, user_id: i64, order_id: i64, reason: String) -> Result<i64> {
    let order = WebsiteOrderModel::find_by_id(db, order_id)
        .await?
        .ok_or_else(|| Error::from("订单不存在"))?;
    if order.user_id != user_id {
        return Err(Error::from("无权操作该订单"));
    }
    if order.status.unwrap_or(0) != STATUS_PENDING_PAY {
        return Err(Error::from("当前订单状态不允许取消"));
    }

    db.transaction::<_, i64, DbErr>(|txn| {
        Box::pin(async move {
            WebsiteOrderModel::update_cancelled(txn, order_id, reason).await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;
    Ok(order_id)
}

/// 用户确认收货（仅待收货状态可确认）
pub async fn user_confirm_receive(db: &DbConn, user_id: i64, order_id: i64) -> Result<i64> {
    let order = WebsiteOrderModel::find_by_id(db, order_id)
        .await?
        .ok_or_else(|| Error::from("订单不存在"))?;
    if order.user_id != user_id {
        return Err(Error::from("无权操作该订单"));
    }
    if order.status.unwrap_or(0) != STATUS_PENDING_RECEIVE {
        return Err(Error::from("当前订单状态不允许确认收货"));
    }

    let pay_amount = order.pay_amount;
    let uid = order.user_id;

    db.transaction::<_, i64, DbErr>(|txn| {
        Box::pin(async move {
            let affected = WebsiteOrderModel::update_received(txn, order_id).await?;
            if affected > 0 {
                // 累计用户消费金额与订单数
                let _ = crate::modules::website::model::website_user::WebsiteUserModel::add_spent(txn, uid, pay_amount).await;
            }
            Ok(affected)
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;
    Ok(order_id)
}

/// 支付成功（回调调用）
pub async fn mark_order_paid(db: &DbConn, order_no: &str, pay_type: i32, transaction_id: String) -> Result<i64> {
    let order = WebsiteOrderModel::find_by_order_no(db, order_no)
        .await?
        .ok_or_else(|| Error::from("订单不存在"))?;
    if order.pay_status.unwrap_or(0) != 0 {
        // 已支付，幂等返回
        return Ok(order.id);
    }
    let order_id = order.id;
    db.transaction::<_, i64, DbErr>(|txn| {
        Box::pin(async move {
            WebsiteOrderModel::update_paid(txn, order_id, pay_type, transaction_id).await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;
    Ok(order_id)
}

// ==================== 后台管理 ====================

/// 后台订单列表
pub async fn admin_order_list(db: &DbConn, query: OrderListQuery) -> Result<ResultPage<Vec<OrderVO>>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).max(1).min(100);
    let (orders, total) = WebsiteOrderModel::select_admin_page(db, page, page_size, &query).await?;
    let order_ids: Vec<i64> = orders.iter().map(|o| o.id).collect();
    let all_items = WebsiteOrderItemModel::find_by_order_ids(db, order_ids).await?;
    let mut vo_list: Vec<OrderVO> = Vec::with_capacity(orders.len());
    for order in orders {
        let oid = order.id;
        let mut vo: OrderVO = order.into();
        vo.items = all_items
            .iter()
            .filter(|i| i.order_id == oid)
            .cloned()
            .map(|m| m.into())
            .collect();
        vo_list.push(vo);
    }
    Ok(ResultPage::new(vo_list, total, page, page_size))
}

/// 后台订单详情
pub async fn admin_order_detail(db: &DbConn, order_id: i64) -> Result<OrderVO> {
    let order = WebsiteOrderModel::find_by_id(db, order_id)
        .await?
        .ok_or_else(|| Error::from("订单不存在"))?;
    let mut vo: OrderVO = order.into();
    let items = WebsiteOrderItemModel::find_by_order_id(db, order_id).await?;
    vo.items = items.into_iter().map(|m| m.into()).collect();
    Ok(vo)
}

/// 后台更新订单（卖家备注等）
pub async fn admin_update_order(db: &DbConn, order_id: i64, req: OrderUpdateRequest) -> Result<i64> {
    let order = WebsiteOrderModel::find_by_id(db, order_id)
        .await?
        .ok_or_else(|| Error::from("订单不存在"))?;

    if let Some(remark) = req.seller_remark {
        let remark_clone = remark.clone();
        db.transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move {
                WebsiteOrderModel::update_seller_remark(txn, order_id, remark_clone).await
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    }
    Ok(order_id)
}

/// 后台批量删除订单（软删除）
pub async fn admin_batch_delete_orders(db: &DbConn, ids: Vec<i64>) -> Result<i64> {
    db.transaction::<_, i64, DbErr>(|txn| {
        let ids_clone = ids.clone();
        Box::pin(async move {
            WebsiteOrderModel::batch_soft_delete(txn, ids_clone).await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))
}

#[allow(dead_code)]
pub async fn find_user_by_id(db: &DbConn, user_id: i64) -> Result<crate::modules::website::entity::website_user::Model> {
    website_user_service::find_by_id(db, user_id).await
}

#[allow(dead_code)]
pub fn get_pending_pay_status() -> i32 {
    STATUS_PENDING_PAY
}
