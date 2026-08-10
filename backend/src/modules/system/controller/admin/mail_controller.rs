//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::system::model::mail::{
    MailConfigListQuery, MailConfigSaveRequest, MailConfigUpdateRequest, MailLogListQuery,
    MailTemplateListQuery, MailTemplateSaveRequest, MailTemplateUpdateRequest, SendMailRequest,
};
use crate::modules::system::service::{mail_config_service, mail_log_service, mail_service, mail_template_service};
use actix_web::{web, HttpRequest, HttpResponse};

// ============================ 邮箱账号配置 ============================

pub async fn mail_config_list(state: web::Data<AppState>, query: web::Query<MailConfigListQuery>) -> HttpResponse {
    let db = &state.db;
    match mail_config_service::list(&db, query.into_inner()).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success_with_page(page_data, "local", page, total))
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn mail_config_info(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.into_inner();
    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "邮箱配置ID不能为空", "local"));
    }
    match mail_config_service::find_by_id(&db, item.id.unwrap()).await {
        Ok(Some(vo)) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(vo, "local")),
        Ok(None) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "邮箱配置不存在或已删除", "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn mail_config_insert(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<MailConfigSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let result = mail_config_service::insert(&db, form_data.into_inner(), jwt_token.id).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn mail_config_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<MailConfigUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let result = mail_config_service::update(&db, form_data.into_inner(), jwt_token.id).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn mail_config_bath_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
    let db = &state.db;
    let delete_item = item.into_inner();
    if delete_item.ids.is_none() || delete_item.ids.as_ref().unwrap().is_empty() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "未获取到删除的邮箱配置ID", "local"));
    }
    let filtered_ids: Vec<i64> = delete_item
        .ids
        .unwrap_or_default()
        .iter()
        .filter_map(|item| item.as_ref().and_then(|s| s.trim().parse().ok()))
        .collect();
    let result = mail_config_service::batch_delete_by_ids(&db, filtered_ids).await;
    HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result))
}

pub async fn mail_config_set_default(state: web::Data<AppState>, req: HttpRequest, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.into_inner();
    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "邮箱配置ID不能为空", "local"));
    }
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let result = mail_config_service::set_default(&db, item.id.unwrap(), jwt_token.id).await;
    HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result))
}

// ============================ 邮件模板 ============================

pub async fn mail_template_list(state: web::Data<AppState>, query: web::Query<MailTemplateListQuery>) -> HttpResponse {
    let db = &state.db;
    match mail_template_service::list(&db, query.into_inner()).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success_with_page(page_data, "local", page, total))
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn mail_template_info(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.into_inner();
    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "邮件模板ID不能为空", "local"));
    }
    match mail_template_service::find_by_id(&db, item.id.unwrap()).await {
        Ok(Some(vo)) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(vo, "local")),
        Ok(None) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "邮件模板不存在或已删除", "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn mail_template_options(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match mail_template_service::options(&db).await {
        Ok(list) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(list, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn mail_template_insert(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<MailTemplateSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let result = mail_template_service::insert(&db, form_data.into_inner(), jwt_token.id).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn mail_template_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<MailTemplateUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let result = mail_template_service::update(&db, form_data.into_inner(), jwt_token.id).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn mail_template_bath_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> HttpResponse {
    let db = &state.db;
    let delete_item = item.into_inner();
    if delete_item.ids.is_none() || delete_item.ids.as_ref().unwrap().is_empty() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "未获取到删除的邮件模板ID", "local"));
    }
    let filtered_ids: Vec<i64> = delete_item
        .ids
        .unwrap_or_default()
        .iter()
        .filter_map(|item| item.as_ref().and_then(|s| s.trim().parse().ok()))
        .collect();
    let result = mail_template_service::batch_delete_by_ids(&db, filtered_ids).await;
    HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result))
}

// ============================ 发送邮件 ============================

pub async fn send_mail(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<SendMailRequest>) -> HttpResponse {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    match mail_service::send_mail(&db, form_data.into_inner(), jwt_token.id, jwt_token.username).await {
        Ok(id) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

// ============================ 邮件日志 ============================

pub async fn mail_log_list(state: web::Data<AppState>, query: web::Query<MailLogListQuery>) -> HttpResponse {
    let db = &state.db;
    match mail_log_service::list(&db, query.into_inner()).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success_with_page(page_data, "local", page, total))
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn mail_log_by_customer(state: web::Data<AppState>, query: web::Query<MailLogListQuery>) -> HttpResponse {
    let db = &state.db;
    let query = query.into_inner();
    let customer_id = match query.customer_id {
        Some(id) if id > 0 => id,
        _ => return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "客户ID不能为空", "local")),
    };
    match mail_log_service::list_by_customer(&db, customer_id).await {
        Ok(list) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(list, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

// ==================== 路由注册 ====================

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/mail")
            // ---- 邮箱账号（SMTP）配置 ----
            // 已统一迁移到「第三方接口配置」integration_config（code=smtp_email）
            // 旧的 /config/* CRUD 路由已移除，相关前端入口改为跳转 integration-config 页面
            // ---- 邮件模板 ----
            .route(
                "/template/list",
                web::get()
                    .to(mail_template_list)
                    .wrap(require_permission("system:mail:template")),
            )
            .route(
                "/template/info",
                web::get()
                    .to(mail_template_info)
                    .wrap(require_permission("system:mail:template")),
            )
            .route(
                "/template/options",
                web::get()
                    .to(mail_template_options)
                    .wrap(require_permission("system:mail:template")),
            )
            .route(
                "/template/save",
                web::post()
                    .to(mail_template_insert)
                    .wrap(require_permission("system:mail:template")),
            )
            .route(
                "/template/update",
                web::put()
                    .to(mail_template_update)
                    .wrap(require_permission("system:mail:template")),
            )
            .route(
                "/template/bath_delete",
                web::delete()
                    .to(mail_template_bath_delete)
                    .wrap(require_permission("system:mail:template")),
            )
            // ---- 发送邮件 ----
            .route(
                "/send",
                web::post()
                    .to(send_mail)
                    .wrap(require_permission("crm:mail:send")),
            )
            // ---- 邮件日志 ----
            .route(
                "/log/list",
                web::get()
                    .to(mail_log_list)
                    .wrap(require_permission("system:mail:log")),
            )
            .route(
                "/log/by_customer",
                web::get()
                    .to(mail_log_by_customer)
                    .wrap(require_permission("crm:mail:log")),
            ),
    );
}
