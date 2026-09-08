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
use crate::core::kit::global::AppState;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, ResultPage, MPACK};
use crate::modules::product::model::sku_template::*;
use crate::modules::product::service::sku_template_service;
use actix_web::{web, HttpRequest, HttpResponse};

/// SKU模板列表查询
pub async fn template_list(
    state: web::Data<AppState>,
    query: web::Query<SkuTemplateListQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.0;

    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    match sku_template_service::list_templates(db, page, page_size, query.keywords, query.product_type).await {
        Ok((items, total)) => {
            let page_data = ResultPage::new(items, total, page, page_size);
            Ok(HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::success(page_data, "local")))
        }
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// SKU模板详情（含规格）
pub async fn template_info(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let db = &state.db;
    let template_id = req.query_string()
        .split("&")
        .find(|s| s.starts_with("id="))
        .and_then(|s| s.split("=").nth(1))
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);

    if template_id <= 0 {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "模板ID无效", "local")));
    }

    match sku_template_service::get_template_detail(db, template_id).await {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 保存SKU模板
pub async fn template_save(
    state: web::Data<AppState>,
    form_data: web::Json<SkuTemplateSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match sku_template_service::insert_template(db, &form_data.0).await {
        Ok(id) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success_with_msg(id, "模板创建成功", "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 更新SKU模板
pub async fn template_update(
    state: web::Data<AppState>,
    form_data: web::Json<SkuTemplateUpdateRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let item = form_data.0;
    let id = item.id.unwrap_or(0);

    if id <= 0 {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "模板ID无效", "local")));
    }

    match sku_template_service::update_template(db, id, &item).await {
        Ok(_) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::success("更新成功".to_string(), "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 删除SKU模板（软删除）
pub async fn template_delete(
    state: web::Data<AppState>,
    form_data: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let id = form_data.0.get("id").and_then(|v| v.as_i64()).unwrap_or(0);

    if id <= 0 {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "模板ID无效", "local")));
    }

    match sku_template_service::delete_template(db, id).await {
        Ok(_) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::success("删除成功".to_string(), "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 保存SKU模板规格
pub async fn template_spec_save(
    state: web::Data<AppState>,
    form_data: web::Json<TemplateSpecBatchSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let result = sku_template_service::save_template_specs(db, &form_data.0).await;
    match result {
        Ok(_) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::success("规格保存成功".to_string(), "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册 SKU 模板模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(sku_template_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sku/template")
            // GET /sku/template/list - SKU模板列表查询
            .route(
                "/list",
                web::get()
                    .to(template_list)
                    .wrap(require_permission("product:product:view")),
            )
            // GET /sku/template/info - SKU模板详情
            .route(
                "/info",
                web::get()
                    .to(template_info)
                    .wrap(require_permission("product:product:view")),
            )
            // POST /sku/template/save - 保存SKU模板
            .route(
                "/save",
                web::post()
                    .to(template_save)
                    .wrap(require_permission("product:product:save")),
            )
            // PUT /sku/template/update - 更新SKU模板
            .route(
                "/update",
                web::put()
                    .to(template_update)
                    .wrap(require_permission("product:product:save")),
            )
            // POST /sku/template/delete - 删除SKU模板
            .route(
                "/delete",
                web::post()
                    .to(template_delete)
                    .wrap(require_permission("product:product:delete")),
            )
            // POST /sku/template/spec/save - 保存SKU模板规格
            .route(
                "/spec/save",
                web::post()
                    .to(template_spec_save)
                    .wrap(require_permission("product:product:save")),
            ),
    );
}
