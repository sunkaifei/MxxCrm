use actix_web::{get, post, put, delete, web, HttpResponse, Result};
use serde::{Serialize, Deserialize};
use crate::core::kit::global::AppState;
use crate::modules::system::service::area_service::AreaService;
use crate::modules::system::model::area::{AreaSaveRequest, AreaUpdateRequest, AreaSaveDTO, AreaListQuery};


#[derive(Debug, Serialize, Deserialize)]
pub struct BatchDeleteRequest {
    pub ids: Vec<String>,
}

fn msgpack_response<T: Serialize>(data: T) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(rmp_serde::to_vec_named(&data).unwrap_or_default())
}

#[get("/area/tree")]
pub async fn get_area_tree(state: web::Data<AppState>) -> Result<HttpResponse> {
    let result = AreaService::get_area_tree(&state.db).await;
    Ok(msgpack_response(result))
}

#[get("/area/cascader")]
pub async fn get_cascader_data(state: web::Data<AppState>) -> Result<HttpResponse> {
    let result = AreaService::get_cascader_data(&state.db).await;
    Ok(msgpack_response(result))
}

#[get("/area/countries")]
pub async fn get_countries(state: web::Data<AppState>) -> Result<HttpResponse> {
    let result = AreaService::get_countries(&state.db).await;
    Ok(msgpack_response(result))
}

#[get("/area/provinces")]
pub async fn get_provinces(state: web::Data<AppState>, query: web::Query<ProvinceQuery>) -> Result<HttpResponse> {
    let result = AreaService::get_provinces(&state.db, query.country_code.clone()).await;
    Ok(msgpack_response(result))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProvinceQuery {
    pub country_code: String,
}

#[get("/area/children")]
pub async fn get_children(state: web::Data<AppState>, query: web::Query<ChildrenQuery>) -> Result<HttpResponse> {
    let result = AreaService::get_children(&state.db, query.parent_id.clone()).await;
    Ok(msgpack_response(result))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChildrenQuery {
    pub parent_id: String,
}

#[get("/area/detail/{id}")]
pub async fn get_detail(state: web::Data<AppState>, id: web::Path<String>) -> Result<HttpResponse> {
    let result = AreaService::get_detail(&state.db, id.into_inner()).await;
    Ok(msgpack_response(result))
}

#[get("/area/list")]
pub async fn get_by_page(state: web::Data<AppState>, query: web::Query<AreaListQuery>) -> Result<HttpResponse> {
    let result = AreaService::get_by_page(&state.db, query.into_inner()).await;
    Ok(msgpack_response(result))
}

#[post("/area/save")]
pub async fn insert(state: web::Data<AppState>, body: web::Json<AreaSaveRequest>) -> Result<HttpResponse> {
    let form_data: AreaSaveDTO = body.into_inner().into();
    let result = AreaService::insert(&state.db, form_data).await;
    Ok(msgpack_response(result))
}

#[put("/area/update/{id}")]
pub async fn update(state: web::Data<AppState>, id: web::Path<String>, body: web::Json<AreaUpdateRequest>) -> Result<HttpResponse> {
    let form_data: AreaSaveDTO = body.into_inner().into();
    let result = AreaService::update(&state.db, id.into_inner(), form_data).await;
    Ok(msgpack_response(result))
}

#[delete("/area/batch_delete")]
pub async fn batch_delete(state: web::Data<AppState>, body: web::Json<BatchDeleteRequest>) -> Result<HttpResponse> {
    let result = AreaService::batch_delete(&state.db, body.into_inner().ids).await;
    Ok(msgpack_response(result))
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_area_tree);
    cfg.service(get_cascader_data);
    cfg.service(get_countries);
    cfg.service(get_provinces);
    cfg.service(get_children);
    cfg.service(get_detail);
    cfg.service(get_by_page);
    cfg.service(insert);
    cfg.service(update);
    cfg.service(batch_delete);
}
