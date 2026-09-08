//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 交付通知 Service（邮件 / 站内信）
//!
//! 用于在虚拟商品交付后，通过邮件及站内信通知客户与订单负责人。
//!

use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set};

use crate::core::errors::error::{Error, Result};
use crate::modules::crm::entity::customer::Entity as CustomerEntity;
use crate::modules::sale::entity::order_delivery::{Column, Entity};
use crate::modules::sale::entity::order::Entity as OrderEntity;
use crate::modules::system::model::mail::SendMailRequest;
use crate::modules::system::service::mail_service;
use crate::modules::system::service::integration_config_service;

/// 交付状态：1=待发送
const DELIVERY_STATUS_PENDING: i32 = 1;
/// 交付状态：2=已发送
const DELIVERY_STATUS_SENT: i32 = 2;

/// 第三方接口配置编码：SMTP 邮件
const INTEGRATION_CODE_SMTP_EMAIL: &str = "smtp_email";

/// 检查 SMTP 邮件是否已配置且已启用
///
/// 用于在发送交付通知前判断是否需要尝试发送邮件：
/// - 若 integration_config 中存在 code="smtp_email" 且 enabled=1，认为已配置
/// - 否则视为未配置，跳过邮件发送
async fn is_smtp_configured(db: &DbConn) -> bool {
    match integration_config_service::get_by_code(db, INTEGRATION_CODE_SMTP_EMAIL).await {
        Ok(Some(cfg)) => cfg.enabled.unwrap_or(0) == 1,
        _ => false,
    }
}

/// 发送交付通知给客户
///
/// 流程：
/// 1. 查询交付记录（order_delivery），不存在则报错
/// 2. 关联订单 → 查客户邮箱
/// 3. 调用 mail_service 发送邮件（标题：`虚拟商品交付通知 - [商品名]`）
/// 4. 同时发送站内信给订单负责人
/// 5. 更新交付记录状态为"已发送"
pub async fn notify_customer_delivery(db: &DbConn, delivery_id: i64) -> Result<i64> {
    // 1. 查询交付记录
    let delivery = Entity::find()
        .filter(Column::Id.eq(delivery_id))
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("交付记录不存在"))?;

    let order_id = delivery.order_id
        .ok_or_else(|| Error::from("交付记录缺少订单ID"))?;
    let product_name = delivery.product_name.clone().unwrap_or_default();
    let delivery_no = delivery.delivery_no.clone().unwrap_or_default();

    // 2. 关联订单 → 客户邮箱
    let order = OrderEntity::find_by_id(order_id)
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("关联订单不存在"))?;

    let order_no = order.order_no.clone().unwrap_or_default();
    let customer_id = order.customer_id.unwrap_or(0);
    let owner_user_id = order.owner_user_id.unwrap_or(0);

    // 查客户邮箱
    let customer_email = if customer_id > 0 {
        CustomerEntity::find_by_id(customer_id)
            .one(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?
            .and_then(|c| c.personal_email)
            .filter(|e| !e.is_empty())
    } else {
        None
    };

    // 解密交付内容（卡密/账号密码等），用于邮件内容
    let decrypted_card_key = delivery.card_key
        .as_deref()
        .map(crate::utils::encryption_utils::decrypt_card);
    let decrypted_account_pwd = delivery.account_password
        .as_deref()
        .map(crate::utils::encryption_utils::decrypt_card);

    // 3. 构造邮件内容
    let subject = format!("虚拟商品交付通知 - {}", product_name);
    let mut body_parts: Vec<String> = Vec::new();
    body_parts.push(format!("<p>尊敬的客户，您好：</p>"));
    body_parts.push(format!("<p>您的订单 <strong>{}</strong> 中的虚拟商品 <strong>{}</strong> 已完成交付，详情如下：</p>", order_no, product_name));
    body_parts.push(format!("<p>交付单号：<strong>{}</strong></p>", delivery_no));
    if let Some(url) = &delivery.download_url {
        body_parts.push(format!("<p>下载链接：<a href=\"{}\">{}</a></p>", url, url));
    }
    if let Some(key) = &decrypted_card_key {
        body_parts.push(format!("<p>卡密/激活码：<strong>{}</strong></p>", key));
    }
    if let Some(name) = &delivery.account_name {
        body_parts.push(format!("<p>账号：<strong>{}</strong></p>", name));
    }
    if let Some(pwd) = &decrypted_account_pwd {
        body_parts.push(format!("<p>密码：<strong>{}</strong></p>", pwd));
    }
    if let Some(extra) = &delivery.extra_content {
        body_parts.push(format!("<p>补充说明：{}</p>", extra));
    }
    body_parts.push(format!("<p style=\"color:#999;margin-top:24px;\">此邮件由系统自动发送，请勿直接回复。</p>"));
    let body = body_parts.join("\n");

    // 发送邮件（若 SMTP 已配置且客户邮箱存在）
    let mut mail_log_id: Option<i64> = None;
    let smtp_configured = is_smtp_configured(db).await;
    if !smtp_configured {
        log::info!("[delivery_notification] 交付{} SMTP 邮件未配置，跳过邮件发送，仅发站内信", delivery_id);
    }
    if smtp_configured {
        if let Some(email) = customer_email.clone() {
            let req = SendMailRequest {
                customer_id: Some(customer_id),
                to_emails: vec![email],
                cc_emails: None,
                subject: Some(subject.clone()),
                body: Some(body.clone()),
                doc_url: None,
                contact_ids: None,
            };
            match mail_service::send_mail(db, req, None, Some("系统".to_string())).await {
                Ok(id) => mail_log_id = Some(id),
                Err(e) => log::warn!("[delivery_notification] 交付{}邮件发送失败：{}", delivery_id, e),
            }
        }
    }

    // 4. 站内信通知订单负责人
    if owner_user_id > 0 {
        let title = format!("虚拟商品交付通知 [{}]", delivery_no);
        let content = format!(
            "订单 [{}] 的虚拟商品 [{}] 已通知客户交付，交付单号 [{}]。",
            order_no, product_name, delivery_no
        );
        let _ = crate::modules::message::service::notification_service::NotificationService::send_system_notification(
            db, owner_user_id, title, content, 3, None
        ).await;
    }

    // 5. 更新交付状态为"已发送"
    let now = chrono::Local::now().naive_local();
    let mut active: crate::modules::sale::entity::order_delivery::ActiveModel = Default::default();
    active.id = Set(delivery_id);
    active.status = Set(Some(DELIVERY_STATUS_SENT));
    active.sent_time = Set(Some(now));
    let _ = active.update(db).await;

    Ok(mail_log_id.unwrap_or(0))
}

/// 批量通知待发送的交付记录（status=1）
///
/// 返回成功通知的记录数。
pub async fn batch_notify_pending(db: &DbConn) -> Result<i64> {
    // 分批查询所有待发送记录
    let page_size: u64 = 50;
    let paginator = Entity::find()
        .filter(Column::Status.eq(DELIVERY_STATUS_PENDING))
        .filter(Column::Deleted.eq(0))
        .order_by_asc(Column::Id)
        .paginate(db, page_size);

    let total = paginator.num_items().await
        .map_err(|e| Error::from(e.to_string()))? as i64;

    let mut success: i64 = 0;
    let total_pages = ((total as u64) + page_size - 1) / page_size;
    for page_idx in 0..total_pages {
        let rows = paginator.fetch_page(page_idx).await
            .map_err(|e| Error::from(e.to_string()))?;
        for row in rows {
            let id = row.id;
            // 单条失败不影响整体批次，记录日志后继续
            match notify_customer_delivery(db, id).await {
                Ok(_) => success += 1,
                Err(e) => log::warn!("[delivery_notification] 批量通知交付{}失败：{}", id, e),
            }
        }
    }

    Ok(success)
}
