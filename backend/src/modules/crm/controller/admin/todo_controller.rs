use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::response::{MetaResp, MPACK};
use actix_web::{web, HttpRequest, HttpResponse};

use crate::modules::crm::model::todo::*;
use crate::modules::crm::service::todo_service::TodoService;

/// GET /todo/summary - 待办汇总
pub async fn todo_summary(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    match TodoService::summary(db, jwt_token.id.unwrap_or_default()).await {
        Ok(data) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// GET /todo/approval - 审批待办列表
pub async fn todo_approval_list(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<ApprovalTodoQuery>,
) -> HttpResponse {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let query = query.0;
    match TodoService::approval_list(db, jwt_token.id.unwrap_or_default(), &query).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success_with_page(page_data, "local", page, total))
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// GET /todo/follow-up - 跟进待办列表
pub async fn todo_follow_up_list(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<FollowUpTodoQuery>,
) -> HttpResponse {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let query = query.0;
    match TodoService::follow_up_list(db, jwt_token.id.unwrap_or_default(), &query).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success_with_page(page_data, "local", page, total))
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// GET /todo/payment - 待回款提醒
pub async fn todo_payment_list(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<PaymentTodoQuery>,
) -> HttpResponse {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let query = query.0;
    match TodoService::payment_list(db, jwt_token.id.unwrap_or_default(), &query).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success_with_page(page_data, "local", page, total))
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// GET /todo/contract - 合同到期提醒
pub async fn todo_contract_list(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<ContractTodoQuery>,
) -> HttpResponse {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let query = query.0;
    match TodoService::contract_list(db, jwt_token.id.unwrap_or_default(), &query).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success_with_page(page_data, "local", page, total))
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// GET /todo/opportunity - 停滞商机
pub async fn todo_opportunity_list(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<OpportunityTodoQuery>,
) -> HttpResponse {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let query = query.0;
    match TodoService::opportunity_list(db, jwt_token.id.unwrap_or_default(), &query).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success_with_page(page_data, "local", page, total))
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/todo")
            .route("/summary", web::get().to(todo_summary))
            .route("/approval", web::get().to(todo_approval_list))
            .route("/follow-up", web::get().to(todo_follow_up_list))
            .route("/payment", web::get().to(todo_payment_list))
            .route("/contract", web::get().to(todo_contract_list))
            .route("/opportunity", web::get().to(todo_opportunity_list)),
    );
}
