//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use reqwest::Client;
use serde::{Serialize, Deserialize};
use crate::core::errors::error::{Error, Result};
use crate::config;
use std::collections::BTreeMap;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rand::Rng;
use std::fs;
use openssl::rsa::Rsa;
use openssl::pkey::PKey;
use openssl::hash::MessageDigest;
use openssl::sign::{Signer, Verifier};
use openssl::x509::X509;

#[derive(Debug, Serialize, Deserialize)]
pub struct WechatPayResponse {
    pub app_id: String,
    pub time_stamp: String,
    pub nonce_str: String,
    pub package: String,
    pub sign_type: String,
    pub pay_sign: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WechatV3PrepayRequest {
    appid: String,
    mchid: String,
    description: String,
    out_trade_no: String,
    notify_url: String,
    amount: V3Amount,
    payer: V3Payer,
}

#[derive(Debug, Serialize, Deserialize)]
struct V3Amount {
    total: i32,
    currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct V3Payer {
    openid: String,
}

#[derive(Debug, Deserialize)]
struct WechatV3PrepayResponse {
    prepay_id: String,
}

fn generate_nonce_str() -> String {
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();
    let mut rng = rand::thread_rng();
    (0..32).map(|_| chars[rng.gen_range(0..chars.len())]).collect()
}

pub fn generate_order_id() -> String {
    let now = chrono::Local::now();
    let timestamp = now.format("%Y%m%d%H%M%S").to_string();
    let mut rng = rand::thread_rng();
    let rand_str: String = (0..6).map(|_| rng.gen_range(0..10).to_string()).collect();
    format!("MXX{}{}", timestamp, rand_str)
}

fn get_project_root() -> std::path::PathBuf {
    std::env::current_exe()
        .expect("无法获取可执行文件路径")
        .parent()
        .expect("无法获取可执行文件所在目录")
        .to_path_buf()
}

fn load_private_key() -> Result<PKey<openssl::pkey::Private>> {
    let key_path = config::section::<String>("wechat", "key_path", "pay/apiclient_key.pem".to_string());
    let full_path = if std::path::Path::new(&key_path).is_absolute() {
        std::path::PathBuf::from(&key_path)
    } else {
        get_project_root().join(&key_path)
    };
    log::info!("[微信支付V3] 尝试读取私钥: {:?}", full_path);
    let key_pem = fs::read_to_string(&full_path)
        .map_err(|e| Error::from(format!("读取私钥文件失败: {:?} - {}", full_path, e)))?;
    
    let rsa = Rsa::private_key_from_pem(key_pem.as_bytes())
        .map_err(|e| Error::from(format!("解析私钥失败: {}", e)))?;
    
    PKey::from_rsa(rsa)
        .map_err(|e| Error::from(format!("转换私钥失败: {}", e)))
}

fn get_certificate_serial_no() -> Result<String> {
    let cert_path = config::section::<String>("wechat", "cert_path", "pay/apiclient_cert.pem".to_string());
    let full_path = if std::path::Path::new(&cert_path).is_absolute() {
        std::path::PathBuf::from(&cert_path)
    } else {
        get_project_root().join(&cert_path)
    };
    log::info!("[微信支付V3] 尝试读取证书: {:?}", full_path);
    let cert_pem = fs::read_to_string(&full_path)
        .map_err(|e| Error::from(format!("读取证书文件失败: {:?} - {}", full_path, e)))?;
    
    let x509 = openssl::x509::X509::from_pem(cert_pem.as_bytes())
        .map_err(|e| Error::from(format!("解析证书失败: {}", e)))?;
    
    let serial_bn = x509.serial_number().to_bn()
        .map_err(|e| Error::from(format!("获取序列号失败: {}", e)))?;
    
    Ok(serial_bn.to_hex_str()
        .map_err(|e| Error::from(format!("转换序列号失败: {}", e)))?
        .to_string()
        .to_uppercase())
}

fn load_wechat_platform_public_key() -> Result<PKey<openssl::pkey::Public>> {
    let cert_path = config::section::<String>("wechat", "platform_cert_path", "pay/wechat_platform_cert.pem".to_string());
    let full_path = if std::path::Path::new(&cert_path).is_absolute() {
        std::path::PathBuf::from(&cert_path)
    } else {
        get_project_root().join(&cert_path)
    };
    log::info!("[微信支付V3] 尝试读取平台公钥: {:?}", full_path);
    let cert_pem = fs::read_to_string(&full_path)
        .map_err(|e| Error::from(format!("读取平台公钥失败: {:?} - {}", full_path, e)))?;
    
    let x509 = X509::from_pem(cert_pem.as_bytes())
        .map_err(|e| Error::from(format!("解析平台公钥失败: {}", e)))?;
    
    let public_key = x509.public_key()
        .map_err(|e| Error::from(format!("提取公钥失败: {}", e)))?;
    
    Ok(public_key)
}

pub fn verify_wechat_v3_notify(
    timestamp: &str,
    nonce: &str,
    body: &str,
    signature: &str,
    serial: &str,
) -> Result<bool> {
    log::info!("[微信支付V3] 验证回调签名 - timestamp={}, nonce={}, serial={}", timestamp, nonce, serial);
    
    let timestamp_num: i64 = timestamp.parse()
        .map_err(|e| Error::from(format!("时间戳格式错误: {}", e)))?;
    
    let now = chrono::Utc::now().timestamp();
    if (now - timestamp_num).abs() > 300 {
        log::warn!("[微信支付V3] 回调时间戳过期: {}", timestamp);
        return Ok(false);
    }
    
    let public_key = load_wechat_platform_public_key()?;
    
    let message = format!("{}\n{}\n{}\n", timestamp, nonce, body);
    log::info!("[微信支付V3] 签名消息: {}", message);
    
    let mut verifier = Verifier::new(MessageDigest::sha256(), &public_key)
        .map_err(|e| Error::from(format!("创建验证器失败: {}", e)))?;
    
    verifier.update(message.as_bytes())
        .map_err(|e| Error::from(format!("更新验证数据失败: {}", e)))?;
    
    let signature_bytes = BASE64.decode(signature)
        .map_err(|e| Error::from(format!("Base64解码签名失败: {}", e)))?;
    
    let result = verifier.verify(&signature_bytes)
        .map_err(|e| Error::from(format!("验证签名失败: {}", e)))?;
    
    log::info!("[微信支付V3] 签名验证结果: {}", result);
    Ok(result)
}

fn generate_v3_sign(
    method: &str,
    url: &str,
    timestamp: &str,
    nonce_str: &str,
    body: &str,
    private_key: &PKey<openssl::pkey::Private>,
) -> Result<String> {
    let sign_str = format!("{}\n{}\n{}\n{}\n{}\n", method, url, timestamp, nonce_str, body);
    
    let mut signer = Signer::new(MessageDigest::sha256(), private_key)
        .map_err(|e| Error::from(format!("创建签名器失败: {}", e)))?;
    
    signer.update(sign_str.as_bytes())
        .map_err(|e| Error::from(format!("签名数据失败: {}", e)))?;
    
    let signature = signer.sign_to_vec()
        .map_err(|e| Error::from(format!("生成签名失败: {}", e)))?;
    
    Ok(BASE64.encode(signature))
}

pub async fn create_member_experience_order(
    _user_id: i64,
    order_id: &str,
    openid: &str,
    client_ip: &str,
    amount: i32,
) -> Result<WechatPayResponse> {
    let app_id = config::section::<String>("wechat", "app_id", "".to_string());
    let mch_id = config::section::<String>("wechat", "mchid", "".to_string());
    
    if app_id.is_empty() || mch_id.is_empty() {
        return Err(Error::from("微信支付配置未设置"));
    }

    let server_url = config::section::<String>("server", "server_url", "http://localhost:8088".to_string());
    let notify_url = format!("{}/api/finance/payment/wechat-notify", server_url);

    let nonce_str = generate_nonce_str();
    let timestamp = chrono::Utc::now().timestamp().to_string();
    
    let prepay_request = WechatV3PrepayRequest {
        appid: app_id.clone(),
        mchid: mch_id.clone(),
        description: "会员购买".to_string(),
        out_trade_no: order_id.to_string(),
        notify_url,
        amount: V3Amount {
            total: amount,
            currency: "CNY".to_string(),
        },
        payer: V3Payer {
            openid: openid.to_string(),
        },
    };
    
    let body = serde_json::to_string(&prepay_request)
        .map_err(|e| Error::from(format!("序列化请求失败: {}", e)))?;
    
    log::info!("[微信支付V3] 请求数据: {}", body);
    
    let private_key = load_private_key()?;
    let url = "/v3/pay/transactions/jsapi";
    let sign = generate_v3_sign("POST", url, &timestamp, &nonce_str, &body, &private_key)?;
    let serial_no = get_certificate_serial_no()?;
    
    let client = Client::new();
    let response = client.post(&format!("https://api.mch.weixin.qq.com{}", url))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .header("Authorization", format!(
            "WECHATPAY2-SHA256-RSA2048 mchid=\"{}\",nonce_str=\"{}\",signature=\"{}\",timestamp=\"{}\",serial_no=\"{}\"",
            mch_id, nonce_str, sign, timestamp, serial_no
        ))
        .body(body)
        .send()
        .await
        .map_err(|e| Error::from(format!("微信统一下单API请求失败: {}", e)))?;

    let status = response.status();
    let response_text = response.text().await.map_err(|e| Error::from(format!("微信响应解析失败: {}", e)))?;
    
    log::info!("[微信支付V3] 原始响应内容: {}", response_text);
    
    if !status.is_success() {
        return Err(Error::from(format!("微信支付请求失败: {}", response_text)));
    }
    
    let resp: WechatV3PrepayResponse = serde_json::from_str(&response_text)
        .map_err(|e| Error::from(format!("解析微信响应失败: {}", e)))?;
    
    let prepay_id = resp.prepay_id;
    log::info!("[微信支付V3] 预支付ID: {}", prepay_id);
    
    let pay_nonce_str = generate_nonce_str();
    let pay_timestamp = chrono::Utc::now().timestamp().to_string();
    let package = format!("prepay_id={}", prepay_id);
    
    let pay_sign_str = format!("{}\n{}\n{}\n{}\n", app_id, pay_timestamp, pay_nonce_str, package);
    let mut signer = Signer::new(MessageDigest::sha256(), &private_key)
        .map_err(|e| Error::from(format!("创建签名器失败: {}", e)))?;
    signer.update(pay_sign_str.as_bytes())
        .map_err(|e| Error::from(format!("签名数据失败: {}", e)))?;
    let pay_signature = signer.sign_to_vec()
        .map_err(|e| Error::from(format!("生成签名失败: {}", e)))?;
    let pay_sign = BASE64.encode(pay_signature);

    let pay_response = WechatPayResponse {
        app_id,
        time_stamp: pay_timestamp,
        nonce_str: pay_nonce_str,
        package,
        sign_type: "RSA".to_string(),
        pay_sign,
    };

    Ok(pay_response)
}

pub fn verify_wechat_notify_sign(_params: &BTreeMap<String, String>, _api_key: &str) -> bool {
    true
}

pub fn parse_notify_json(json_str: &str) -> Result<BTreeMap<String, String>> {
    let value: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| Error::from(format!("解析通知JSON失败: {}", e)))?;
    
    let mut result = BTreeMap::new();
    if let Some(obj) = value.as_object() {
        for (k, v) in obj {
            result.insert(k.clone(), v.to_string());
        }
    }
    
    Ok(result)
}

#[allow(dead_code)]
pub fn parse_notify_xml(xml_str: &str) -> Result<BTreeMap<String, String>> {
    parse_xml_response(xml_str)
}

fn parse_xml_response(xml_str: &str) -> Result<BTreeMap<String, String>> {
    let mut result = BTreeMap::new();
    let xml_str = xml_str.trim();
    
    let mut chars = xml_str.chars().peekable();
    while let Some(_) = chars.peek() {
        if !skip_until(&mut chars, '<') {
            break;
        }
        
        let tag_start = chars.position(|c| c == '>');
        if tag_start.is_none() {
            break;
        }
        
        let tag_name: String = chars.by_ref().take(tag_start.unwrap()).collect();
        
        if tag_name.starts_with('/') || tag_name.starts_with('?') {
            continue;
        }
        
        if !skip_until(&mut chars, '>') {
            break;
        }
        
        let end_tag = format!("</{}>", tag_name);
        let mut value = String::new();
        let mut buffer = String::new();
        
        while let Some(c) = chars.next() {
            buffer.push(c);
            if buffer.ends_with(&end_tag) {
                value = buffer[..buffer.len() - end_tag.len()].trim().to_string();
                break;
            }
            if buffer.len() > end_tag.len() + 100 {
                buffer.remove(0);
            }
        }
        
        value = value.replace("<![CDATA[", "").replace("]]>", "");
        
        result.insert(tag_name, value);
    }
    
    Ok(result)
}

fn skip_until<I: Iterator<Item = char>>(iter: &mut std::iter::Peekable<I>, target: char) -> bool {
    while let Some(&c) = iter.peek() {
        if c == target {
            iter.next();
            return true;
        }
        iter.next();
    }
    false
}
