use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::response::MetaResp;
use crate::modules::statistics::model::performance_target::{PerformanceTargetQuery, PerformanceTargetBatchSaveRequest, PerformanceRankingQuery};
use crate::modules::statistics::model::customer_stats::CustomerStatsQuery;
use crate::modules::statistics::model::employee_stats::EmployeeStatsQuery;
use crate::modules::statistics::model::contract_stats::ContractStatsQuery;
use crate::modules::statistics::model::payment_stats::PaymentStatsQuery;
use crate::modules::statistics::service::{performance_target_service, customer_stats_service, employee_stats_service, contract_stats_service, payment_stats_service};
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;

#[get("/statistics/performance/target")]
#[protect("statistics:performance:view")]
pub async fn get_performance_target(state: web::Data<AppState>, query: web::Query<PerformanceTargetQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match performance_target_service::get_targets(db, query.employee_id, query.year, query.month).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[post("/statistics/performance/target/save")]
#[protect("statistics:performance:manage")]
pub async fn save_performance_target(state: web::Data<AppState>, form_data: web::Json<PerformanceTargetBatchSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    
    if form_data.targets.is_none() || form_data.targets.as_ref().unwrap().is_empty() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "目标数据不能为空", "local")));
    }
    
    match performance_target_service::save_targets(db, form_data.targets.as_ref().unwrap()).await {
        Ok((saved_count, updated_count)) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success({
            serde_json::json!({
                "saved_count": saved_count,
                "updated_count": updated_count
            })
        }, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[get("/statistics/performance/monthly")]
#[protect("statistics:performance:view")]
pub async fn get_monthly_performance(state: web::Data<AppState>, query: web::Query<PerformanceRankingQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match performance_target_service::get_monthly_performance(db, query.year, query.department_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[get("/statistics/performance/ranking")]
#[protect("statistics:performance:view")]
pub async fn get_performance_ranking(state: web::Data<AppState>, query: web::Query<PerformanceRankingQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match performance_target_service::get_performance_ranking(db, query.year, query.month, query.order_by, query.department_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[get("/statistics/customer/type")]
#[protect("statistics:customer:view")]
pub async fn get_customer_type_stats(state: web::Data<AppState>, query: web::Query<CustomerStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match customer_stats_service::get_customer_type_stats(db, query.year, query.month).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[get("/statistics/customer/source")]
#[protect("statistics:customer:view")]
pub async fn get_customer_source_stats(state: web::Data<AppState>, query: web::Query<CustomerStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match customer_stats_service::get_customer_source_stats(db, query.year, query.month).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[get("/statistics/customer/industry")]
#[protect("statistics:customer:view")]
pub async fn get_customer_industry_stats(state: web::Data<AppState>, query: web::Query<CustomerStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match customer_stats_service::get_customer_industry_stats(db, query.year, query.month).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[get("/statistics/customer/funnel")]
#[protect("statistics:customer:view")]
pub async fn get_customer_funnel(state: web::Data<AppState>, query: web::Query<CustomerStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match customer_stats_service::get_customer_funnel(db, query.year, query.month).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[get("/statistics/employee/customer-count")]
#[protect("statistics:employee:view")]
pub async fn get_employee_customer_count(state: web::Data<AppState>, query: web::Query<EmployeeStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match employee_stats_service::get_employee_customer_count(db, query.department_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[get("/statistics/employee/follow-up")]
#[protect("statistics:employee:view")]
pub async fn get_employee_follow_up(state: web::Data<AppState>, query: web::Query<EmployeeStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match employee_stats_service::get_employee_follow_up(db, query.year, query.month, query.department_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[get("/statistics/employee/conversion")]
#[protect("statistics:employee:view")]
pub async fn get_employee_conversion(state: web::Data<AppState>, query: web::Query<EmployeeStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match employee_stats_service::get_employee_conversion(db, query.year, query.month, query.department_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[get("/statistics/contract/ranking")]
#[protect("statistics:contract:view")]
pub async fn get_contract_ranking(state: web::Data<AppState>, query: web::Query<ContractStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match contract_stats_service::get_contract_ranking(db, query.year, query.month, query.order_by, query.order_type, query.limit).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[get("/statistics/contract/type-distribution")]
#[protect("statistics:contract:view")]
pub async fn get_contract_type_distribution(state: web::Data<AppState>, query: web::Query<CustomerStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match contract_stats_service::get_contract_type_distribution(db, query.year, query.month).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[get("/statistics/contract/status-analysis")]
#[protect("statistics:contract:view")]
pub async fn get_contract_status_analysis(state: web::Data<AppState>, query: web::Query<CustomerStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match contract_stats_service::get_contract_status_analysis(db, query.year, query.month).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[get("/statistics/payment/completion")]
#[protect("statistics:payment:view")]
pub async fn get_payment_completion(state: web::Data<AppState>, query: web::Query<PaymentStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match payment_stats_service::get_payment_completion(db, query.year, query.month).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[get("/statistics/payment/monthly-trend")]
#[protect("statistics:payment:view")]
pub async fn get_payment_monthly_trend(state: web::Data<AppState>, query: web::Query<PaymentStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match payment_stats_service::get_payment_monthly_trend(db, query.year).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[get("/statistics/payment/status-analysis")]
#[protect("statistics:payment:view")]
pub async fn get_payment_status_analysis(state: web::Data<AppState>, query: web::Query<CustomerStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match payment_stats_service::get_payment_status_analysis(db, query.year, query.month).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

#[get("/statistics/payment/ranking")]
#[protect("statistics:payment:view")]
pub async fn get_payment_ranking(state: web::Data<AppState>, query: web::Query<PaymentStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match payment_stats_service::get_payment_ranking(db, query.year, query.month, query.order_by, query.limit).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}