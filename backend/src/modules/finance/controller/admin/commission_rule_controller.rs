//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{get, post, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;

use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::entity::common::InfoId;
use crate::core::web::response::MetaResp;
use crate::modules::finance::model::commission::{CommissionRuleQuery, CommissionRuleSaveDTO};
use crate::modules::finance::service::commission_rule_service;

#[get("/finance/commission-rule/list")]
#[protect("finance:commission:list")]
pub async fn list(
    state: web::Data<AppState>,
    query: web::Query<CommissionRuleQuery>,
) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    let page = query.page.unwrap_or(1) as u32;

    match commission_rule_service::get_list(db, query).await {
        Ok((list, total)) => {
            HttpResponse::Ok().content_type("application/msgpack")
                .body(MetaResp::success_with_page(list, "local", page, total as u32))
        }
        Err(e) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

#[get("/finance/commission-rule/detail")]
#[protect("finance:commission:list")]
pub async fn detail(
    state: web::Data<AppState>,
    query: web::Query<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let item = query.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, "规则ID不能为空", "local"));
    }

    match commission_rule_service::get_detail(db, item.id.unwrap()).await {
        Ok(data) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

#[post("/finance/commission-rule/save")]
#[protect("finance:commission:manage")]
pub async fn save(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<CommissionRuleSaveDTO>,
) -> HttpResponse {
    let db = &state.db;
    let mut dto = form_data.0;

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let user_id = jwt_token.id.unwrap_or_default();

    if dto.id.is_some() {
        dto.updated_by = Some(user_id);
    } else {
        dto.created_by = Some(user_id);
    }

    match commission_rule_service::save(db, dto).await {
        Ok(id) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

#[post("/finance/commission-rule/delete")]
#[protect("finance:commission:manage")]
pub async fn delete(
    state: web::Data<AppState>,
    query: web::Query<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let item = query.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, "规则ID不能为空", "local"));
    }

    match commission_rule_service::delete(db, item.id.unwrap()).await {
        Ok(_) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::success("删除成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

#[post("/finance/commission-rule/toggle")]
#[protect("finance:commission:manage")]
pub async fn toggle(
    state: web::Data<AppState>,
    query: web::Query<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let item = query.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, "规则ID不能为空", "local"));
    }

    match commission_rule_service::toggle(db, item.id.unwrap()).await {
        Ok(_) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::success("操作成功".to_string(), "local")),
        Err(e) => HttpResponse::Ok().content_type("application/msgpack")
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}
