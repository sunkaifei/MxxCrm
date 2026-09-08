//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 批2 认证安全：MFA / 找回密码 / 会话自管理 / API 令牌（PAT）/ SSO 请求响应 DTO

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MfaVerifyRequest {
    pub ticket: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MfaSetupRequest {
    /// 1=TOTP 2=邮箱验证码
    pub mfa_type: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MfaSetupVO {
    pub mfa_type: i32,
    /// TOTP：Base32 密钥（手动录入用）
    pub secret: Option<String>,
    /// TOTP：otpauth:// 链接（认证器扫码用）
    pub otpauth_url: Option<String>,
    /// 邮箱：脱敏收件地址
    pub email_masked: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MfaSetupConfirmRequest {
    pub mfa_type: i32,
    pub code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MfaDisableRequest {
    pub code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForgotPasswordVO {
    pub ticket: String,
    pub email_masked: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordRequest {
    pub ticket: String,
    pub code: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterEmailCodeRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatCreateRequest {
    pub name: String,
    /// 有效天数，NULL/0=永不过期
    pub expire_days: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatCreateVO {
    pub id: i64,
    /// 明文令牌（仅本次返回，服务端只存哈希）
    pub token: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatVO {
    pub id: i64,
    pub name: Option<String>,
    pub token_prefix: Option<String>,
    pub expire_time: Option<String>,
    pub last_used_at: Option<String>,
    pub status: Option<i32>,
    pub create_time: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatRevokeRequest {
    pub id: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionItemVO {
    /// 会话 accessToken（用户自己的会话，用于精确下线）
    pub token: String,
    /// 展示用脱敏串
    pub token_masked: String,
    pub is_current: bool,
    pub login_ip: Option<String>,
    pub login_time: Option<String>,
    pub expire_time: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRevokeRequest {
    pub token: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SsoStatusVO {
    /// 任一 provider 启用即 true（兼容旧前端）
    pub enabled: bool,
    pub providers: Vec<SsoProviderStatusVO>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SsoProviderStatusVO {
    /// wecom / dingtalk / oidc
    pub code: String,
    pub name: String,
    /// 必填字段是否配置齐全
    pub configured: bool,
    /// 是否启用（integration_config.enabled=1）
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SsoAuthorizeQuery {
    /// wecom / dingtalk / oidc，缺省=oidc（兼容旧调用）
    pub provider: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SsoBindItemVO {
    /// wecom / dingtalk
    pub provider: String,
    /// 第三方唯一 ID（脱敏展示）
    pub provider_uid: String,
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SsoBindUnbindRequest {
    pub provider: String,
}

/// SSO 回调结果（登录场景与绑定场景）
pub enum SsoCallbackOutcome {
    /// 登录成功（user_id, user_name, is_admin）
    Login(i64, String, bool),
    /// 绑定成功
    BindSuccess,
}
