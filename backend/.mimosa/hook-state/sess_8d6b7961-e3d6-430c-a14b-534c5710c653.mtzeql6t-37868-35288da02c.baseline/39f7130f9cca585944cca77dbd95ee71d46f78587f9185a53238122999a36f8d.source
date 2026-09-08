use actix_web::{web, HttpResponse, Result};
use serde::{Serialize, Deserialize};
use crate::core::kit::global::AppState;
use crate::modules::system::service::area_service::AreaService;
use crate::modules::system::model::area::{AreaSaveRequest, AreaUpdateRequest, AreaSaveDTO, AreaListQuery};


use crate::core::web::response::MPACK;
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchDeleteRequest {
    pub ids: Vec<String>,
}

fn msgpack_response<T: Serialize>(data: T) -> HttpResponse {
    HttpResponse::Ok()
        .content_type(MPACK)
        .body(rmp_serde::to_vec_named(&data).unwrap_or_default())
}

pub async fn get_area_tree(state: web::Data<AppState>) -> Result<HttpResponse> {
    let result = AreaService::get_area_tree(&state.db).await;
    Ok(msgpack_response(result))
}

pub async fn get_cascader_data(state: web::Data<AppState>) -> Result<HttpResponse> {
    let result = AreaService::get_cascader_data(&state.db).await;
    Ok(msgpack_response(result))
}

pub async fn get_countries(state: web::Data<AppState>) -> Result<HttpResponse> {
    let result = AreaService::get_countries(&state.db).await;
    Ok(msgpack_response(result))
}

pub async fn get_provinces(state: web::Data<AppState>, query: web::Query<ProvinceQuery>) -> Result<HttpResponse> {
    let result = AreaService::get_provinces(&state.db, query.country_code.clone()).await;
    Ok(msgpack_response(result))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProvinceQuery {
    pub country_code: String,
}

pub async fn get_children(state: web::Data<AppState>, query: web::Query<ChildrenQuery>) -> Result<HttpResponse> {
    let result = AreaService::get_children(&state.db, query.parent_id.clone()).await;
    Ok(msgpack_response(result))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChildrenQuery {
    pub parent_id: String,
}

pub async fn get_detail(state: web::Data<AppState>, id: web::Path<String>) -> Result<HttpResponse> {
    let result = AreaService::get_detail(&state.db, id.into_inner()).await;
    Ok(msgpack_response(result))
}

pub async fn get_by_page(state: web::Data<AppState>, query: web::Query<AreaListQuery>) -> Result<HttpResponse> {
    let result = AreaService::get_by_page(&state.db, query.into_inner()).await;
    Ok(msgpack_response(result))
}

pub async fn insert(state: web::Data<AppState>, body: web::Json<AreaSaveRequest>) -> Result<HttpResponse> {
    let form_data: AreaSaveDTO = body.into_inner().into();
    let result = AreaService::insert(&state.db, form_data).await;
    Ok(msgpack_response(result))
}

pub async fn update(state: web::Data<AppState>, id: web::Path<String>, body: web::Json<AreaUpdateRequest>) -> Result<HttpResponse> {
    let form_data: AreaSaveDTO = body.into_inner().into();
    let result = AreaService::update(&state.db, id.into_inner(), form_data).await;
    Ok(msgpack_response(result))
}

pub async fn batch_delete(state: web::Data<AppState>, body: web::Json<BatchDeleteRequest>) -> Result<HttpResponse> {
    let result = AreaService::batch_delete(&state.db, body.into_inner().ids).await;
    Ok(msgpack_response(result))
}

// ==================== 路由注册（单点维护）====================

/// 注册行政区域模块所有路由
///
/// 修改路径、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(area_admin_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/area")
            // GET /area/tree - 获取地区树
            .route("/tree", web::get().to(get_area_tree))
            // GET /area/cascader - 获取级联数据
            .route("/cascader", web::get().to(get_cascader_data))
            // GET /area/countries - 获取国家列表
            .route("/countries", web::get().to(get_countries))
            // GET /area/provinces - 获取省份列表
            .route("/provinces", web::get().to(get_provinces))
            // GET /area/children - 获取子地区
            .route("/children", web::get().to(get_children))
            // GET /area/detail/{id} - 获取地区详情
            .route("/detail/{id}", web::get().to(get_detail))
            // GET /area/list - 分页获取地区列表
            .route("/list", web::get().to(get_by_page))
            // POST /area/save - 新增地区
            .route("/save", web::post().to(insert))
            // PUT /area/update/{id} - 修改地区
            .route("/update/{id}", web::put().to(update))
            // DELETE /area/batch_delete - 批量删除地区
            .route("/batch_delete", web::delete().to(batch_delete)),
    );
}
