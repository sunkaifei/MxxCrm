//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 统一安全验证码（OTP）服务
//!
//! 用于删除备份 / 数据还原 / 下载备份等破坏性或敏感操作的二次身份验证。
//!
//! 安全设计：
//! 1. 仅超级管理员（user_type=1）可发送与使用验证码
//! 2. 验证码为 6 位随机数字，发送至当前管理员绑定邮箱（复用 SMTP 邮件服务）
//! 3. 有效期 5 分钟，一次性使用（校验成功后立即失效）
//! 4. 60 秒内同一操作不可重复发送（防骚扰），错误 5 次自动失效（防爆破）
//! 5. 验证码仅存内存（OnceLock<Mutex>），不落库，服务重启即失效需重新获取

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use sea_orm::DatabaseConnection;

use crate::modules::system::model::admin::AdminModel;
use crate::modules::system::model::mail::SendMailRequest;
use crate::modules::system::service::mail_service;

/// 验证码条目
struct OtpEntry {
    /// 验证码（明文，仅内存）
    code: String,
    /// 发送时刻（用于有效期与重发间隔）
    sent_at: Instant,
    /// 已错误尝试次数
    tries: u32,
}

/// 验证码缓存：key = (admin_id, action)
static OTP_CACHE: OnceLock<Mutex<HashMap<(i64, String), OtpEntry>>> = OnceLock::new();

fn otp_cache() -> &'static Mutex<HashMap<(i64, String), OtpEntry>> {
    OTP_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 验证码有效期（秒）
const OTP_TTL_SECS: u64 = 300;
/// 同一操作重发间隔（秒）
const RESEND_INTERVAL_SECS: u64 = 60;
/// 最大错误次数
const MAX_TRIES: u32 = 5;

/// 支持的操作
const SUPPORTED_ACTIONS: &[&str] = &["delete", "restore", "download"];

/// 生成 6 位随机数字验证码（线性同余，避免额外依赖）
fn gen_code() -> String {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x1234_5678);
    let mut x = seed | 1;
    let mut out = String::with_capacity(6);
    for _ in 0..6 {
        x = x.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        out.push((b'0' + ((x >> 16) % 10) as u8) as char);
    }
    out
}

/// 脱敏邮箱：a***b@domain.com
fn mask_email(email: &str) -> String {
    let (local, domain) = match email.split_once('@') {
        Some((l, d)) => (l, d),
        None => return email.to_string(),
    };
    if local.len() <= 1 {
        return format!("{}***@{}", local, domain);
    }
    let first = &local[..1];
    let last = &local[local.len() - 1..];
    format!("{}***{}@{}", first, last, domain)
}

/// 操作中文名
fn action_label(action: &str) -> &'static str {
    match action {
        "delete" => "删除备份",
        "restore" => "数据还原",
        "download" => "下载备份",
        _ => "安全验证",
    }
}

/// 发送验证码到当前超管绑定邮箱，返回脱敏邮箱用于前端展示
pub async fn send(db: &DatabaseConnection, admin_id: i64, action: &str) -> Result<String, String> {
    if !SUPPORTED_ACTIONS.contains(&action) {
        return Err("不支持的验证码用途".to_string());
    }
    let admin = AdminModel::find_by_id(db, &Some(admin_id))
        .await
        .map_err(|e| format!("查询用户信息失败: {}", e))?
        .ok_or_else(|| "当前登录用户不存在".to_string())?;
    if admin.user_type != Some(1) {
        return Err("仅超级管理员可执行此操作".to_string());
    }
    let email = admin.email.clone().unwrap_or_default();
    if email.trim().is_empty() {
        return Err("当前账号未绑定邮箱，无法发送验证码，请在「系统管理→用户管理」中完善邮箱".to_string());
    }

    // 60 秒重发限制
    {
        let cache = otp_cache().lock().unwrap();
        if let Some(e) = cache.get(&(admin_id, action.to_string())) {
            let elapsed = e.sent_at.elapsed().as_secs();
            if elapsed < RESEND_INTERVAL_SECS {
                return Err(format!(
                    "验证码发送过于频繁，请 {} 秒后再试",
                    RESEND_INTERVAL_SECS - elapsed
                ));
            }
        }
    }

    let code = gen_code();
    let label = action_label(action);
    let req = SendMailRequest {
        customer_id: None,
        to_emails: vec![email.clone()],
        cc_emails: None,
        subject: Some(format!("【Mxx CRM】{}安全验证码", label)),
        body: Some(format!(
            "<p>您正在执行「{}」操作。</p><p>本次安全验证码为：<b style=\"font-size:20px\">{}</b></p><p>验证码 5 分钟内有效且仅可使用一次，请勿向他人泄露。若非本人操作，请立即修改登录密码。</p>",
            label, code
        )),
        doc_url: None,
        contact_ids: None,
    };
    let sender_name = admin.nick_name.clone().unwrap_or_else(|| "系统".to_string());
    mail_service::send_mail(db, req, Some(admin_id), Some(sender_name))
        .await
        .map_err(|e| format!("验证码邮件发送失败: {}", e))?;

    // 写入缓存（先移除旧条目，保证一次只有一个有效码）
    {
        let mut cache = otp_cache().lock().unwrap();
        cache.insert(
            (admin_id, action.to_string()),
            OtpEntry { code, sent_at: Instant::now(), tries: 0 },
        );
        if cache.len() > 500 {
            cache.retain(|_, e| e.sent_at.elapsed().as_secs() <= OTP_TTL_SECS);
        }
    }
    Ok(mask_email(&email))
}

/// 发送验证码到指定邮箱（账号安全场景：email_old 验证旧邮箱 / email_new 验证新邮箱）
///
/// 与 `send` 的区别：
/// 1. 不限制仅超管（任意已登录用户修改自己的邮箱都需要验证码）
/// 2. 收件邮箱由调用方指定（新邮箱验证码要发到用户填的新邮箱）
/// 3. 重发限制 / 有效期 / 错误次数 / 一次性消费规则与 `send` 完全一致
pub async fn send_to_email(
    db: &DatabaseConnection,
    admin_id: i64,
    action: &str,
    target_email: &str,
) -> Result<String, String> {
    if !["email_old", "email_new"].contains(&action) {
        return Err("不支持的验证码用途".to_string());
    }
    let email = target_email.trim().to_string();
    if email.is_empty() || !email.contains('@') {
        return Err("收件邮箱格式不正确".to_string());
    }

    // 60 秒重发限制
    {
        let cache = otp_cache().lock().unwrap();
        if let Some(e) = cache.get(&(admin_id, action.to_string())) {
            let elapsed = e.sent_at.elapsed().as_secs();
            if elapsed < RESEND_INTERVAL_SECS {
                return Err(format!(
                    "验证码发送过于频繁，请 {} 秒后再试",
                    RESEND_INTERVAL_SECS - elapsed
                ));
            }
        }
    }

    let code = gen_code();
    let label = match action {
        "email_old" => "旧邮箱验证",
        "email_new" => "新邮箱验证",
        _ => "安全验证",
    };
    let req = SendMailRequest {
        customer_id: None,
        to_emails: vec![email.clone()],
        cc_emails: None,
        subject: Some(format!("【Mxx CRM】{}安全验证码", label)),
        body: Some(format!(
            "<p>您正在执行「{}」操作。</p><p>本次安全验证码为：<b style=\"font-size:20px\">{}</b></p><p>验证码 5 分钟内有效且仅可使用一次，请勿向他人泄露。若非本人操作，请立即修改登录密码。</p>",
            label, code
        )),
        doc_url: None,
        contact_ids: None,
    };
    mail_service::send_mail(db, req, Some(admin_id), None)
        .await
        .map_err(|e| format!("验证码邮件发送失败: {}", e))?;

    // 写入缓存（先移除旧条目，保证一次只有一个有效码）
    {
        let mut cache = otp_cache().lock().unwrap();
        cache.insert(
            (admin_id, action.to_string()),
            OtpEntry { code, sent_at: Instant::now(), tries: 0 },
        );
        if cache.len() > 500 {
            cache.retain(|_, e| e.sent_at.elapsed().as_secs() <= OTP_TTL_SECS);
        }
    }
    Ok(mask_email(&email))
}

/// 校验验证码（一次性消费；错误 5 次自动失效）
pub fn verify(admin_id: i64, action: &str, code: &str) -> Result<(), String> {
    let mut cache = otp_cache().lock().unwrap();
    let key = (admin_id, action.to_string());
    let Some(entry) = cache.get_mut(&key) else {
        return Err("验证码不存在或已过期，请重新获取".to_string());
    };
    if entry.sent_at.elapsed().as_secs() > OTP_TTL_SECS {
        cache.remove(&key);
        return Err("验证码已过期，请重新获取".to_string());
    }
    if entry.tries >= MAX_TRIES {
        cache.remove(&key);
        return Err("验证码错误次数过多已失效，请重新获取".to_string());
    }
    if entry.code != code.trim() {
        entry.tries += 1;
        let left = MAX_TRIES - entry.tries;
        return Err(if left == 0 {
            "验证码错误次数过多已失效，请重新获取".to_string()
        } else {
            format!("验证码错误，还可尝试 {} 次", left)
        });
    }
    cache.remove(&key);
    Ok(())
}
