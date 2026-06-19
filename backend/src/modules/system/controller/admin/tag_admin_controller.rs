use crate::core::errors::error::Result;
use actix_web::{delete, get, HttpResponse, post, put, web, HttpRequest};
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::MetaResp;
use crate::modules::system::model::tag::{TagSaveDTO, TagSaveRequest, TagUpdateRequest, TagListQuery, TagMoveToGroupRequest};
use crate::modules::system::model::tag_group::{TagGroupSaveDTO, TagGroupSaveRequest, TagGroupUpdateRequest};
use crate::modules::system::model::tag_merge::{TagEntityRequest, TagEntityRemoveRequest, TagEntityBatchRequest};
use crate::modules::system::service::{admin_service, tag_service, tag_group_service};

#[post("/tag/add")]
pub async fn save_tag(state: web::Data<AppState>, req: HttpRequest, payload: web::Json<TagSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let tag_request = payload.0;
    if tag_request.tag_name.as_ref().map_or(true, |tag_name| tag_name.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "标签名称不能为空", "local")));
    }
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let admin = admin_service::get_by_detail(&db, &jwt_token.id).await?;
    let mut form_data = TagSaveDTO::from(tag_request);
    form_data.created_by = admin.id;
    form_data.updated_by = admin.id;
    match tag_service::TagService::save(&db, form_data, admin.id).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("添加标签出错：{:}", e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

#[put("/tag/update")]
pub async fn update_tag(state: web::Data<AppState>, req: HttpRequest, payload: web::Json<TagUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let tag_request = payload.0;
    if tag_request.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "标签ID不能为空", "local")));
    }
    if tag_request.tag_name.as_ref().map_or(true, |tag_name| tag_name.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "标签名称不能为空", "local")));
    }
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let admin = admin_service::get_by_detail(&db, &jwt_token.id).await?;
    let mut form_data = TagSaveDTO::from(tag_request);
    form_data.updated_by = admin.id;
    match tag_service::TagService::save(&db, form_data, admin.id).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("更新标签出错：{:}", e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

#[delete("/tag/delete/{id}")]
pub async fn delete_tag(state: web::Data<AppState>, item: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let id = item.into_inner();
    match tag_service::TagService::delete(&db, id).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("删除标签出错：{:}", e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

#[delete("/tag/batch_delete")]
pub async fn batch_delete_tag(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
        } else {
            let ids: Vec<i64> = ids_vec.into_iter().filter_map(|id| id.and_then(|s| s.parse().ok())).collect();
            match tag_service::TagService::batch_delete(&db, &ids).await {
                Ok(v) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))),
                Err(e) => {
                    log::error!("批量删除标签出错：{:}", e);
                    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
                }
            }
        }
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

#[get("/tag/detail/{id}")]
pub async fn get_tag_detail(state: web::Data<AppState>, item: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let id = item.into_inner();
    match tag_service::TagService::get_by_id(&db, id).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("获取标签详情出错：{:}", e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

#[get("/tag/list")]
pub async fn get_tag_list(state: web::Data<AppState>, query: web::Query<TagListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let page = query.page_num.unwrap_or(1);
    let per_page = query.page_size.unwrap_or(20);
    match tag_service::TagService::get_list(&db, page, per_page, query.tag_name.clone(), query.group_id, query.is_global).await {
        Ok((list, count)) => {
            let page_resp = crate::core::web::entity::page::Page::with_list(list, page, per_page, count);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_resp, "local")))
        }
        Err(e) => {
            log::error!("获取标签列表出错：{:}", e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

#[get("/tag/statistics")]
pub async fn get_tag_statistics(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    match tag_service::TagService::get_statistics(&db).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("获取标签统计出错：{:}", e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

#[get("/tag/all")]
pub async fn get_all_tags(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    match tag_service::TagService::get_all_tags(&db).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("获取所有标签出错：{:}", e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

#[get("/tag/group/{group_id}")]
pub async fn get_tags_by_group(state: web::Data<AppState>, path: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let group_id = path.into_inner();
    match tag_service::TagService::get_tags_by_group(&db, group_id).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("获取分组下标签出错：{:}", e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

#[post("/tag/move-to-group")]
pub async fn move_tags_to_group(state: web::Data<AppState>, payload: web::Json<TagMoveToGroupRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let req = payload.0;
    if req.group_id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "分组ID不能为空", "local")));
    }
    if req.tag_ids.as_ref().map_or(true, |ids| ids.is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "标签ID列表不能为空", "local")));
    }
    match tag_service::TagService::move_to_group(&db, req.group_id.unwrap(), req.tag_ids.as_ref().unwrap()).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("移动标签到分组出错：{:}", e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

#[get("/tag/suggest")]
pub async fn tag_suggest(state: web::Data<AppState>, query: web::Query<(String,)>) -> Result<HttpResponse> {
    let db = &state.db;
    let keyword = &query.0.0;
    match tag_service::TagService::suggest(&db, keyword).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("标签建议出错：{:}", e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

#[post("/tag/group")]
pub async fn save_tag_group(state: web::Data<AppState>, req: HttpRequest, payload: web::Json<TagGroupSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let group_request = payload.0;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let admin = admin_service::get_by_detail(&db, &jwt_token.id).await?;
    let mut form_data = TagGroupSaveDTO::from(group_request);
    form_data.created_by = admin.id;
    form_data.updated_by = admin.id;
    match tag_group_service::TagGroupService::save(&db, form_data, admin.id).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("保存标签分组出错：{:}", e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

#[put("/tag/group")]
pub async fn update_tag_group(state: web::Data<AppState>, req: HttpRequest, payload: web::Json<TagGroupUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let group_request = payload.0;
    if group_request.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "分组ID不能为空", "local")));
    }
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let admin = admin_service::get_by_detail(&db, &jwt_token.id).await?;
    let mut form_data = TagGroupSaveDTO::from(group_request);
    form_data.updated_by = admin.id;
    match tag_group_service::TagGroupService::save(&db, form_data, admin.id).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("更新标签分组出错：{:}", e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

#[delete("/tag/group/{id}")]
pub async fn delete_tag_group(state: web::Data<AppState>, item: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let id = item.into_inner();
    match tag_group_service::TagGroupService::delete(&db, id).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("删除标签分组出错：{:}", e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

#[delete("/tag/group/batch_delete")]
pub async fn batch_delete_tag_group(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
        } else {
            let ids: Vec<i64> = ids_vec.into_iter().filter_map(|id| id.and_then(|s| s.parse().ok())).collect();
            match tag_group_service::TagGroupService::batch_delete(&db, &ids).await {
                Ok(v) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))),
                Err(e) => {
                    log::error!("批量删除标签分组出错：{:}", e);
                    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
                }
            }
        }
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

#[get("/tag/group/list")]
pub async fn get_tag_group_list(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    match tag_group_service::TagGroupService::get_list(&db).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("获取标签分组列表出错：{:}", e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

#[get("/tag/group/detail/{id}")]
pub async fn get_tag_group_detail(state: web::Data<AppState>, item: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let id = item.into_inner();
    match tag_group_service::TagGroupService::get_by_id(&db, id).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("获取标签分组详情出错：{:}", e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

#[post("/tag/entity/add")]
pub async fn add_tags_to_entity(state: web::Data<AppState>, payload: web::Json<TagEntityRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let req = payload.0;
    if req.entity_type.as_ref().map_or(true, |t| t.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "实体类型不能为空", "local")));
    }
    if req.entity_id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "实体ID不能为空", "local")));
    }
    if req.tag_ids.as_ref().map_or(true, |ids| ids.is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "标签ID列表不能为空", "local")));
    }
    match tag_service::TagService::add_tags_to_entity(&db, req.entity_type.as_ref().unwrap(), req.entity_id.unwrap(), req.tag_ids.as_ref().unwrap()).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("添加标签到实体出错：{:}", e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

#[post("/tag/entity/remove")]
pub async fn remove_tags_from_entity(state: web::Data<AppState>, payload: web::Json<TagEntityRemoveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let req = payload.0;
    if req.entity_type.as_ref().map_or(true, |t| t.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "实体类型不能为空", "local")));
    }
    if req.entity_id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "实体ID不能为空", "local")));
    }
    if req.tag_ids.as_ref().map_or(true, |ids| ids.is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "标签ID列表不能为空", "local")));
    }
    match tag_service::TagService::remove_tags_from_entity(&db, req.entity_type.as_ref().unwrap(), req.entity_id.unwrap(), req.tag_ids.as_ref().unwrap()).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("移除实体标签出错：{:}", e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

#[get("/tag/entity/{entity_type}/{entity_id}")]
pub async fn get_tags_by_entity(state: web::Data<AppState>, path: web::Path<(String, i64)>) -> Result<HttpResponse> {
    let db = &state.db;
    let (entity_type, entity_id) = path.into_inner();
    match tag_service::TagService::get_tags_by_entity(&db, &entity_type, entity_id).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("获取实体标签出错：{:}", e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

#[post("/tag/entity/batch")]
pub async fn batch_tag_entity(state: web::Data<AppState>, payload: web::Json<TagEntityBatchRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let req = payload.0;
    if req.entity_type.as_ref().map_or(true, |t| t.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "实体类型不能为空", "local")));
    }
    if req.entity_ids.as_ref().map_or(true, |ids| ids.is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "实体ID列表不能为空", "local")));
    }
    if req.tag_ids.as_ref().map_or(true, |ids| ids.is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "标签ID列表不能为空", "local")));
    }
    let default_action = "add".to_string();
    let action = req.action.as_ref().unwrap_or(&default_action);
    let result = match action.as_str() {
        "add" => tag_service::TagService::batch_add_tags_to_entities(&db, req.entity_type.as_ref().unwrap(), req.entity_ids.as_ref().unwrap(), req.tag_ids.as_ref().unwrap()).await,
        "remove" => tag_service::TagService::batch_remove_tags_from_entities(&db, req.entity_type.as_ref().unwrap(), req.entity_ids.as_ref().unwrap(), req.tag_ids.as_ref().unwrap()).await,
        _ => return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "操作类型无效", "local")))
    };
    match result {
        Ok(v) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("批量操作标签出错：{:}", e);
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}
