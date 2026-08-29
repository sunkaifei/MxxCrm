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
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::product::model::product_unit::{
    ProductUnitDeleteRequest, ProductUnitSaveRequest, ProductUnitUpdateRequest,
};
use crate::modules::product::service::product_unit_service;
use actix_web::{web, HttpRequest, HttpResponse};

/// 单位选项列表（选择器用；低风险共享基础资料，登录即可访问，同字典 options 先例）
pub async fn product_unit_options(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match product_unit_service::get_options(db).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 新增单位（选择器内联新增；登录即可，参照金蝶录单补录计量单位的策略）
pub async fn product_unit_save(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<ProductUnitSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    if form_data.name.as_ref().map_or(true, |name| name.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "单位名称不能为空", "local")));
    }

    let result = product_unit_service::save(&db, &form_data, get_current_user_id(&req)).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

/// 修改单位小数位精度（仅精度；不改名——mxx_product.unit 按名称引用）
pub async fn product_unit_update(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<ProductUnitUpdateRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let result = product_unit_service::update_precision(&db, &form_data, get_current_user_id(&req)).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

/// 删除单位（软删除；默认单位与在用单位禁止删除）
pub async fn product_unit_delete(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<ProductUnitDeleteRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = query.0;
    let result = product_unit_service::delete_unit(&db, &form_data, get_current_user_id(&req)).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/product/unit")
            .route("/options", web::get().to(product_unit_options))
            .route("/save", web::post().to(product_unit_save))
            .route("/update", web::put().to(product_unit_update))
            .route("/delete", web::delete().to(product_unit_delete)),
    );
}
