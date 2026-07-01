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
use crate::core::web::entity::common::BathDeleteIdRequest;
use crate::core::web::response::MetaResp;
use crate::modules::company::model::company::{CompanyAccountSaveRequest, CompanyInfoSaveRequest};
use crate::modules::company::service::company_service;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;

/// 判断当前用户是否有企业信息编辑权限（有编辑权限则不脱敏法人电话）
fn can_edit_company(jwt_token: &JWTToken) -> bool {
    jwt_token.permissions.iter().any(|p| p == "company:info:edit")
}

#[get("/company/info")]
#[protect("company:info:list")]
pub async fn get_company_info(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let mask_sensitive = !can_edit_company(&jwt_token);

    match company_service::get_info(&db, mask_sensitive).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[put("/company/update")]
#[protect("company:info:edit")]
pub async fn update_company_info(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<CompanyInfoSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let form_data = form_data.0;

    let result = company_service::update_info(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[get("/company/account/list")]
#[protect("company:info:list")]
pub async fn get_account_list(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;

    match company_service::get_accounts(&db).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[post("/company/account/save")]
#[protect("company:account:save")]
pub async fn save_account(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<CompanyAccountSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let form_data = form_data.0;

    let result = company_service::save_account(&db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[delete("/company/account/delete")]
#[protect("company:account:delete")]
pub async fn delete_account(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let ids: Vec<i64> = item.0.ids.unwrap_or_default()
        .into_iter()
        .flatten()
        .filter_map(|s| s.parse().ok())
        .collect();
    if ids.is_empty() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "请选择要删除的记录", "local")));
    }

    let mut affected = 0i64;
    for id in &ids {
        match company_service::delete_account(&db, *id).await {
            Ok(rows) => affected += rows,
            Err(e) => {
                return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local")));
            }
        }
    }
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(affected, "local")))
}
