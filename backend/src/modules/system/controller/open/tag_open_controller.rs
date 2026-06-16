use crate::core::errors::error::Result;
use actix_web::{get, HttpResponse, web};
use crate::core::kit::global::AppState;
use crate::core::web::response::MetaResp;
use crate::modules::system::service::tag_service;

#[get("/tag/suggest")]
pub async fn suggest_tag(state: web::Data<AppState>, query: web::Query<(String,)>) -> Result<HttpResponse> {
    let db = &state.db;
    let keyword = &query.0.0;
    match tag_service::suggest(&db, keyword).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("获取标签建议出错：{:}", e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e, "local")))
        }
    }
}

#[get("/tag/by-type/{tag_type}")]
pub async fn get_tag_by_type(state: web::Data<AppState>, path: web::Path<String>) -> Result<HttpResponse> {
    let db = &state.db;
    let tag_type = path.into_inner();
    match tag_service::get_by_type(&db, &tag_type).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("获取标签列表出错：{:}", e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e, "local")))
        }
    }
}

#[get("/tag/all")]
pub async fn get_all_tags(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    match tag_service::get_all(&db).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("获取所有标签出错：{:}", e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e, "local")))
        }
    }
}
