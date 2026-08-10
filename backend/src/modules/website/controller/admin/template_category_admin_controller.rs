//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//

use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use actix_web::{web, HttpRequest, HttpResponse};
use crate::core::web::entity::common::{BathDeleteIdRequest, BathIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::website::model::template_category::{CategorySaveDTO, CategorySaveRequest, CategoryUpdateRequest, ListQuery};
use crate::modules::website::service::{template_category_service};


pub async fn add(state: web::Data<AppState>, _req: HttpRequest, item: web::Json<CategorySaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.into_inner();
    if payload.name.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "名称不能为空", "local")));
    }
    if template_category_service::find_by_name_unique(&db, &payload.name, &None).await? {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "名称已存在", "local")));
    }
    let payload =  CategorySaveDTO::from(payload);
    let result = template_category_service::insert(&db, payload).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("添加成功".to_string(), "local")))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "添加失败", "local")))
    }
}

pub async fn batch_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }

        let result = template_category_service::batch_delete_by_ids(&db, &ids_vec).await?;
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(Ok(result))))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

pub async fn update_by_id(state: web::Data<AppState>, _req: HttpRequest, id: web::Path<i64>, item: web::Json<CategoryUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.into_inner();
    let cat_id = Some(id.into_inner());
    if payload.name.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "名称不能为空", "local")));
    }
    if template_category_service::find_by_name_unique(&db, &payload.name, &cat_id).await?{
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "名称已存在", "local")));
    }
    let mut payload = CategorySaveDTO::from(payload);
    payload.id = cat_id;
    let result = template_category_service::update_by_id(&db, &payload).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("修改成功".to_string(), "local")))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "修改失败", "local")))
    }
}

pub async fn get_by_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    let result = template_category_service::get_by_detail(&db, &item.id).await?;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(result, "local")))
}

pub async fn get_by_options(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    let menu_result = template_category_service::category_all_options(&db).await?;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(menu_result, "local")))
}

pub async fn select_by_parent(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    let menu_result = template_category_service::select_by_parent_id(&db, &Some(0)).await?;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(menu_result, "local")))
}


pub async fn get_by_list(state: web::Data<AppState>, _req: HttpRequest, query: web::Query<ListQuery>,) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = query.into_inner();
    template_category_service::category_all_tree_list(&db, &form_data).await.map(|tree_data| {
        HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(tree_data, "local"))
    })
}

// ==================== 路由注册（单点维护）====================

/// 注册模板分类模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(template_category_admin_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/template_category")
            // POST /template_category/add - 新建模板分类
            .route(
                "/add",
                web::post()
                    .to(add)
                    .wrap(require_permission("template:category:add")),
            )
            // DELETE /template_category/batch_delete - 批量删除模板分类
            .route(
                "/batch_delete",
                web::delete()
                    .to(batch_delete)
                    .wrap(require_permission("template:category:delete")),
            )
            // PUT /template_category/update/{id} - 修改模板分类
            .route(
                "/update/{id}",
                web::put()
                    .to(update_by_id)
                    .wrap(require_permission("template:category:update")),
            )
            // GET /template_category/detail/{id} - 模板分类详情
            .route(
                "/detail/{id}",
                web::get()
                    .to(get_by_detail)
                    .wrap(require_permission("template:category:view")),
            )
            // GET /template_category/options - 模板分类下拉
            .route("/options", web::get().to(get_by_options))
            // GET /template_category/parent - 父级模板分类
            .route("/parent", web::get().to(select_by_parent))
            // GET /template_category/list - 模板分类列表
            .route(
                "/list",
                web::get()
                    .to(get_by_list)
                    .wrap(require_permission("template:category:list")),
            ),
    );
}
