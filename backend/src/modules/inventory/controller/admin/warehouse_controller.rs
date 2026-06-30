use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::entity::common::BathDeleteIdRequest;
use crate::core::web::response::MetaResp;
use crate::modules::inventory::model::warehouse::{WarehouseDetailVO, WarehouseListQuery, WarehouseSaveRequest, WarehouseUpdateRequest};
use crate::modules::inventory::service::warehouse_service;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;

#[post("/warehouse/save")]
#[protect("product:warehouse:create")]
pub async fn warehouse_insert(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<WarehouseSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let form_data = form_data.0;

    let result = warehouse_service::insert(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[put("/warehouse/update")]
#[protect("product:warehouse:edit")]
pub async fn warehouse_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<WarehouseUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let form_data = form_data.0;

    if form_data.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "仓库ID不能为空", "local")));
    }

    let result = warehouse_service::update(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[delete("/warehouse/bath_delete")]
#[protect("product:warehouse:delete")]
pub async fn batch_delete_warehouse(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let ids: Vec<i64> = item.0.ids.unwrap_or_default()
        .into_iter()
        .flatten()
        .filter_map(|s| s.parse().ok())
        .collect();
    if ids.is_empty() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "请选择要删除的记录", "local")));
    }
    let result = warehouse_service::batch_delete(&db, &ids).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[get("/warehouse/info")]
#[protect("product:warehouse:edit")]
pub async fn warehouse_info(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let id = req.query_string().split("&").find(|s| s.starts_with("id=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())).unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }

    match warehouse_service::get_detail(&db, id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[get("/warehouse/list")]
#[protect("product:warehouse:list")]
pub async fn warehouse_list(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let query_str = req.query_string();

    fn q<'a>(qs: &'a str, key: &str) -> Option<&'a str> {
        qs.split('&').find(|s| s.starts_with(&format!("{}=", key)))
            .and_then(|s| s.split('=').nth(1))
    }

    let query = WarehouseListQuery {
        page_num: q(query_str, "page").and_then(|s| s.parse().ok()),
        page_size: q(query_str, "pageSize").and_then(|s| s.parse().ok()),
        warehouse_name: q(query_str, "warehouseName").map(|s| s.to_string()),
    };

    match warehouse_service::get_list(&db, &query).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}
