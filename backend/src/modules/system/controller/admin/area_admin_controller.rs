use actix_web::{get, post, put, delete, web, HttpResponse};
use serde::{Serialize, Deserialize};
use crate::modules::system::service::area_service::AreaService;
use crate::modules::system::model::area::{AreaSaveRequest, AreaUpdateRequest, AreaSaveDTO, AreaListQuery};
use crate::utils::response::ApiResponse;

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchDeleteRequest {
    pub ids: Vec<String>,
}

#[get("/area/tree")]
pub async fn get_area_tree(db: web::Data<crate::core::kit::db::DbConn>) -> HttpResponse {
    let result = AreaService::get_area_tree(db).await;
    HttpResponse::Ok().json(result)
}

#[get("/area/cascader")]
pub async fn get_cascader_data(db: web::Data<crate::core::kit::db::DbConn>) -> HttpResponse {
    let result = AreaService::get_cascader_data(db).await;
    HttpResponse::Ok().json(result)
}

#[get("/area/countries")]
pub async fn get_countries(db: web::Data<crate::core::kit::db::DbConn>) -> HttpResponse {
    let result = AreaService::get_countries(db).await;
    HttpResponse::Ok().json(result)
}

#[get("/area/provinces")]
pub async fn get_provinces(db: web::Data<crate::core::kit::db::DbConn>, query: web::Query<ProvinceQuery>) -> HttpResponse {
    let result = AreaService::get_provinces(db, query.country_code.clone()).await;
    HttpResponse::Ok().json(result)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProvinceQuery {
    pub country_code: String,
}

#[get("/area/children")]
pub async fn get_children(db: web::Data<crate::core::kit::db::DbConn>, query: web::Query<ChildrenQuery>) -> HttpResponse {
    let result = AreaService::get_children(db, query.parent_id.clone()).await;
    HttpResponse::Ok().json(result)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChildrenQuery {
    pub parent_id: String,
}

#[get("/area/detail/{id}")]
pub async fn get_detail(db: web::Data<crate::core::kit::db::DbConn>, id: web::Path<String>) -> HttpResponse {
    let result = AreaService::get_detail(db, id.into_inner()).await;
    HttpResponse::Ok().json(result)
}

#[get("/area/list")]
pub async fn get_by_page(db: web::Data<crate::core::kit::db::DbConn>, query: web::Query<AreaListQuery>) -> HttpResponse {
    let result = AreaService::get_by_page(db, query.into_inner()).await;
    HttpResponse::Ok().json(result)
}

#[post("/area/save")]
pub async fn insert(db: web::Data<crate::core::kit::db::DbConn>, body: web::Json<AreaSaveRequest>) -> HttpResponse {
    let form_data: AreaSaveDTO = body.into_inner().into();
    let result = AreaService::insert(db, form_data).await;
    HttpResponse::Ok().json(result)
}

#[put("/area/update/{id}")]
pub async fn update(db: web::Data<crate::core::kit::db::DbConn>, id: web::Path<String>, body: web::Json<AreaUpdateRequest>) -> HttpResponse {
    let form_data: AreaSaveDTO = body.into_inner().into();
    let result = AreaService::update(db, id.into_inner(), form_data).await;
    HttpResponse::Ok().json(result)
}

#[delete("/area/batch_delete")]
pub async fn batch_delete(db: web::Data<crate::core::kit::db::DbConn>, body: web::Json<BatchDeleteRequest>) -> HttpResponse {
    let result = AreaService::batch_delete(db, body.into_inner().ids).await;
    HttpResponse::Ok().json(result)
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
