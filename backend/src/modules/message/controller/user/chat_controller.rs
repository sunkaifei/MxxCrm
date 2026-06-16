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
use crate::modules::message::model::chat::*;
use crate::modules::message::service::chat_service::{ChatService, ChatServiceError};
use crate::core::web::response::MetaResp;
use crate::core::kit::global::AppState;
use crate::core::kit::user_auth::get_user_id_from_request;

#[post("/chat/send")]
pub async fn send_message_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    request: web::Json<SendChatMessageRequest>,
) -> HttpResponse {
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let db = &state.db;

    log::info!("[发送聊天消息] 用户ID: {}, 内容长度: {}", user_id, request.content.len());

    let result = ChatService::send_message(
        db,
        user_id,
        request.session_id,
        request.receiver_id,
        request.content.clone(),
    ).await;

    match result {
        Ok(response) => {
            log::info!("[发送聊天消息] 成功: session_id={}, message_id={}", response.session_id, response.message_id);
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(response, "local"))
        }
        Err(e) => {
            log::error!("[发送聊天消息] 失败: {}", format_error(&e));
            match e {
                ChatServiceError::InvalidParameter(msg) => HttpResponse::BadRequest().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &msg, "local")),
                ChatServiceError::UserNotFound => HttpResponse::BadRequest().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "用户不存在", "local")),
                _ => HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "发送消息失败", "local")),
            }
        }
    }
}

#[get("/chat/sessions")]
pub async fn get_session_list_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    params: web::Query<GetSessionListParams>,
) -> HttpResponse {
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let db = &state.db;
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(20);

    log::info!("[会话列表] 用户ID: {}, 页码: {}, 每页: {}", user_id, page, page_size);

    let result = ChatService::get_session_list(db, user_id, page, page_size).await;

    match result {
        Ok(response) => {
            log::info!("[会话列表] 成功: 总数={}", response.total);
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(response, "local"))
        }
        Err(e) => {
            log::error!("[会话列表] 失败: {}", format_error(&e));
            HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "获取会话列表失败", "local"))
        }
    }
}

#[get("/chat/messages")]
pub async fn get_chat_messages_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    params: web::Query<GetChatMessagesParams>,
) -> HttpResponse {
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let db = &state.db;
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(20);

    log::info!("[聊天记录] 用户ID: {}, 会话ID: {}, 页码: {}", user_id, params.session_id, page);

    let result = ChatService::get_chat_messages(db, user_id, params.session_id, page, page_size).await;

    match result {
        Ok(response) => {
            log::info!("[聊天记录] 成功: 总数={}", response.total);
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(response, "local"))
        }
        Err(e) => {
            log::error!("[聊天记录] 失败: {}", format_error(&e));
            match e {
                ChatServiceError::SessionNotFound => HttpResponse::NotFound().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "会话不存在", "local")),
                _ => HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "获取聊天记录失败", "local")),
            }
        }
    }
}

#[post("/chat/mark-read")]
pub async fn mark_read_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    request: web::Json<MarkReadRequest>,
) -> HttpResponse {
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let db = &state.db;

    log::info!("[标记已读] 用户ID: {}, 会话ID: {}", user_id, request.session_id);

    let result = ChatService::mark_session_read(db, user_id, request.session_id).await;

    match result {
        Ok(_) => {
            log::info!("[标记已读] 成功");
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(serde_json::json!({"success": true}), "local"))
        }
        Err(e) => {
            log::error!("[标记已读] 失败: {}", format_error(&e));
            HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "标记已读失败", "local"))
        }
    }
}

#[post("/chat/delete-session")]
pub async fn delete_session_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    request: web::Json<DeleteSessionRequest>,
) -> HttpResponse {
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let db = &state.db;

    log::info!("[删除会话] 用户ID: {}, 会话ID: {}", user_id, request.session_id);

    let result = ChatService::delete_session(db, user_id, request.session_id).await;

    match result {
        Ok(_) => {
            log::info!("[删除会话] 成功");
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(serde_json::json!({"success": true}), "local"))
        }
        Err(e) => {
            log::error!("[删除会话] 失败: {}", format_error(&e));
            HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除会话失败", "local"))
        }
    }
}

#[get("/chat/search-users")]
pub async fn search_users_handler(
    state: web::Data<AppState>,
    params: web::Query<SearchUserParams>,
) -> HttpResponse {
    let db = &state.db;
    log::info!("[用户搜索] 关键词: {}", params.keyword);

    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(10);

    let result = ChatService::search_users(db, params.keyword.clone(), page, page_size).await;

    match result {
        Ok(users) => {
            log::info!("[用户搜索] 成功: 找到{}个用户", users.len());
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(users, "local"))
        }
        Err(e) => {
            log::error!("[用户搜索] 失败: {}", format_error(&e));
            match e {
                ChatServiceError::InvalidParameter(msg) => HttpResponse::BadRequest().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &msg, "local")),
                _ => HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "搜索用户失败", "local")),
            }
        }
    }
}

#[get("/chat/unread-count")]
pub async fn get_unread_count_handler(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let db = &state.db;

    log::info!("[未读数量] 用户ID: {}", user_id);

    let result = ChatService::get_total_unread_count(db, user_id).await;

    match result {
        Ok(count) => {
            log::info!("[未读数量] 成功: {}", count);
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(serde_json::json!({"unread_count": count}), "local"))
        }
        Err(e) => {
            log::error!("[未读数量] 失败: {}", format_error(&e));
            HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "获取未读数量失败", "local"))
        }
    }
}

#[post("/chat/start-session")]
pub async fn start_session_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    request: web::Json<serde_json::Value>,
) -> HttpResponse {
    let user_id = match get_user_id_from_request(&req).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let db = &state.db;

    let receiver_id = request["receiver_id"].as_i64().unwrap_or(0);
    if receiver_id == 0 {
        return HttpResponse::BadRequest().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "接收人ID不能为空", "local"));
    }

    log::info!("[开始会话] 用户ID: {}, 对方ID: {}", user_id, receiver_id);

    let result = ChatService::get_or_create_session(db, user_id, receiver_id).await;

    match result {
        Ok(session_id) => {
            log::info!("[开始会话] 成功: session_id={}", session_id);
            HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(serde_json::json!({"session_id": session_id}), "local"))
        }
        Err(e) => {
            log::error!("[开始会话] 失败: {}", format_error(&e));
            match e {
                ChatServiceError::InvalidParameter(msg) => HttpResponse::BadRequest().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &msg, "local")),
                _ => HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "创建会话失败", "local")),
            }
        }
    }
}

fn format_error(e: &ChatServiceError) -> String {
    match e {
        ChatServiceError::InvalidParameter(msg) => format!("参数错误: {}", msg),
        ChatServiceError::UserNotFound => "用户不存在".to_string(),
        ChatServiceError::SessionNotFound => "会话不存在".to_string(),
        ChatServiceError::PermissionDenied => "无权限".to_string(),
        ChatServiceError::DatabaseError(err) => format!("数据库错误: {}", err),
    }
}
