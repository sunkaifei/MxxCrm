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
use crate::core::web::response::MetaResp;
use crate::modules::product::model::product::ProductModel;
use crate::modules::product::model::spec::{SpecBatchSaveRequest, SkuBatchSaveRequest};
use crate::modules::product::service::spec_service;
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;

/// 获取产品规格定义和SKU列表
#[get("/product/spec/list")]
#[protect("product:product:view")]
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
#[post("/product/spec/save")]
#[protect("product:product:save")]
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
#[get("/product/sku/generate")]
#[protect("product:product:view")]
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
#[post("/product/sku/batchSave")]
#[protect("product:product:save")]
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
