//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究。
//!

use std::time::{Duration, Instant};

use actix_web::{web, HttpRequest, HttpResponse};
use actix_ws::{Message, MessageStream, Session};
use futures_util::StreamExt;
use tokio::sync::mpsc;
use tokio::time::interval;

use crate::core::kit::config;
use crate::core::kit::jwt_util::JWTToken;
use crate::modules::message::websocket::registry::ConnectionRegistry;

/// 心跳间隔（秒）
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
/// 客户端无心跳超时（秒）
const CLIENT_TIMEOUT: Duration = Duration::from_secs(90);

/// 从 query 参数中解析 JWT token，返回 user_id
fn extract_user_id_from_query(req: &HttpRequest) -> Option<i64> {
    let token_str = web::Query::<std::collections::HashMap<String, String>>::from_query(req.query_string())
        .ok()?
        .get("token")
        .cloned()?;

    if token_str.is_empty() {
        return None;
    }

    // 先用 admin secret 校验
    let admin_secret = config::section::<String>("server", "jwt_secret_admin", "".to_string());
    if let Ok(claims) = JWTToken::verify(&admin_secret, &token_str) {
        return claims.id;
    }

    // 再用 user secret 校验
    let user_secret = config::section::<String>("server", "jwt_secret_user", "".to_string());
    if let Ok(claims) = JWTToken::verify(&user_secret, &token_str) {
        return claims.id;
    }

    None
}

/// WebSocket 握手处理函数
/// URL: ws://host/ws/message?token=xxx
pub async fn ws_handler(req: HttpRequest, body: web::Payload) -> Result<HttpResponse, actix_web::Error> {
    // 从 query 参数解析 token，浏览器 WebSocket 不支持自定义 Header
    let user_id = match extract_user_id_from_query(&req) {
        Some(id) if id > 0 => id,
        _ => {
            log::warn!("[WebSocket] 握手失败：未认证");
            return Ok(HttpResponse::Unauthorized().finish());
        }
    };

    let (response, mut session, mut msg_stream) = match actix_ws::handle(&req, body) {
        Ok(tuple) => tuple,
        Err(e) => {
            log::warn!("[WebSocket] 握手失败：{}", e);
            return Ok(HttpResponse::BadRequest().finish());
        }
    };

    log::info!("[WebSocket] 用户 {} 握手成功", user_id);

    // 创建向客户端推送消息的通道
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    // 注册到全局连接管理器
    let registry = ConnectionRegistry::global();
    registry.register(user_id, tx);

    // 拆出 session 用于推送任务
    let mut push_session = session.clone();

    // 任务1：从通道读取推送消息，写到 WebSocket
    actix_web::rt::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if push_session.text(msg).await.is_err() {
                break;
            }
        }
    });

    // 任务2：处理客户端消息 + 心跳检测
    actix_web::rt::spawn(async move {
        let mut last_heartbeat = Instant::now();
        let mut tick = interval(HEARTBEAT_INTERVAL);

        loop {
            tokio::select! {
                // 客户端消息
                msg = msg_stream.next() => {
                    match msg {
                        Some(Ok(Message::Ping(p))) => {
                            last_heartbeat = Instant::now();
                            let _ = session.pong(&p).await;
                        }
                        Some(Ok(Message::Pong(_))) => {
                            last_heartbeat = Instant::now();
                        }
                        Some(Ok(Message::Text(text))) => {
                            log::debug!("[WebSocket] 收到用户 {} 消息: {}", user_id, text);
                        }
                        Some(Ok(Message::Close(reason))) => {
                            log::info!("[WebSocket] 用户 {} 主动关闭连接: {:?}", user_id, reason);
                            let _ = session.close(reason).await;
                            break;
                        }
                        Some(Err(e)) => {
                            log::warn!("[WebSocket] 用户 {} 协议错误: {}", user_id, e);
                            break;
                        }
                        None => break,
                        _ => {}
                    }
                }
                // 心跳定时器
                _ = tick.tick() => {
                    if Instant::now().duration_since(last_heartbeat) > CLIENT_TIMEOUT {
                        log::warn!("[WebSocket] 用户 {} 心跳超时，关闭连接", user_id);
                        break;
                    }
                    // 发送 Ping
                    if session.ping(b"").await.is_err() {
                        break;
                    }
                }
            }
        }

        // 任务结束，注销连接
        let registry = ConnectionRegistry::global();
        registry.unregister(user_id);
        log::info!("[WebSocket] 用户 {} 连接已断开", user_id);
    });

    Ok(response)
}
