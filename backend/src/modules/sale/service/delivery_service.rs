//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 虚拟商品交付 Service
//!

use rust_decimal::prelude::ToPrimitive;
use sea_orm::{ColumnTrait, Condition, ConnectionTrait, DbConn, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, TransactionTrait};

use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::sale::entity::order_delivery::{Column as DeliveryColumn, Entity as DeliveryEntity};
use crate::modules::sale::model::order::{
    self, ORDER_STATUS_PARTIAL_DELIVERED, ORDER_STATUS_PENDING_DELIVERY,
    FULFILLMENT_AUTO_DELIVER, PRODUCT_TYPE_PHYSICAL, PRODUCT_TYPE_SUBSCRIPTION,
    PRODUCT_TYPE_VIRTUAL, OrderItemModel, OrderModel,
};
use crate::modules::sale::model::order_item::OrderItemModel as OrderItemModelById;
use crate::modules::sale::model::order_delivery::{
    mask_card_key, DeliveryDetailVO, DeliveryListVO, DeliveryListQuery, DeliveryModel,
    DeliverySaveDTO, DeliverySaveRequest, delivery_method_name, delivery_status_name,
};
use crate::modules::sale::model::card_pool::CardPoolModel;

/// 列表查询
pub async fn get_list(
    db: &DbConn, query: &DeliveryListQuery
) -> Result<ResultPage<Vec<DeliveryListVO>>> {
    let page = query.page_num.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).max(1);

    let mut cond = Condition::all();
    cond = cond.add(DeliveryColumn::Deleted.eq(0));
    if let Some(oid) = query.order_id { cond = cond.add(DeliveryColumn::OrderId.eq(oid)); }
    if let Some(cid) = query.customer_id { cond = cond.add(DeliveryColumn::CustomerId.eq(cid)); }
    if let Some(s) = query.status { cond = cond.add(DeliveryColumn::Status.eq(s)); }
    if let Some(m) = query.delivery_method { cond = cond.add(DeliveryColumn::DeliveryMethod.eq(m)); }

    let paginator = DeliveryEntity::find()
        .filter(cond)
        .order_by_desc(DeliveryColumn::Id)
        .paginate(db, page_size as u64);

    let total = paginator.num_items().await.map_err(|e| Error::from(e.to_string()))? as i64;
    let rows = paginator.fetch_page((page - 1) as u64).await
        .map_err(|e| Error::from(e.to_string()))?;

    let items: Vec<DeliveryListVO> = rows.into_iter().map(|m| DeliveryListVO {
        id: m.id,
        delivery_no: m.delivery_no,
        order_id: m.order_id,
        order_item_id: m.order_item_id,
        customer_id: m.customer_id,
        product_id: m.product_id,
        product_name: m.product_name,
        delivery_method: m.delivery_method,
        delivery_method_name: m.delivery_method.map(|v| delivery_method_name(v).to_string()),
        card_key_masked: m.card_key.as_deref().map(mask_card_key),
        download_url: m.download_url,
        account_name: m.account_name,
        extra_content: m.extra_content,
        status: m.status,
        status_name: m.status.map(|v| delivery_status_name(v).to_string()),
        deliver_type: m.deliver_type,
        sent_time: m.sent_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        received_time: m.received_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        remark: m.remark,
        create_time: m.create_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
    }).collect();

    Ok(ResultPage::new(items, total, page, page_size))
}

/// 详情（脱敏版）
pub async fn get_detail(db: &DbConn, id: i64) -> Result<DeliveryDetailVO> {
    let m = DeliveryModel::find_by_id(db, id).await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("交付记录不存在"))?;
    Ok(DeliveryDetailVO {
        id: m.id,
        delivery_no: m.delivery_no,
        order_id: m.order_id,
        order_item_id: m.order_item_id,
        customer_id: m.customer_id,
        product_id: m.product_id,
        product_name: m.product_name,
        delivery_method: m.delivery_method,
        delivery_method_name: m.delivery_method.map(|v| delivery_method_name(v).to_string()),
        card_key_masked: m.card_key.as_deref().map(mask_card_key),
        download_url: m.download_url,
        account_name: m.account_name,
        extra_content: m.extra_content,
        status: m.status,
        status_name: m.status.map(|v| delivery_status_name(v).to_string()),
        deliver_type: m.deliver_type,
        sent_time: m.sent_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        received_time: m.received_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        expire_time: m.expire_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        remark: m.remark,
        create_by: m.create_by,
        create_time: m.create_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        update_time: m.update_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
    })
}

/// 查看完整卡密
pub async fn view_full(db: &DbConn, id: i64) -> Result<serde_json::Value> {
    let m = DeliveryModel::find_by_id(db, id).await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("交付记录不存在"))?;
    // 解密卡密
    let decrypted_key = m.card_key.as_deref()
        .map(|k| crate::utils::encryption_utils::decrypt_card(k));
    Ok(serde_json::json!({
        "id": m.id,
        "deliveryNo": m.delivery_no,
        "cardKey": decrypted_key,
        "downloadUrl": m.download_url,
        "accountName": m.account_name,
        "accountPassword": m.account_password,
        "extraContent": m.extra_content,
    }))
}

/// 手动录入交付记录
pub async fn create(
    db: &DbConn, req: DeliverySaveRequest, user_id: i64
) -> Result<i64> {
    let order_id = req.order_id.ok_or_else(|| Error::from("订单ID不能为空"))?;
    let order_item_id = req.order_item_id.ok_or_else(|| Error::from("订单明细ID不能为空"))?;
    let _delivery_method = req.delivery_method.ok_or_else(|| Error::from("交付方式不能为空"))?;

    let order = OrderModel::find_by_id(db, order_id).await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("订单不存在"))?;

    let order_item = OrderItemModelById::find_by_id(db, order_item_id).await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("订单明细不存在"))?;
    let product_type = order_item.product_type.unwrap_or(PRODUCT_TYPE_PHYSICAL);
    if product_type == PRODUCT_TYPE_PHYSICAL {
        return Err(Error::from("实物商品不能创建虚拟交付记录"));
    }

    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;

    let date_prefix = format!("VD{}", chrono::Local::now().format("%Y%m%d"));
    let max_seq = DeliveryModel::get_max_delivery_no_today(&txn, &date_prefix).await
        .map_err(|e| Error::from(e.to_string()))?;
    let seq = max_seq.unwrap_or(0) + 1;
    let delivery_no = format!("{}{:04}", date_prefix, seq);

    let mut dto: DeliverySaveDTO = req.clone().into();
    dto.delivery_no = Some(delivery_no);
    dto.customer_id = order.customer_id;
    dto.product_id = order_item.product_id;
    dto.product_name = order_item.product_name.clone();
    dto.status = Some(2);
    dto.deliver_type = Some(2);
    dto.create_by = Some(user_id);

    let delivery_id = DeliveryModel::insert(&txn, &dto).await
        .map_err(|e| Error::from(e.to_string()))?;

    refresh_order_delivery_status(&txn, order_id).await?;

    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;

    Ok(delivery_id)
}

/// 自动交付服务
pub async fn trigger_auto_delivery<C: ConnectionTrait>(
    db: &C, order_id: i64
) -> Result<Vec<i64>> {
    let items = OrderItemModel::find_by_order_id(db, order_id).await
        .map_err(|e| Error::from(e.to_string()))?;
    let mut delivery_ids = Vec::new();

    for item in items.iter() {
        let item_id = item.id;
        let product_id = match item.product_id { Some(id) => id, None => continue };
        let product_type = item.product_type.unwrap_or(PRODUCT_TYPE_PHYSICAL);

        if item.fulfillment_type.unwrap_or(1) != FULFILLMENT_AUTO_DELIVER { continue; }
        if !matches!(product_type, PRODUCT_TYPE_VIRTUAL | PRODUCT_TYPE_SUBSCRIPTION) { continue; }

        let needed: i64 = item.quantity.unwrap_or(rust_decimal::Decimal::ZERO).to_i64().unwrap_or(0);
        for _ in 0..needed {
            let card = CardPoolModel::lock_one(db, product_id).await
                .map_err(|e| Error::from(e.to_string()))?;
            if let Some(c) = card {
                let card_id = c.id;
                let date_prefix = format!("VD{}", chrono::Local::now().format("%Y%m%d"));
                let max_seq = DeliveryModel::get_max_delivery_no_today(db, &date_prefix).await
                    .map_err(|e| Error::from(e.to_string()))?;
                let seq = max_seq.unwrap_or(0) + 1;
                let delivery_no = format!("{}{:04}", date_prefix, seq);

                let dto = DeliverySaveDTO {
                    delivery_no: Some(delivery_no),
                    order_id: Some(order_id),
                    order_item_id: Some(item_id),
                    customer_id: None,
                    product_id: Some(product_id),
                    product_name: item.product_name.clone(),
                    delivery_method: Some(1),
                    card_key: c.card_key.clone(),
                    download_url: None,
                    account_name: None,
                    account_password: c.card_password.clone(),
                    extra_content: None,
                    status: Some(2),
                    deliver_type: Some(1),
                    sent_time: Some(chrono::Local::now().naive_local()),
                    card_pool_id: Some(card_id),
                    remark: Some("系统自动交付".to_string()),
                    create_by: None,
                };
                let delivery_id = DeliveryModel::insert(db, &dto).await
                    .map_err(|e| Error::from(e.to_string()))?;
                CardPoolModel::mark_sold(db, card_id, order_id).await
                    .map_err(|e| Error::from(e.to_string()))?;
                delivery_ids.push(delivery_id);
            }
        }
    }

    refresh_order_delivery_status(db, order_id).await?;

    Ok(delivery_ids)
}

/// 推进订单状态
pub async fn refresh_order_delivery_status<C: ConnectionTrait>(
    db: &C, order_id: i64
) -> Result<()> {
    let items = OrderItemModel::find_by_order_id(db, order_id).await
        .map_err(|e| Error::from(e.to_string()))?;
    let mut all_delivered = true;
    let mut any_delivered = false;

    for item in items.iter() {
        let product_type = item.product_type.unwrap_or(PRODUCT_TYPE_PHYSICAL);
        if !matches!(product_type, PRODUCT_TYPE_VIRTUAL | PRODUCT_TYPE_SUBSCRIPTION) { continue; }
        let item_id = item.id;
        let delivered = DeliveryModel::count_by_item(db, item_id).await
            .map_err(|e| Error::from(e.to_string()))?;
        let needed = item.quantity.unwrap_or(rust_decimal::Decimal::ZERO).to_i64().unwrap_or(0);
        if delivered >= needed && needed > 0 { any_delivered = true; }
        else { all_delivered = false; }
    }

    let order = OrderModel::find_by_id(db, order_id).await
        .map_err(|e| Error::from(e.to_string()))?;
    if let Some(o) = order {
        let current = o.order_status.unwrap_or(0);
        let new_status = if all_delivered { 8 }
                         else if any_delivered { ORDER_STATUS_PARTIAL_DELIVERED }
                         else { ORDER_STATUS_PENDING_DELIVERY };
        if matches!(current, 0..=3 | 12 | 14) && current != new_status {
            OrderModel::update_status(db, order_id, new_status, None, None).await
                .map_err(|e| Error::from(e.to_string()))?;
        }
    }

    Ok(())
}

/// 修改交付状态
pub async fn update_status(db: &DbConn, id: i64, status: i32) -> Result<i64> {
    DeliveryModel::update_status(db, id, status).await
        .map_err(|e| Error::from(e.to_string()))
}

/// 重发通知：通过站内信通知订单负责人
pub async fn resend_notification(db: &DbConn, id: i64) -> Result<()> {
    let m = DeliveryModel::find_by_id(db, id).await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("交付记录不存在"))?;

    // 查询订单负责人
    let order_id = m.order_id.unwrap_or(0);
    if order_id > 0 {
        let order = OrderModel::find_by_id(db, order_id).await
            .map_err(|e| Error::from(e.to_string()))?;
        if let Some(o) = order {
            let owner_id = o.owner_user_id.unwrap_or(0);
            let order_no = o.order_no.unwrap_or_default();
            let product_name = m.product_name.unwrap_or_default();
            let delivery_no = m.delivery_no.unwrap_or_default();
            if owner_id > 0 {
                let title = format!("虚拟商品交付提醒 [{}]", delivery_no);
                let content = format!(
                    "订单 [{}] 的虚拟商品 [{}] 交付记录已{}，请及时跟进客户确认。",
                    order_no,
                    product_name,
                    delivery_status_name(m.status.unwrap_or(1))
                );
                let _ = crate::modules::message::service::notification_service::NotificationService::send_system_notification(
                    db, owner_id, title, content, 3, None
                ).await;
            }
        }
    }
    Ok(())
}

// 防止未使用导入告警
#[allow(unused_imports)]
use order as _order_mod;
