use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::permission_guard::require_permission;

use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::product::model::category::{CategoryDetailVO, CategoryListQuery, CategoryListVO, CategorySaveRequest, CategoryUpdateRequest};
use crate::modules::product::service::category_service;
use actix_web::{web, HttpResponse};

pub async fn category_insert(state: web::Data<AppState>, form_data: web::Json<CategorySaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    let result = category_service::insert(&db, &form_data).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn category_update(state: web::Data<AppState>, form_data: web::Json<CategoryUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;

    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "分类ID不能为空", "local")));
    }

    let result = category_service::update(&db, &form_data).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn batch_delete_category(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
    let db = &state.db;
    let delete_item = item.0;

    if delete_item.ids.is_none() || delete_item.ids.as_ref().unwrap().is_empty() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "未获取到删除的分类ID", "local"));
    }

    let filtered_ids: Vec<i64> = delete_item.ids.unwrap_or_default()
        .iter()
        .filter_map(|item| item.as_ref().and_then(|s| s.trim().parse().ok()))
        .collect();

    let result = category_service::batch_delete_by_ids(&db, &filtered_ids).await;
    HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result))
}

pub async fn info_category(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "分类ID不能为空", "local"));
    }

    match category_service::find_by_id(&db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn list_category(state: web::Data<AppState>, query: web::Query<CategoryListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.0;

    match category_service::list(&db, &query).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success_with_page(page_data, "local", page, total))
        },
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册产品分类模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(product_category_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/product/category")
            // POST /product/category/save - 新建分类
            .route(
                "/save",
                web::post()
                    .to(category_insert)
                    .wrap(require_permission("product:category:save")),
            )
            // PUT /product/category/update - 修改分类
            .route(
                "/update",
                web::put()
                    .to(category_update)
                    .wrap(require_permission("product:category:update")),
            )
            // DELETE /product/category/bath_delete - 批量删除分类
            .route(
                "/bath_delete",
                web::delete()
                    .to(batch_delete_category)
                    .wrap(require_permission("product:category:delete")),
            )
            // GET /product/category/info - 分类详情
            .route(
                "/info",
                web::get()
                    .to(info_category)
                    .wrap(require_permission("product:category:info")),
            )
            // GET /product/category/list - 分类列表
            .route(
                "/list",
                web::get()
                    .to(list_category)
                    .wrap(require_permission("product:category:list")),
            ),
    );
}
