//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::{Error, Result};
use actix_web::{web, HttpRequest, HttpResponse};
use crate::core::kit::global::AppState;
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp};
use crate::modules::website::model::template_data::{ListQuery, TemplateDataSaveDTO, TemplateDataSaveRequest, TemplateDataUpdateRequest};
use crate::modules::website::service::{template_data_service, template_user_data_service};

/// 新增
pub async fn add(state: web::Data<AppState>, req: HttpRequest, item: web::Json<TemplateDataSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.into_inner();

    let form_data = TemplateDataSaveDTO::from(payload);

    let website_id = req.headers().get("website_id")
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);


    let result = template_data_service::insert(&db, &form_data).await?;

    if result > 0 {
        template_user_data_service::save_website_template_merge(&db, &Some(website_id), &Some(result)).await?;
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::success("添加成功".to_string(), "local")))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "添加失败", "local")))
    }
}

/// 批量删除
pub async fn batch_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }

        let result = template_data_service::batch_delete_by_ids(&db, &ids_vec).await?;
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(Ok(result))))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

/// 修改
pub async fn update_by_id(state: web::Data<AppState>, _req: HttpRequest, id: web::Path<i64>, item: web::Json<TemplateDataUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.into_inner();

    let mut form_data = TemplateDataSaveDTO::from(payload);
    form_data.id = Some(id.into_inner());

    let result = template_data_service::update_by_id(&db, &form_data).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::success("修改成功".to_string(), "local")))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "修改失败", "local")))
    }
}

/// 详情
pub async fn get_by_detail(state: web::Data<AppState>, item: web::Path<InfoId>) ->Result<HttpResponse> {
    let db = &state.db;
    let result = template_data_service::get_by_detail(&db, &item.id).await?;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")))
}

/// 分页
pub async fn get_by_page(state: web::Data<AppState>, _req: HttpRequest, query: web::Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    template_data_service::get_by_page(&db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local"))
    })
}

// ==================== 路由注册（单点维护）====================

/// 注册模板数据模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(template_data_admin_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/template/data")
            // POST /template/data/add - 新增模板数据
            .route(
                "/add",
                web::post()
                    .to(add)
                    .wrap(require_permission("template:data:add")),
            )
            // DELETE /template/data/batch_delete - 批量删除模板数据
            .route(
                "/batch_delete",
                web::delete()
                    .to(batch_delete)
                    .wrap(require_permission("template:data:delete")),
            )
            // PUT /template/data/update/{id} - 修改模板数据
            .route(
                "/update/{id}",
                web::put()
                    .to(update_by_id)
                    .wrap(require_permission("template:data:update")),
            )
            // GET /template/data/detail/{id} - 模板数据详情
            .route(
                "/detail/{id}",
                web::get()
                    .to(get_by_detail)
                    .wrap(require_permission("template:data:view")),
            )
            // GET /template/data/list - 模板数据分页
            .route(
                "/list",
                web::get()
                    .to(get_by_page)
                    .wrap(require_permission("template:data:list")),
            ),
    );
}
