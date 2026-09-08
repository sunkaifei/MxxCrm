//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 批2 认证安全控制器：MFA / 找回密码 / 注册邮箱验证 / 会话自管理 / API 令牌 / SSO
//!
//! 路由分两类：
//! - 公开（白名单）：/auth/mfa/verify、/auth/forgot_password、/auth/reset_password、
//!   /auth/register/email_code、/auth/sso/*（sso 走前缀白名单）
//! - 仅登录：MFA 绑定/解绑、/auth/my-sessions、/auth/api-token/*

use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_user;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::system::controller::admin::system_admin_controller::issue_login_tokens;
use crate::modules::system::model::auth_security::*;
use crate::modules::system::service::{admin_service, auth_security_service};
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use serde::Deserialize;

fn ok<T: serde::Serialize>(data: T) -> HttpResponse {
    HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))
}

fn fail(msg: &str) -> HttpResponse {
    HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, msg, "local"))
}

// ==================== MFA ====================

/// POST /auth/mfa/verify - 登录二次验证（公开：凭一次性 ticket）
#[post("/mfa/verify")]
pub async fn mfa_verify(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<MfaVerifyRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let Some((user_id, remember_me)) = auth_security_service::consume_mfa_ticket(item.ticket.trim()).await else {
        return Ok(fail("MFA 验证已过期，请重新登录"));
    };
    if let Err(msg) = auth_security_service::verify_mfa_code(db, user_id, &item.code).await {
        return Ok(fail(&msg));
    }
    let admin = admin_service::get_by_detail(db, &Some(user_id)).await?;
    let is_admin = admin.user_type == Some(1);
    let oper_param = format!("{{\"username\":\"{}\"}}", admin.user_name.clone().unwrap_or_default());
    issue_login_tokens(db, &req, user_id, admin.user_name.clone().unwrap_or_default(), is_admin, remember_me, None, oper_param).await
}

/// POST /auth/mfa/setup - 发起 MFA 绑定（仅登录）
#[post("/mfa/setup")]
pub async fn mfa_setup(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<MfaSetupRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let Ok(token) = get_user(&req) else {
        return Ok(fail("登录状态已失效"));
    };
    let Some(user_id) = token.id else {
        return Ok(fail("无效的用户身份"));
    };
    match auth_security_service::begin_setup(db, user_id, item.mfa_type).await {
        Ok(vo) => Ok(ok(vo)),
        Err(msg) => Ok(fail(&msg)),
    }
}

/// POST /auth/mfa/setup/confirm - 确认 MFA 绑定（仅登录）
#[post("/mfa/setup/confirm")]
pub async fn mfa_setup_confirm(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<MfaSetupConfirmRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let Ok(token) = get_user(&req) else {
        return Ok(fail("登录状态已失效"));
    };
    let Some(user_id) = token.id else {
        return Ok(fail("无效的用户身份"));
    };
    match auth_security_service::confirm_setup(db, user_id, item.mfa_type, &item.code).await {
        Ok(()) => Ok(ok("MFA 绑定成功")),
        Err(msg) => Ok(fail(&msg)),
    }
}

/// POST /auth/mfa/enroll - 强制 MFA 场景：未绑定用户凭票据发起绑定（公开）
#[post("/mfa/enroll")]
pub async fn mfa_enroll(
    state: web::Data<AppState>,
    item: web::Json<MfaEnrollRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match auth_security_service::enroll_begin(db, &item.ticket, item.mfa_type).await {
        Ok(vo) => Ok(ok(vo)),
        Err(msg) => Ok(fail(&msg)),
    }
}

/// POST /auth/mfa/enroll/confirm - 强制 MFA 场景：确认绑定并完成登录（公开）
#[post("/mfa/enroll/confirm")]
pub async fn mfa_enroll_confirm(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<MfaSetupConfirmWithTicketRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match auth_security_service::enroll_confirm(db, &item.ticket, item.mfa_type, &item.code).await {
        Ok((user_id, remember_me)) => {
            let admin = admin_service::get_by_detail(db, &Some(user_id)).await?;
            let is_admin = admin.user_type == Some(1);
            let oper_param = format!("{{\"username\":\"{}\"}}", admin.user_name.clone().unwrap_or_default());
            issue_login_tokens(db, &req, user_id, admin.user_name.clone().unwrap_or_default(), is_admin, remember_me, None, oper_param).await
        }
        Err(msg) => Ok(fail(&msg)),
    }
}

/// POST /auth/mfa/disable - 解绑 MFA（仅登录，需当前动态码）
#[post("/mfa/disable")]
pub async fn mfa_disable(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<MfaDisableRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let Ok(token) = get_user(&req) else {
        return Ok(fail("登录状态已失效"));
    };
    let Some(user_id) = token.id else {
        return Ok(fail("无效的用户身份"));
    };
    match auth_security_service::disable(db, user_id, &item.code).await {
        Ok(()) => Ok(ok("MFA 已解绑")),
        Err(msg) => Ok(fail(&msg)),
    }
}

// ==================== 找回密码 / 注册邮箱验证 ====================

/// POST /auth/forgot_password - 发起找回密码（公开）
#[post("/forgot_password")]
pub async fn forgot_password(
    state: web::Data<AppState>,
    item: web::Json<ForgotPasswordRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match auth_security_service::forgot_password(db, &item.email).await {
        Ok(vo) => Ok(ok(vo)),
        Err(msg) => Ok(fail(&msg)),
    }
}

/// POST /auth/reset_password - 重置密码（公开：凭票据 + 邮箱验证码）
#[post("/reset_password")]
pub async fn reset_password(
    state: web::Data<AppState>,
    item: web::Json<ResetPasswordRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match auth_security_service::reset_password(db, &item.ticket, &item.code, &item.password).await {
        Ok(_) => Ok(ok("密码重置成功，请使用新密码登录")),
        Err(msg) => Ok(fail(&msg)),
    }
}

/// POST /auth/register/email_code - 注册邮箱验证码（公开；受注册开关约束由前端控制展示）
#[post("/register/email_code")]
pub async fn register_email_code(
    state: web::Data<AppState>,
    item: web::Json<RegisterEmailCodeRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match auth_security_service::send_register_email_code(db, &item.email).await {
        Ok(masked) => Ok(ok(masked)),
        Err(msg) => Ok(fail(&msg)),
    }
}

// ==================== 会话自管理 ====================

/// GET /auth/my-sessions - 我的在线会话（仅登录）
#[get("/my-sessions")]
pub async fn my_sessions(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let Ok(token) = get_user(&req) else {
        return Ok(fail("登录状态已失效"));
    };
    let Some(user_id) = token.id else {
        return Ok(fail("无效的用户身份"));
    };
    let current_token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default()
        .trim_start_matches("Bearer ")
        .to_string();
    match auth_security_service::my_sessions(db, user_id, &current_token).await {
        Ok(list) => Ok(ok(list)),
        Err(msg) => Ok(fail(&msg)),
    }
}

/// POST /auth/my-session/revoke - 下线自己的指定会话（仅登录）
#[post("/my-session/revoke")]
pub async fn revoke_my_session(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<SessionRevokeRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    // 注意：admin 路由用 get_user（admin JWT），勿用 user_auth（user 端 JWT）
    let Ok(token) = get_user(&req) else {
        return Ok(fail("登录状态已失效"));
    };
    let Some(user_id) = token.id else {
        return Ok(fail("无效的用户身份"));
    };
    match auth_security_service::revoke_my_session(db, user_id, &item.token).await {
        Ok(()) => Ok(ok("会话已下线")),
        Err(msg) => Ok(fail(&msg)),
    }
}

// ==================== API 个人访问令牌（PAT）===================

/// GET /auth/api-token/list - 我的 API 令牌（仅登录）
#[get("/api-token/list")]
pub async fn api_token_list(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let Ok(token) = get_user(&req) else {
        return Ok(fail("登录状态已失效"));
    };
    let Some(user_id) = token.id else {
        return Ok(fail("无效的用户身份"));
    };
    match auth_security_service::list_pats(db, user_id).await {
        Ok(list) => Ok(ok(list)),
        Err(msg) => Ok(fail(&msg)),
    }
}

/// POST /auth/api-token/create - 创建 PAT（仅登录，明文仅返回一次）
#[post("/api-token/create")]
pub async fn api_token_create(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<PatCreateRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let Ok(token) = get_user(&req) else {
        return Ok(fail("登录状态已失效"));
    };
    let Some(user_id) = token.id else {
        return Ok(fail("无效的用户身份"));
    };
    match auth_security_service::create_pat(db, user_id, &item.name, item.expire_days).await {
        Ok(vo) => Ok(ok(vo)),
        Err(msg) => Ok(fail(&msg)),
    }
}

/// POST /auth/api-token/revoke - 停用 PAT（仅登录）
#[post("/api-token/revoke")]
pub async fn api_token_revoke(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<PatRevokeRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let Ok(token) = get_user(&req) else {
        return Ok(fail("登录状态已失效"));
    };
    let Some(user_id) = token.id else {
        return Ok(fail("无效的用户身份"));
    };
    match auth_security_service::revoke_pat(db, user_id, item.id).await {
        Ok(()) => Ok(ok("令牌已停用")),
        Err(msg) => Ok(fail(&msg)),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MfaEnrollRequest {
    pub ticket: String,
    pub mfa_type: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MfaSetupConfirmWithTicketRequest {
    pub ticket: String,
    pub mfa_type: i32,
    pub code: String,
}

// ==================== SSO（OIDC / 企业微信 / 钉钉）====================

/// GET /auth/sso/status - 查询 SSO 状态（公开，登录页判断是否显示第三方图标）
#[get("/sso/status")]
pub async fn sso_status(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    let providers = auth_security_service::sso_status_vos(db).await;
    // enabled 由 providers 推导（任一开启即 true），避免再查一遍配置
    let enabled = providers.iter().any(|p| p.enabled);
    Ok(ok(SsoStatusVO { enabled, providers }))
}

/// GET /auth/sso/authorize?provider=wecom|dingtalk|oidc - 跳转 IdP 授权页（公开，302）
#[get("/sso/authorize")]
pub async fn sso_authorize(
    state: web::Data<AppState>,
    query: web::Query<SsoAuthorizeQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let provider = query.provider.clone().unwrap_or_default();
    match auth_security_service::build_authorize_url(db, &provider).await {
        Ok(url) => Ok(HttpResponse::TemporaryRedirect().insert_header(("Location", url)).finish()),
        Err(msg) => Ok(fail(&msg)),
    }
}

/// GET /auth/sso/callback - IdP 回调（公开，302 回前端；登录场景带双 Token，绑定场景带 bind_result）
#[get("/sso/callback")]
pub async fn sso_callback(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<SsoCallbackQuery>,
) -> Result<HttpResponse> {
    use crate::core::kit::jwt_util::JWTToken;
    use crate::modules::system::service::{menu_service, permission_cache_service, session_service};

    let db = &state.db;
    let frontend_url = auth_security_service::frontend_base_url(db).await;
    let base = format!("{}/auth/sso-callback", frontend_url);
    if let Some(err) = &query.error {
        let url = format!("{}?error={}", base, urlencoding(err));
        return Ok(HttpResponse::TemporaryRedirect().insert_header(("Location", url)).finish());
    }
    let (Some(code), Some(state)) = (&query.code, &query.state) else {
        let url = format!("{}?error=missing_code", base);
        return Ok(HttpResponse::TemporaryRedirect().insert_header(("Location", url)).finish());
    };
    let provider = query.provider.clone().unwrap_or_default();

    let outcome = match auth_security_service::sso_callback(db, &provider, code, state).await {
        Ok(v) => v,
        Err(msg) => {
            let url = format!("{}?error={}", base, urlencoding(&msg));
            return Ok(HttpResponse::TemporaryRedirect().insert_header(("Location", url)).finish());
        }
    };
    // 绑定场景：不签 Token，回个人中心提示绑定成功
    if let SsoCallbackOutcome::BindSuccess = outcome {
        let url = format!("{}?bind_result=success", base);
        return Ok(HttpResponse::TemporaryRedirect().insert_header(("Location", url)).finish());
    }
    let (user_id, user_name, is_admin) = match outcome {
        SsoCallbackOutcome::Login(uid, uname, ia) => (uid, uname, ia),
        SsoCallbackOutcome::BindSuccess => unreachable!(),
    };

    // 签发双 Token（与登录链路同一套原语；SSO 会话不涉及"记住我"档）
    let access_expire_secs = permission_cache_service::get_access_token_expire_secs().await;
    let session_expire_secs = permission_cache_service::get_session_timeout_secs().await;
    let user_role_keys = menu_service::find_user_role_keys(db, &is_admin, &Some(user_id)).await.unwrap_or_default();
    let _ = permission_cache_service::set_permissions(user_id, &user_role_keys).await;
    let refresh_plain = session_service::generate_refresh_token();
    let refresh_hash = session_service::sha256_hex(&refresh_plain);
    let token = match JWTToken::new_with_expire(Some(user_id), Some(user_name.clone()), None, access_expire_secs)
        .create_token(&crate::config::section::<String>("server", "jwt_secret_admin", "".to_string()))
    {
        Ok(t) => t,
        Err(_) => {
            let url = format!("{}?error=issue_failed", base);
            return Ok(HttpResponse::TemporaryRedirect().insert_header(("Location", url)).finish());
        }
    };
    let multi = permission_cache_service::is_multi_device_mode().await;
    let _ = if multi {
        let key = format!("user_tokens_{}", user_id);
        let mut tokens: Vec<String> = crate::core::kit::CONTEXT.cache_service.get_json(&key).await.unwrap_or_default();
        tokens.push(token.clone());
        crate::core::kit::CONTEXT.cache_service.set_json_ex(&key, &tokens, Some(std::time::Duration::from_secs(access_expire_secs))).await
    } else {
        let _ = session_service::get_session_store().remove_session(db, user_id).await;
        crate::core::kit::CONTEXT.cache_service.set_string_ex(&format!("user_{}", user_id), &token, Some(std::time::Duration::from_secs(access_expire_secs))).await
    };
    let _ = session_service::get_session_store()
        .create_session(db, user_id, &token, &refresh_hash, &req.connection_info().realip_remote_addr().unwrap_or_default().to_string(), access_expire_secs as i64, session_expire_secs as i64)
        .await;

    let url = format!(
        "{}#accessToken={}&refreshToken={}",
        base,
        urlencoding(&token),
        urlencoding(&refresh_plain)
    );
    Ok(HttpResponse::TemporaryRedirect().insert_header(("Location", url)).finish())
}

// ==================== 账号绑定（企业微信 / 钉钉）====================

/// GET /auth/sso/bind/list - 已绑定列表（仅登录）
#[get("/sso/bind/list")]
pub async fn sso_bind_list(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let Ok(token) = get_user(&req) else {
        return Ok(fail("登录状态已失效"));
    };
    let Some(user_id) = token.id else {
        return Ok(fail("无效的用户身份"));
    };
    match auth_security_service::sso_bind_list(db, user_id).await {
        Ok(items) => Ok(ok(items)),
        Err(msg) => Ok(fail(&msg)),
    }
}

/// GET /auth/sso/bind/start?provider=wecom|dingtalk - 发起绑定（仅登录，返回授权 URL 由前端跳转）
///
/// 不做 302：浏览器跳转无法携带 Authorization 头；且跨域第三方授权页非 XHR 可跟随目标。
#[get("/sso/bind/start")]
pub async fn sso_bind_start(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<SsoAuthorizeQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let Ok(token) = get_user(&req) else {
        return Ok(fail("登录状态已失效"));
    };
    let Some(user_id) = token.id else {
        return Ok(fail("无效的用户身份"));
    };
    let provider = query.provider.clone().unwrap_or_default();
    match auth_security_service::sso_bind_start(db, user_id, &provider).await {
        Ok(url) => Ok(ok(serde_json::json!({ "url": url }))),
        Err(msg) => Ok(fail(&msg)),
    }
}

/// POST /auth/sso/bind/unbind - 解绑（仅登录，幂等）
#[post("/sso/bind/unbind")]
pub async fn sso_bind_unbind(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<SsoBindUnbindRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let Ok(token) = get_user(&req) else {
        return Ok(fail("登录状态已失效"));
    };
    let Some(user_id) = token.id else {
        return Ok(fail("无效的用户身份"));
    };
    match auth_security_service::sso_bind_unbind(db, user_id, &item.provider).await {
        Ok(()) => Ok(ok("解绑成功")),
        Err(msg) => Ok(fail(&msg)),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SsoCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    /// wecom / dingtalk / oidc（企业微信、钉钉跳转时携带）
    pub provider: Option<String>,
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "%20".to_string(),
            ':' => "%3A".to_string(),
            '/' => "%2F".to_string(),
            '?' => "%3F".to_string(),
            '&' => "%26".to_string(),
            '=' => "%3D".to_string(),
            '#' => "%23".to_string(),
            c => c.to_string(),
        })
        .collect()
}

/// 路由注册（挂载在 /api/system/auth 之下；公开路径已列白名单）
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(mfa_verify)
        .service(mfa_enroll)
        .service(mfa_enroll_confirm)
        .service(mfa_setup)
        .service(mfa_setup_confirm)
        .service(mfa_disable)
        .service(forgot_password)
        .service(reset_password)
        .service(register_email_code)
        .service(my_sessions)
        .service(revoke_my_session)
        .service(api_token_list)
        .service(api_token_create)
        .service(api_token_revoke)
        .service(sso_status)
        .service(sso_authorize)
        .service(sso_callback)
        .service(sso_bind_list)
        .service(sso_bind_start)
        .service(sso_bind_unbind);
}
