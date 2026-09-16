//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 可视化 PDF 模板设计器 —— 控制器。
//!
//! 路由前缀：`/api/system/pdf-designer`、`/api/system/pdf-asset`
//! 设计依据：设计文档 §7.1（接口清单）、§25（报文详例）、§33（软锁/模板包）。
//!
//! 约定：
//! - 除 `preview-svg`（返回 SVG 文本）与 `bundle/export`（返回 zip 二进制）外，其余走 msgpack。
//! - `pdf-asset/upload` 沿用附件上传惯例，返回 **JSON**（`JsonResp`），便于前端原生 fetch。

use crate::core::errors::error::{Error, Result};
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::{get_current_user, get_current_user_id};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MPACK, MetaResp, ResultPage};
use crate::modules::system::service::{
    pdf_asset_service, pdf_designer_service as svc, pdf_designer_service::{
        LockRequest, ParsePdfRequest, PreviewRequest, SaveLayoutRequest, ValidateBindingsRequest,
    },
};
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::text::Text;
use actix_multipart::form::MultipartForm;
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

// ============================================================================
// 查询参数
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocTypeQuery {
    pub doc_type: Option<String>,
    pub preset: Option<String>,
    pub item_rows: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RealDataQuery {
    pub doc_type: String,
    pub doc_id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdQuery {
    pub id: Option<i64>,
    pub template_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SvgQuery {
    pub token: String,
    pub page: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RollbackRequest {
    pub template_id: i64,
    pub version: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetDefaultRequest {
    pub id: i64,
    /// 纸外元素（C 档）存在时的二次确认标记（§34.10）
    #[serde(default)]
    pub confirm_out_of_paper: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateRequest {
    pub id: i64,
    pub name: Option<String>,
}

/// 素材列表查询
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetListQuery {
    pub category: Option<String>,
    pub name: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetSaveRequest {
    pub id: Option<i64>,
    pub name: String,
    pub category: String,
    pub file_url: String,
    pub file_path: Option<String>,
    pub file_size: Option<i64>,
    pub width_px: Option<i32>,
    pub height_px: Option<i32>,
    pub sort: Option<i32>,
    pub status: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetDeleteRequest {
    pub ids: Vec<i64>,
}

/// 素材上传（multipart）
#[derive(Debug, MultipartForm)]
pub struct PdfAssetUploadForm {
    #[multipart(rename = "file")]
    pub file: TempFile,
    pub name: Option<Text<String>>,
    /// logo | seal | signature | background | texture
    pub category: Option<Text<String>>,
}

/// 模板包导入（multipart）
#[derive(Debug, MultipartForm)]
pub struct BundleImportForm {
    #[multipart(rename = "file")]
    pub file: TempFile,
}

// ============================================================================
// JSON 响应体（素材上传接口沿用项目上传惯例，返回 JSON 而非 msgpack）
// ============================================================================

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

// ============================================================================
// 元数据类（无副作用）
// ============================================================================

fn mpack_ok<T: Serialize>(data: T) -> HttpResponse {
    HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::success(data, "local"))
}

fn mpack_fail(msg: &str) -> HttpResponse {
    HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<String>::fail(400, msg, "local"))
}

/// # 单据类型列表
pub async fn doc_types() -> Result<HttpResponse> {
    Ok(mpack_ok(svc::doc_types()))
}

/// # 格式化器列表（money / money_cn / money_en_cn / date …）
pub async fn formatters() -> Result<HttpResponse> {
    Ok(mpack_ok(svc::formatters()))
}

/// # 样本预设集（typical / long / empty / zero / mixed / unicode / longlist）
pub async fn presets() -> Result<HttpResponse> {
    Ok(mpack_ok(svc::presets()))
}

/// # 字段树（按 docType）
pub async fn field_tree(state: web::Data<AppState>, q: web::Query<DocTypeQuery>) -> Result<HttpResponse> {
    let doc_type = q.doc_type.clone().unwrap_or_else(|| "order".to_string());
    let v = svc::field_tree(&state.db, &doc_type).await?;
    Ok(mpack_ok(v))
}

/// # 样例数据（§32）
pub async fn sample_data(state: web::Data<AppState>, q: web::Query<DocTypeQuery>) -> Result<HttpResponse> {
    let doc_type = q.doc_type.clone().unwrap_or_else(|| "order".to_string());
    let preset = q.preset.clone().unwrap_or_else(|| "typical".to_string());
    let rows = q.item_rows.unwrap_or(3);
    let v = svc::sample_data(&state.db, &doc_type, Some(preset.as_str()), Some(rows), None).await?;
    Ok(mpack_ok(v))
}

/// # 真实数据（上线前验收用；需数据权限）
pub async fn real_data(state: web::Data<AppState>, q: web::Query<RealDataQuery>) -> Result<HttpResponse> {
    let v = svc::real_data(&state.db, &q.doc_type, q.doc_id).await?;
    Ok(mpack_ok(v))
}

/// # 绑定健康度检查（§32.11）
pub async fn validate_bindings(
    state: web::Data<AppState>,
    body: web::Json<ValidateBindingsRequest>,
) -> Result<HttpResponse> {
    let _ = &state;
    match svc::validate_bindings(&body.doc_type, &body.layout_json) {
        Ok(v) => Ok(mpack_ok(v)),
        Err(e) => Ok(mpack_fail(&e.to_string())),
    }
}

// ============================================================================
// 模板读写
// ============================================================================

/// # 模板详情（含 layout_json / 版本 / 素材清单）
pub async fn template_info(state: web::Data<AppState>, q: web::Query<IdQuery>) -> Result<HttpResponse> {
    let id = q
        .id
        .or(q.template_id)
        .ok_or_else(|| Error::from("缺少 id"))?;
    let v = svc::template_detail(&state.db, id).await?;
    Ok(mpack_ok(v))
}

/// # 保存 layout_json
///
/// 防呆（§32.15）：`layout_json.settings.designSample` 在设计态可任意存在，
/// 但正式出图路径会强制忽略；此处仅做 schema 校验与落库。
pub async fn save_layout(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<SaveLayoutRequest>,
) -> Result<HttpResponse> {
    let (uid, _name) = get_current_user(&req);
    let operator = if uid > 0 { Some(uid) } else { None };
    match svc::save_layout(&state.db, body.into_inner(), operator).await {
        Ok(v) => Ok(mpack_ok(v)),
        Err(e) => {
            let msg = e.to_string();
            // 乐观锁冲突：前端识别 __CONFLICT__ 前缀后弹「是否覆盖」
            if msg.starts_with("__CONFLICT__") {
                return Ok(HttpResponse::Ok()
                    .content_type(MPACK)
                    .body(MetaResp::<String>::fail(409, msg.trim_start_matches("__CONFLICT__"), "local")));
            }
            Ok(mpack_fail(&msg))
        }
    }
}

/// # 精确预览 / 分页预览（§25.3、§34.6）
pub async fn precise_preview(
    state: web::Data<AppState>,
    body: web::Json<PreviewRequest>,
) -> Result<HttpResponse> {
    match svc::precise_preview(&state.db, body.into_inner()).await {
        Ok(v) => Ok(mpack_ok(v)),
        Err(e) => Ok(mpack_fail(&e.to_string())),
    }
}

/// # 取分页预览的某一页 SVG（原始文本，非 msgpack）
pub async fn preview_svg(q: web::Query<SvgQuery>) -> Result<HttpResponse> {
    let page = q.page.unwrap_or(1);
    match svc::take_preview_page(&q.token, page) {
        Some(svg) => Ok(HttpResponse::Ok()
            .content_type("image/svg+xml; charset=utf-8")
            .insert_header(("Cache-Control", "private, max-age=300"))
            .body(svg)),
        None => Ok(HttpResponse::NotFound().body("预览已过期，请重新编译")),
    }
}

/// # 版本列表
pub async fn versions(state: web::Data<AppState>, q: web::Query<IdQuery>) -> Result<HttpResponse> {
    let tid = q
        .template_id
        .or(q.id)
        .ok_or_else(|| Error::from("缺少 templateId"))?;
    let v = svc::versions(&state.db, tid).await?;
    Ok(mpack_ok(v))
}

/// # 回滚到指定版本
pub async fn rollback(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<RollbackRequest>,
) -> Result<HttpResponse> {
    let (uid, _) = get_current_user(&req);
    let operator = if uid > 0 { Some(uid) } else { None };
    match svc::rollback(&state.db, body.template_id, body.version, operator).await {
        Ok(v) => Ok(mpack_ok(v)),
        Err(e) => Ok(mpack_fail(&e.to_string())),
    }
}

/// # 复制模板
pub async fn duplicate(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<DuplicateRequest>,
) -> Result<HttpResponse> {
    let (uid, _) = get_current_user(&req);
    let operator = if uid > 0 { Some(uid) } else { None };
    match svc::duplicate(&state.db, body.id, operator).await {
        Ok(v) => Ok(mpack_ok(v)),
        Err(e) => Ok(mpack_fail(&e.to_string())),
    }
}

/// # 设为默认（独立权限 `system:pdf-designer:set-default`，§22.1）
///
/// 纸外元素（C 档）存在时，前端需带 `confirmOutOfPaper=true` 二次确认（§34.10）。
pub async fn set_default(
    state: web::Data<AppState>,
    body: web::Json<SetDefaultRequest>,
) -> Result<HttpResponse> {
    match svc::set_default(&state.db, body.id, body.confirm_out_of_paper).await {
        Ok(v) => Ok(mpack_ok(v)),
        Err(e) => Ok(mpack_fail(&e.to_string())),
    }
}

/// # 上传 PDF 反设计（阶段 A：底图模式）
pub async fn parse_pdf(
    state: web::Data<AppState>,
    body: web::Json<ParsePdfRequest>,
) -> Result<HttpResponse> {
    match svc::parse_pdf(&state.db, body.into_inner()).await {
        Ok(v) => Ok(mpack_ok(v)),
        Err(e) => Ok(mpack_fail(&e.to_string())),
    }
}

// ============================================================================
// 编辑期软锁（§33.1）
// ============================================================================

/// # 抢占编辑锁
pub async fn lock_acquire(req: HttpRequest, body: web::Json<LockRequest>) -> Result<HttpResponse> {
    let (uid, name) = get_current_user(&req);
    match svc::acquire_lock(body.template_id, uid, &name) {
        None => Ok(mpack_ok(serde_json::json!({ "granted": true }))),
        Some(lock) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(
            409,
            &format!(
                "模板正被 {} 编辑（锁将于 {} 自动释放）。可等待，或选择强制接管。",
                lock.user_name, lock.expire_at
            ),
            "local",
        ))),
    }
}

/// # 续约（15s 一次）
pub async fn lock_heartbeat(req: HttpRequest, body: web::Json<LockRequest>) -> Result<HttpResponse> {
    let uid = get_current_user_id(&req);
    let ok = svc::heartbeat_lock(body.template_id, uid);
    Ok(mpack_ok(serde_json::json!({ "granted": ok })))
}

/// # 释放锁
pub async fn lock_release(req: HttpRequest, body: web::Json<LockRequest>) -> Result<HttpResponse> {
    let uid = get_current_user_id(&req);
    svc::release_lock(body.template_id, uid);
    Ok(mpack_ok(serde_json::json!({ "released": true })))
}

/// # 查询当前锁
pub async fn lock_current(q: web::Query<IdQuery>) -> Result<HttpResponse> {
    let tid = q
        .template_id
        .or(q.id)
        .ok_or_else(|| Error::from("缺少 templateId"))?;
    Ok(mpack_ok(match svc::current_lock(tid) {
        Some(l) => serde_json::json!({ "locked": true, "userName": l.user_name, "userId": l.user_id.to_string(), "expireAt": l.expire_at }),
        None => serde_json::json!({ "locked": false }),
    }))
}

// ============================================================================
// 模板包导出 / 导入（§33.2）
// ============================================================================

/// # 导出模板包（zip 二进制）
pub async fn bundle_export(state: web::Data<AppState>, q: web::Query<IdQuery>) -> Result<HttpResponse> {
    let id = q.id.or(q.template_id).ok_or_else(|| Error::from("缺少 id"))?;
    let (file_name, bytes) = svc::export_bundle(&state.db, id).await?;
    Ok(HttpResponse::Ok()
        .content_type("application/zip")
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", file_name),
        ))
        .body(bytes))
}

/// # 导入模板包
pub async fn bundle_import(
    state: web::Data<AppState>,
    req: HttpRequest,
    MultipartForm(form): MultipartForm<BundleImportForm>,
) -> Result<HttpResponse> {
    let (uid, _) = get_current_user(&req);
    let operator = if uid > 0 { Some(uid) } else { None };
    let path = form
        .file
        .file
        .path()
        .to_path_buf();
    let bytes = std::fs::read(&path).map_err(|e| Error::from(format!("读取上传文件失败: {}", e)))?;
    match svc::import_bundle(&state.db, &bytes, operator).await {
        Ok(v) => Ok(mpack_ok(v)),
        Err(e) => Ok(mpack_fail(&e.to_string())),
    }
}

// ============================================================================
// 素材管理
// ============================================================================

/// # 素材列表
pub async fn asset_list(state: web::Data<AppState>, q: web::Query<AssetListQuery>) -> Result<HttpResponse> {
    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(20);
    let (items, total) = pdf_asset_service::list(
        &state.db,
        q.category.as_deref(),
        q.name.as_deref(),
        page,
        page_size,
    )
    .await?;
    let rp = ResultPage::new(items, total, page, page_size);
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success_with_page(
        rp,
        "local",
        page as u32,
        total as u32,
    )))
}

/// # 素材上传（返回 JSON，符合项目上传接口惯例）
pub async fn asset_upload(
    state: web::Data<AppState>,
    req: HttpRequest,
    MultipartForm(form): MultipartForm<PdfAssetUploadForm>,
) -> Result<HttpResponse> {
    let uid = get_current_user_id(&req);
    let operator = if uid > 0 { Some(uid) } else { None };
    let category = form
        .category
        .as_ref()
        .map(|t| t.0.trim().to_string())
        .filter(|c| !c.is_empty())
        .unwrap_or_else(|| "logo".to_string());
    let name = form.name.as_ref().map(|t| t.0.clone());
    match pdf_asset_service::upload(&state.db, form.file, name, category, operator).await {
        Ok(v) => Ok(HttpResponse::Ok().json(JsonResp::ok(v))),
        Err(e) => Ok(HttpResponse::Ok().json(JsonResp::<()>::err(e.to_string()))),
    }
}

/// # 素材登记（已有附件的 URL 登记为设计素材）
pub async fn asset_save(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<AssetSaveRequest>,
) -> Result<HttpResponse> {
    let uid = get_current_user_id(&req);
    let operator = if uid > 0 { Some(uid) } else { None };
    let b = body.into_inner();
    let req = pdf_asset_service::PdfAssetSaveRequest {
        id: b.id,
        name: b.name,
        category: b.category,
        file_url: b.file_url,
        file_path: b.file_path,
        file_size: b.file_size,
        width_px: b.width_px,
        height_px: b.height_px,
        sort: b.sort,
        status: b.status,
    };
    match pdf_asset_service::save(&state.db, req, operator).await {
        Ok(id) => Ok(mpack_ok(serde_json::json!({ "id": id.to_string() }))),
        Err(e) => Ok(mpack_fail(&e.to_string())),
    }
}

/// # 素材删除（软删除）
pub async fn asset_delete(
    state: web::Data<AppState>,
    body: web::Json<AssetDeleteRequest>,
) -> Result<HttpResponse> {
    match pdf_asset_service::delete(&state.db, &body.ids).await {
        Ok(n) => Ok(mpack_ok(serde_json::json!({ "affected": n }))),
        Err(e) => Ok(mpack_fail(&e.to_string())),
    }
}

// ============================================================================
// 路由注册
// ============================================================================

pub fn register(cfg: &mut web::ServiceConfig) {
    const P_LIST: &str = "system:pdf-designer:list";
    const P_DESIGN: &str = "system:pdf-designer:design";
    const P_SAVE: &str = "system:pdf-designer:save";
    const P_SET_DEFAULT: &str = "system:pdf-designer:set-default";
    const P_BUNDLE: &str = "system:pdf-designer:bundle";
    const A_LIST: &str = "system:pdf-asset:list";
    const A_CREATE: &str = "system:pdf-asset:create";
    const A_UPDATE: &str = "system:pdf-asset:update";
    const A_DELETE: &str = "system:pdf-asset:delete";

    cfg.service(
        web::scope("/pdf-designer")
            // ---- 元数据 ----
            .route("/doc-types", web::get().to(doc_types).wrap(require_permission(P_LIST)))
            .route("/formatters", web::get().to(formatters).wrap(require_permission(P_DESIGN)))
            .route("/presets", web::get().to(presets).wrap(require_permission(P_DESIGN)))
            .route("/field-tree", web::get().to(field_tree).wrap(require_permission(P_DESIGN)))
            .route("/sample-data", web::get().to(sample_data).wrap(require_permission(P_DESIGN)))
            .route("/real-data", web::get().to(real_data).wrap(require_permission(P_DESIGN)))
            .route(
                "/validate-bindings",
                web::post().to(validate_bindings).wrap(require_permission(P_DESIGN)),
            )
            // ---- 模板读写 ----
            .route("/template-info", web::get().to(template_info).wrap(require_permission(P_DESIGN)))
            .route("/save-layout", web::post().to(save_layout).wrap(require_permission(P_SAVE)))
            .route(
                "/precise-preview",
                web::post().to(precise_preview).wrap(require_permission(P_DESIGN)),
            )
            .route("/preview-svg", web::get().to(preview_svg).wrap(require_permission(P_DESIGN)))
            .route("/versions", web::get().to(versions).wrap(require_permission(P_DESIGN)))
            .route("/rollback", web::post().to(rollback).wrap(require_permission(P_SAVE)))
            .route("/duplicate", web::post().to(duplicate).wrap(require_permission(P_SAVE)))
            .route(
                "/set-default",
                web::put().to(set_default).wrap(require_permission(P_SET_DEFAULT)),
            )
            .route("/parse-pdf", web::post().to(parse_pdf).wrap(require_permission(P_DESIGN)))
            // ---- 软锁 ----
            .route("/lock/acquire", web::post().to(lock_acquire).wrap(require_permission(P_DESIGN)))
            .route(
                "/lock/heartbeat",
                web::post().to(lock_heartbeat).wrap(require_permission(P_DESIGN)),
            )
            .route("/lock/release", web::post().to(lock_release).wrap(require_permission(P_DESIGN)))
            .route("/lock/current", web::get().to(lock_current).wrap(require_permission(P_DESIGN)))
            // ---- 模板包 ----
            .route("/bundle/export", web::get().to(bundle_export).wrap(require_permission(P_BUNDLE)))
            .route("/bundle/import", web::post().to(bundle_import).wrap(require_permission(P_BUNDLE))),
    );

    cfg.service(
        web::scope("/pdf-asset")
            .route("/list", web::get().to(asset_list).wrap(require_permission(A_LIST)))
            .route("/upload", web::post().to(asset_upload).wrap(require_permission(A_CREATE)))
            .route("/save", web::post().to(asset_save).wrap(require_permission(A_UPDATE)))
            .route("/delete", web::post().to(asset_delete).wrap(require_permission(A_DELETE))),
    );
}
