use crate::core::errors::error::Result;
use actix_web::{web, HttpRequest, HttpResponse};
use crate::core::web::permission_guard::require_permission;
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::ai::model::ai_config::AiConfigSaveDTO;
use crate::modules::ai::service::ai_config_service;
use crate::modules::system::service::admin_service;

pub async fn insert_ai_config(state: web::Data<AppState>, req: HttpRequest, item: web::Json<AiConfigSaveDTO>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.config_key.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "配置key不能为空", "local")));
    }
    if ai_config_service::find_by_key_unique(&db, &item.config_key, &None).await.unwrap_or(false) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "配置key已存在", "local")));
    }
    if item.config_name.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "配置名称不能为空", "local")));
    }
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let admin = admin_service::get_by_detail(&db, &jwt_token.id).await?;
    let mut form_data = item.into_inner();
    form_data.created_by = admin.user_name.clone();
    form_data.updated_by = admin.user_name;
    match ai_config_service::insert(&db, &form_data).await {
        Ok(id) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(id, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn update_ai_config(state: web::Data<AppState>, req: HttpRequest, item: web::Json<AiConfigSaveDTO>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    if item.config_key.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "配置key不能为空", "local")));
    }
    if ai_config_service::find_by_key_unique(&db, &item.config_key, &item.id).await.unwrap_or(false) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "配置key已存在", "local")));
    }
    if item.config_name.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "配置名称不能为空", "local")));
    }
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let admin = admin_service::get_by_detail(&db, &jwt_token.id).await?;
    let mut form_data = item.into_inner();
    form_data.updated_by = admin.user_name;
    match ai_config_service::update_by_id(&db, &form_data.id, &form_data).await {
        Ok(id) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(id, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_ai_config_list(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    match ai_config_service::get_all(&db).await {
        Ok(list) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(list, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_ai_config_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    match ai_config_service::get_by_detail(&db, &item.id).await {
        Ok(Some(detail)) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(detail, "local"))),
        Ok(None) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "配置不存在", "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn delete_ai_config(state: web::Data<AppState>, id: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    match ai_config_service::delete_by_id(&db, id.into_inner()).await {
        Ok(count) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(count, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn batch_delete_ai_config(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除ID不能为空", "local")));
        }
        let ids: Vec<i64> = ids_vec.iter().filter_map(|id| id.clone().and_then(|s| s.parse().ok())).collect();
        match ai_config_service::batch_delete_by_ids(&db, ids).await {
            Ok(count) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(count, "local"))),
            Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
        }
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除ID不能为空", "local")))
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/ai-config")
            .route("/add", web::post().to(insert_ai_config).wrap(require_permission("crm:ai:update")))
            .route("/update", web::put().to(update_ai_config).wrap(require_permission("crm:ai:update")))
            .route("/list", web::get().to(get_ai_config_list).wrap(require_permission("crm:ai:view")))
            .route("/detail/{id}", web::get().to(get_ai_config_detail).wrap(require_permission("crm:ai:view")))
            .route("/delete/{id}", web::delete().to(delete_ai_config).wrap(require_permission("crm:ai:update")))
            .route("/batch-delete", web::delete().to(batch_delete_ai_config).wrap(require_permission("crm:ai:update"))),
    );
}
