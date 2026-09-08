//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 批2 认证安全服务：MFA / 找回密码 / 注册邮箱验证 / 会话自管理 / API 令牌（PAT）/ OIDC SSO / 角色登录策略

use rand::Rng;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, QueryFilter, QueryOrder, Set};

use crate::core::kit::password_util;
use crate::core::kit::totp;
use crate::core::kit::CONTEXT;
use crate::modules::system::entity::{admin, api_token, role, user_bind};
use crate::modules::system::model::auth_security::*;
use crate::modules::system::service::{
    admin_service, integration_config_service, otp_service, permission_cache_service, session_service,
    sso_provider,
};

const MFA_TICKET_TTL: u64 = 300;
const MFA_SETUP_TTL: u64 = 600;
const RESET_TICKET_TTL: u64 = 900;
const SSO_STATE_TTL: u64 = 600;

fn gen_hex(len: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| format!("{:x}", rng.gen_range(0..16)))
        .collect()
}

fn now_secs() -> u64 {
    chrono::Utc::now().timestamp().max(0) as u64
}

// ==================== MFA ====================

/// 判定登录是否需要 MFA：本人开启 / 超管强制开关 / 任一有效角色开启
pub async fn is_mfa_required(db: &DbConn, user_id: i64, is_admin: bool) -> std::result::Result<bool, String> {
    let admin = admin_service::get_by_detail(db, &Some(user_id))
        .await
        .map_err(|e| e.to_string())?;
    if admin.mfa_type.unwrap_or(0) > 0 {
        return Ok(true);
    }
    if is_admin {
        let enforce = CONTEXT
            .cache_service
            .get_string("config:mfa_enforce_superadmin")
            .await
            .ok()
            .unwrap_or_default();
        let enforce = if !enforce.is_empty() {
            enforce == "1"
        } else {
            crate::modules::system::service::config_service::find_value_by_key_from_db("mfa_enforce_superadmin")
                .await
                .unwrap_or_else(|| "1".to_string())
                == "1"
        };
        if enforce {
            return Ok(true);
        }
    }
    let merges = crate::modules::system::model::admin_role_merge::AdminRoleMergeModel::find_by_admin_id(db, &Some(user_id))
        .await
        .unwrap_or_default();
    for m in merges {
        let Some(rid) = m.role_id else { continue };
        if let Ok(Some(r)) = role::Entity::find_by_id(rid).one(db).await {
            if r.status == Some(1) && r.mfa_required == Some(1) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// 创建 MFA 挑战票据（5 分钟一次性，携带 remember_me 档位）
pub async fn create_mfa_ticket(user_id: i64, remember_me: bool) -> std::result::Result<String, String> {
    let ticket = gen_hex(48);
    let payload = serde_json::json!({ "user_id": user_id, "remember_me": remember_me });
    CONTEXT
        .cache_service
        .set_string_ex(
            &format!("mfa_ticket:{}", ticket),
            &payload.to_string(),
            Some(std::time::Duration::from_secs(MFA_TICKET_TTL)),
        )
        .await
        .map_err(|e| e.to_string())?;
    Ok(ticket)
}

/// 批2: 强制 MFA 场景——未绑定用户凭票据发起绑定（公开端点用）
pub async fn enroll_begin(db: &DbConn, ticket: &str, mfa_type: i32) -> Result<MfaSetupVO, String> {
    let Some((user_id, _)) = consume_mfa_ticket_peek(ticket).await else {
        return Err("MFA 会话已过期，请重新登录".to_string());
    };
    begin_setup(db, user_id, mfa_type).await
}

/// 批2: 强制 MFA 场景——凭票据确认绑定（成功后由调用方签发令牌）
pub async fn enroll_confirm(db: &DbConn, ticket: &str, mfa_type: i32, code: &str) -> std::result::Result<(i64, bool), String> {
    let Some((user_id, remember_me)) = consume_mfa_ticket_peek(ticket).await else {
        return Err("MFA 会话已过期，请重新登录".to_string());
    };
    confirm_setup(db, user_id, mfa_type, code).await?;
    Ok((user_id, remember_me))
}

/// 读取票据但不消费（enroll 两步之间复用同一票据）
async fn consume_mfa_ticket_peek(ticket: &str) -> Option<(i64, bool)> {
    let key = format!("mfa_ticket:{}", ticket);
    let raw = CONTEXT.cache_service.get_string(&key).await.ok()?;
    if raw.is_empty() {
        return None;
    }
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    Some((v.get("user_id")?.as_i64()?, v.get("remember_me")?.as_bool()?))
}

/// 消费 MFA 票据（一次性）
pub async fn consume_mfa_ticket(ticket: &str) -> Option<(i64, bool)> {
    let key = format!("mfa_ticket:{}", ticket);
    let raw = CONTEXT.cache_service.get_string(&key).await.ok()?;
    if raw.is_empty() {
        return None;
    }
    let _ = CONTEXT.cache_service.del(&key).await;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    Some((v.get("user_id")?.as_i64()?, v.get("remember_me")?.as_bool()?))
}

/// 校验 MFA 动态码（TOTP 或邮箱码）
pub async fn verify_mfa_code(db: &DbConn, user_id: i64, code: &str) -> std::result::Result<(), String> {
    let admin = admin_service::get_by_detail(db, &Some(user_id))
        .await
        .map_err(|e| e.to_string())?;
    match admin.mfa_type.unwrap_or(0) {
        1 => {
            let secret = admin
                .mfa_secret
                .clone()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "MFA 未正确配置，请联系管理员".to_string())?;
            if totp::verify_totp(&secret, code.trim(), now_secs()) {
                Ok(())
            } else {
                Err("动态验证码不正确或已过期".to_string())
            }
        }
        2 => otp_service::verify(user_id, "mfa_login", code.trim()),
        _ => Err("当前账号未启用 MFA".to_string()),
    }
}

/// 发起 MFA 绑定（TOTP 返回密钥与 otpauth 链接；邮箱发送验证码）
pub async fn begin_setup(db: &DbConn, user_id: i64, mfa_type: i32) -> std::result::Result<MfaSetupVO, String> {
    let admin = admin_service::get_by_detail(db, &Some(user_id))
        .await
        .map_err(|e| e.to_string())?;
    match mfa_type {
        1 => {
            let secret = totp::generate_secret();
            let url = totp::build_otpauth_url(&secret, admin.user_name.as_deref().unwrap_or("mxx"), "MxxCRM");
            CONTEXT
                .cache_service
                .set_string_ex(
                    &format!("mfa_setup:{}", user_id),
                    &secret,
                    Some(std::time::Duration::from_secs(MFA_SETUP_TTL)),
                )
                .await
                .map_err(|e| e.to_string())?;
            Ok(MfaSetupVO { mfa_type: 1, secret: Some(secret), otpauth_url: Some(url), email_masked: None })
        }
        2 => {
            let email = admin.email.clone().unwrap_or_default();
            let masked = otp_service::send_to_email(db, user_id, "mfa_setup", &email).await?;
            Ok(MfaSetupVO { mfa_type: 2, secret: None, otpauth_url: None, email_masked: Some(masked) })
        }
        _ => Err("不支持的 MFA 类型".to_string()),
    }
}

/// 确认 MFA 绑定（验证动态码后落库）
pub async fn confirm_setup(db: &DbConn, user_id: i64, mfa_type: i32, code: &str) -> std::result::Result<(), String> {
    match mfa_type {
        1 => {
            let key = format!("mfa_setup:{}", user_id);
            let secret = CONTEXT.cache_service.get_string(&key).await.unwrap_or_default();
            if secret.is_empty() {
                return Err("绑定会话已过期，请重新获取密钥".to_string());
            }
            if !totp::verify_totp(&secret, code.trim(), now_secs()) {
                return Err("动态验证码不正确，请核对认证器时间或重新扫码".to_string());
            }
            let _ = CONTEXT.cache_service.del(&key).await;
            update_mfa(db, user_id, &secret, 1).await
        }
        2 => {
            otp_service::verify(user_id, "mfa_setup", code.trim())?;
            update_mfa(db, user_id, "", 2).await
        }
        _ => Err("不支持的 MFA 类型".to_string()),
    }
}

/// 解绑 MFA（需先通过当前验证方式的一次动态码）
pub async fn disable(db: &DbConn, user_id: i64, code: &str) -> std::result::Result<(), String> {
    verify_mfa_code(db, user_id, code).await?;
    update_mfa(db, user_id, "", 0).await
}

async fn update_mfa(db: &DbConn, user_id: i64, secret: &str, mfa_type: i32) -> std::result::Result<(), String> {
    use sea_orm::sea_query::Expr;
    admin::Entity::update_many()
        .col_expr(admin::Column::MfaSecret, Expr::value(if secret.is_empty() { None } else { Some(secret.to_string()) }))
        .col_expr(admin::Column::MfaType, Expr::value(mfa_type))
        .filter(admin::Column::Id.eq(user_id))
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ==================== 找回密码 / 注册邮箱验证 ====================

/// 发起找回密码：按邮箱定位账号 → 发送验证码 → 返回一次性票据
pub async fn forgot_password(db: &DbConn, email: &str) -> std::result::Result<ForgotPasswordVO, String> {
    let email = email.trim().to_string();
    let admin = admin::Entity::find()
        .filter(admin::Column::Email.eq(&email))
        .filter(admin::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "该邮箱未绑定任何账号".to_string())?;
    if admin.status != Some(1) {
        return Err("账号已被停用，请联系管理员".to_string());
    }
    let masked = otp_service::send_to_email(db, admin.id, "forgot_password", &email)
        .await?;
    let ticket = gen_hex(48);
    CONTEXT
        .cache_service
        .set_string_ex(
            &format!("forgot_ticket:{}", ticket),
            &admin.id.to_string(),
            Some(std::time::Duration::from_secs(RESET_TICKET_TTL)),
        )
        .await
        .map_err(|e| e.to_string())?;
    Ok(ForgotPasswordVO { ticket, email_masked: masked })
}

/// 重置密码：校验票据 + 邮箱验证码 → 更新密码 → 全端下线
pub async fn reset_password(db: &DbConn, ticket: &str, code: &str, new_password: &str) -> std::result::Result<i64, String> {
    let key = format!("forgot_ticket:{}", ticket);
    let raw = CONTEXT.cache_service.get_string(&key).await.unwrap_or_default();
    if raw.is_empty() {
        return Err("重置链接已失效，请重新发起找回密码".to_string());
    }
    let user_id: i64 = raw.parse().map_err(|_| "重置票据异常".to_string())?;
    otp_service::verify(user_id, "forgot_password", code.trim())?;
    password_util::validate_password_strength(new_password).map_err(|e| e.to_string())?;

    use bcrypt::DEFAULT_COST;
    let hashed = bcrypt::hash(new_password, DEFAULT_COST).map_err(|e| format!("密码加密失败: {}", e))?;
    admin::Entity::update_many()
        .col_expr(admin::Column::Password, sea_orm::sea_query::Expr::value(hashed))
        .filter(admin::Column::Id.eq(user_id))
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;

    let _ = CONTEXT.cache_service.del(&key).await;
    // 改密即全端下线（含被重置账号的所有会话）
    permission_cache_service::revoke_user_session(db, user_id).await;
    Ok(user_id)
}

/// 注册邮箱验证码（注册流程未登录，按邮箱维度发码/校验）
pub async fn send_register_email_code(db: &DbConn, email: &str) -> std::result::Result<String, String> {
    let email = email.trim().to_string();
    if email.is_empty() || !email.contains('@') {
        return Err("邮箱格式不正确".to_string());
    }
    // 60 秒重发限制
    let last_key = format!("regcode_last:{}", email.to_lowercase());
    let last = CONTEXT.cache_service.get_string(&last_key).await.unwrap_or_default();
    if !last.is_empty() {
        let elapsed = chrono::Local::now().timestamp().saturating_sub(last.parse::<i64>().unwrap_or(0));
        if elapsed < 60 {
            return Err(format!("验证码发送过于频繁，请 {} 秒后再试", 60 - elapsed));
        }
    }
    let mut rng = rand::thread_rng();
    let code: String = (0..6).map(|_| rng.gen_range(0..10).to_string()).collect();
    let req = crate::modules::system::model::mail::SendMailRequest {
        customer_id: None,
        to_emails: vec![email.clone()],
        cc_emails: None,
        subject: Some("【Mxx CRM】注册邮箱验证码".to_string()),
        body: Some(format!(
            "<p>您正在注册 Mxx CRM 账号。</p><p>本次验证码为：<b style=\"font-size:20px\">{}</b></p><p>验证码 5 分钟内有效，请勿向他人泄露。</p>",
            code
        )),
        doc_url: None,
        contact_ids: None,
    };
    crate::modules::system::service::mail_service::send_mail(db, req, None, None)
        .await
        .map_err(|e| format!("验证码邮件发送失败: {}", e))?;
    let payload = serde_json::json!({ "code": code, "tries": 0, "sent_at": chrono::Local::now().timestamp() });
    CONTEXT
        .cache_service
        .set_string_ex(&format!("regcode:{}", email.to_lowercase()), &payload.to_string(), Some(std::time::Duration::from_secs(600)))
        .await
        .map_err(|e| e.to_string())?;
    let _ = CONTEXT.cache_service.set_string_ex(&last_key, &chrono::Local::now().timestamp().to_string(), Some(std::time::Duration::from_secs(120))).await;
    let mut masked = email.clone();
    if let Some(at) = email.find('@') {
        let name = &email[..at];
        masked = format!("{}***{}", &name[..name.len().min(2)], &email[at..]);
    }
    Ok(masked)
}

/// 校验并消费注册邮箱验证码
pub async fn verify_register_email_code(email: &str, code: &str) -> std::result::Result<(), String> {
    let key = format!("regcode:{}", email.trim().to_lowercase());
    let raw = CONTEXT.cache_service.get_string(&key).await.unwrap_or_default();
    if raw.is_empty() {
        return Err("验证码已过期，请重新获取".to_string());
    }
    let mut v: serde_json::Value = serde_json::from_str(&raw).map_err(|_| "验证码数据异常".to_string())?;
    if v["code"].as_str() != Some(code.trim()) {
        let tries = v["tries"].as_u64().unwrap_or(0) + 1;
        if tries >= 5 {
            let _ = CONTEXT.cache_service.del(&key).await;
            return Err("错误次数过多，验证码已失效，请重新获取".to_string());
        }
        v["tries"] = serde_json::json!(tries);
        let _ = CONTEXT.cache_service.set_string_ex(&key, &v.to_string(), Some(std::time::Duration::from_secs(600))).await;
        return Err("验证码不正确".to_string());
    }
    let _ = CONTEXT.cache_service.del(&key).await;
    Ok(())
}

// ==================== 会话自管理 ====================

/// 当前用户会话列表（current_token 用于标记当前会话）
pub async fn my_sessions(db: &DbConn, user_id: i64, current_token: &str) -> std::result::Result<Vec<SessionItemVO>, String> {
    let rows = session_service::list_user_sessions(db, user_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(rows
        .into_iter()
        .map(|m| {
            let is_current = m.token == current_token;
            let token = m.token.clone();
            let token_masked = if token.len() > 16 {
                format!("{}...{}", &token[..6], &token[token.len() - 6..])
            } else {
                token.clone()
            };
            SessionItemVO {
                token,
                token_masked,
                is_current,
                login_ip: m.login_ip.clone(),
                login_time: m.login_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
                expire_time: m.expire_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
            }
        })
        .collect())
}

/// 下线自己的指定会话（校验归属）
pub async fn revoke_my_session(db: &DbConn, user_id: i64, token: &str) -> std::result::Result<(), String> {
    let rows = session_service::list_user_sessions(db, user_id)
        .await
        .map_err(|e| e.to_string())?;
    if !rows.iter().any(|m| m.token == token) {
        return Err("会话不存在或不属于当前用户".to_string());
    }
    session_service::get_session_store()
        .remove_by_token(db, user_id, token)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ==================== API 个人访问令牌（PAT）====================

pub const PAT_PREFIX: &str = "mxxpat_";

/// 创建 PAT：返回明文（仅此一次）
pub async fn create_pat(db: &DbConn, user_id: i64, name: &str, expire_days: Option<i64>) -> std::result::Result<PatCreateVO, String> {
    if name.trim().is_empty() {
        return Err("令牌名称不能为空".to_string());
    }
    let mut rng = rand::thread_rng();
    let suffix: String = (0..64).map(|_| format!("{:x}", rng.gen_range(0..16))).collect();
    let plain = format!("{}{}", PAT_PREFIX, suffix);
    let hash = session_service::sha256_hex(&plain);
    let prefix = format!("{}{}", PAT_PREFIX, &suffix[..8]);
    let expire_time = expire_days
        .filter(|d| *d > 0)
        .map(|d| (chrono::Local::now().naive_local() + chrono::Duration::days(d)));

    let row = api_token::ActiveModel {
        user_id: Set(user_id),
        name: Set(Some(name.trim().to_string())),
        token_prefix: Set(Some(prefix)),
        token_hash: Set(Some(hash)),
        rate_limit: Set(Some(1000)),
        expire_time: Set(expire_time),
        status: Set(Some(1)),
        deleted: Set(Some(0)),
        create_time: Set(Some(chrono::Local::now().naive_local())),
        update_time: Set(Some(chrono::Local::now().naive_local())),
        ..Default::default()
    };
    let res = api_token::Entity::insert(row).exec(db).await.map_err(|e| e.to_string())?;
    Ok(PatCreateVO { id: res.last_insert_id, token: plain })
}

pub async fn list_pats(db: &DbConn, user_id: i64) -> std::result::Result<Vec<PatVO>, String> {
    let rows = api_token::Entity::find()
        .filter(api_token::Column::UserId.eq(user_id))
        .filter(api_token::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(rows
        .into_iter()
        .map(|m| PatVO {
            id: m.id,
            name: m.name,
            token_prefix: m.token_prefix,
            expire_time: m.expire_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
            last_used_at: m.last_used_at.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
            status: m.status,
            create_time: m.create_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        })
        .collect())
}

pub async fn revoke_pat(db: &DbConn, user_id: i64, id: i64) -> std::result::Result<(), String> {
    api_token::Entity::update_many()
        .col_expr(api_token::Column::Status, sea_orm::sea_query::Expr::value(0))
        .filter(api_token::Column::Id.eq(id))
        .filter(api_token::Column::UserId.eq(user_id))
        .filter(api_token::Column::Deleted.eq(0))
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// PAT 鉴权（extract 中间件调用）：校验哈希/有效期/限流，返回 user_id
pub async fn authenticate_pat(db: &DbConn, token_plain: &str) -> std::result::Result<i64, String> {
    let hash = session_service::sha256_hex(token_plain);
    let row = api_token::Entity::find()
        .filter(api_token::Column::TokenHash.eq(&hash))
        .filter(api_token::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "API 令牌无效".to_string())?;
    if row.status != Some(1) {
        return Err("API 令牌已停用".to_string());
    }
    if let Some(exp) = row.expire_time {
        if exp < chrono::Local::now().naive_local() {
            return Err("API 令牌已过期".to_string());
        }
    }
    // 固定窗口限流（每小时）
    let limit = row.rate_limit.unwrap_or(1000).max(1) as u64;
    let hour = chrono::Utc::now().timestamp() / 3600;
    let rl_key = format!("patrl:{}:{}", row.id, hour);
    let used: u64 = CONTEXT.cache_service.get_string(&rl_key).await.ok().and_then(|s| s.parse().ok()).unwrap_or(0);
    if used >= limit {
        return Err(format!("API 令牌请求超过每小时上限（{}）", limit));
    }
    let _ = CONTEXT.cache_service.set_string_ex(&rl_key, &(used + 1).to_string(), Some(std::time::Duration::from_secs(3600))).await;

    // 最近使用时间（best-effort，每分钟最多写一次）
    let lu_key = format!("patlu:{}", row.id);
    if CONTEXT.cache_service.get_string(&lu_key).await.unwrap_or_default().is_empty() {
        let _ = api_token::Entity::update_many()
            .col_expr(api_token::Column::LastUsedAt, sea_orm::sea_query::Expr::value(chrono::Local::now().naive_local()))
            .filter(api_token::Column::Id.eq(row.id))
            .exec(db)
            .await;
        let _ = CONTEXT.cache_service.set_string_ex(&lu_key, "1", Some(std::time::Duration::from_secs(60))).await;
    }
    Ok(row.user_id)
}

// ==================== SSO（OIDC / 企业微信 / 钉钉）====================

/// 各 provider 状态（设置页卡片用：configured=必填字段齐全，enabled=开关开启）
pub async fn sso_status_vos(db: &DbConn) -> Vec<SsoProviderStatusVO> {
    let mut vos = Vec::new();
    let oidc_configured = !sso_config_db(db, "sso_oidc_issuer").await.is_empty()
        && !sso_config_db(db, "sso_oidc_client_id").await.is_empty();
    vos.push(SsoProviderStatusVO {
        code: "oidc".to_string(),
        name: "OIDC 通用协议".to_string(),
        configured: oidc_configured,
        enabled: sso_config_db(db, "sso_oidc_enabled").await == "1",
    });
    for (code, name, required) in [
        ("wecom", "企业微信", &["corp_id", "agent_id", "corp_secret"][..]),
        ("dingtalk", "钉钉", &["app_key", "app_secret"][..]),
    ] {
        let (configured, enabled) = match integration_config_service::get_by_code(db, code).await {
            Ok(Some(cfg)) => {
                let configured = cfg.config_json.as_ref().map(|j| {
                    required
                        .iter()
                        .all(|k| j.get(*k).and_then(|v| v.as_str()).map(|s| !s.trim().is_empty()).unwrap_or(false))
                }).unwrap_or(false);
                (configured, cfg.enabled == Some(1))
            }
            _ => (false, false),
        };
        vos.push(SsoProviderStatusVO {
            code: code.to_string(),
            name: name.to_string(),
            configured,
            enabled,
        });
    }
    vos
}

async fn sso_config_db(_db: &DbConn, key: &str) -> String {
    let cached = CONTEXT.cache_service.get_string(&format!("config:{}", key)).await.unwrap_or_default();
    if !cached.is_empty() {
        return cached;
    }
    let val = crate::modules::system::service::config_service::find_value_by_key_from_db(key)
        .await
        .unwrap_or_default();
    let _ = CONTEXT.cache_service.set_string(&format!("config:{}", key), &val).await;
    val
}

/// provider 归一化（空 → oidc，兼容旧调用）
fn normalize_provider(provider: &str) -> String {
    let p = provider.trim();
    if p.is_empty() {
        "oidc".to_string()
    } else {
        p.to_string()
    }
}

/// 构造 IdP 授权跳转 URL（oidc 走发现文档 1h 缓存；wecom/dingtalk 走适配层）
pub async fn build_authorize_url(db: &DbConn, provider: &str) -> std::result::Result<String, String> {
    let provider = normalize_provider(provider);
    let server_url = crate::config::section::<String>("server", "server_url", "http://localhost:8088".to_string());
    let state = gen_hex(32);
    CONTEXT
        .cache_service
        .set_string_ex(&format!("sso_state:{}", state), "1", Some(std::time::Duration::from_secs(SSO_STATE_TTL)))
        .await
        .map_err(|e| e.to_string())?;
    if provider != "oidc" {
        let redirect_uri = format!("{}/api/system/auth/sso/callback?provider={}", server_url.trim_end_matches('/'), provider);
        return sso_provider::build_authorize_url(db, &provider, &redirect_uri, &state).await;
    }
    let issuer = sso_config_db(db, "sso_oidc_issuer").await.trim_end_matches('/').to_string();
    let client_id = sso_config_db(db, "sso_oidc_client_id").await;
    if issuer.is_empty() || client_id.is_empty() {
        return Err("SSO 未配置完整（issuer/client_id）".to_string());
    }
    let disc_key = format!("sso:discovery:{}", issuer);
    let mut discovery = CONTEXT.cache_service.get_string(&disc_key).await.unwrap_or_default();
    if discovery.is_empty() {
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("{}/.well-known/openid-configuration", issuer))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| format!("获取 OIDC 发现文档失败: {}", e))?;
        discovery = resp.text().await.map_err(|e| e.to_string())?;
        let _ = CONTEXT.cache_service.set_string_ex(&disc_key, &discovery, Some(std::time::Duration::from_secs(3600))).await;
    }
    let doc: serde_json::Value = serde_json::from_str(&discovery).map_err(|e| format!("OIDC 发现文档解析失败: {}", e))?;
    let authorization_endpoint = doc["authorization_endpoint"]
        .as_str()
        .ok_or_else(|| "OIDC 发现文档缺少 authorization_endpoint".to_string())?
        .to_string();
    let redirect_uri = format!("{}/api/system/auth/sso/callback", server_url.trim_end_matches('/'));

    Ok(format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}",
        authorization_endpoint,
        client_id,
        urlencoding_min(&redirect_uri),
        "openid%20profile%20email",
        state
    ))
}

fn urlencoding_min(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "%20".to_string(),
            ':' => "%3A".to_string(),
            '/' => "%2F".to_string(),
            '?' => "%3F".to_string(),
            '&' => "%26".to_string(),
            '=' => "%3D".to_string(),
            c => c.to_string(),
        })
        .collect()
}

/// 前端基地址（SSO 回跳用）
pub async fn frontend_base_url(db: &DbConn) -> String {
    sso_config_db(db, "frontend_url").await.trim_end_matches('/').to_string()
}

/// SSO 回调：code 换用户 → 绑定场景写绑定 / 登录场景匹配账号（含离职拦截）
///
/// provider: wecom / dingtalk / oidc（缺省 oidc）；state 编码绑定场景（bind:{user_id}:{rand}）。
pub async fn sso_callback(
    db: &DbConn,
    provider: &str,
    code: &str,
    state: &str,
) -> std::result::Result<SsoCallbackOutcome, String> {
    let state_key = format!("sso_state:{}", state);
    if CONTEXT.cache_service.get_string(&state_key).await.unwrap_or_default().is_empty() {
        return Err("SSO state 无效或已过期".to_string());
    }
    let _ = CONTEXT.cache_service.del(&state_key).await;

    let provider = normalize_provider(provider);
    let (is_bind, bind_user_id) = parse_sso_state(state);

    let sso_user = match provider.as_str() {
        "oidc" => oidc_exchange_user(db, code).await?,
        _ => sso_provider::exchange_and_fetch_user(db, &provider, code).await?,
    };

    if is_bind {
        let target_id = bind_user_id.ok_or_else(|| "绑定参数无效".to_string())?;
        // R7: 绑定目标账号必须在职（发起绑定到回调完成期间可能已离职/停用）
        ensure_user_active(db, target_id, "完成绑定").await?;
        // 防串号：provider + provider_uid 已被其他账号绑定则拒
        let existing = user_bind::Entity::find()
            .filter(user_bind::Column::Provider.eq(provider.clone()))
            .filter(user_bind::Column::ProviderUid.eq(sso_user.id.clone()))
            .filter(user_bind::Column::Deleted.eq(0))
            .one(db)
            .await
            .map_err(|e| e.to_string())?;
        if let Some(bind) = existing {
            if bind.user_id != target_id {
                return Err("该第三方账号已被其他系统账号绑定，请先解绑".to_string());
            }
            return Ok(SsoCallbackOutcome::BindSuccess);
        }
        let now = chrono::Local::now().naive_local();
        let row = user_bind::ActiveModel {
            user_id: Set(target_id),
            provider: Set(Some(provider.clone())),
            provider_uid: Set(Some(sso_user.id.clone())),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        user_bind::Entity::insert(row).exec(db).await.map_err(|e| {
            // 并发双绑兜底：唯一索引冲突转译为业务文案，不透传 DB 原始错误
            if e.to_string().contains("uk_user_bind") {
                "该第三方账号已被其他系统账号绑定，请刷新后重试".to_string()
            } else {
                format!("绑定失败: {}", e)
            }
        })?;
        return Ok(SsoCallbackOutcome::BindSuccess);
    }

    // 登录场景：1) 绑定表精准匹配（优先，扫码识别）
    let mut admin_row = match user_bind::Entity::find()
        .filter(user_bind::Column::Provider.eq(provider.clone()))
        .filter(user_bind::Column::ProviderUid.eq(sso_user.id.clone()))
        .filter(user_bind::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| e.to_string())?
    {
        Some(bind) => admin::Entity::find_by_id(bind.user_id).one(db).await.map_err(|e| e.to_string())?,
        None => None,
    };

    // 2) 邮箱 / 用户名匹配
    if admin_row.is_none() {
        let cond = sea_orm::Condition::all().add(admin::Column::Deleted.eq(0));
        let mut any = sea_orm::Condition::any();
        if !sso_user.email.is_empty() {
            any = any.add(admin::Column::Email.eq(sso_user.email.clone()));
        }
        if !sso_user.username.is_empty() {
            any = any.add(admin::Column::UserName.eq(sso_user.username.clone()));
        }
        admin_row = admin::Entity::find().filter(cond.add(any)).one(db).await.map_err(|e| e.to_string())?;
    }

    // 3) 可选自动开通（仅限有邮箱可标识身份者）
    if admin_row.is_none() {
        let auto = sso_config_db(db, "sso_auto_create").await == "1";
        if !auto || sso_user.email.is_empty() {
            return Err("SSO 账号未绑定系统用户，请联系管理员分配".to_string());
        }
        let username = if sso_user.username.is_empty() {
            sso_user.email.split('@').next().unwrap_or("sso_user").to_string()
        } else {
            sso_user.username.clone()
        };
        // 自动开通：无角色账号，随机密码（不可用密码登录，随机 64 位）
        let rand_pwd = gen_hex(64);
        let hashed = bcrypt::hash(&rand_pwd, bcrypt::DEFAULT_COST).map_err(|e| e.to_string())?;
        let row = admin::ActiveModel {
            user_name: Set(Some(username.clone())),
            password: Set(Some(hashed)),
            email: Set(Some(sso_user.email.clone())),
            status: Set(Some(1)),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        let new_id = admin::Entity::insert(row).exec(db).await.map_err(|e| format!("SSO 自动开通失败: {}", e))?.last_insert_id;
        admin_row = admin::Entity::find_by_id(new_id).one(db).await.map_err(|e| e.to_string())?;
    }

    let admin_row = admin_row.ok_or_else(|| "SSO 账号未匹配到系统用户".to_string())?;
    // 离职/停用拦截（R7：离职员工不得通过第三方登录进入）
    if admin_row.status != Some(1) {
        return Err("账号已被停用，请联系管理员".to_string());
    }
    Ok(SsoCallbackOutcome::Login(admin_row.id, admin_row.user_name.clone().unwrap_or_default(), admin_row.user_type == Some(1)))
}

/// 解析 SSO state：返回（是否绑定场景, 绑定目标 user_id）
fn parse_sso_state(state: &str) -> (bool, Option<i64>) {
    if let Some(rest) = state.strip_prefix("bind:") {
        if let Some(uid) = rest.splitn(2, ':').next().and_then(|s| s.parse::<i64>().ok()) {
            return (true, Some(uid));
        }
    }
    (false, None)
}

/// OIDC：code 换 token → userinfo（发现文档复用 1h 缓存）
async fn oidc_exchange_user(db: &DbConn, code: &str) -> std::result::Result<sso_provider::SsoProviderUser, String> {
    let issuer = sso_config_db(db, "sso_oidc_issuer").await.trim_end_matches('/').to_string();
    let client_id = sso_config_db(db, "sso_oidc_client_id").await;
    let client_secret = sso_config_db(db, "sso_oidc_client_secret").await;
    let disc_key = format!("sso:discovery:{}", issuer);
    let discovery = CONTEXT.cache_service.get_string(&disc_key).await.unwrap_or_default();
    let doc: serde_json::Value = serde_json::from_str(&discovery).map_err(|_| "OIDC 发现文档缺失，请重新发起登录".to_string())?;
    let token_endpoint = doc["token_endpoint"].as_str().ok_or_else(|| "OIDC 发现文档缺少 token_endpoint".to_string())?;
    let userinfo_endpoint = doc["userinfo_endpoint"].as_str().ok_or_else(|| "OIDC 发现文档缺少 userinfo_endpoint".to_string())?;
    let server_url = crate::config::section::<String>("server", "server_url", "http://localhost:8088".to_string());
    let redirect_uri = format!("{}/api/system/auth/sso/callback", server_url.trim_end_matches('/'));

    let client = reqwest::Client::new();
    let token_resp = client
        .post(token_endpoint)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri.as_str()),
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
        ])
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("OIDC 令牌交换失败: {}", e))?;
    if !token_resp.status().is_success() {
        return Err(format!("OIDC 令牌交换失败: HTTP {}", token_resp.status()));
    }
    let token_json: serde_json::Value = token_resp.json().await.map_err(|e| e.to_string())?;
    let access_token = token_json["access_token"]
        .as_str()
        .ok_or_else(|| "OIDC 令牌响应缺少 access_token".to_string())?;

    let userinfo: serde_json::Value = client
        .get(userinfo_endpoint)
        .bearer_auth(access_token)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("获取 OIDC 用户信息失败: {}", e))?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let email = userinfo["email"].as_str().unwrap_or_default().to_string();
    let preferred_username = userinfo["preferred_username"].as_str().unwrap_or_default().to_string();
    let sub = userinfo["sub"].as_str().unwrap_or("").to_string();
    let id = if sub.is_empty() { preferred_username.clone() } else { sub };
    Ok(sso_provider::SsoProviderUser {
        id,
        email,
        username: preferred_username,
    })
}

// ==================== 账号绑定（企业微信 / 钉钉）====================

/// 校验账号在职可用（存在、未删除、status=1）；离职/停用拒绝（R7）
///
/// 返回错误消息形如"账号已停用/离职，无法{action}"
async fn ensure_user_active(db: &DbConn, user_id: i64, action: &str) -> std::result::Result<(), String> {
    let user = admin::Entity::find()
        .filter(admin::Column::Id.eq(user_id))
        .filter(admin::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "账号不存在或已删除".to_string())?;
    if user.status != Some(1) {
        return Err(format!("账号已停用/离职，无法{}", action));
    }
    Ok(())
}

/// 已绑定列表（provider_uid 脱敏展示）
pub async fn sso_bind_list(db: &DbConn, user_id: i64) -> std::result::Result<Vec<SsoBindItemVO>, String> {
    let binds = user_bind::Entity::find()
        .filter(user_bind::Column::UserId.eq(user_id))
        .filter(user_bind::Column::Deleted.eq(0))
        .order_by_asc(user_bind::Column::Provider)
        .all(db)
        .await
        .map_err(|e| e.to_string())?;
    let mut items = Vec::new();
    for b in binds {
        items.push(SsoBindItemVO {
            provider: b.provider.clone().unwrap_or_default(),
            provider_uid: mask_uid(&b.provider_uid.unwrap_or_default()),
            created_at: b.created_at.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        });
    }
    Ok(items)
}

/// 脱敏第三方 ID（首 3 尾 3，其余掩码；过短全掩）
fn mask_uid(uid: &str) -> String {
    let chars: Vec<char> = uid.chars().collect();
    if chars.len() <= 6 {
        "******".to_string()
    } else {
        let head: String = chars[..3].iter().collect();
        let tail: String = chars[chars.len() - 3..].iter().collect();
        format!("{}******{}", head, tail)
    }
}

/// 发起绑定：构造扫码授权 URL（state 编码绑定目标 user_id，TTL 600s）
///
/// 返回授权 URL 交由前端跳转（浏览器跳转无法携带 Authorization 头，不走 302）。
pub async fn sso_bind_start(db: &DbConn, user_id: i64, provider: &str) -> std::result::Result<String, String> {
    let provider = normalize_provider(provider);
    if provider == "oidc" {
        return Err("OIDC 不支持账号绑定".to_string());
    }
    // R7: 离职/停用员工不可发起绑定
    ensure_user_active(db, user_id, "绑定").await?;
    let server_url = crate::config::section::<String>("server", "server_url", "http://localhost:8088".to_string());
    let redirect_uri = format!("{}/api/system/auth/sso/callback?provider={}", server_url.trim_end_matches('/'), provider);
    let state = format!("bind:{}:{}", user_id, gen_hex(32));
    CONTEXT
        .cache_service
        .set_string_ex(&format!("sso_state:{}", state), "1", Some(std::time::Duration::from_secs(SSO_STATE_TTL)))
        .await
        .map_err(|e| e.to_string())?;
    sso_provider::build_authorize_url(db, &provider, &redirect_uri, &state).await
}

/// 解绑（软删，幂等）
pub async fn sso_bind_unbind(db: &DbConn, user_id: i64, provider: &str) -> std::result::Result<(), String> {
    let provider = normalize_provider(provider);
    // R7: 离职/停用员工不可操作解绑
    ensure_user_active(db, user_id, "操作").await?;
    let bind = user_bind::Entity::find()
        .filter(user_bind::Column::UserId.eq(user_id))
        .filter(user_bind::Column::Provider.eq(provider.clone()))
        .filter(user_bind::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| e.to_string())?;
    if let Some(row) = bind {
        let am = user_bind::ActiveModel {
            id: Set(row.id),
            user_id: Set(row.user_id),
            provider: Set(row.provider),
            provider_uid: Set(row.provider_uid),
            created_at: Set(row.created_at),
            updated_at: Set(Some(chrono::Local::now().naive_local())),
            deleted: Set(Some(1)),
        };
        am.update(db).await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ==================== 角色登录策略（IP 白名单 / 登录时段）====================

/// 登录策略校验：取用户全部有效角色的限制，允许集取并集（任一角色放行即放行）
pub async fn check_role_login_policy(db: &DbConn, user_id: i64, client_ip: &str) -> std::result::Result<(), String> {
    let merges = crate::modules::system::model::admin_role_merge::AdminRoleMergeModel::find_by_admin_id(db, &Some(user_id))
        .await
        .unwrap_or_default();
    let mut ip_rules: Vec<String> = Vec::new();
    let mut hour_rules: Vec<String> = Vec::new();
    for m in merges {
        let Some(rid) = m.role_id else { continue };
        if let Ok(Some(r)) = role::Entity::find_by_id(rid).one(db).await {
            if r.status != Some(1) {
                continue;
            }
            if let Some(w) = r.ip_whitelist {
                if !w.trim().is_empty() {
                    ip_rules.extend(w.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()));
                }
            }
            if let Some(h) = r.login_hours {
                if !h.trim().is_empty() {
                    hour_rules.extend(h.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()));
                }
            }
        }
    }
    if !ip_rules.is_empty() && !ip_rules.iter().any(|rule| ip_matches(client_ip, rule)) {
        return Err("当前 IP 不在允许的登录范围内".to_string());
    }
    if !hour_rules.is_empty() && !hour_rules.iter().any(|rule| in_login_hours(rule)) {
        return Err("当前时间不在允许的登录时段内".to_string());
    }
    Ok(())
}

/// IP 匹配：支持精确 IP 与 IPv4 CIDR（如 192.168.1.0/24）
fn ip_matches(client_ip: &str, rule: &str) -> bool {
    let ip = client_ip.split(':').next().unwrap_or(client_ip); // 去端口
    if ip == rule {
        return true;
    }
    if let Some((net, mask)) = rule.split_once('/') {
        let (ip_u32, mask_u32) = match (parse_ipv4(ip), mask.parse::<u32>()) {
            (Some(a), Ok(m)) if m <= 32 => (a, m),
            _ => return false,
        };
        let net_u32 = match parse_ipv4(net) {
            Some(v) => v,
            None => return false,
        };
        let shift = 32 - mask_u32;
        (ip_u32 >> shift) == (net_u32 >> shift)
    } else {
        false
    }
}

fn parse_ipv4(s: &str) -> Option<u32> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return None;
    }
    let mut out: u32 = 0;
    for p in parts {
        let v: u32 = p.parse().ok()?;
        if v > 255 {
            return None;
        }
        out = (out << 8) | v;
    }
    Some(out)
}

/// 时段匹配：rule 形如 "09:00-18:00"，当前本地时间落入任一窗口即允许
fn in_login_hours(rule: &str) -> bool {
    use chrono::Timelike;
    let Some((start, end)) = rule.split_once('-') else { return false };
    let parse_hm = |s: &str| -> Option<u32> {
        let (h, m) = s.trim().split_once(':')?;
        Some(h.trim().parse::<u32>().ok()? * 60 + m.trim().parse::<u32>().ok()?)
    };
    let (Some(s), Some(e)) = (parse_hm(start), parse_hm(end)) else { return false };
    let now = chrono::Local::now().time();
    let cur = now.hour() * 60 + now.minute();
    if s <= e {
        cur >= s && cur <= e
    } else {
        cur >= s || cur <= e // 跨零点窗口
    }
}

/// 新设备登录提醒（有邮箱 + 配置开启时发送；设备指纹由前端持久化提供）
pub async fn remind_new_device(db: &DbConn, user_id: i64, device_id: &str) {
    let enabled = {
        let v = CONTEXT.cache_service.get_string("config:new_device_reminder").await.unwrap_or_default();
        if v.is_empty() {
            crate::modules::system::service::config_service::find_value_by_key_from_db("new_device_reminder")
                .await
                .unwrap_or_else(|| "1".to_string())
                == "1"
        } else {
            v == "1"
        }
    };
    if !enabled {
        return;
    }
    let Ok(admin) = admin_service::get_by_detail(db, &Some(user_id)).await else { return };
    let Some(email) = admin.email.filter(|e| !e.trim().is_empty()) else { return };
    let known_key = format!("device:{}", session_service::sha256_hex(&format!("{}:{}", user_id, device_id)));
    if !CONTEXT.cache_service.get_string(&known_key).await.unwrap_or_default().is_empty() {
        return; // 已知设备
    }
    let _ = CONTEXT.cache_service.set_string(&known_key, "1").await;
    let req = crate::modules::system::model::mail::SendMailRequest {
        customer_id: None,
        to_emails: vec![email],
        cc_emails: None,
        subject: Some("【Mxx CRM】新设备登录提醒".to_string()),
        body: Some(format!(
            "<p>您的账号于 {} 从新设备登录（IP: {}）。</p><p>若非本人操作，请立即修改密码并联系管理员。</p>",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            admin.login_ip.clone().unwrap_or_default()
        )),
        doc_url: None,
        contact_ids: None,
    };
    if let Err(e) = crate::modules::system::service::mail_service::send_mail(db, req, Some(user_id), None).await {
        log::warn!("[新设备提醒] 发送失败 user_id={}: {}", user_id, e);
    }
}
