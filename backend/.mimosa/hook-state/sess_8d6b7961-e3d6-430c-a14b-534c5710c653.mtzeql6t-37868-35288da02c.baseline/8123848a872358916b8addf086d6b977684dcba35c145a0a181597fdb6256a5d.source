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
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::website::model::template::{ListQuery, TemplateSaveDTO, TemplateSaveRequest, TemplateUpdateRequest};
use crate::modules::website::service::{website_service, template_service, template_user_data_service};
use crate::validate;

pub async fn add(state: web::Data<AppState>, req: HttpRequest, item: web::Json<TemplateSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.into_inner();
    validate!(payload.name.is_none(), t!("website.template.name_empty", locale = "zh-CN").to_string());
    if template_service::find_by_name_unique(&db, &payload.name, &None).await? > 0 {
        validate!(true, t!("website.template.name_exists", locale = "zh-CN").to_string());
    }
    let mut form_data = TemplateSaveDTO::from(payload);

    let admin_token:JWTToken = get_user(&req).unwrap_or_default();
    form_data.user_id = admin_token.id;

    let result =  template_service::insert(&db, &form_data).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(result, "local")))
    }else{
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "文章发布失败", "local")))
    }
}

pub async fn batch_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }

        let result = template_service::batch_delete_by_ids(&db, &ids_vec).await?;
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(Ok(result))))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

pub async fn update_by_id(state: web::Data<AppState>, req: HttpRequest, item: web::Json<TemplateUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.into_inner();
    validate!(payload.id.is_none(), t!("website.template.id_empty", locale = "zh-CN").to_string());
    validate!(payload.name.is_none(), t!("website.template.name_empty", locale = "zh-CN").to_string());
    if template_service::find_by_name_unique(&db, &payload.name, &payload.id).await? > 0 {
        validate!(true, t!("website.template.name_exists", locale = "zh-CN").to_string());
    }
    let mut form_data = TemplateSaveDTO::from(payload);

    let admin_token:JWTToken = get_user(&req).unwrap_or_default();
    form_data.user_id = admin_token.id;

    let result = template_service::update_by_id(&db, &form_data).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(result, "local")))
    }else{
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "文章修改失败", "local")))
    }
}

pub async fn get_by_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    let result = template_service::get_by_detail(&db, &item.id).await?;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(result, "local")))
}

pub async fn get_by_options(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    let result = template_service::select_by_iscommon(db, &Some(1)).await?;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(result, "local")))
}

pub async fn get_by_page(state: web::Data<AppState>, query: web::Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    template_service::get_by_page(&db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(page_data, "local"))
    })
}

// ==================== 路由注册（单点维护）====================

/// 注册模板模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(template_admin_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/template")
            // POST /template/add - 新建模板
            .route(
                "/add",
                web::post()
                    .to(add)
                    .wrap(require_permission("template:add")),
            )
            // DELETE /template/batch_delete - 批量删除模板
            .route(
                "/batch_delete",
                web::delete()
                    .to(batch_delete)
                    .wrap(require_permission("template:delete")),
            )
            // POST /template/update - 修改模板
            .route(
                "/update",
                web::post()
                    .to(update_by_id)
                    .wrap(require_permission("template:update")),
            )
            // GET /template/detail/{id} - 模板详情
            .route(
                "/detail/{id}",
                web::get()
                    .to(get_by_detail)
                    .wrap(require_permission("template:view")),
            )
            // GET /template/common_options - 模板下拉
            .route("/common_options", web::get().to(get_by_options))
            // GET /template/list - 模板列表
            .route(
                "/list",
                web::get()
                    .to(get_by_page)
                    .wrap(require_permission("template:list")),
            )
            // GET /template/buy_list - 购买模板列表
            .route(
                "/buy_list",
                web::get()
                    .to(crate::modules::website::controller::admin::my_template_admin_controller::get_buy_by_page)
                    .wrap(require_permission("buy:template:list")),
            )
            // ========== 模板数据子路由（嵌套避免 scope 前缀冲突）==========
            .service(
                web::scope("/data")
                    // POST /template/data/add - 新增模板数据
                    .route(
                        "/add",
                        web::post()
                            .to(crate::modules::website::controller::admin::template_data_admin_controller::add)
                            .wrap(require_permission("template:data:add")),
                    )
                    // DELETE /template/data/batch_delete - 批量删除模板数据
                    .route(
                        "/batch_delete",
                        web::delete()
                            .to(crate::modules::website::controller::admin::template_data_admin_controller::batch_delete)
                            .wrap(require_permission("template:data:delete")),
                    )
                    // PUT /template/data/update/{id} - 修改模板数据
                    .route(
                        "/update/{id}",
                        web::put()
                            .to(crate::modules::website::controller::admin::template_data_admin_controller::update_by_id)
                            .wrap(require_permission("template:data:update")),
                    )
                    // GET /template/data/detail/{id} - 模板数据详情
                    .route(
                        "/detail/{id}",
                        web::get()
                            .to(crate::modules::website::controller::admin::template_data_admin_controller::get_by_detail)
                            .wrap(require_permission("template:data:view")),
                    )
                    // GET /template/data/list - 模板数据分页
                    .route(
                        "/list",
                        web::get()
                            .to(crate::modules::website::controller::admin::template_data_admin_controller::get_by_page)
                            .wrap(require_permission("template:data:list")),
                    )
                    // GET /template/data/list_by_template - 按模板ID查询（不分页）
                    .route(
                        "/list_by_template",
                        web::get()
                            .to(crate::modules::website::controller::admin::template_data_admin_controller::list_by_template)
                            .wrap(require_permission("template:data:list")),
                    )
                    // GET /template/data/export/{template_id} - 导出模板方案
                    .route(
                        "/export/{template_id}",
                        web::get()
                            .to(crate::modules::website::controller::admin::template_data_admin_controller::export_scheme)
                            .wrap(require_permission("template:data:list")),
                    )
                    // POST /template/data/import - 导入模板方案
                    .route(
                        "/import",
                        web::post()
                            .to(crate::modules::website::controller::admin::template_data_admin_controller::import_scheme)
                            .wrap(require_permission("template:data:add")),
                    )
                    // POST /template/data/preview - 模板预览
                    .route(
                        "/preview",
                        web::post()
                            .to(crate::modules::website::controller::admin::template_data_admin_controller::preview)
                            .wrap(require_permission("template:data:list")),
                    ),
            ),
    );
}
