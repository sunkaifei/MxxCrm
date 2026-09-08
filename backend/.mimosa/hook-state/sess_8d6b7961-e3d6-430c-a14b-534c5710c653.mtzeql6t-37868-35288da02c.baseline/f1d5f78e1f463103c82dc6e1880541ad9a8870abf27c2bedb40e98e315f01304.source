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
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::message::model::chat::*;
use crate::modules::message::service::chat_service::{ChatService, ChatServiceError};
use actix_web::{web, HttpRequest, HttpResponse};

pub async fn send_message_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    request: web::Json<SendChatMessageRequest>,
) -> Result<HttpResponse> {
    let admin_token = get_user(&req).unwrap_or_default();
    let user_id = admin_token.id.unwrap_or(0);
    let db = &state.db;

    let result = ChatService::send_message(
        db,
        user_id,
        request.session_id,
        request.receiver_id,
        request.content.clone(),
        request.content_type,
        request.file_url.clone(),
        request.file_name.clone(),
    ).await;

    match result {
        Ok(response) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(response, "local")))
        }
        Err(e) => {
            let msg = match e {
                ChatServiceError::InvalidParameter(msg) => msg,
                ChatServiceError::UserNotFound => "用户不存在".to_string(),
                _ => "发送消息失败".to_string(),
            };
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &msg, "local")))
        }
    }
}

pub async fn get_session_list_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    params: web::Query<GetSessionListParams>,
) -> Result<HttpResponse> {
    let admin_token = get_user(&req).unwrap_or_default();
    let user_id = admin_token.id.unwrap_or(0);
    let db = &state.db;
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(20);

    let result = ChatService::get_session_list(db, user_id, page, page_size).await;

    match result {
        Ok(response) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(response, "local")))
        }
        Err(_) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "获取会话列表失败", "local")))
        }
    }
}

pub async fn get_chat_messages_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    params: web::Query<GetChatMessagesParams>,
) -> Result<HttpResponse> {
    let admin_token = get_user(&req).unwrap_or_default();
    let user_id = admin_token.id.unwrap_or(0);
    let db = &state.db;
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(20);

    let result = ChatService::get_chat_messages(db, user_id, params.session_id, page, page_size).await;

    match result {
        Ok(response) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(response, "local")))
        }
        Err(e) => {
            let msg = match e {
                ChatServiceError::SessionNotFound => "会话不存在".to_string(),
                _ => "获取聊天记录失败".to_string(),
            };
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &msg, "local")))
        }
    }
}

pub async fn mark_read_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    request: web::Json<MarkReadRequest>,
) -> Result<HttpResponse> {
    let admin_token = get_user(&req).unwrap_or_default();
    let user_id = admin_token.id.unwrap_or(0);
    let db = &state.db;

    let result = ChatService::mark_session_read(db, user_id, request.session_id).await;

    match result {
        Ok(_) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(serde_json::json!({"success": true}), "local")))
        }
        Err(_) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "标记已读失败", "local")))
        }
    }
}

pub async fn delete_session_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    request: web::Json<DeleteSessionRequest>,
) -> Result<HttpResponse> {
    let admin_token = get_user(&req).unwrap_or_default();
    let user_id = admin_token.id.unwrap_or(0);
    let db = &state.db;

    let result = ChatService::delete_session(db, user_id, request.session_id).await;

    match result {
        Ok(_) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(serde_json::json!({"success": true}), "local")))
        }
        Err(_) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除会话失败", "local")))
        }
    }
}

pub async fn search_users_handler(
    state: web::Data<AppState>,
    params: web::Query<SearchUserParams>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(10);

    let result = ChatService::search_users(db, params.keyword.clone(), page, page_size).await;

    match result {
        Ok(users) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(users, "local")))
        }
        Err(e) => {
            let msg = match e {
                ChatServiceError::InvalidParameter(msg) => msg,
                _ => "搜索用户失败".to_string(),
            };
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &msg, "local")))
        }
    }
}

pub async fn get_colleague_list_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    params: web::Query<ColleagueListParams>,
) -> Result<HttpResponse> {
    let admin_token = get_user(&req).unwrap_or_default();
    let current_user_id = admin_token.id.unwrap_or(0);
    let db = &state.db;
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(200);

    let result = ChatService::get_colleague_list(db, current_user_id, params.keyword.clone(), page, page_size).await;

    match result {
        Ok(users) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(users, "local")))
        }
        Err(_) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "获取同事列表失败", "local")))
        }
    }
}

pub async fn get_unread_count_handler(req: HttpRequest, state: web::Data<AppState>) -> Result<HttpResponse> {
    let admin_token = get_user(&req).unwrap_or_default();
    let user_id = admin_token.id.unwrap_or(0);
    let db = &state.db;

    let result = ChatService::get_total_unread_count(db, user_id).await;

    match result {
        Ok(count) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(serde_json::json!({"unreadCount": count}), "local")))
        }
        Err(_) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "获取未读数量失败", "local")))
        }
    }
}

pub async fn start_session_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    request: web::Json<StartSessionRequest>,
) -> Result<HttpResponse> {
    let admin_token = get_user(&req).unwrap_or_default();
    let user_id = admin_token.id.unwrap_or(0);
    let db = &state.db;
    let receiver_id = request.receiver_id;

    log::info!("[开始会话(admin)] 用户ID: {}, 对方ID: {}", user_id, receiver_id);

    if receiver_id == 0 {
        log::warn!("[开始会话(admin)] 接收人ID为空");
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "接收人ID不能为空", "local")));
    }

    let result = ChatService::get_or_create_session(db, user_id, receiver_id).await;

    match result {
        Ok(session_id) => {
            log::info!("[开始会话(admin)] 成功: session_id={}", session_id);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(serde_json::json!({"sessionId": session_id}), "local")))
        }
        Err(e) => {
            let msg = match e {
                ChatServiceError::InvalidParameter(msg) => {
                    log::warn!("[开始会话(admin)] 参数错误: {}", msg);
                    msg
                },
                ChatServiceError::DatabaseError(db_err) => {
                    log::error!("[开始会话(admin)] 数据库错误: {:?}", db_err);
                    format!("数据库错误: {}", db_err)
                },
                ChatServiceError::UserNotFound => {
                    log::warn!("[开始会话(admin)] 用户不存在");
                    "用户不存在".to_string()
                },
                ChatServiceError::SessionNotFound => {
                    log::warn!("[开始会话(admin)] 会话不存在");
                    "会话不存在".to_string()
                },
                ChatServiceError::PermissionDenied => {
                    log::warn!("[开始会话(admin)] 权限不足");
                    "权限不足".to_string()
                },
            };
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &msg, "local")))
        }
    }
}

pub async fn recall_message_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    request: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let admin_token = get_user(&req).unwrap_or_default();
    let user_id = admin_token.id.unwrap_or(0);
    let db = &state.db;

    let message_id = request["messageId"].as_i64().unwrap_or(0);
    if message_id == 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "消息ID不能为空", "local")));
    }

    let result = ChatService::recall_message(db, user_id, message_id).await;

    match result {
        Ok(success) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(serde_json::json!({"success": success}), "local")))
        }
        Err(e) => {
            let msg = match e {
                ChatServiceError::InvalidParameter(msg) => msg,
                _ => "撤回消息失败".to_string(),
            };
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &msg, "local")))
        }
    }
}

pub async fn pin_session_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    request: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let admin_token = get_user(&req).unwrap_or_default();
    let user_id = admin_token.id.unwrap_or(0);
    let db = &state.db;

    let session_id = request["sessionId"].as_i64().unwrap_or(0);
    let is_pinned = request["isPinned"].as_bool().unwrap_or(false);

    if session_id == 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "会话ID不能为空", "local")));
    }

    let result = ChatService::toggle_pin(db, user_id, session_id, is_pinned).await;

    match result {
        Ok(_) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(serde_json::json!({"success": true}), "local")))
        }
        Err(_) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "操作失败", "local")))
        }
    }
}

pub async fn mute_session_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    request: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let admin_token = get_user(&req).unwrap_or_default();
    let user_id = admin_token.id.unwrap_or(0);
    let db = &state.db;

    let session_id = request["sessionId"].as_i64().unwrap_or(0);
    let is_muted = request["isMuted"].as_bool().unwrap_or(false);

    if session_id == 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "会话ID不能为空", "local")));
    }

    let result = ChatService::toggle_mute(db, user_id, session_id, is_muted).await;

    match result {
        Ok(_) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(serde_json::json!({"success": true}), "local")))
        }
        Err(_) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "操作失败", "local")))
        }
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/chat")
            .route("/send", web::post().to(send_message_handler))
            .route("/sessions", web::get().to(get_session_list_handler))
            .route("/messages", web::get().to(get_chat_messages_handler))
            .route("/mark-read", web::post().to(mark_read_handler))
            .route("/delete-session", web::post().to(delete_session_handler))
            .route("/search-users", web::get().to(search_users_handler))
            .route("/colleague-list", web::get().to(get_colleague_list_handler))
            .route("/unread-count", web::get().to(get_unread_count_handler))
            .route("/start-session", web::post().to(start_session_handler))
            .route("/recall", web::post().to(recall_message_handler))
            .route("/pin", web::post().to(pin_session_handler))
            .route("/mute", web::post().to(mute_session_handler))
    );
}
