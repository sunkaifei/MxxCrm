//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 附件管理控制器（新文件管理系统）
//! - 上传/下载/绑定/解绑/分页查询/详情/删除/修改
//! - 上传接口返回 JSON（其余接口沿用 msgpack）
//! - 通过 JWT 提取上传者用户ID
//!
use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_user;
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::upload::model::attachment::{
    AttachmentBindRequest, AttachmentByEntityQuery, AttachmentPageRequest,
    AttachmentUnbindRequest, AttachmentUpdateRequest, BatchMoveRequest, ImageFormRequest,
};
use crate::modules::upload::controller::admin::attachment_category_admin_controller;
use crate::modules::upload::service::{attachment_category_service, attachment_service};
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;
use crate::validate;
use actix_multipart::form::MultipartForm;
use actix_web::{web, HttpRequest, HttpResponse};
use serde::Serialize;

/// JSON 业务响应体（上传接口使用）
#[derive(Debug, Serialize)]
struct JsonResp<T: Serialize> {
    pub code: i32,
    pub msg: String,
    pub data: Option<T>,
}

impl<T: Serialize> JsonResp<T> {
    fn ok(data: T) -> Self {
        Self {
            code: 200,
            msg: "success".to_string(),
            data: Some(data),
        }
    }

    fn err(msg: String) -> Self {
        Self {
            code: 400,
            msg,
            data: None,
        }
    }
}

/// # 上传文件
///
/// - multipart 表单字段：file / entity_type / entity_id（可选）
/// - 从 JWT token 提取 uploaded_by
/// - 返回 JSON（不再使用 msgpack）
pub async fn upload_attachment(
    state: web::Data<AppState>,
    req: HttpRequest,
    MultipartForm(form): MultipartForm<ImageFormRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    // 从 JWT 提取上传者用户ID
    let uploaded_by = match get_user(&req) {
        Ok(token) => token.id,
        Err(_) => None,
    };

    match attachment_service::upload_file(db, form, uploaded_by).await {
        Ok(data) => Ok(HttpResponse::Ok().json(JsonResp::ok(data))),
        Err(e) => Ok(HttpResponse::Ok().json(JsonResp::<()>::err(e.to_string()))),
    }
}

/// # 删除附件
///
/// 删除规则：count_quote > 0 拒绝；软删除 + 删除物理文件
pub async fn delete_attachment(
    state: web::Data<AppState>,
    item: web::Json<BathDeleteIdRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }

        let ids = convert_vec_option_string_to_vec_u64(ids_vec);
        let result = attachment_service::batch_delete_by_ids(&db, ids).await?;
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(Ok(result))))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

/// # 修改附件信息
pub async fn update(
    state: web::Data<AppState>,
    id: web::Path<i64>,
    item: web::Json<AttachmentUpdateRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let mut data = item.into_inner();
    data.id = Some(id.into_inner());
    validate!(data.name.is_none(), t!("attachment.category.id_empty", locale = "zh-CN").to_string());
    let result = attachment_service::update(&db, data).await?;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(Ok(result))))
}

pub async fn batch_move(
    state: web::Data<AppState>,
    item: web::Json<BatchMoveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    validate!(item.type_id.is_none(), t!("attachment.category.id_empty", locale = "zh-CN").to_string());

    let category_info = attachment_category_service::find_by_id(db, &item.type_id).await?;
    if category_info.is_none() {
        validate!(true, t!("attachment.category.info_empty", locale = "zh-CN").to_string());
    }
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
        } else {
            let ids = convert_vec_option_string_to_vec_u64(ids_vec);
            let result = attachment_service::batch_update(db, item.type_id, ids).await;
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
        }
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

/// # 获取附件详情
pub async fn get_detail(
    state: web::Data<AppState>,
    item: web::Path<InfoId>,
) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    let result = attachment_service::get_by_detail(&db, &item.id).await?;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(result, "local")))
}

/// # 分页获取附件列表
pub async fn get_page_list(
    state: web::Data<AppState>,
    query: web::Query<AttachmentPageRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = query.into_inner();

    attachment_service::get_by_page(&db, payload)
        .await
        .map(|page_data| {
            HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::success(page_data, "local"))
        })
}

/// # 下载附件
///
/// 返回二进制文件流，带 Content-Disposition 头
pub async fn download_attachment(
    state: web::Data<AppState>,
    id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let id = id.into_inner();
    match attachment_service::download_file(db, id).await {
        Ok((data, file_name, mime_type)) => {
            let safe_name = file_name.replace('"', "_");
            let disposition = format!("attachment; filename=\"{}\"", safe_name);
            Ok(HttpResponse::Ok()
                .content_type(mime_type.as_str())
                .append_header(("Content-Disposition", disposition))
                .append_header(("Content-Length", data.len()))
                .body(data))
        }
        Err(e) => {
            log::warn!("Download file {} failed: {}", id, e);
            Ok(HttpResponse::NotFound()
                .content_type("application/json")
                .body(format!(r#"{{"code":404,"msg":"{}"}}"#, e.to_string().replace('"', "'"))))
        }
    }
}

/// # 按业务实体查询附件
pub async fn get_by_entity(
    state: web::Data<AppState>,
    query: web::Query<AttachmentByEntityQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match attachment_service::get_by_entity(db, query.into_inner()).await {
        Ok(list) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(list, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// # 绑定附件到业务实体
///
/// 更新附件的 entity_type / entity_id，count_quote += 1
pub async fn bind_attachment(
    state: web::Data<AppState>,
    item: web::Json<AttachmentBindRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let result = attachment_service::bind_attachments(db, item.into_inner()).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

/// # 解绑附件
///
/// count_quote -= 1（最小为 0）
pub async fn unbind_attachment(
    state: web::Data<AppState>,
    item: web::Json<AttachmentUnbindRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let result = attachment_service::unbind_attachments(db, item.into_inner()).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

// ==================== 路由注册（单点维护）====================

/// 注册附件模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(attachment_admin_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/attachment")
            // POST /attachment/upload - 上传附件
            .route(
                "/upload",
                web::post()
                    .to(upload_attachment)
                    .wrap(require_permission("attachment:file:upload")),
            )
            // DELETE /attachment/batch_delete - 批量删除附件
            .route(
                "/batch_delete",
                web::delete()
                    .to(delete_attachment)
                    .wrap(require_permission("attachment:file:delete")),
            )
            // PUT /attachment/update/{id} - 修改附件
            .route(
                "/update/{id}",
                web::put()
                    .to(update)
                    .wrap(require_permission("attachment:update")),
            )
            // PUT /attachment/batchmove - 批量移动附件
            .route(
                "/batchmove",
                web::put()
                    .to(batch_move)
                    .wrap(require_permission("attachment:update")),
            )
            // GET /attachment/detail/{id} - 附件详情
            .route(
                "/detail/{id}",
                web::get()
                    .to(get_detail)
                    .wrap(require_permission("attachment:list")),
            )
            // GET /attachment/list - 附件列表
            .route(
                "/list",
                web::get()
                    .to(get_page_list)
                    .wrap(require_permission("attachment:list")),
            )
            // GET /attachment/download/{id} - 下载附件
            .route(
                "/download/{id}",
                web::get()
                    .to(download_attachment)
                    .wrap(require_permission("attachment:list")),
            )
            // GET /attachment/by-entity - 按业务实体查询附件
            .route(
                "/by-entity",
                web::get()
                    .to(get_by_entity)
                    .wrap(require_permission("attachment:list")),
            )
            // POST /attachment/bind - 绑定附件
            .route(
                "/bind",
                web::post()
                    .to(bind_attachment)
                    .wrap(require_permission("attachment:update")),
            )
            // POST /attachment/unbind - 解绑附件
            .route(
                "/unbind",
                web::post()
                    .to(unbind_attachment)
                    .wrap(require_permission("attachment:update")),
            )
            // 附件分类管理（注册在 /attachment scope 内，避免 /attachment scope 吞掉 /attachment/category 路由）
            .configure(attachment_category_admin_controller::register),
    );
}
