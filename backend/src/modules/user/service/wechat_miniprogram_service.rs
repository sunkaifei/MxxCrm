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
use std::fs;
use std::path::Path;
use chrono::Datelike;

#[derive(Debug, Serialize)]
struct WxaCodeUnlimitRequest {
    scene: String,
    page: String,
    width: Option<i32>,
    auto_color: Option<bool>,
    line_color: Option<LineColor>,
    is_hyaline: Option<bool>,
}

#[derive(Debug, Serialize)]
struct LineColor {
    r: i32,
    g: i32,
    b: i32,
}

#[derive(Debug, Deserialize)]
struct AccessTokenResponse {
    access_token: String,
    expires_in: i64,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    errcode: i32,
    errmsg: String,
}

/// 获取微信小程序access_token
async fn get_access_token(client: &Client) -> Result<String> {
    let app_id = config::section::<String>("wechat", "app_id", "".to_string());
    let app_secret = config::section::<String>("wechat", "app_secret", "".to_string());
    
    if app_id.is_empty() || app_secret.is_empty() {
        return Err(Error::from("微信小程序配置未设置"));
    }

    let url = format!(
        "https://api.weixin.qq.com/cgi-bin/token?grant_type=client_credential&appid={}&secret={}",
        app_id, app_secret
    );

    let response = client.get(&url).send().await.map_err(|e| {
        Error::from(format!("请求微信access_token失败: {:?}", e))
    })?;

    let status = response.status();
    if !status.is_success() {
        let error: ErrorResponse = response.json().await.map_err(|e| {
            Error::from(format!("解析微信错误响应失败: {:?}", e))
        })?;
        return Err(Error::from(format!("微信API错误: {} - {}", error.errcode, error.errmsg)));
    }

    let result: AccessTokenResponse = response.json().await.map_err(|e| {
        Error::from(format!("解析access_token响应失败: {:?}", e))
    })?;

    Ok(result.access_token)
}

/// 获取小程序二维码并保存到指定路径
/// 使用 getwxacodeunlimit 接口（无数量限制），scene 参数传递 query 值
pub async fn get_qrcode_and_save(page: &str, scene: &str, short_code: &str, save_dir: &str) -> Result<String> {
    let client = Client::new();
    
    // 获取 access_token
    let access_token = get_access_token(&client).await?;
    
    // 构建请求URL（使用 getwxacodeunlimit 接口）
    let url = format!(
        "https://api.weixin.qq.com/wxa/getwxacodeunlimit?access_token={}",
        access_token
    );

    // 构建请求体
    let request = WxaCodeUnlimitRequest {
        page: page.to_string(),
        scene: scene.to_string(),
        width: Some(430),
        auto_color: Some(false),
        line_color: Some(LineColor { r: 0, g: 0, b: 0 }),
        is_hyaline: Some(false),
    };

    log::info!("[微信小程序] 请求参数: page={}, scene={}", page, scene);

    // 发送请求
    let response = client.post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| Error::from(format!("请求微信二维码失败: {:?}", e)))?;

    let status = response.status();
    
    // 获取响应头信息（先克隆，避免后续操作消费）
    let content_type = response.headers().get("content-type").map(|v| v.to_str().unwrap_or("unknown").to_string());
    let content_encoding = response.headers().get("content-encoding").map(|v| v.to_str().unwrap_or("unknown").to_string());
    
    // 获取响应字节
    let bytes = response.bytes().await.map_err(|e| {
        Error::from(format!("读取二维码数据失败: {:?}", e))
    })?;

    // 记录响应字节大小和状态码
    let bytes_len = bytes.len();
    log::info!("[微信小程序] 响应数据大小: {} 字节, HTTP状态码: {}, Content-Type: {:?}, Content-Encoding: {:?}", 
        bytes_len, status, content_type, content_encoding);
    
    // 验证是否为有效图片（检查文件头）
    let png_header = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    let jpeg_header = [0xFF, 0xD8];
    
    // 检查是否为PNG或JPEG格式
    let is_png = bytes.len() >= 8 && bytes.starts_with(&png_header);
    let is_jpeg = bytes.len() >= 2 && bytes.starts_with(&jpeg_header);
    
    if !is_png && !is_jpeg {
        // 不是图片格式，尝试解析JSON错误
        log::error!("[微信小程序] 响应不是有效的图片格式");
        
        // 先记录文件头十六进制（用于调试）
        log::error!("[微信小程序] 文件头十六进制: {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}", 
            bytes.get(0).copied().unwrap_or(0), 
            bytes.get(1).copied().unwrap_or(0), 
            bytes.get(2).copied().unwrap_or(0), 
            bytes.get(3).copied().unwrap_or(0), 
            bytes.get(4).copied().unwrap_or(0), 
            bytes.get(5).copied().unwrap_or(0), 
            bytes.get(6).copied().unwrap_or(0), 
            bytes.get(7).copied().unwrap_or(0));
        
        if let Ok(error) = serde_json::from_slice::<ErrorResponse>(&bytes) {
            log::error!("[微信小程序] 微信API错误: {} - {}", error.errcode, error.errmsg);
            return Err(Error::from(format!("微信API错误: {} - {}", error.errcode, error.errmsg)));
        } else if let Ok(text) = String::from_utf8(bytes.to_vec()) {
            // 截取前500个字符避免日志过长
            let text_preview = if text.len() > 500 { &text[..500] } else { &text };
            log::error!("[微信小程序] 响应内容(前500字符): {}", text_preview);
            return Err(Error::from(format!("微信API返回非图片数据: {}", text_preview)));
        } else {
            return Err(Error::from("微信API返回的不是有效的图片"));
        }
    }
    
    let format = if is_png { "PNG" } else { "JPEG" };
    log::info!("[微信小程序] 验证通过，是有效的{}图片", format);


    // 确保保存目录存在
    fs::create_dir_all(save_dir).map_err(|e| {
        Error::from(format!("创建保存目录失败: {:?}", e))
    })?;

    // 生成文件名（使用 short_code，根据图片格式选择扩展名）
    let ext = if is_png { "png" } else { "jpg" };
    let file_name = format!("qrcode_{}.{}", short_code, ext);
    let file_path = Path::new(save_dir).join(&file_name);

    // 保存文件
    fs::write(&file_path, &bytes).map_err(|e| {
        log::error!("[微信小程序] 保存二维码文件失败: {:?}", e);
        Error::from(format!("保存二维码文件失败: {:?}", e))
    })?;

    log::info!("[微信小程序] 二维码保存成功: {}", file_path.display());
    
    // 验证保存的文件
    if let Ok(saved_bytes) = fs::read(&file_path) {
        log::info!("[微信小程序] 保存后文件大小: {} 字节", saved_bytes.len());
        
        if saved_bytes.len() != bytes_len {
            log::warn!("[微信小程序] 保存文件大小不一致: 原始: {}, 保存后: {}", bytes_len, saved_bytes.len());
        } else {
            log::info!("[微信小程序] 文件大小验证通过");
        }
        
        // 验证文件头（支持PNG和JPEG）
        let saved_is_png = saved_bytes.starts_with(&png_header);
        let saved_is_jpeg = saved_bytes.len() >= 2 && saved_bytes.starts_with(&jpeg_header);
        
        if saved_is_png || saved_is_jpeg {
            let saved_format = if saved_is_png { "PNG" } else { "JPEG" };
            log::info!("[微信小程序] 保存后文件头验证通过，格式: {}", saved_format);
        } else {
            log::error!("[微信小程序] 保存后文件头验证失败");
        }
    } else {
        log::warn!("[微信小程序] 无法读取保存的文件进行验证");
    }
    
    Ok(file_path.to_string_lossy().to_string())
}

/// 获取龟类记录的小程序二维码
pub async fn get_turtle_qrcode(short_code: &str, create_time: chrono::NaiveDateTime) -> Result<String> {
    let base_path = config::section::<String>("attach", "upload_path", "storage/upload/".to_string());
    let static_url = config::section::<String>("attach", "static_url", "https://static.s88.cn/".to_string());
    let base_url = config::section::<String>("attach", "upload_url", "/upload/".to_string());
    
    let year = create_time.year();
    let month = create_time.month();
    let day = create_time.day();
    
    let save_dir = format!("{}qrcode/{}/{}/{}/", base_path, year, month, day);
    let url_prefix = format!("{}qrcode/{}/{}/{}/", base_url, year, month, day);
    
    // page 不带 / 开头，不支持 code=xxx
    // scene 参数传递 query 值，小程序在 onLoad 时通过 options.scene 获取
    let page = "pages/farm/record-list/index";
    let scene = format!("code={}", short_code);
    
    log::info!("[微信小程序] 开始生成二维码 short_code={}, page={}, scene={}, save_dir={}", short_code, page, scene, save_dir);
    
    // 调用保存函数，获取本地文件路径
    let file_path = get_qrcode_and_save(page, &scene, short_code, &save_dir).await?;
    
    // 提取文件名并构建可访问的URL
    let file_name = Path::new(&file_path).file_name()
        .ok_or(Error::from("无法获取文件名"))?
        .to_string_lossy();
    
    let qrcode_url = format!("{}{}{}", static_url.trim_end_matches('/'), url_prefix, file_name);
    
    log::info!("[微信小程序] 二维码生成完成 file_path={}, qrcode_url={}", file_path, qrcode_url);
    
    Ok(qrcode_url)
}
