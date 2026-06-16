//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::Result;
use crate::core::kit::config;
use crate::core::kit::global::AppState;
use crate::core::web::response::MetaResp;
use crate::modules::upload::model::attachment::{
    AttachmentModel, AttachmentSaveDTO, ImageFormRequest, StorageType, UploadImageResult,
};
use crate::modules::upload::service::upload_service;
use crate::modules::user::model::user::UserModel;
use crate::utils::encryption_utils;
use crate::utils::time_utils;
use crate::SNOWFLAKE;
use actix_multipart::form::MultipartForm;
use actix_web::{post, web, HttpRequest, HttpResponse};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;

use chrono::Datelike;
use std::fs;
use std::path::Path;

/// 内部辅助：将 MessagePack 字节使用 Base64 封装后，以 application/msgpack 内容类型返回。
fn pack_msgpack_response(bytes: Vec<u8>) -> HttpResponse {
    let b64 = STANDARD.encode(&bytes);
    HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(b64)
}

/// # 用户上传头像
#[post("/upload/avatar")]
pub async fn upload_avatar(
    state: web::Data<AppState>,
    req: HttpRequest,
    MultipartForm(form): MultipartForm<ImageFormRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;

    log::info!("[upload_avatar] 接收到头像上传请求");

    // 验证用户登录
    let user_id = match upload_service::verify_auth(&req) {
        Ok(id) => {
            log::info!("[upload_avatar] 用户ID: {}", id);
            id
        }
        Err(resp) => return Ok(resp),
    };

    // 查询用户信息获取注册日期
    let user = match UserModel::find_by_id(db, &Some(user_id))
        .await
        .map_err(|e| {
            log::error!("[upload_avatar] 查询用户失败: {}", e);
            e
        })? {
        Some(u) => u,
        None => {
            log::error!("[upload_avatar] 用户不存在 user_id={}", user_id);
            return Ok(pack_msgpack_response(MetaResp::<String>::fail(
                400,
                "用户不存在",
                "local",
            )));
        }
    };

    // 校验文件
    let file_info = match upload_service::check_file(&form.file) {
        Ok(info) => {
            log::info!(
                "[upload_avatar] 文件: {}, 大小: {}",
                info.file_name,
                info.file_size
            );
            info
        }
        Err(resp) => return Ok(resp),
    };

    // 从用户注册日期提取年月日
    let (year, month, day) = match user.create_time {
        Some(dt) => (dt.year(), dt.month(), dt.day()),
        None => {
            let now = time_utils::now();
            (now.year(), now.month(), now.day())
        }
    };

    // 构建目录结构：头像/年/月/日
    let directory = format!("avatar/{}/{:02}/{:02}", year, month, day);
    let name = format!("avatar_{}.{}", user_id, file_info.ext);

    let base_path =
        config::section::<String>("attach", "upload_path", "./storage/upload/".to_string());
    let base_url = config::section::<String>("attach", "upload_url", "/upload/".to_string());
    let path = format!("{}{}/{}", base_path, directory, name);
    let url = format!("{}{}/{}", base_url, directory, name);

    // 获取存储配置
    let storage_cfg = upload_service::get_storage_config(db)
        .await
        .unwrap_or_else(|_| upload_service::StorageConfig {
            storage_type: StorageType::Local,
            storage_url: String::new(),
        });

    // 上传到存储
    match &storage_cfg.storage_type {
        StorageType::Local => {
            // 删除旧头像文件（如果存在）
            if Path::new(&path).exists() {
                if let Err(e) = fs::remove_file(&path) {
                    log::warn!("[upload_avatar] 删除旧头像失败 {}", e);
                }
            }
            if let Err(e) = upload_service::upload_to_storage(&form.file, &path) {
                log::error!(
                    "[upload_avatar] 上传到本地存储失败 path={}, err={}",
                    path,
                    e
                );
                return Ok(pack_msgpack_response(MetaResp::<String>::fail(
                    400,
                    "保存文件失败",
                    "local",
                )));
            }
        }
        StorageType::Qiniu => {
            return Ok(pack_msgpack_response(MetaResp::<String>::fail(
                400,
                "七牛云存储暂未实现",
                "local",
            )));
        }
        StorageType::Aliyun => {
            return Ok(pack_msgpack_response(MetaResp::<String>::fail(
                400,
                "阿里云存储暂未实现",
                "local",
            )));
        }
        StorageType::Tencent => {
            return Ok(pack_msgpack_response(MetaResp::<String>::fail(
                400,
                "腾讯云存储暂未实现",
                "local",
            )));
        }
    }

    // 更新用户头像字段
    let img_url = format!("{}{}", storage_cfg.storage_url, url);
    match UserModel::update_avatar(db, user_id, Some(img_url.clone())).await {
        Err(e) => {
            log::error!("[upload_avatar] 更新用户头像失败: {:?}", e);
            if fs::remove_file(&path).is_err() {
                log::warn!("[upload_avatar] 更新失败，图片删除失败 {}", path);
            }
            return Ok(pack_msgpack_response(MetaResp::<String>::fail(
                400,
                "更新头像失败",
                "local",
            )));
        }
        Ok(_) => {
            let result = UploadImageResult {
                file_name: name,
                url: img_url,
                id: user_id.to_string(),
            };
            Ok(pack_msgpack_response(MetaResp::success(result, "local")))
        }
    }
}

/// # 用户上传海龟图片
#[post("/upload")]
pub async fn upload_turtle_image(
    state: web::Data<AppState>,
    req: HttpRequest,
    MultipartForm(form): MultipartForm<ImageFormRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;

    log::info!("[upload_turtle_image] 接收到图片上传请求");

    // 验证用户登录
    let user_id = match upload_service::verify_auth(&req) {
        Ok(id) => {
            log::info!("[upload_turtle_image] 用户ID: {}", id);
            id
        }
        Err(resp) => return Ok(resp),
    };

    // 校验文件
    let file_info = match upload_service::check_file(&form.file) {
        Ok(info) => {
            log::info!(
                "[upload_turtle_image] 文件: {}, 大小: {}",
                info.file_name,
                info.file_size
            );
            info
        }
        Err(resp) => return Ok(resp),
    };

    // 读取文件内容并计算MD5
    let buffer = match fs::read(&form.file.file.path()) {
        Ok(b) => b,
        Err(e) => {
            log::error!("[upload_turtle_image] 读取文件失败: {}", e);
            return Ok(pack_msgpack_response(MetaResp::<String>::fail(
                400,
                "读取文件失败",
                "local",
            )));
        }
    };
    let md5 = encryption_utils::md5_bytes(&buffer);
    let size = buffer.len() as i64;

    // 使用MD5作为文件名
    let name = format!("{}.{}", md5, file_info.ext);
    let date_str = time_utils::current_date();
    let directory = format!("{}/{}", date_str, user_id);

    let base_path =
        config::section::<String>("attach", "upload_path", "./storage/upload/".to_string());
    let base_url = config::section::<String>("attach", "upload_url", "/upload/".to_string());
    let path = format!("{}{}/{}", base_path, directory, name);
    let url = format!("{}{}/{}", base_url, directory, name);

    // 获取存储配置
    let storage_cfg = upload_service::get_storage_config(db)
        .await
        .unwrap_or_else(|_| upload_service::StorageConfig {
            storage_type: StorageType::Local,
            storage_url: String::new(),
        });

    // 检查是否已存在相同用户ID+MD5的图片（避免重复上传）
    let existing = match AttachmentModel::select_by_user_id_and_md5(&db, user_id, md5.clone()).await
    {
        Ok(v) => v,
        Err(e) => {
            log::error!("[upload_turtle_image] 查询已存在图片失败 {}", e);
            return Ok(pack_msgpack_response(MetaResp::<String>::fail(
                400,
                "查询图片信息失败",
                "local",
            )));
        }
    };
    if let Some(existing_attachment) = existing {
        let static_url =
            config::section::<String>("attach", "static_url", "https://static.s88.cn/".to_string());
        let img_url = format!(
            "{}{}",
            static_url.trim_end_matches('/'),
            existing_attachment.upload_url.unwrap_or_default()
        );

        let result = UploadImageResult {
            file_name: existing_attachment.name.unwrap_or_default(),
            url: img_url,
            id: existing_attachment.id.to_string(),
        };

        return Ok(pack_msgpack_response(MetaResp::success(result, "local")));
    }

    // 上传到存储
    match &storage_cfg.storage_type {
        StorageType::Local => {
            if let Err(e) = upload_service::upload_to_storage(&form.file, &path) {
                log::error!(
                    "[upload_turtle_image] 保存文件失败: path={}, err={}",
                    path,
                    e
                );
                return Ok(pack_msgpack_response(MetaResp::<String>::fail(
                    400,
                    "保存文件失败",
                    "local",
                )));
            }
        }
        StorageType::Qiniu => {
            return Ok(pack_msgpack_response(MetaResp::<String>::fail(
                400,
                "七牛云存储暂未实现",
                "local",
            )));
        }
        StorageType::Aliyun => {
            return Ok(pack_msgpack_response(MetaResp::<String>::fail(
                400,
                "阿里云存储暂未实现",
                "local",
            )));
        }
        StorageType::Tencent => {
            return Ok(pack_msgpack_response(MetaResp::<String>::fail(
                400,
                "腾讯云存储暂未实现",
                "local",
            )));
        }
    }

    // 生成ID
    let id = SNOWFLAKE.generate();

    // 创建附件记录
    let create_data = AttachmentSaveDTO {
        id,
        user_id: Some(user_id),
        type_id: None,
        name: Some(file_info.file_name),
        path: Some(path.clone()),
        upload_url: Some(url.clone()),
        ext: Some(file_info.ext),
        size: Some(size),
        md5: Some(md5),
        storage_type: match storage_cfg.storage_type {
            StorageType::Local => Some(1),
            StorageType::Qiniu => Some(2),
            StorageType::Aliyun => Some(3),
            StorageType::Tencent => Some(4),
        },
        r#type: Some(2),
        status: Some(1),
        create_time: None,
    };

    // 保存到数据库
    let rows = match AttachmentModel::insert(&db, &create_data).await {
        Ok(v) => v,
        Err(e) => {
            log::error!("[upload_turtle_image] 保存附件记录失败: {}", e);
            let _ = fs::remove_file(&path);
            return Ok(pack_msgpack_response(MetaResp::<String>::fail(
                400,
                "保存附件信息失败",
                "local",
            )));
        }
    };

    if rows > 0 {
        let static_url =
            config::section::<String>("attach", "static_url", "https://static.s88.cn/".to_string());
        let img_url = format!("{}{}", static_url.trim_end_matches('/'), &url);

        let result = UploadImageResult {
            file_name: name,
            url: img_url,
            id: id.to_string(),
        };

        Ok(pack_msgpack_response(MetaResp::success(result, "local")))
    } else {
        if fs::remove_file(&path).is_err() {
            log::warn!("保存失败，图片删除失败 {}", path);
        }
        Ok(pack_msgpack_response(MetaResp::<String>::fail(
            400,
            "上传图片保存失败",
            "local",
        )))
    }
}

/// 获取完整的上传URL
pub fn get_upload_url(
    static_url: &str,
    upload_url: &str,
    module: &str,
    directory_url: &str,
    file_name: &str,
) -> String {
    format!(
        "{}{}{}{}{}",
        static_url.trim_end_matches('/'),
        upload_url,
        module,
        directory_url,
        file_name
    )
}
