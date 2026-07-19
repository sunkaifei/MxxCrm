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
use crate::core::web::response::MetaResp;
use crate::modules::product::model::product::ProductModel;
use crate::modules::product::model::spec::{SpecBatchSaveRequest, SkuBatchSaveRequest};
use crate::modules::product::service::spec_service;
use actix_web::{web, HttpRequest, HttpResponse};
use serde_json;

/// 获取产品规格定义和SKU列表
pub async fn get_product_specs(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let product_id = req.query_string()
        .split("&")
        .find(|s| s.starts_with("productId="))
        .and_then(|s| s.split("=").nth(1))
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);

    if product_id <= 0 {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(
            MetaResp::<String>::fail(400, "产品ID无效", "local"),
        ));
    }

    match spec_service::get_specs(db, product_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 保存产品规格定义
pub async fn save_product_specs(
    state: web::Data<AppState>,
    form_data: web::Json<SpecBatchSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    let result = spec_service::save_specs(db, &form_data).await;
    match result {
        Ok(_) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::success("保存成功".to_string(), "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 根据规格组合自动生成SKU
pub async fn generate_skus(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let product_id = req.query_string()
        .split("&")
        .find(|s| s.starts_with("productId="))
        .and_then(|s| s.split("=").nth(1))
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);

    if product_id <= 0 {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(
            MetaResp::<String>::fail(400, "产品ID无效", "local"),
        ));
    }

    match spec_service::generate_skus(db, product_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 批量保存SKU（独立保存，不影响产品主数据）
pub async fn batch_save_skus(
    state: web::Data<AppState>,
    form_data: web::Json<SkuBatchSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let item = form_data.0;

    if item.product_id <= 0 || item.skus.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(
            MetaResp::<String>::fail(400, "参数无效", "local"),
        ));
    }

    let result = ProductModel::batch_save_skus(db, item.product_id, &item.skus.unwrap_or_default()).await;
    match result {
        Ok(_) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::success("SKU保存成功".to_string(), "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &format!("SKU保存失败: {}", e), "local"))),
    }
}

/// 获取可用规格值（淘宝式级联选择）
/// 根据已选择的规格，返回剩余规格的可用值
pub async fn get_available_spec_values(
    state: web::Data<AppState>,
    form_data: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let data = form_data.0;

    let product_id = data.get("productId")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let selected_specs = data.get("selectedSpecs")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    if product_id <= 0 {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(
            MetaResp::<String>::fail(400, "产品ID无效", "local"),
        ));
    }

    match spec_service::get_available_spec_values(db, product_id, selected_specs).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 根据规格组合获取对应的SKU
pub async fn get_sku_by_specs(
    state: web::Data<AppState>,
    form_data: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let data = form_data.0;

    let product_id = data.get("productId")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let specs = data.get("specs")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    if product_id <= 0 {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(
            MetaResp::<String>::fail(400, "产品ID无效", "local"),
        ));
    }

    match spec_service::get_sku_by_specs(db, product_id, specs).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册产品规格/SKU 模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(spec_controller::register)` 注册。
///
/// 注意：本模块包含两个路径前缀（/product/spec 和 /product/sku），
/// 因此在 register 中使用两个独立的 scope。
pub fn register(cfg: &mut web::ServiceConfig) {
    // 规格定义
    cfg.service(
        web::scope("/product/spec")
            // GET /product/spec/list - 获取产品规格定义和SKU列表
            .route(
                "/list",
                web::get()
                    .to(get_product_specs)
                    .wrap(require_permission("product:product:view")),
            )
            // POST /product/spec/save - 保存产品规格定义
            .route(
                "/save",
                web::post()
                    .to(save_product_specs)
                    .wrap(require_permission("product:product:save")),
            )
            // POST /product/spec/availableValues - 获取可用规格值
            .route(
                "/availableValues",
                web::post()
                    .to(get_available_spec_values)
                    .wrap(require_permission("product:product:view")),
            ),
    );
    // SKU 管理
    cfg.service(
        web::scope("/product/sku")
            // GET /product/sku/generate - 根据规格组合自动生成SKU
            .route(
                "/generate",
                web::get()
                    .to(generate_skus)
                    .wrap(require_permission("product:product:view")),
            )
            // POST /product/sku/batchSave - 批量保存SKU
            .route(
                "/batchSave",
                web::post()
                    .to(batch_save_skus)
                    .wrap(require_permission("product:product:save")),
            )
            // POST /product/sku/getBySpecs - 根据规格组合获取对应的SKU
            .route(
                "/getBySpecs",
                web::post()
                    .to(get_sku_by_specs)
                    .wrap(require_permission("product:product:view")),
            ),
    );
}
