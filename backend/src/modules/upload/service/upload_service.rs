//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_multipart::form::tempfile::TempFile;
use actix_web::{HttpResponse, HttpRequest};
use sea_orm::DatabaseConnection;

use crate::core::errors::error::{Error, Result};
use crate::core::kit::config;
use crate::core::web::base_controller;
use crate::core::web::response::MetaResp;
use crate::modules::system::service::config_service;
use crate::modules::upload::model::attachment::StorageType;

use std::ffi::OsStr;
use std::fs;
use std::path::Path;

/// 文件检测结果
pub struct FileInfo {
    pub file_name: String,
    pub ext: String,
    pub file_size: i64,
}

/// 存储配置结果
pub struct StorageConfig {
    pub storage_type: StorageType,
    pub storage_url: String,
}

/// 验证用户登录并获取用户ID
pub fn verify_auth(req: &HttpRequest) -> std::result::Result<i64, HttpResponse> {
    match base_controller::get_user_client_id(req) {
        Ok(id) => Ok(id),
        Err(_) => Err(HttpResponse::Unauthorized()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, "用户未登录", "local")))
    }
}

/// 检测上传文件：存在性、文件名、扩展名、大小
pub fn check_file(file: &TempFile) -> std::result::Result<FileInfo, HttpResponse> {
    if file.file_name.is_none() {
        return Err(HttpResponse::BadRequest()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, "表单中没有文件字段", "local")));
    }

    let file_name = file.file_name.clone().unwrap_or_default();
    if file_name.is_empty() {
        return Err(HttpResponse::BadRequest()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, "没有获取到文件名，上传失败", "local")));
    }

    let ext = get_extension(&file_name);
    if !is_image(&ext) {
        return Err(HttpResponse::BadRequest()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, "请上传正确的图片类型(png, jpg, jpeg, gif, bmp, svg)", "local")));
    }

    let file_size = fs::metadata(&file.file.path()).map(|m| m.len()).unwrap_or(0) as i64;
    if file_size == 0 {
        return Err(HttpResponse::BadRequest()
            .content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, "上传的文件为空", "local")));
    }

    Ok(FileInfo { file_name, ext, file_size })
}

/// 获取存储配置(uploadType + storageURL)
pub async fn get_storage_config(db: &DatabaseConnection) -> Result<StorageConfig> {
    let config_detail = config_service::select_by_key(db, &"uploadType".to_string()).await?;

    let storage_type = config_detail.config_value
        .as_deref()
        .and_then(|s| s.parse::<i32>().ok())
        .and_then(StorageType::from_i16)
        .ok_or_else(|| Error::from("无效的存储类型配置"))?;

    let storage_key = match storage_type {
        StorageType::Local => "localStorage".to_string(),
        StorageType::Qiniu => "qiniuStorage".to_string(),
        StorageType::Aliyun => "aliyunStorage".to_string(),
        StorageType::Tencent => "tencentStorage".to_string(),
    };

    let storage_config = config_service::select_by_key(db, &storage_key).await?;
    let storage_url = storage_config.config_value.unwrap_or_default();

    Ok(StorageConfig { storage_type, storage_url })
}

/// 上传文件到存储(本地/云存储)
pub fn upload_to_storage(file: &TempFile, save_path: &str) -> Result<bool> {
    let path = save_path.trim();
    if path.is_empty() {
        return Err(Error::from("保存路径不能为空"));
    }

    if let Some(parent) = Path::new(path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| Error::from(format!("创建目录失败: {}", e)))?;
        }
    }

    fs::copy(&file.file, path).map_err(|e| Error::from(format!("文件复制失败: {}", e)))?;

    if !Path::new(path).exists() {
        return Err(Error::from("上传失败"));
    }

    if cfg!(target_os = "linux") {
        let target_path = config::section::<String>("attach", "upload_path", "".to_string());
        if let Ok(metadata) = fs::metadata(&target_path) {
            let permissions = metadata.permissions();
            let _ = fs::set_permissions(path, permissions);
        }
    }

    Ok(true)
}

/// 获取文件扩展名
fn get_extension(file_name: &str) -> String {
    Path::new(file_name)
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("")
        .to_lowercase()
}

/// 检查是否为图片
fn is_image(ext: &str) -> bool {
    matches!(ext, "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg")
}
