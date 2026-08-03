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
use crate::core::kit::global::AppState;
use crate::core::web::entity::common::{BathDeleteIdRequest, BathIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::articles::model::category::{CategoryModel, CategoryPageDTO, CategoryPageRequest, CategorySaveDTO, CategorySaveRequest, CategoryUpdateRequest};
use crate::modules::articles::service::category_service;
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;
use actix_web::{web, HttpRequest, HttpResponse};

pub async fn save_category(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<CategorySaveRequest>
) -> Result<HttpResponse> {
    let db = &state.db;
    let item = item.0;
    let mut category_data = CategorySaveDTO::from(item);
    let website_id = req.headers().get("website_id").and_then(|value| value.to_str().ok());
    category_data.website_id = website_id.map(|s| s.parse::<i64>().unwrap_or_default());
    match category_service::save_category(&db, category_data).await {
        Ok(_) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(200, "保存成功", "local")))
        }
        Err(_err) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "保存失败", "local")))
        }
    }
}


pub async fn batch_delete(state: web::Data<AppState>, req: HttpRequest, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let website_id = req.headers().get("website_id").and_then(|value| value.to_str().ok());
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }
        let ids = convert_vec_option_string_to_vec_u64(ids_vec);
        let result = CategoryModel::batch_delete_by_ids(&db,&website_id.map(|s| s.parse::<i64>().unwrap_or_default()), ids).await?;
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(Ok(result))))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }

}

pub async fn update_category(
    state: web::Data<AppState>,
    req: HttpRequest,
    id: web::Path<i64>,
    item: web::Json<CategoryUpdateRequest>
) -> Result<HttpResponse> {
    let db = &state.db;
    let item = item.0;
    let mut category_data = CategorySaveDTO::from(item);
    category_data.id = Some(id.into_inner());
    let website_id = req.headers().get("website_id").and_then(|value| value.to_str().ok());
    category_data.website_id = website_id.map(|s| s.parse::<i64>().unwrap_or_default());

    let category_data = category_service::update_by_id(&db, category_data).await?;
    if category_data == 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "更新失败", "local")));
    }
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("修改成功".to_string(), "local")))
}

/// 兼容前端 PUT /category/update（id 在 body 内，无路径参数）
pub async fn update_category_compat(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<CategoryUpdateRequest>
) -> Result<HttpResponse> {
    let db = &state.db;
    let item = item.0;
    let category_data = CategorySaveDTO::from(item);
    let website_id = req.headers().get("website_id").and_then(|value| value.to_str().ok());
    let mut dto = category_data;
    dto.website_id = website_id.map(|s| s.parse::<i64>().unwrap_or_default());

    let result = category_service::update_by_id(&db, dto).await?;
    if result == 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "更新失败", "local")));
    }
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("修改成功".to_string(), "local")))
}

/// 兼容前端 DELETE /category/delete?id=（query 参数传 id）
pub async fn delete_category_compat(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<std::collections::HashMap<String, String>>
) -> Result<HttpResponse> {
    let db = &state.db;
    let website_id = req.headers().get("website_id").and_then(|value| value.to_str().ok());
    let id_str = query.get("id").cloned().unwrap_or_default();
    let id: i64 = id_str.parse().unwrap_or(0);
    if id == 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
    }
    let ids = vec![id];
    let result = CategoryModel::batch_delete_by_ids(&db, &website_id.map(|s| s.parse::<i64>().unwrap_or_default()), ids).await?;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(Ok(result))))
}

pub async fn category_option(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let website_id = req.headers().get("website_id").and_then(|value| value.to_str().ok());

    match category_service::all_category_tree(&db, website_id.map(|s| s.parse::<i64>().unwrap_or_default()).unwrap_or_default()).await {
        Ok(router_list) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(router_list, "local")))
        }
        Err(_err) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "未获取到文章分类列表", "local")))
        }
    }
}

pub async fn get_category_detail(state: web::Data<AppState>, req: HttpRequest,item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    let id = item.id;
    //log::info!("----------------find_by_id:{:?}", id);
    let _website_id = req.headers().get("website_id").and_then(|value| value.to_str().ok());
    let _website_id = _website_id.map(|s| s.parse::<i64>().unwrap_or_default());

    let result = category_service::find_by_id(db,&id).await?;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(result, "local")))
}

pub async fn category_list_tree(state: web::Data<AppState>, req: HttpRequest, item: web::Query<CategoryPageRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.0;
    let mut category_dto = CategoryPageDTO::from(payload);

    let website_id = req.headers().get("website_id").and_then(|value| value.to_str().ok());

    category_dto.website_id = website_id.map(|s| s.parse::<i64>().unwrap_or_default());

    match category_service::select_all_list(&db,category_dto).await{
        Ok(router_list) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(router_list, "local")))
        }
        Err(_err) => {
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "未获取到文章分类列表", "local")))
        }
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册文章分类模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(category_admin_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/category")
            // POST /category/save - 新建分类
            .route(
                "/save",
                web::post()
                    .to(save_category)
                    .wrap(require_permission("website:category:add")),
            )
            // DELETE /category/batch_delete - 批量删除分类
            .route(
                "/batch_delete",
                web::delete()
                    .to(batch_delete)
                    .wrap(require_permission("website:category:delete")),
            )
            // PUT /category/update/{id} - 修改分类
            .route(
                "/update/{id}",
                web::put()
                    .to(update_category)
                    .wrap(require_permission("website:category:update")),
            )
            // GET /category/Option - 分类下拉
            .route("/Option", web::get().to(category_option))
            // GET /category/detail/{id} - 分类详情
            .route(
                "/detail/{id}",
                web::get()
                    .to(get_category_detail)
                    .wrap(require_permission("website:category:view")),
            )
            // GET /category/list - 分类列表
            .route(
                "/list",
                web::get()
                    .to(category_list_tree)
                    .wrap(require_permission("website:category:list")),
            )
            // GET /category/tree - 分类树（前端兼容别名，与 /list 相同）
            .route(
                "/tree",
                web::get()
                    .to(category_list_tree)
                    .wrap(require_permission("website:category:list")),
            )
            // PUT /category/update - 兼容前端无路径参数的更新（id 在 body 内）
            .route(
                "/update",
                web::put()
                    .to(update_category_compat)
                    .wrap(require_permission("website:category:update")),
            )
            // DELETE /category/delete - 兼容前端 query 参数删除?id=
            .route(
                "/delete",
                web::delete()
                    .to(delete_category_compat)
                    .wrap(require_permission("website:category:delete")),
            ),
    );
}
