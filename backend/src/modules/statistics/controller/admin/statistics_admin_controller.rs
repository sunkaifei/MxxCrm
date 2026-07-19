use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::MetaResp;
use crate::modules::statistics::model::performance_target::{PerformanceTargetQuery, PerformanceTargetBatchSaveRequest, PerformanceRankingQuery};
use crate::modules::statistics::model::customer_stats::CustomerStatsQuery;
use crate::modules::statistics::model::employee_stats::EmployeeStatsQuery;
use crate::modules::statistics::model::contract_stats::ContractStatsQuery;
use crate::modules::statistics::model::payment_stats::PaymentStatsQuery;
use crate::modules::statistics::service::{performance_target_service, customer_stats_service, employee_stats_service, contract_stats_service, payment_stats_service};
use actix_web::{web, HttpResponse};

pub async fn get_performance_target(state: web::Data<AppState>, query: web::Query<PerformanceTargetQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match performance_target_service::get_targets(db, query.employee_id, query.year, query.month).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

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

pub async fn get_monthly_performance(state: web::Data<AppState>, query: web::Query<PerformanceRankingQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match performance_target_service::get_monthly_performance(db, query.year, query.department_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_performance_ranking(state: web::Data<AppState>, query: web::Query<PerformanceRankingQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match performance_target_service::get_performance_ranking(db, query.year, query.month, query.order_by, query.department_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_customer_type_stats(state: web::Data<AppState>, query: web::Query<CustomerStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match customer_stats_service::get_customer_type_stats(db, query.year, query.month).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_customer_source_stats(state: web::Data<AppState>, query: web::Query<CustomerStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match customer_stats_service::get_customer_source_stats(db, query.year, query.month).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_customer_industry_stats(state: web::Data<AppState>, query: web::Query<CustomerStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match customer_stats_service::get_customer_industry_stats(db, query.year, query.month).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_customer_funnel(state: web::Data<AppState>, query: web::Query<CustomerStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match customer_stats_service::get_customer_funnel(db, query.year, query.month).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_employee_customer_count(state: web::Data<AppState>, query: web::Query<EmployeeStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match employee_stats_service::get_employee_customer_count(db, query.department_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_employee_follow_up(state: web::Data<AppState>, query: web::Query<EmployeeStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match employee_stats_service::get_employee_follow_up(db, query.year, query.month, query.department_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_employee_conversion(state: web::Data<AppState>, query: web::Query<EmployeeStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match employee_stats_service::get_employee_conversion(db, query.year, query.month, query.department_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_contract_ranking(state: web::Data<AppState>, query: web::Query<ContractStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match contract_stats_service::get_contract_ranking(db, query.year, query.month, query.order_by, query.order_type, query.limit).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_contract_type_distribution(state: web::Data<AppState>, query: web::Query<CustomerStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match contract_stats_service::get_contract_type_distribution(db, query.year, query.month).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_contract_status_analysis(state: web::Data<AppState>, query: web::Query<CustomerStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match contract_stats_service::get_contract_status_analysis(db, query.year, query.month).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_payment_completion(state: web::Data<AppState>, query: web::Query<PaymentStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match payment_stats_service::get_payment_completion(db, query.year, query.month).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_payment_monthly_trend(state: web::Data<AppState>, query: web::Query<PaymentStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match payment_stats_service::get_payment_monthly_trend(db, query.year).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_payment_status_analysis(state: web::Data<AppState>, query: web::Query<CustomerStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match payment_stats_service::get_payment_status_analysis(db, query.year, query.month).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_payment_ranking(state: web::Data<AppState>, query: web::Query<PaymentStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match payment_stats_service::get_payment_ranking(db, query.year, query.month, query.order_by, query.limit).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册统计分析模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(sys_statistics_admin_controller::register)` 注册。
///
/// 注意：本模块包含五个子领域（performance/customer/employee/contract/payment），
/// 因此在 register 中使用五个独立的 scope。
pub fn register(cfg: &mut web::ServiceConfig) {
    // 业绩目标统计
    cfg.service(
        web::scope("/statistics/performance")
            // GET /statistics/performance/target - 业绩目标查询
            .route(
                "/target",
                web::get()
                    .to(get_performance_target)
                    .wrap(require_permission("statistics:performance:view")),
            )
            // POST /statistics/performance/target/save - 保存业绩目标
            .route(
                "/target/save",
                web::post()
                    .to(save_performance_target)
                    .wrap(require_permission("statistics:performance:manage")),
            )
            // GET /statistics/performance/monthly - 月度业绩
            .route(
                "/monthly",
                web::get()
                    .to(get_monthly_performance)
                    .wrap(require_permission("statistics:performance:view")),
            )
            // GET /statistics/performance/ranking - 业绩排行
            .route(
                "/ranking",
                web::get()
                    .to(get_performance_ranking)
                    .wrap(require_permission("statistics:performance:view")),
            ),
    );
    // 客户统计
    cfg.service(
        web::scope("/statistics/customer")
            // GET /statistics/customer/type - 客户类型统计
            .route(
                "/type",
                web::get()
                    .to(get_customer_type_stats)
                    .wrap(require_permission("statistics:customer:view")),
            )
            // GET /statistics/customer/source - 客户来源统计
            .route(
                "/source",
                web::get()
                    .to(get_customer_source_stats)
                    .wrap(require_permission("statistics:customer:view")),
            )
            // GET /statistics/customer/industry - 客户行业统计
            .route(
                "/industry",
                web::get()
                    .to(get_customer_industry_stats)
                    .wrap(require_permission("statistics:customer:view")),
            )
            // GET /statistics/customer/funnel - 客户漏斗
            .route(
                "/funnel",
                web::get()
                    .to(get_customer_funnel)
                    .wrap(require_permission("statistics:customer:view")),
            ),
    );
    // 员工统计
    cfg.service(
        web::scope("/statistics/employee")
            // GET /statistics/employee/customer-count - 员工客户数
            .route(
                "/customer-count",
                web::get()
                    .to(get_employee_customer_count)
                    .wrap(require_permission("statistics:employee:view")),
            )
            // GET /statistics/employee/follow-up - 员工跟进统计
            .route(
                "/follow-up",
                web::get()
                    .to(get_employee_follow_up)
                    .wrap(require_permission("statistics:employee:view")),
            )
            // GET /statistics/employee/conversion - 员工转化率
            .route(
                "/conversion",
                web::get()
                    .to(get_employee_conversion)
                    .wrap(require_permission("statistics:employee:view")),
            ),
    );
    // 合同统计
    cfg.service(
        web::scope("/statistics/contract")
            // GET /statistics/contract/ranking - 合同排行
            .route(
                "/ranking",
                web::get()
                    .to(get_contract_ranking)
                    .wrap(require_permission("statistics:contract:view")),
            )
            // GET /statistics/contract/type-distribution - 合同类型分布
            .route(
                "/type-distribution",
                web::get()
                    .to(get_contract_type_distribution)
                    .wrap(require_permission("statistics:contract:view")),
            )
            // GET /statistics/contract/status-analysis - 合同状态分析
            .route(
                "/status-analysis",
                web::get()
                    .to(get_contract_status_analysis)
                    .wrap(require_permission("statistics:contract:view")),
            ),
    );
    // 回款统计
    cfg.service(
        web::scope("/statistics/payment")
            // GET /statistics/payment/completion - 回款完成情况
            .route(
                "/completion",
                web::get()
                    .to(get_payment_completion)
                    .wrap(require_permission("statistics:payment:view")),
            )
            // GET /statistics/payment/monthly-trend - 回款月度趋势
            .route(
                "/monthly-trend",
                web::get()
                    .to(get_payment_monthly_trend)
                    .wrap(require_permission("statistics:payment:view")),
            )
            // GET /statistics/payment/status-analysis - 回款状态分析
            .route(
                "/status-analysis",
                web::get()
                    .to(get_payment_status_analysis)
                    .wrap(require_permission("statistics:payment:view")),
            )
            // GET /statistics/payment/ranking - 回款排行
            .route(
                "/ranking",
                web::get()
                    .to(get_payment_ranking)
                    .wrap(require_permission("statistics:payment:view")),
            ),
    );
}
