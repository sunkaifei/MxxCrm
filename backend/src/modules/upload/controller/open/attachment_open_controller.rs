//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 公开附件访问（附件URL统一方案 v1.1 §5.2）
//!
//! 安全规则：
//! - 仅 is_public=1 的附件可访问；其余一律 404（不泄露存在性）
//! - 不返回目录列表；路径仅来自 DB（上传时生成的服务端路径）
//! - 图片 inline 预览，其余 attachment 下载
//! - NamedFile 流式响应，带 ETag/Last-Modified（浏览器缓存生效）

use actix_files::NamedFile;
use actix_web::{get, web, HttpRequest, HttpResponse};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::core::kit::global::AppState;
use crate::modules::upload::entity::attachment::{self, Entity as Attachment};

/// 按附件 ID 读取公开文件（无需登录；scope 已带 /file 前缀，此处为 /{id}）
#[get("/{id}")]
pub async fn open_file(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> HttpResponse {
    let db = &state.db;
    let id = path.into_inner();

    let not_found = || HttpResponse::NotFound().finish();

    let Ok(Some(att)) = Attachment::find_by_id(id)
        .filter(attachment::Column::Deleted.eq(0))
        .one(db)
        .await
    else {
        return not_found();
    };

    // 关键安全闸门：非公开附件统一 404，不暴露存在性
    if att.is_public != Some(true) {
        return not_found();
    }

    let Some(file_path) = att.path.clone().filter(|p| !p.is_empty()) else {
        return not_found();
    };
    if !std::path::Path::new(&file_path).exists() {
        return not_found();
    }

    // 图片 inline 预览，其余 attachment 下载
    let file_name = att
        .original_name
        .clone()
        .or(att.name.clone())
        .unwrap_or_else(|| format!("file-{}", id));
    let mime_type = att
        .mime_type
        .clone()
        .unwrap_or_else(|| "application/octet-stream".to_string());
    // HeaderValue 拒绝非 ASCII；中文文件名回退 ASCII 兜底名（原始名仍可从 DB 追溯）
    let ascii_name = format!("file-{}.{}", id, att.ext.clone().unwrap_or_default());
    let disposition = if mime_type.starts_with("image/") {
        format!("inline; filename=\"{}\"", ascii_name)
    } else {
        format!("attachment; filename=\"{}\"", ascii_name)
    };

    match NamedFile::open(&file_path) {
        Ok(nf) => {
            // NamedFile 自带扩展名推断的 Content-Type，此处用 DB 记录的 MIME 覆盖（更准）
            let mut response = nf.into_response(&req);
            let headers = response.headers_mut();
            if let Ok(hv) = actix_web::http::header::HeaderValue::from_str(&mime_type) {
                headers.insert(actix_web::http::header::CONTENT_TYPE, hv);
            }
            if let Ok(hv) = actix_web::http::header::HeaderValue::from_str(&disposition) {
                headers.insert(actix_web::http::header::CONTENT_DISPOSITION, hv);
            }
            response
        }
        Err(_) => not_found(),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(open_file);
}
