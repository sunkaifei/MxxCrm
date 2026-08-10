//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crypto::digest::Digest;
use crypto::md5::Md5;
use crypto::sha1::Sha1;
use uuid::Uuid;
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use base64::{Engine as _, engine::general_purpose};

// md5
pub fn md5(data: &str) -> String {
    let mut h = Md5::new();
    h.input_str(data);
    h.result_str()
}

// md5 for binary data
pub fn md5_bytes(data: &[u8]) -> String {
    let mut h = Md5::new();
    h.input(data);
    h.result_str()
}

// sha1
pub fn sha1(data: &str) -> String {
    let mut h = Sha1::new();
    h.input_str(data);
    h.result_str()
}

pub fn uuid() -> String {
    Uuid::new_v4().to_string()
}

/// AES-256-GCM 密钥（从固定 secret 派生 32 字节）
const CARD_ENCRYPT_SECRET: &str = "mxxcrm_card_key_v1_secret_2026";

fn card_key_bytes() -> [u8; 32] {
    let h = md5(CARD_ENCRYPT_SECRET);
    let h2 = md5(&format!("{}{}", h, CARD_ENCRYPT_SECRET));
    let mut k = [0u8; 32];
    let bytes: Vec<u8> = (h + &h2).bytes().collect();
    for (i, b) in bytes.iter().enumerate().take(32) {
        k[i] = *b;
    }
    k
}

/// AES-256-GCM 加密，返回 base64
pub fn encrypt_card(plaintext: &str) -> String {
    let key_bytes = card_key_bytes();
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce_source = md5("card_nonce");
    let mut nonce_arr = [0u8; 12];
    for (i, b) in nonce_source.bytes().enumerate().take(12) {
        nonce_arr[i] = b;
    }
    let nonce = Nonce::from_slice(&nonce_arr);
    match cipher.encrypt(nonce, plaintext.as_bytes()) {
        Ok(ct) => general_purpose::STANDARD.encode(ct),
        Err(_) => plaintext.to_string(),
    }
}

/// AES-256-GCM 解密
pub fn decrypt_card(ciphertext_b64: &str) -> String {
    let key_bytes = card_key_bytes();
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce_source = md5("card_nonce");
    let mut nonce_arr = [0u8; 12];
    for (i, b) in nonce_source.bytes().enumerate().take(12) {
        nonce_arr[i] = b;
    }
    let nonce = Nonce::from_slice(&nonce_arr);
    match general_purpose::STANDARD.decode(ciphertext_b64) {
        Ok(ct) => match cipher.decrypt(nonce, ct.as_ref()) {
            Ok(pt) => String::from_utf8_lossy(&pt).to_string(),
            Err(_) => ciphertext_b64.to_string(),
        },
        Err(_) => ciphertext_b64.to_string(),
    }
}









