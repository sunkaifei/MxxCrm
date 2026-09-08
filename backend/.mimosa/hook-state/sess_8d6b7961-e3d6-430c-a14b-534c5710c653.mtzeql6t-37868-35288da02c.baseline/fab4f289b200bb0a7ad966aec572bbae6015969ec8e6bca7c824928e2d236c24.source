//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! TOTP（RFC 6238，SHA1/6位/30秒步长）纯 Rust 实现（批2 MFA）
//!
//! 零新增依赖：HMAC-SHA1 走 openssl，Base32 手写实现。
//! 与 Google Authenticator / Microsoft Authenticator 兼容。

use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::sign::Signer;

const BASE32_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

/// 生成随机 MFA 密钥并编码为 Base32（20 字节 → 32 字符）
pub fn generate_secret() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..20).map(|_| rng.gen_range(0..=255)).collect();
    base32_encode(&bytes)
}

/// Base32 编码（RFC 4648，无填充）
pub fn base32_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity((data.len() * 8 + 4) / 5);
    let mut buffer: u32 = 0;
    let mut bits = 0u32;
    for &b in data {
        buffer = (buffer << 8) | b as u32;
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            out.push(BASE32_ALPHABET[((buffer >> bits) & 0x1f) as usize] as char);
        }
    }
    if bits > 0 {
        out.push(BASE32_ALPHABET[((buffer << (5 - bits)) & 0x1f) as usize] as char);
    }
    out
}

/// Base32 解码（RFC 4648，忽略填充与非法字符）
pub fn base32_decode(encoded: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut buffer: u32 = 0;
    let mut bits = 0u32;
    for ch in encoded.to_ascii_uppercase().chars() {
        if ch == '=' {
            continue;
        }
        let val = BASE32_ALPHABET.iter().position(|&c| c as char == ch)? as u32;
        buffer = (buffer << 5) | val;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push(((buffer >> bits) & 0xff) as u8);
        }
    }
    Some(out)
}

/// 计算指定时刻的 TOTP 码
pub fn generate_totp(secret_base32: &str, unix_time_secs: u64) -> Result<String, String> {
    let step: u64 = 30;
    let digits = 6usize;
    let key = base32_decode(secret_base32).ok_or_else(|| "MFA密钥不是合法的Base32".to_string())?;
    let counter = (unix_time_secs / step).to_be_bytes();

    let pkey = PKey::hmac(&key).map_err(|e| format!("HMAC初始化失败: {}", e))?;
    let mut signer = Signer::new(MessageDigest::sha1(), &pkey)
        .map_err(|e| format!("HMAC签名失败: {}", e))?;
    signer
        .update(&counter)
        .map_err(|e| format!("HMAC更新失败: {}", e))?;
    let hmac = signer.sign_to_vec().map_err(|e| format!("HMAC完成失败: {}", e))?;

    let offset = (*hmac.last().unwrap_or(&0) & 0x0f) as usize;
    let code = ((hmac[offset] as u32) << 24
        | (hmac[offset + 1] as u32) << 16
        | (hmac[offset + 2] as u32) << 8
        | (hmac[offset + 3] as u32))
        & 0x7fff_ffff;
    Ok(format!("{:0width$}", code % 1_000_000, width = digits))
}

/// 校验 TOTP 码（允许相邻一个步长 ±30s，容忍时钟偏差）
pub fn verify_totp(secret_base32: &str, code: &str, unix_time_secs: u64) -> bool {
    if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    for drift in [0i64, -1, 1] {
        let t = (unix_time_secs as i64 + drift * 30).max(0) as u64;
        if let Ok(expected) = generate_totp(secret_base32, t) {
            if expected == code {
                return true;
            }
        }
    }
    false
}

/// 构造 otpauth:// 迁移链接（供认证器 App 扫码/手动添加）
pub fn build_otpauth_url(secret_base32: &str, account: &str, issuer: &str) -> String {
    format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm=SHA1&digits=6&period=30",
        urlencoding_min(issuer),
        urlencoding_min(account),
        secret_base32,
        urlencoding_min(issuer)
    )
}

/// 极简 URL 编码（仅处理 otpauth 常见特殊字符，避免引入新依赖）
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

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 6238 测试向量（SHA1）：seed "12345678901234567890"，T=59s → code 287082
    #[test]
    fn test_rfc6238_vector() {
        // "12345678901234567890" 的 Base32
        let secret = base32_encode(b"12345678901234567890");
        assert_eq!(secret, "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ");
        let code = generate_totp(&secret, 59).unwrap();
        assert_eq!(code, "287082");
    }

    #[test]
    fn test_base32_roundtrip() {
        let data = b"MxxCrm-2026-secret!";
        let encoded = base32_encode(data);
        let decoded = base32_decode(&encoded).unwrap();
        assert_eq!(decoded, data.to_vec());
    }

    #[test]
    fn test_verify_window() {
        let secret = generate_secret();
        let code = generate_totp(&secret, 1_000_000).unwrap();
        assert!(verify_totp(&secret, &code, 1_000_000));
        assert!(verify_totp(&secret, &code, 1_000_030)); // 下一窗口容忍
        assert!(!verify_totp(&secret, "000000", 1_000_000) || code == "000000");
    }
}
