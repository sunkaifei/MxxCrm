//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 交付通知控制器
//!
//! ## 路由表
//!
//! | 方法   | 路径                                       | 权限码                | handler      | 说明                |
//! |--------|-------------------------------------------|----------------------|--------------|---------------------|
//! | POST   | /sale/delivery-notification/notify        | sale:delivery:save   | notify       | 通知指定交付记录     |
//! | POST   | /sale/delivery-notification/batch-notify  | sale:delivery:save   | batch_notify | 批量通知待发送记录   |
//!

use actix_web::{web, HttpResponse};
use crate::core::kit::global::AppState;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::sale::service::delivery_notification_service;

/// 通知指定交付记录（邮件 + 站内信）
pub async fn notify(
    state: web::Data<AppState>,
    form_data: web::Json<serde_json::Value>,
) -> HttpResponse {
    let db = &state.db;
    let id = form_data.get("id").and_then(|v| v.as_i64());
    if id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "交付记录ID不能为空", "local"));
    }
    match delivery_notification_service::notify_customer_delivery(db, id.unwrap()).await {
        Ok(log_id) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(serde_json::json!({ "success": true, "mailLogId": log_id }), "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 批量通知待发送的交付记录
pub async fn batch_notify(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match delivery_notification_service::batch_notify_pending(db).await {
        Ok(count) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(serde_json::json!({ "success": true, "notifiedCount": count }), "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 注册交付通知模块所有路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sale/delivery-notification")
            .route("/notify", web::post().to(notify).wrap(require_permission("sale:delivery:save")))
            .route("/batch-notify", web::post().to(batch_notify).wrap(require_permission("sale:delivery:save"))),
    );
}
