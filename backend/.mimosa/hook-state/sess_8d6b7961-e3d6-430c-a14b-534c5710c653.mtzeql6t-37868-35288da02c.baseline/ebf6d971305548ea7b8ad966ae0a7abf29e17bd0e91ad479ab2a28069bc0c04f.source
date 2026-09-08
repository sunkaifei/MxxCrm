use crate::core::errors::error::Result;
use actix_web::{HttpResponse, web, HttpRequest};
use sea_orm::DbConn;
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::system::model::tag::{TagSaveDTO, TagSaveRequest, TagUpdateRequest, TagListQuery, TagMoveToGroupRequest, UpdateTagStatusRequest};
use crate::modules::system::model::tag_group::{TagGroupSaveDTO, TagGroupSaveRequest, TagGroupUpdateRequest};
use crate::modules::system::model::tag_merge::{TagEntityRequest, TagEntityRemoveRequest, TagEntityBatchRequest};
use crate::modules::system::service::{admin_service, tag_service, tag_group_service};

/// 获取当前用户 id 与是否超管（user_type=1）
async fn current_user_context(db: &DbConn, req: &HttpRequest) -> Result<(i64, bool)> {
    let admin = admin_service::get_by_detail(db, &Some(get_current_user_id(req))).await?;
    Ok((admin.id.unwrap_or(0), admin.user_type == Some(1)))
}

/// 权限拒绝响应（MPACK 403）
fn forbidden(msg: &str) -> HttpResponse {
    HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(403, msg, "local"))
}

pub async fn save_tag(state: web::Data<AppState>, req: HttpRequest, payload: web::Json<TagSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let tag_request = payload.0;
    if tag_request.tag_name.as_ref().map_or(true, |tag_name| tag_name.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "标签名称不能为空", "local")));
    }
    let (user_id, is_admin) = current_user_context(&db, &req).await?;
    // 系统标签仅超管可创建
    if tag_request.is_global == Some(true) && !is_admin {
        return Ok(forbidden("系统标签仅管理员可创建"));
    }
    let mut form_data = TagSaveDTO::from(tag_request);
    if !is_admin {
        form_data.is_global = Some(false);
    }
    form_data.created_by = Some(user_id);
    form_data.updated_by = Some(user_id);
    match tag_service::TagService::save(&db, form_data, Some(user_id)).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("添加标签出错：{:}", e);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

pub async fn update_tag(state: web::Data<AppState>, req: HttpRequest, payload: web::Json<TagUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let tag_request = payload.0;
    if tag_request.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "标签ID不能为空", "local")));
    }
    if tag_request.tag_name.as_ref().map_or(true, |tag_name| tag_name.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "标签名称不能为空", "local")));
    }
    let (user_id, is_admin) = current_user_context(&db, &req).await?;
    // 目标标签权限校验：系统标签仅超管，个人标签仅 owner
    let tag_id = tag_request.id.unwrap();
    if let Err(e) = tag_service::TagService::validate_tags_manageable(&db, &vec![tag_id], user_id, is_admin).await {
        return Ok(forbidden(&e.to_string()));
    }
    // 普通用户不允许把标签改成系统标签
    if tag_request.is_global == Some(true) && !is_admin {
        return Ok(forbidden("系统标签仅管理员可创建"));
    }
    let mut form_data = TagSaveDTO::from(tag_request);
    if !is_admin {
        form_data.is_global = Some(false);
    }
    form_data.updated_by = Some(user_id);
    match tag_service::TagService::save(&db, form_data, Some(user_id)).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("更新标签出错：{:}", e);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

pub async fn delete_tag(state: web::Data<AppState>, req: HttpRequest, item: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let id = item.into_inner();
    let (user_id, is_admin) = current_user_context(&db, &req).await?;
    if let Err(e) = tag_service::TagService::validate_tags_manageable(&db, &vec![id], user_id, is_admin).await {
        return Ok(forbidden(&e.to_string()));
    }
    match tag_service::TagService::delete(&db, id).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("删除标签出错：{:}", e);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

pub async fn batch_delete_tag(state: web::Data<AppState>, req: HttpRequest, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
        } else {
            let ids: Vec<i64> = ids_vec.into_iter().filter_map(|id| id.and_then(|s| s.parse().ok())).collect();
            let (user_id, is_admin) = current_user_context(&db, &req).await?;
            if let Err(e) = tag_service::TagService::validate_tags_manageable(&db, &ids, user_id, is_admin).await {
                return Ok(forbidden(&e.to_string()));
            }
            match tag_service::TagService::batch_delete(&db, &ids).await {
                Ok(v) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local"))),
                Err(e) => {
                    log::error!("批量删除标签出错：{:}", e);
                    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
                }
            }
        }
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

/// 修改标签状态（启用/禁用）
pub async fn update_tag_status(state: web::Data<AppState>, req: HttpRequest, payload: web::Json<UpdateTagStatusRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let req_data = payload.0;
    let id = match req_data.id {
        Some(id) if id > 0 => id,
        _ => return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "标签ID不能为空", "local"))),
    };
    let status = match req_data.status {
        Some(s) => s,
        None => return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "状态不能为空", "local"))),
    };
    let (user_id, is_admin) = current_user_context(&db, &req).await?;
    if let Err(e) = tag_service::TagService::validate_tags_manageable(&db, &vec![id], user_id, is_admin).await {
        return Ok(forbidden(&e.to_string()));
    }
    match tag_service::TagService::update_status(&db, id, status).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("修改标签状态出错：{:}", e);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

pub async fn get_tag_detail(state: web::Data<AppState>, req: HttpRequest, item: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let id = item.into_inner();
    let (user_id, is_admin) = current_user_context(&db, &req).await?;
    match tag_service::TagService::get_by_id(&db, id).await {
        Ok(vo) => {
            // 非超管仅可查看系统标签或自己创建的标签
            if !is_admin {
                if vo.is_global != Some(true) && vo.created_by != Some(user_id) {
                    return Ok(forbidden("无权查看他人创建的标签"));
                }
            }
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(vo, "local")))
        }
        Err(e) => {
            log::error!("获取标签详情出错：{:}", e);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

pub async fn get_tag_list(state: web::Data<AppState>, req: HttpRequest, query: web::Query<TagListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let page = query.page_num.unwrap_or(1);
    let per_page = query.page_size.unwrap_or(20);
    let (user_id, is_admin) = current_user_context(&db, &req).await?;
    match tag_service::TagService::get_list(&db, page, per_page, query.tag_name.clone(), query.group_id, query.is_global, Some(user_id), is_admin).await {
        Ok(page_data) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(page_data, "local")))
        }
        Err(e) => {
            log::error!("获取标签列表出错：{:}", e);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

pub async fn get_tag_statistics(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    match tag_service::TagService::get_statistics(&db).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("获取标签统计出错：{:}", e);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

pub async fn get_all_tags(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let (user_id, is_admin) = current_user_context(&db, &req).await?;
    match tag_service::TagService::get_all_tags(&db, Some(user_id), is_admin).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("获取所有标签出错：{:}", e);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

pub async fn get_tags_by_group(state: web::Data<AppState>, req: HttpRequest, path: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let group_id = path.into_inner();
    let (user_id, is_admin) = current_user_context(&db, &req).await?;
    match tag_service::TagService::get_tags_by_group(&db, group_id, Some(user_id), is_admin).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("获取分组下标签出错：{:}", e);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

pub async fn move_tags_to_group(state: web::Data<AppState>, req: HttpRequest, payload: web::Json<TagMoveToGroupRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let req_data = payload.0;
    if req_data.group_id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "分组ID不能为空", "local")));
    }
    if req_data.tag_ids.as_ref().map_or(true, |ids| ids.is_empty()) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "标签ID列表不能为空", "local")));
    }
    let (user_id, is_admin) = current_user_context(&db, &req).await?;
    let tag_ids = req_data.tag_ids.clone().unwrap();
    if let Err(e) = tag_service::TagService::validate_tags_manageable(&db, &tag_ids, user_id, is_admin).await {
        return Ok(forbidden(&e.to_string()));
    }
    match tag_service::TagService::move_to_group(&db, req_data.group_id.unwrap(), &tag_ids).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("移动标签到分组出错：{:}", e);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

pub async fn tag_suggest(state: web::Data<AppState>, req: HttpRequest, query: web::Query<(String,)>) -> Result<HttpResponse> {
    let db = &state.db;
    let keyword = &query.0.0;
    let (user_id, is_admin) = current_user_context(&db, &req).await?;
    match tag_service::TagService::suggest(&db, keyword, Some(user_id), is_admin).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("标签建议出错：{:}", e);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

pub async fn save_tag_group(state: web::Data<AppState>, req: HttpRequest, payload: web::Json<TagGroupSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let group_request = payload.0;
    let admin = admin_service::get_by_detail(&db, &Some(get_current_user_id(&req))).await?;
    let mut form_data = TagGroupSaveDTO::from(group_request);
    form_data.created_by = admin.id;
    form_data.updated_by = admin.id;
    match tag_group_service::TagGroupService::save(&db, form_data, admin.id).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("保存标签分组出错：{:}", e);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

pub async fn update_tag_group(state: web::Data<AppState>, req: HttpRequest, payload: web::Json<TagGroupUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let group_request = payload.0;
    if group_request.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "分组ID不能为空", "local")));
    }
    let admin = admin_service::get_by_detail(&db, &Some(get_current_user_id(&req))).await?;
    let mut form_data = TagGroupSaveDTO::from(group_request);
    form_data.updated_by = admin.id;
    match tag_group_service::TagGroupService::save(&db, form_data, admin.id).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("更新标签分组出错：{:}", e);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

pub async fn delete_tag_group(state: web::Data<AppState>, item: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let id = item.into_inner();
    match tag_group_service::TagGroupService::delete(&db, id).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("删除标签分组出错：{:}", e);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

pub async fn batch_delete_tag_group(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
        } else {
            let ids: Vec<i64> = ids_vec.into_iter().filter_map(|id| id.and_then(|s| s.parse().ok())).collect();
            match tag_group_service::TagGroupService::batch_delete(&db, &ids).await {
                Ok(v) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local"))),
                Err(e) => {
                    log::error!("批量删除标签分组出错：{:}", e);
                    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
                }
            }
        }
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

pub async fn get_tag_group_list(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    match tag_group_service::TagGroupService::get_list(&db).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("获取标签分组列表出错：{:}", e);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

pub async fn get_tag_group_detail(state: web::Data<AppState>, item: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let id = item.into_inner();
    match tag_group_service::TagGroupService::get_by_id(&db, id).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("获取标签分组详情出错：{:}", e);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

pub async fn add_tags_to_entity(state: web::Data<AppState>, req: HttpRequest, payload: web::Json<TagEntityRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let req_data = payload.0;
    if req_data.entity_type.as_ref().map_or(true, |t| t.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "实体类型不能为空", "local")));
    }
    if req_data.entity_id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "实体ID不能为空", "local")));
    }
    if req_data.tag_ids.as_ref().map_or(true, |ids| ids.is_empty()) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "标签ID列表不能为空", "local")));
    }
    let (user_id, is_admin) = current_user_context(&db, &req).await?;
    let tag_ids = req_data.tag_ids.clone().unwrap();
    if let Err(e) = tag_service::TagService::validate_tags_visible(&db, &tag_ids, user_id, is_admin).await {
        return Ok(forbidden(&e.to_string()));
    }
    match tag_service::TagService::add_tags_to_entity(&db, req_data.entity_type.as_ref().unwrap(), req_data.entity_id.unwrap(), &tag_ids).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("添加标签到实体出错：{:}", e);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

pub async fn remove_tags_from_entity(state: web::Data<AppState>, req: HttpRequest, payload: web::Json<TagEntityRemoveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let req_data = payload.0;
    if req_data.entity_type.as_ref().map_or(true, |t| t.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "实体类型不能为空", "local")));
    }
    if req_data.entity_id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "实体ID不能为空", "local")));
    }
    if req_data.tag_ids.as_ref().map_or(true, |ids| ids.is_empty()) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "标签ID列表不能为空", "local")));
    }
    let (user_id, is_admin) = current_user_context(&db, &req).await?;
    let tag_ids = req_data.tag_ids.clone().unwrap();
    if let Err(e) = tag_service::TagService::validate_tags_visible(&db, &tag_ids, user_id, is_admin).await {
        return Ok(forbidden(&e.to_string()));
    }
    match tag_service::TagService::remove_tags_from_entity(&db, req_data.entity_type.as_ref().unwrap(), req_data.entity_id.unwrap(), &tag_ids).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("移除实体标签出错：{:}", e);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

pub async fn get_tags_by_entity(state: web::Data<AppState>, path: web::Path<(String, i64)>) -> Result<HttpResponse> {
    let db = &state.db;
    let (entity_type, entity_id) = path.into_inner();
    match tag_service::TagService::get_tags_by_entity(&db, &entity_type, entity_id).await {
        Ok(v) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("获取实体标签出错：{:}", e);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

pub async fn batch_tag_entity(state: web::Data<AppState>, req: HttpRequest, payload: web::Json<TagEntityBatchRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let req_data = payload.0;
    if req_data.entity_type.as_ref().map_or(true, |t| t.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "实体类型不能为空", "local")));
    }
    if req_data.entity_ids.as_ref().map_or(true, |ids| ids.is_empty()) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "实体ID列表不能为空", "local")));
    }
    if req_data.tag_ids.as_ref().map_or(true, |ids| ids.is_empty()) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "标签ID列表不能为空", "local")));
    }
    let (user_id, is_admin) = current_user_context(&db, &req).await?;
    let tag_ids = req_data.tag_ids.clone().unwrap();
    if let Err(e) = tag_service::TagService::validate_tags_visible(&db, &tag_ids, user_id, is_admin).await {
        return Ok(forbidden(&e.to_string()));
    }
    let default_action = "add".to_string();
    let action = req_data.action.as_ref().unwrap_or(&default_action);
    let result = match action.as_str() {
        "add" => tag_service::TagService::batch_add_tags_to_entities(&db, req_data.entity_type.as_ref().unwrap(), req_data.entity_ids.as_ref().unwrap(), &tag_ids).await,
        "remove" => tag_service::TagService::batch_remove_tags_from_entities(&db, req_data.entity_type.as_ref().unwrap(), req_data.entity_ids.as_ref().unwrap(), &tag_ids).await,
        _ => return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "操作类型无效", "local")))
    };
    match result {
        Ok(v) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local"))),
        Err(e) => {
            log::error!("批量操作标签出错：{:}", e);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册标签模块所有路由
///
/// 修改路径、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(tag_admin_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tag")
            // POST /tag/add - 添加标签
            .route("/add", web::post().to(save_tag))
            // PUT /tag/update - 更新标签
            .route("/update", web::put().to(update_tag))
            // DELETE /tag/delete/{id} - 删除标签
            .route("/delete/{id}", web::delete().to(delete_tag))
            // DELETE /tag/batch_delete - 批量删除标签
            .route("/batch_delete", web::delete().to(batch_delete_tag))
            // PUT /tag/status - 修改标签状态
            .route("/status", web::put().to(update_tag_status))
            // GET /tag/detail/{id} - 获取标签详情
            .route("/detail/{id}", web::get().to(get_tag_detail))
            // GET /tag/list - 获取标签列表
            .route("/list", web::get().to(get_tag_list))
            // GET /tag/statistics - 获取标签统计
            .route("/statistics", web::get().to(get_tag_statistics))
            // GET /tag/all - 获取所有标签
            .route("/all", web::get().to(get_all_tags))
            // POST /tag/move-to-group - 移动标签到分组
            .route("/move-to-group", web::post().to(move_tags_to_group))
            // GET /tag/suggest - 标签建议
            .route("/suggest", web::get().to(tag_suggest))
            // POST+PUT /tag/group - 保存/更新标签分组（同一路径两种方法，使用 resource 合并）
            .service(
                web::resource("/group")
                    .route(web::post().to(save_tag_group))
                    .route(web::put().to(update_tag_group)),
            )
            // GET /tag/group/list - 获取标签分组列表（精确路由，必须放在参数化路由 /group/{id} 之前）
            .route("/group/list", web::get().to(get_tag_group_list))
            // DELETE /tag/group/batch_delete - 批量删除标签分组（精确路由，必须放在参数化路由 /group/{id} 之前）
            .route("/group/batch_delete", web::delete().to(batch_delete_tag_group))
            // GET /tag/group/detail/{id} - 获取标签分组详情（参数化路由，放在精确路由之后）
            .route("/group/detail/{id}", web::get().to(get_tag_group_detail))
            // GET /tag/group/{group_id} - 获取分组下标签（参数化路由，放在精确路由之后，避免误匹配 list/batch_delete）
            .route("/group/{group_id}", web::get().to(get_tags_by_group))
            // DELETE /tag/group/{id} - 删除标签分组（参数化路由，放在精确路由之后）
            .route("/group/{id}", web::delete().to(delete_tag_group))
            // POST /tag/entity/add - 添加标签到实体
            .route("/entity/add", web::post().to(add_tags_to_entity))
            // POST /tag/entity/remove - 移除实体标签
            .route("/entity/remove", web::post().to(remove_tags_from_entity))
            // POST /tag/entity/batch - 批量操作标签
            .route("/entity/batch", web::post().to(batch_tag_entity))
            // GET /tag/entity/{entity_type}/{entity_id} - 获取实体标签（参数化路由，放在最后）
            .route("/entity/{entity_type}/{entity_id}", web::get().to(get_tags_by_entity)),
    );
}
