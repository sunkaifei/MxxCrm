//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 操作日志中间件：拦截后台 POST/PUT/DELETE 请求，自动写 mxx_system_log。
//!
//! 设计要点：
//! - 排除登录/注销/验证码/上传/日志自身等无需记录或会自循环的接口
//! - 通过 JWT 解析 oper_name、operator_type（admin=1, user=2）
//! - 读取请求体和响应体并记录（脱敏后截断到 2000 字符）
//! - 敏感字段（password/token/secret 等）值自动替换为 "***"
//! - 异步落库失败仅记 log，绝不返回错误打断主流程
//! - 使用 struct + impl Transform 形式，避免 from_fn 在泛型 B 上的限制
//!
//! 注册位置：在 `admin_routes::configure_routes` 中通过 `.wrap(OperationLog)` 挂载。

use std::future::{ready, Ready};
use std::rc::Rc;

use actix_web::body::{BoxBody, MessageBody};
use actix_web::dev::{Payload, Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::http::Method;
use actix_web::web;
use actix_web::Error;
use futures_util::future::LocalBoxFuture;
use futures_util::StreamExt;

use crate::core::kit::config;
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::kit::sensitive;
use crate::modules::system::entity::system_log as log_entity;
use crate::SNOWFLAKE;
use sea_orm::{EntityTrait, Set};

/// 需要记录的操作类型
const BUSINESS_TYPE_OTHER: i32 = 0;
const BUSINESS_TYPE_INSERT: i32 = 1;
const BUSINESS_TYPE_UPDATE: i32 = 2;
const BUSINESS_TYPE_DELETE: i32 = 3;

/// 中间件主结构体（无字段，纯行为）
#[derive(Debug, Clone, Default)]
pub struct OperationLog;

impl<S, B> Transform<S, ServiceRequest> for OperationLog
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    // 注意：这里把响应体统一成 BoxBody，因为我们需要先 to_bytes 读出原始 body 再重新放回去，
    // 原本的泛型 B 在读取后就无法复原。BoxBody 是 actix-web 4 中通用的 body 类型，
    // 外层 actix 接受 BoxBody 没有问题。
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = OperationLogMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(OperationLogMiddleware { service: Rc::new(service) }))
    }
}

pub struct OperationLogMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for OperationLogMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let method = req.method().clone();
        let path = req.path().to_string();

        // ---- 过滤规则 ----
        // 1. 只记录写操作；GET / HEAD / OPTIONS 不记
        if !matches!(method, Method::POST | Method::PUT | Method::DELETE) {
            // 直接调用下游，但需要把 Response 类型统一成 BoxBody
            let svc = self.service.clone();
            return Box::pin(async move {
                let res = svc.call(req).await?;
                Ok(res.map_into_boxed_body())
            });
        }
        // 2. 只处理 admin 后台路径
        if !path.starts_with("/api/system/") {
            let svc = self.service.clone();
            return Box::pin(async move {
                let res = svc.call(req).await?;
                Ok(res.map_into_boxed_body())
            });
        }
        // 3. 排除名单（登录/注销/验证码/上传/日志自身）
        if is_excluded_path(&path) {
            let svc = self.service.clone();
            return Box::pin(async move {
                let res = svc.call(req).await?;
                Ok(res.map_into_boxed_body())
            });
        }

        // 提取所需上下文（在调用 service 之前，req 还没被消费）
        let app_state = match req.app_data::<web::Data<AppState>>() {
            Some(s) => s.clone(),
            None => {
                let svc = self.service.clone();
                return Box::pin(async move {
                    let res = svc.call(req).await?;
                    Ok(res.map_into_boxed_body())
                });
            }
        };
        let db = app_state.db.clone();

        let (oper_name, operator_type) = extract_user_from_request(&req);
        let title = title_from_path(&path);
        let business_type = business_type_from_method_and_path(&method, &path);
        let method_name = method.to_string();
        let oper_ip = req
            .connection_info()
            .realip_remote_addr()
            .map(|s| s.to_string());

        // ---- 提取请求体 ----
        // 把 ServiceRequest 拆成 (HttpRequest, Payload)，读完 payload 后再重组
        let (http_req_parts, mut payload) = req.into_parts();
        let svc = self.service.clone();
        Box::pin(async move {
            // 读取请求体 bytes（用 StreamExt::next 异步拉取所有 chunk）
            let mut req_buf = Vec::new();
            while let Some(chunk) = payload.next().await {
                match chunk {
                    Ok(c) => req_buf.extend_from_slice(&c),
                    Err(_) => break,
                }
            }
            let req_bytes = bytes::Bytes::from(req_buf);

            // 把请求体重新放回 req，让下游 handler 能正常解析
            // Payload::from(bytes) 创建一个一次性返回所有 bytes 的 payload
            let new_payload = Payload::from(req_bytes.clone());
            let req = ServiceRequest::from_parts(http_req_parts, new_payload);

            // 解析请求体为可读字符串并脱敏（支持 JSON / msgpack / form-urlencoded）
            let oper_param = sensitive::mask_body_bytes(&req_bytes);

            // 调用下游 service
            let start = std::time::Instant::now();
            let res = svc.call(req).await?;
            let elapsed = start.elapsed().as_millis() as i64;
            let status_code = res.status().as_u16() as i32;

            // 操作状态：HTTP 2xx 视为正常(0)，其它视为异常(1)
            let log_status = if (200..300).contains(&status_code) { Some(0) } else { Some(1) };
            let error_msg = if log_status == Some(1) {
                Some(format!("HTTP {}", status_code))
            } else {
                None
            };

            // ---- 提取响应体 ----
            // ServiceResponse::into_parts 返回 (HttpRequest, HttpResponse<B>)
            // 然后用 HttpResponse::into_body 拿到真正的 body B
            let (http_req, resp) = res.into_parts();
            let status = resp.status();
            let headers_clone = resp.headers().clone();
            let body = resp.into_body();
            // 读取响应体 bytes（限制大小 10MB，避免大响应撑爆内存）
            let resp_bytes_full = match actix_web::body::to_bytes(body).await {
                Ok(b) => b,
                Err(_) => bytes::Bytes::new(),
            };
            // 截断到 10MB 用于日志记录（不影响返回给前端的完整数据）
            let max_resp_bytes = 10 * 1024 * 1024;
            let resp_bytes = if resp_bytes_full.len() > max_resp_bytes {
                resp_bytes_full.slice(0..max_resp_bytes)
            } else {
                resp_bytes_full.clone()
            };
            let json_result = sensitive::mask_body_bytes(&resp_bytes);

            // 用 HttpResponse::with_body 重新构造响应，保留原 status 和 headers
            // resp_bytes_full 是 bytes::Bytes，实现了 MessageBody
            let mut new_resp = actix_web::HttpResponse::with_body(status, resp_bytes_full);
            *new_resp.headers_mut() = headers_clone;
            // map_into_boxed_body 把 ServiceResponse<Bytes> 转成 ServiceResponse<BoxBody>
            // 与 type Response = ServiceResponse<BoxBody> 类型匹配
            let new_res = ServiceResponse::new(http_req, new_resp).map_into_boxed_body();


            // 异步写日志，不阻塞响应返回
            actix_web::rt::spawn(async move {
                let payload = log_entity::ActiveModel {
                    id: Set(SNOWFLAKE.generate() as i64),
                    title: Set(title),
                    business_type: Set(business_type),
                    method: Set(Some(method_name)),
                    request_method: Set(Some(method.to_string())),
                    operator_type: Set(operator_type),
                    oper_name: Set(oper_name),
                    dept_name: Set(None),
                    oper_url: Set(Some(path)),
                    oper_ip: Set(oper_ip),
                    oper_location: Set(None),
                    oper_param: Set(oper_param),
                    json_result: Set(json_result),
                    status: Set(log_status),
                    error_msg: Set(error_msg),
                    status_code: Set(Some(status_code)),
                    elapsed: Set(Some(elapsed)),
                    create_time: Set(Some(chrono::Local::now().naive_local())),
                    ..Default::default()
                };
                if let Err(e) = log_entity::Entity::insert(payload).exec(&db).await {
                    log::warn!("[操作日志] 写入失败: {}", e);
                }
            });

            Ok(new_res)
        })
    }
}

/// 从请求中提取用户信息（oper_name、operator_type）
///
/// - 解析 Authorization: Bearer <token>
/// - admin token → operator_type=1
/// - user token → operator_type=2
/// - 没有 token 或解析失败 → None / None
fn extract_user_from_request(req: &ServiceRequest) -> (Option<String>, Option<i32>) {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    if token.is_empty() {
        return (None, None);
    }

    let token = token.strip_prefix("Bearer ").unwrap_or(token);
    let secret = config::section::<String>("server", "jwt_secret_admin", "".to_string());
    match JWTToken::verify(&secret, token) {
        Ok(claims) => (claims.username, Some(1)),
        Err(_) => {
            let user_secret = config::section::<String>("server", "jwt_secret_user", "".to_string());
            match JWTToken::verify(&user_secret, token) {
                Ok(claims) => (claims.username, Some(2)),
                Err(_) => (None, None),
            }
        }
    }
}

/// 根据路径推断模块标题
///
/// `/api/system/{module}/...` → `{module}`
fn title_from_path(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    // ["api", "system", "menu", "list"] → "menu"
    if parts.len() >= 3 {
        Some(parts[2].to_string())
    } else {
        Some("system".to_string())
    }
}

/// 根据 HTTP 方法和路径推断业务类型
///
/// - DELETE → 删除(3)
/// - PUT → 修改(2)
/// - POST → 新增(1)，除非路径包含 list/page/info/detail/options/get/by_id → 其它(0)
fn business_type_from_method_and_path(method: &Method, path: &str) -> Option<i32> {
    match *method {
        Method::DELETE => Some(BUSINESS_TYPE_DELETE),
        Method::PUT => Some(BUSINESS_TYPE_UPDATE),
        Method::POST => {
            let p = path.to_lowercase();
            let read_only_markers = [
                "/list", "/page", "/info", "/detail", "/options", "/get", "/by_id",
                "/tree", "/select", "/all", "/export", "/login", "/logout", "/captcha",
                "/check", "/verify",
            ];
            if read_only_markers.iter().any(|m| p.contains(m)) {
                Some(BUSINESS_TYPE_OTHER)
            } else {
                Some(BUSINESS_TYPE_INSERT)
            }
        }
        _ => Some(BUSINESS_TYPE_OTHER),
    }
}

/// 判断是否是排除路径（不记录日志）
///
/// 排除项：
/// - 登录/注销/验证码/注册（无 token 或自循环）
/// - 文件上传/下载（body 巨大或二进制）
/// - 系统日志自身的查询/删除（避免写入循环）
fn is_excluded_path(path: &str) -> bool {
    const EXCLUDE_SUFFIX: &[&str] = &[
        "/auth/login",
        "/auth/logout",
        "/auth/captcha",
        "/auth/register",
        "/captcha",
        "/upload",
        "/attachment/upload",
        "/import",
        "/export",
        "/logs/list",
        "/logs/bath_delete",
        "/logs/detail",
        "/wechat/login",
    ];

    EXCLUDE_SUFFIX.iter().any(|s| path.ends_with(s))
}

/// 用于消除未使用导入警告（BoxBody 在 Transform 中使用）
#[allow(dead_code)]
fn _assert_box_body() {
    let _: Option<BoxBody> = None;
}
