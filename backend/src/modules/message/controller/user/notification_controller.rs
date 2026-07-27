//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{get, post, web, HttpResponse, HttpRequest};
use crate::modules::message::model::notification::*;
use crate::modules::message::service::notification_service::{NotificationService, NotificationServiceError};
use crate::core::web::response::MetaResp;
use crate::core::kit::global::AppState;
use crate::core::kit::user_auth::get_user_id_from_request;

#[get("/notification/list")]
pub async fn get_notification_list_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    params: web::Query<NotificationListQuery>,
) -> HttpResponse {
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let db = &state.db;

    log::info!("[通知列表] 用户ID: {}", user_id);

    let result = NotificationService::get_notification_list(db, user_id, params.into_inner()).await;

    match result {
        Ok(response) => {
            log::info!("[通知列表] 成功: 总数={}", response.total);
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(response, "local"))
        }
        Err(e) => {
            log::error!("[通知列表] 失败: {}", format_error(&e));
            HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "获取通知列表失败", "local"))
        }
    }
}

#[post("/notification/read")]
pub async fn mark_read_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    request: web::Json<ReadNotificationRequest>,
) -> HttpResponse {
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let db = &state.db;

    log::info!("[标记已读] 用户ID: {}, 通知ID: {}", user_id, request.id);

    let result = NotificationService::mark_as_read(db, user_id, request.id).await;

    match result {
        Ok(_) => {
            log::info!("[标记已读] 成功");
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(serde_json::json!({"success": true}), "local"))
        }
        Err(e) => {
            log::error!("[标记已读] 失败: {}", format_error(&e));
            match e {
                NotificationServiceError::InvalidParameter(msg) => HttpResponse::BadRequest().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &msg, "local")),
                _ => HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "标记已读失败", "local")),
            }
        }
    }
}

#[post("/notification/read-all")]
pub async fn mark_all_read_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> HttpResponse {
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let db = &state.db;

    log::info!("[全部已读] 用户ID: {}", user_id);

    let result = NotificationService::mark_all_read(db, user_id).await;

    match result {
        Ok(count) => {
            log::info!("[全部已读] 成功: {}条", count);
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(serde_json::json!({"success": true, "count": count}), "local"))
        }
        Err(e) => {
            log::error!("[全部已读] 失败: {}", format_error(&e));
            HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "全部已读失败", "local"))
        }
    }
}

#[get("/notification/unread-count")]
pub async fn get_unread_count_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> HttpResponse {
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let db = &state.db;

    log::info!("[未读数量] 用户ID: {}", user_id);

    let result = NotificationService::get_unread_count(db, user_id).await;

    match result {
        Ok(count) => {
            log::info!("[未读数量] 成功: {}", count);
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(serde_json::json!({"unreadCount": count}), "local"))
        }
        Err(e) => {
            log::error!("[未读数量] 失败: {}", format_error(&e));
            HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "获取未读数量失败", "local"))
        }
    }
}

#[post("/notification/delete")]
pub async fn delete_notification_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    request: web::Json<DeleteNotificationRequest>,
) -> HttpResponse {
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let db = &state.db;

    log::info!("[删除通知] 用户ID: {}, 通知ID: {}", user_id, request.id);

    let result = NotificationService::delete_notification(db, user_id, request.id).await;

    match result {
        Ok(_) => {
            log::info!("[删除通知] 成功");
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(serde_json::json!({"success": true}), "local"))
        }
        Err(e) => {
            log::error!("[删除通知] 失败: {}", format_error(&e));
            match e {
                NotificationServiceError::InvalidParameter(msg) => HttpResponse::BadRequest().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &msg, "local")),
                _ => HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除通知失败", "local")),
            }
        }
    }
}

fn format_error(e: &NotificationServiceError) -> String {
    match e {
        NotificationServiceError::InvalidParameter(msg) => format!("参数错误: {}", msg),
        NotificationServiceError::DatabaseError(err) => format!("数据库错误: {}", err),
    }
}
