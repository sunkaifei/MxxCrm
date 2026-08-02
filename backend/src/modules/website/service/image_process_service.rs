//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::{Error, Result};
use std::path::Path;

/// 图片处理服务
pub struct ImageProcessService;

impl ImageProcessService {
    /// 从文件名生成ALT文字
    /// 例如: "product-iphone-15-pro.jpg" -> "Product Iphone 15 Pro"
    pub fn generate_alt_text(filename: &str) -> String {
        // 移除扩展名
        let stem = Path::new(filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(filename);
        
        // 替换分隔符为空格
        let cleaned = stem
            .replace(['-', '_'], " ")
            .replace('.', " ");
        
        // 首字母大写
        let result: String = cleaned
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        
        result
    }

    /// 生成缩略图文件名
    /// 例如: "photo.jpg" -> "photo_small.jpg", "photo_medium.jpg", "photo_large.jpg"
    pub fn get_thumb_name(original_name: &str, size: &str) -> String {
        let path = Path::new(original_name);
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("jpg");
        format!("{}_{}.{}", stem, size, ext)
    }

    /// 获取文件扩展名
    pub fn get_file_ext(filename: &str) -> String {
        Path::new(filename)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase()
    }

    /// 判断是否为图片文件
    pub fn is_image(ext: &str) -> bool {
        matches!(ext.to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "svg")
    }

    /// 判断是否为视频文件
    pub fn is_video(ext: &str) -> bool {
        matches!(ext.to_lowercase().as_str(), "mp4" | "avi" | "mov" | "wmv" | "flv" | "mkv" | "webm")
    }

    /// 判断是否为文档文件
    pub fn is_document(ext: &str) -> bool {
        matches!(ext.to_lowercase().as_str(), "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt")
    }

    /// 获取文件类型编号
    /// 1=图片, 2=视频, 3=文档, 4=音频, 5=其他
    pub fn get_file_type(ext: &str) -> i32 {
        if Self::is_image(ext) { 1 }
        else if Self::is_video(ext) { 2 }
        else if Self::is_document(ext) { 3 }
        else if matches!(ext.to_lowercase().as_str(), "mp3" | "wav" | "ogg" | "flac" | "aac") { 4 }
        else { 5 }
    }

    /// 获取MIME类型
    pub fn get_mime_type(ext: &str) -> String {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" => "image/jpeg".to_string(),
            "png" => "image/png".to_string(),
            "gif" => "image/gif".to_string(),
            "webp" => "image/webp".to_string(),
            "svg" => "image/svg+xml".to_string(),
            "mp4" => "video/mp4".to_string(),
            "avi" => "video/x-msvideo".to_string(),
            "pdf" => "application/pdf".to_string(),
            "doc" => "application/msword".to_string(),
            "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
            "mp3" => "audio/mpeg".to_string(),
            "wav" => "audio/wav".to_string(),
            _ => "application/octet-stream".to_string(),
        }
    }

    /// 格式化文件大小
    pub fn format_file_size(size: i64) -> String {
        if size < 1024 {
            format!("{} B", size)
        } else if size < 1024 * 1024 {
            format!("{:.1} KB", size as f64 / 1024.0)
        } else if size < 1024 * 1024 * 1024 {
            format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
}
