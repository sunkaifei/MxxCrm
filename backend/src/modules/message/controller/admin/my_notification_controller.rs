//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_user;
use crate::core::web::response::MetaResp;
use crate::modules::message::model::notification::*;
use crate::modules::message::service::notification_service::NotificationService;
use actix_web::{web, HttpRequest, HttpResponse};

pub async fn get_my_notification_list(
    req: HttpRequest,
    state: web::Data<AppState>,
    params: web::Query<NotificationListQuery>,
) -> Result<HttpResponse> {
    let admin_token = get_user(&req).unwrap_or_default();
    let user_id = admin_token.id.unwrap_or(0);
    let db = &state.db;

    let result = NotificationService::get_notification_list(db, user_id, params.into_inner()).await;

    match result {
        Ok(data) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local")))
        }
        Err(e) => {
            log::error!("[通知] 获取列表失败: user_id={}, err={:?}", user_id, e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "获取通知列表失败", "local")))
        }
    }
}

pub async fn mark_as_read_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    request: web::Json<ReadNotificationRequest>,
) -> Result<HttpResponse> {
    let admin_token = get_user(&req).unwrap_or_default();
    let user_id = admin_token.id.unwrap_or(0);
    let db = &state.db;

    let result = NotificationService::mark_as_read(db, user_id, request.id).await;

    match result {
        Ok(_) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(serde_json::json!({"success": true}), "local")))
        }
        Err(_) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "标记已读失败", "local")))
        }
    }
}

pub async fn mark_all_read_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let admin_token = get_user(&req).unwrap_or_default();
    let user_id = admin_token.id.unwrap_or(0);
    let db = &state.db;

    let result = NotificationService::mark_all_read(db, user_id).await;

    match result {
        Ok(count) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(serde_json::json!({"success": true, "count": count}), "local")))
        }
        Err(_) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "操作失败", "local")))
        }
    }
}

pub async fn get_unread_count_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let admin_token = get_user(&req).unwrap_or_default();
    let user_id = admin_token.id.unwrap_or(0);
    let db = &state.db;

    let result = NotificationService::get_unread_count(db, user_id).await;

    match result {
        Ok(count) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(serde_json::json!({"unreadCount": count}), "local")))
        }
        Err(_) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "获取未读数量失败", "local")))
        }
    }
}

pub async fn delete_notification_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    request: web::Json<DeleteNotificationRequest>,
) -> Result<HttpResponse> {
    let admin_token = get_user(&req).unwrap_or_default();
    let user_id = admin_token.id.unwrap_or(0);
    let db = &state.db;

    let result = NotificationService::delete_notification(db, user_id, request.id).await;

    match result {
        Ok(_) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(serde_json::json!({"success": true}), "local")))
        }
        Err(_) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除失败", "local")))
        }
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/my-notification")
            .route("/list", web::get().to(get_my_notification_list))
            .route("/read", web::post().to(mark_as_read_handler))
            .route("/read-all", web::post().to(mark_all_read_handler))
            .route("/unread-count", web::get().to(get_unread_count_handler))
            .route("/delete", web::post().to(delete_notification_handler))
    );
}
