//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.!
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::Result;
use actix_web::{HttpResponse, web, HttpRequest};
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::system::model::dict_data::{DataListQuery, DictDataSaveDTO, DictDataSaveRequest, DictDataUpdateRequest};
use crate::modules::system::model::dict::{DictSaveDTO, DictSaveRequest, DictUpdateRequest, TypeListQuery};
use crate::modules::system::service::{admin_service, dict_service};

pub async fn save_dict(state: web::Data<AppState>, req: HttpRequest, payload: web::Json<DictSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let dict_request = payload.0;
    if dict_request.dict_name.as_ref().map_or(true, |dict_name| dict_name.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "字典名称不能为空", "local")));
    }
    if dict_service::find_by_name_unique(&db, &dict_request.dict_name,&None).await? {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "字典名称已存在", "local")));
    }
    if dict_request.dict_code.as_ref().map_or(true, |dict_code| dict_code.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "字典编码不能为空", "local")));
    }
    if dict_service::find_by_code_unique(&db, &dict_request.dict_code,&None).await? {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "字典编码已存在", "local")));
    }
    //获取用户信息
    let admin = admin_service::get_by_detail(&db, &Some(get_current_user_id(&req))).await?;
    let mut form_data = DictSaveDTO::from(dict_request);
    form_data.create_by = admin.user_name.clone();
    form_data.update_by = admin.user_name;
    match dict_service::insert(&db, &form_data).await {
        Ok(v) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local")))
        }
        Err(e) => {
            log::error!("添加字典出错：{:}",e);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

pub async fn save_dict_data(state: web::Data<AppState>, req: HttpRequest, payload: web::Json<DictDataSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let data_request = payload.0;
    if data_request.dict_label.as_ref().map_or(true, |dict_label| dict_label.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "字典标签名称不能为空", "local")));
    }
    if dict_service::find_data_by_label_unique(&db, &data_request.dict_code, &data_request.dict_label, &None).await? {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "字典标签名称已存在", "local")));
    }
    if data_request.dict_value.as_ref().map_or(true, |dict_value| dict_value.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "字典值不能为空", "local")));
    }
    //获取用户信息
    let admin = admin_service::get_by_detail(&db, &Some(get_current_user_id(&req))).await?;
    let mut form_data = DictDataSaveDTO::from(data_request);
    form_data.create_by = admin.user_name;
    match dict_service::insert_data(&db, &form_data).await {
        Ok(v) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local")))
        }
        Err(e) => {
            log::error!("添加字典出错：{:}",e);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

pub async fn batch_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
        } else {
            let result = dict_service::batch_delete_by_ids(&db, ids_vec).await;
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
        }
    }else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

pub async fn batch_delete_data(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
        } else {
            let result = dict_service::batch_delete_data_by_ids(&db, ids_vec).await;
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
        }
    }else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

pub async fn update_dict(state: web::Data<AppState>, req: HttpRequest, id: web::Path<i64>, item: web::Json<DictUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.dict_name.as_ref().map_or(true, |dict_name| dict_name.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "字典名称不能为空", "local")));
    }
    let dict_id = Some(id.into_inner());
    if dict_service::find_by_name_unique(&db, &item.dict_name, &dict_id).await? {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "字典名称已存在", "local")));
    }
    if item.dict_code.as_ref().map_or(true, |dict_code| dict_code.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "字典编码不能为空", "local")));
    }
    if dict_service::find_by_code_unique(&db, &item.dict_code, &dict_id).await? {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "字典编码已存在", "local")));
    }

    //获取用户信息
    let admin = admin_service::get_by_detail(&db, &Some(get_current_user_id(&req))).await?;

    let mut form_data = DictSaveDTO::from(item.0);
    form_data.id = dict_id;
    form_data.update_by = admin.user_name;
    match dict_service::update_by_id(&db, &form_data).await {
        Ok(v) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local")))
        }
        Err(e) => {
            log::error!("更新字典出错：{:}",e);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

/// 更新字典数据
pub async fn update_dict_data(state: web::Data<AppState>, req: HttpRequest, id: web::Path<i64>, item: web::Json<DictDataUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let dict_data_id = Some(id.into_inner());
    if item.dict_label.as_ref().map_or(true, |dict_label| dict_label.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "字典标签名称不能为空", "local")));
    }
    if dict_service::find_data_by_label_unique(&db, &item.dict_code, &item.dict_label, &dict_data_id).await? {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "字典标签名称已存在", "local")));
    }
    if item.dict_value.as_ref().map_or(true, |dict_value| dict_value.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "字典值不能为空", "local")));
    }

    //获取用户信息
    let admin = admin_service::get_by_detail(&db, &Some(get_current_user_id(&req))).await?;

    let mut form_data = DictDataSaveDTO::from(item.0);
    form_data.id = dict_data_id;
    form_data.update_by = admin.user_name;
    match dict_service::update_data_by_id(&db, &form_data).await{
        Ok(v) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local")))
        }
        Err(e) => {
            log::error!("更新字典出错：{:}",e);
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(500, &e.to_string(), "local")))
        }
    }
}

/// 获取字典类型详情
pub async fn get_dict_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    match dict_service::get_by_id(&db, &item.id).await {
        Ok(dict_type) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(dict_type, "local")))
        }
        Err(err) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &err.to_string(), "local")))
        }
    }
}

/// 获取字典数据详情
pub async fn get_dict_data_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    match dict_service::get_data_by_id(&db, &item.id).await {
        Ok(dict_data) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(dict_data, "local")))
        }
        Err(err) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &err.to_string(), "local")))
        }
    }
}

pub async fn get_dict_data_list_by_code(state: web::Data<AppState>, dict_code: web::Path<String>) -> Result<HttpResponse> {
    let db = &state.db;
    let dict_code = dict_code.into_inner();
    if dict_code.is_empty() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "字典编码不能为空", "local")));
    }
    dict_service::get_dict_data_list_by_code(&db, &Some(dict_code)).await.map(|dict_data| {
        HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(dict_data, "local"))
    })
}

pub async fn get_dict_page(state: web::Data<AppState>, query: web::Query<TypeListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    dict_service::get_dict_page(&db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(page_data, "local"))
    })
}

pub async fn get_dict_data_list(state: web::Data<AppState>, query: web::Query<DataListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    dict_service::get_dict_data_page(&db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(page_data, "local"))
    })
}

// ==================== 路由注册（方案 C：单点维护）====================

/// 注册字典管理模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(system_dict_controller::register)` 注册。
///
/// 注意：静态路径（如 /list、/add）注册在动态路径（如 /{id}）之前，
/// actix-web 路由匹配时静态路径优先于动态路径。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/dict")
            // ---- 字典类型相关路由 ----
            // POST /dict/add - 新增字典
            // 注意：Route::to() 会覆盖之前 wrap() 设置的中间件，所以必须先 to() 再 wrap()
            .route(
                "/add",
                web::post()
                    .to(save_dict)
                    .wrap(require_permission("system:dict:save")),
            )
            // GET /dict/list - 字典类型列表
            .route(
                "/list",
                web::get()
                    .to(get_dict_page)
                    .wrap(require_permission("system:dict:list")),
            )
            // DELETE /dict/batch_delete - 批量删除字典
            .route(
                "/batch_delete",
                web::delete()
                    .to(batch_delete)
                    .wrap(require_permission("system:dict:delete")),
            )
            // PUT /dict/update/{id} - 更新字典
            .route(
                "/update/{id}",
                web::put()
                    .to(update_dict)
                    .wrap(require_permission("system:dict:update")),
            )
            // GET /dict/{id} - 字典类型详情（动态路径，放后面）
            .route(
                "/{id}",
                web::get()
                    .to(get_dict_detail)
                    .wrap(require_permission("system:dict:view")),
            )
            // ---- 字典数据相关路由 ----
            // POST /dict/data/save - 新增字典数据
            .route("/data/save", web::post().to(save_dict_data))
            // GET /dict/data/list - 字典数据列表
            .route(
                "/data/list",
                web::get()
                    .to(get_dict_data_list)
                    .wrap(require_permission("dict:data:list")),
            )
            // DELETE /dict/data/batch_delete - 批量删除字典数据
            .route("/data/batch_delete", web::delete().to(batch_delete_data))
            // PUT /dict/data/update/{id} - 更新字典数据
            .route("/data/update/{id}", web::put().to(update_dict_data))
            // GET /dict/data/{dict_code}/options - 按编码获取字典数据选项
            .route(
                "/data/{dict_code}/options",
                web::get().to(get_dict_data_list_by_code),
            )
            // GET /dict/data/{id} - 字典数据详情（动态路径，放后面）
            .route(
                "/data/{id}",
                web::get()
                    .to(get_dict_data_detail)
                    .wrap(require_permission("dict:data:detail:view")),
            ),
    );
}
