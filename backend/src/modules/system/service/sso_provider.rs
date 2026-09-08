//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 第三方登录 Provider 适配层：企业微信 / 钉钉
//!
//! 职责边界：只处理"协议差异"（构造授权 URL、code 换 token、取用户信息），
//! 不做账号匹配/离职判断/会话签发（交给 auth 模块 SSO 回调链路）。
//! 配置读取自 integration_config（auth 分类），敏感字段自动解密。

use sea_orm::DbConn;
use serde_json::Value;

use crate::modules::system::service::integration_config_service;

/// 统一 Provider 用户信息
pub struct SsoProviderUser {
    /// 第三方唯一 ID（企业微信 userid / 钉钉 unionId）
    pub id: String,
    /// 邮箱（企业微信默认无，需后台配置）
    pub email: String,
    /// 用户名（优先手机号/邮箱前缀，兜底第三方 ID）
    pub username: String,
}

/// 企业微信配置（auth 分类 integration_code=wecom）
struct WecomConfig {
    corp_id: String,
    agent_id: String,
    corp_secret: String,
}

/// 钉钉配置（auth 分类 integration_code=dingtalk）
struct DingtalkConfig {
    app_key: String,
    app_secret: String,
}

async fn get_config_json(db: &DbConn, code: &str) -> Result<Value, String> {
    let config = integration_config_service::get_by_code(db, code)
        .await
        .map_err(|e| e.to_string())?;
    let config = config.ok_or_else(|| format!("{} 未配置（请先在第三方接口配置中完善）", code))?;
    if config.enabled != Some(1) {
        return Err(format!("{} 未启用", code));
    }
    Ok(config.config_json.clone().unwrap_or(Value::Null))
}

fn get_required(json: &Value, keys: &[&str]) -> Result<Vec<String>, String> {
    let mut vals = Vec::new();
    for k in keys {
        let v = json
            .get(*k)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if v.is_empty() {
            return Err(format!("{} 未配置", *k));
        }
        vals.push(v);
    }
    Ok(vals)
}

async fn wecom_config(db: &DbConn) -> Result<WecomConfig, String> {
    let json = get_config_json(db, "wecom").await?;
    let vals = get_required(&json, &["corp_id", "agent_id", "corp_secret"])?;
    Ok(WecomConfig {
        corp_id: vals[0].clone(),
        agent_id: vals[1].clone(),
        corp_secret: vals[2].clone(),
    })
}

async fn dingtalk_config(db: &DbConn) -> Result<DingtalkConfig, String> {
    let json = get_config_json(db, "dingtalk").await?;
    let vals = get_required(&json, &["app_key", "app_secret"])?;
    Ok(DingtalkConfig {
        app_key: vals[0].clone(),
        app_secret: vals[1].clone(),
    })
}

/// URL 编码（SSO 重定向参数用）
fn urlenc(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "%20".to_string(),
            ':' => "%3A".to_string(),
            '/' => "%2F".to_string(),
            '?' => "%3F".to_string(),
            '&' => "%26".to_string(),
            '=' => "%3D".to_string(),
            '#' => "%23".to_string(),
            '%' => "%25".to_string(),
            '+' => "%2B".to_string(),
            '@' => "%40".to_string(),
            c => c.to_string(),
        })
        .collect()
}

/// 共享 HTTP 客户端（连接池复用，避免每次扫码登录重复 TLS 握手）
fn http_client() -> &'static reqwest::Client {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    CLIENT.get_or_init(reqwest::Client::new)
}

/// 构造第三方扫码授权 URL
///
/// provider: wecom / dingtalk；redirect_uri 为本系统回调地址；state 原样带回（防 CSRF）。
pub async fn build_authorize_url(
    db: &DbConn,
    provider: &str,
    redirect_uri: &str,
    state: &str,
) -> Result<String, String> {
    match provider {
        "wecom" => {
            let cfg = wecom_config(db).await?;
            // 企业微信应用扫码登录（独立窗口二维码模式）
            Ok(format!(
                "https://login.work.weixin.qq.com/wwlogin/sso/login?login_type=CorpApp&appid={}&agentid={}&redirect_uri={}&state={}",
                urlenc(&cfg.corp_id),
                urlenc(&cfg.agent_id),
                urlenc(redirect_uri),
                urlenc(state),
            ))
        }
        "dingtalk" => {
            let cfg = dingtalk_config(db).await?;
            // 钉钉 OAuth2 扫码授权
            Ok(format!(
                "https://login.dingtalk.com/oauth2/auth?redirect_uri={}&response_type=code&client_id={}&scope=openid&state={}&prompt=consent",
                urlenc(redirect_uri),
                urlenc(&cfg.app_key),
                urlenc(state),
            ))
        }
        other => Err(format!("不支持的 provider: {}", other)),
    }
}

/// 企业 access_token（2h 有效，Redis 缓存避免高频 gettoken）
async fn wecom_corp_access_token(_db: &DbConn, cfg: &WecomConfig) -> Result<String, String> {
    let cache_key = format!("sso:wecom:corp_at:{}", cfg.corp_id);
    if let Ok(v) = crate::core::kit::CONTEXT.cache_service.get_string(&cache_key).await {
        if !v.is_empty() {
            return Ok(v);
        }
    }
    let client = http_client();
    let resp = client
        .get("https://qyapi.weixin.qq.com/cgi-bin/gettoken")
        .query(&[("corpid", &cfg.corp_id), ("corpsecret", &cfg.corp_secret)])
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("企业微信 gettoken 请求失败: {}", e))?;
    let json: Value = resp.json().await.map_err(|e| e.to_string())?;
    let errcode = json["errcode"].as_i64().unwrap_or(-1);
    if errcode != 0 {
        return Err(format!(
            "企业微信 gettoken 失败: {} {}",
            errcode,
            json["errmsg"].as_str().unwrap_or("")
        ));
    }
    let token = json["access_token"]
        .as_str()
        .ok_or_else(|| "企业微信 gettoken 响应缺少 access_token".to_string())?
        .to_string();
    let _ = crate::core::kit::CONTEXT
        .cache_service
        // 提前 200s 过期（官方有效期 7200s），避免边界时刻用到刚失效的 token
        .set_string_ex(&cache_key, &token, Some(std::time::Duration::from_secs(7000)))
        .await;
    Ok(token)
}

/// code 换 token 并取用户信息（统一出口）
pub async fn exchange_and_fetch_user(
    db: &DbConn,
    provider: &str,
    code: &str,
) -> Result<SsoProviderUser, String> {
    match provider {
        "wecom" => wecom_fetch_user(db, code).await,
        "dingtalk" => dingtalk_fetch_user(db, code).await,
        other => Err(format!("不支持的 provider: {}", other)),
    }
}

async fn wecom_fetch_user(db: &DbConn, code: &str) -> Result<SsoProviderUser, String> {
    let cfg = wecom_config(db).await?;
    let access_token = wecom_corp_access_token(db, &cfg).await?;
    let client = http_client();

    // 用 code 换成员 UserID
    let resp = client
        .get("https://qyapi.weixin.qq.com/cgi-bin/auth/getuserinfo")
        .query(&[("access_token", access_token.as_str()), ("code", code)])
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("企业微信 getuserinfo 请求失败: {}", e))?;
    let json: Value = resp.json().await.map_err(|e| e.to_string())?;
    let errcode = json["errcode"].as_i64().unwrap_or(-1);
    if errcode != 0 {
        return Err(format!(
            "企业微信 getuserinfo 失败: {} {}",
            errcode,
            json["errmsg"].as_str().unwrap_or("")
        ));
    }
    let userid = json["userid"]
        .as_str()
        .ok_or_else(|| "企业微信 getuserinfo 响应缺少 userid".to_string())?
        .to_string();

    // 取成员详情（手机号/邮箱/姓名）
    let mut email = String::new();
    let mut mobile = String::new();
    let mut name = String::new();
    let detail = client
        .get("https://qyapi.weixin.qq.com/cgi-bin/user/get")
        .query(&[("access_token", &access_token), ("userid", &userid)])
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;
    if let Ok(resp) = detail {
        if let Ok(json) = resp.json::<Value>().await {
            if json["errcode"].as_i64().unwrap_or(-1) == 0 {
                email = json["email"].as_str().unwrap_or("").to_string();
                mobile = json["mobile"].as_str().unwrap_or("").to_string();
                name = json["name"].as_str().unwrap_or("").to_string();
            }
        }
    }
    let username = if !mobile.is_empty() {
        mobile.clone()
    } else if !email.is_empty() {
        email.split('@').next().unwrap_or(&userid).to_string()
    } else if !name.is_empty() {
        name
    } else {
        userid.clone()
    };
    Ok(SsoProviderUser {
        id: userid,
        email,
        username,
    })
}

async fn dingtalk_fetch_user(db: &DbConn, code: &str) -> Result<SsoProviderUser, String> {
    let cfg = dingtalk_config(db).await?;
    let client = http_client();

    // code 换用户 access_token
    let resp = client
        .post("https://api.dingtalk.com/v1.0/oauth2/userAccessToken")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "clientId": cfg.app_key,
            "clientSecret": cfg.app_secret,
            "code": code,
            "grantType": "authorization_code",
        }))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("钉钉 userAccessToken 请求失败: {}", e))?;
    let status = resp.status();
    let json: Value = resp.json().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!(
            "钉钉 userAccessToken 失败: {} {}",
            status,
            json["message"].as_str().unwrap_or("")
        ));
    }
    let access_token = json["accessToken"]
        .as_str()
        .ok_or_else(|| "钉钉 userAccessToken 响应缺少 accessToken".to_string())?;

    // 取当前用户信息
    let resp = client
        .get("https://api.dingtalk.com/v1.0/contact/users/me")
        .header("x-acs-dingtalk-access-token", access_token)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("钉钉获取用户信息请求失败: {}", e))?;
    let json: Value = resp.json().await.map_err(|e| e.to_string())?;
    let union_id = json["unionId"]
        .as_str()
        .ok_or_else(|| "钉钉用户信息响应缺少 unionId".to_string())?
        .to_string();
    let nick = json["nick"].as_str().unwrap_or("").to_string();
    let mobile = json["mobile"].as_str().unwrap_or("").to_string();
    let email = json["email"].as_str().unwrap_or("").to_string();
    let username = if !mobile.is_empty() {
        mobile
    } else if !email.is_empty() {
        email.split('@').next().unwrap_or(&union_id).to_string()
    } else if !nick.is_empty() {
        nick
    } else {
        union_id.clone()
    };
    Ok(SsoProviderUser {
        id: union_id,
        email,
        username,
    })
}
