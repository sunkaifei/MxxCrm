use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::MetaResp;
use crate::modules::statistics::model::performance_target::{PerformanceTargetQuery, PerformanceTargetBatchSaveRequest, PerformanceRankingQuery};
use crate::modules::statistics::model::performance_overview::PerformanceOverviewQuery;
use crate::modules::statistics::model::customer_stats::CustomerStatsQuery;
use crate::modules::statistics::model::employee_stats::EmployeeStatsQuery;
use crate::modules::statistics::model::contract_stats::ContractStatsQuery;
use crate::modules::statistics::model::payment_stats::PaymentStatsQuery;
use crate::modules::statistics::service::{performance_target_service, customer_stats_service, employee_stats_service, contract_stats_service, payment_stats_service, performance_overview_service};
use crate::modules::system::service::data_scope_service;
use actix_web::{web, HttpRequest, HttpResponse};

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

pub async fn get_monthly_performance(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PerformanceRankingQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();

    // 获取当前用户可访问的用户ID列表（按 data_scope 过滤）
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let current_user_id = jwt_token.id.unwrap_or_default();
    let accessible_user_ids = data_scope_service::get_accessible_user_ids(db, current_user_id).await.unwrap_or(None);

    match performance_target_service::get_monthly_performance(db, query.year, query.department_id, accessible_user_ids).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_performance_ranking(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PerformanceRankingQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();

    // 获取当前用户可访问的用户ID列表（按 data_scope 过滤）
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let current_user_id = jwt_token.id.unwrap_or_default();
    let accessible_user_ids = data_scope_service::get_accessible_user_ids(db, current_user_id).await.unwrap_or(None);

    match performance_target_service::get_performance_ranking(db, query.year, query.month, query.order_by, query.department_id, accessible_user_ids).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

// ==================== 业绩概览扩展接口（15项补充功能）====================

/// GET /statistics/performance/comparison - 业绩对比（同比/环比）
pub async fn get_performance_comparison(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PerformanceOverviewQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();

    // 获取当前用户可访问的用户ID列表（按 data_scope 过滤）
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let current_user_id = jwt_token.id.unwrap_or_default();
    let accessible_user_ids = data_scope_service::get_accessible_user_ids(db, current_user_id).await.unwrap_or(None);

    match performance_overview_service::get_comparison(db, q.year, q.month, q.time_dimension, accessible_user_ids).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// GET /statistics/performance/forecast - 业绩预测
pub async fn get_performance_forecast(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PerformanceOverviewQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();

    // 获取当前用户可访问的用户ID列表（按 data_scope 过滤）
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let current_user_id = jwt_token.id.unwrap_or_default();
    let accessible_user_ids = data_scope_service::get_accessible_user_ids(db, current_user_id).await.unwrap_or(None);

    match performance_overview_service::get_forecast(db, q.year, q.month, q.time_dimension, accessible_user_ids).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// GET /statistics/performance/funnel - 销售漏斗
pub async fn get_sales_funnel(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PerformanceOverviewQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();

    // 获取当前用户可访问的用户ID列表（按 data_scope 过滤）
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let current_user_id = jwt_token.id.unwrap_or_default();
    let accessible_user_ids = data_scope_service::get_accessible_user_ids(db, current_user_id).await.unwrap_or(None);

    match performance_overview_service::get_funnel(db, q.year, q.month, q.time_dimension, accessible_user_ids).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// GET /statistics/performance/customer-breakdown - 客户维度拆解
pub async fn get_customer_breakdown(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PerformanceOverviewQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();

    // 获取当前用户可访问的用户ID列表（按 data_scope 过滤）
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let current_user_id = jwt_token.id.unwrap_or_default();
    let accessible_user_ids = data_scope_service::get_accessible_user_ids(db, current_user_id).await.unwrap_or(None);

    match performance_overview_service::get_customer_breakdown(db, q.year, q.month, q.time_dimension, accessible_user_ids).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// GET /statistics/performance/product-breakdown - 产品维度拆解
pub async fn get_product_breakdown(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PerformanceOverviewQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();

    // 获取当前用户可访问的用户ID列表（按 data_scope 过滤）
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let current_user_id = jwt_token.id.unwrap_or_default();
    let accessible_user_ids = data_scope_service::get_accessible_user_ids(db, current_user_id).await.unwrap_or(None);

    match performance_overview_service::get_product_breakdown(db, q.year, q.month, q.time_dimension, accessible_user_ids).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// GET /statistics/performance/behavior - 行为指标
pub async fn get_behavior_metrics(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PerformanceOverviewQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();

    // 获取当前用户可访问的用户ID列表（按 data_scope 过滤）
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let current_user_id = jwt_token.id.unwrap_or_default();
    let accessible_user_ids = data_scope_service::get_accessible_user_ids(db, current_user_id).await.unwrap_or(None);

    match performance_overview_service::get_behavior_metrics(db, q.year, q.month, q.time_dimension, accessible_user_ids).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// GET /statistics/performance/region-breakdown - 区域维度拆解
pub async fn get_region_breakdown(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PerformanceOverviewQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();

    // 获取当前用户可访问的用户ID列表（按 data_scope 过滤）
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let current_user_id = jwt_token.id.unwrap_or_default();
    let accessible_user_ids = data_scope_service::get_accessible_user_ids(db, current_user_id).await.unwrap_or(None);

    match performance_overview_service::get_region_breakdown(db, q.year, q.month, q.time_dimension, accessible_user_ids).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// GET /statistics/performance/personal-growth - 个人成长档案
pub async fn get_personal_growth(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PerformanceOverviewQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();

    // 个人成长档案：未指定 employee_id 时使用当前用户ID
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let current_user_id = jwt_token.id.unwrap_or_default();
    let employee_id = q.employee_id.unwrap_or(current_user_id);

    match performance_overview_service::get_personal_growth(db, Some(employee_id)).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// GET /statistics/performance/milestone - 业绩里程碑
pub async fn get_performance_milestone(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PerformanceOverviewQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();

    // 业绩里程碑：未指定 employee_id 时使用当前用户ID
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let current_user_id = jwt_token.id.unwrap_or_default();
    let employee_id = q.employee_id.unwrap_or(current_user_id);

    match performance_overview_service::get_milestone(db, q.year, Some(employee_id), current_user_id).await {
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
            )
            // GET /statistics/performance/comparison - 业绩对比（同比/环比）
            .route(
                "/comparison",
                web::get()
                    .to(get_performance_comparison)
                    .wrap(require_permission("statistics:performance:view")),
            )
            // GET /statistics/performance/forecast - 业绩预测
            .route(
                "/forecast",
                web::get()
                    .to(get_performance_forecast)
                    .wrap(require_permission("statistics:performance:view")),
            )
            // GET /statistics/performance/funnel - 销售漏斗
            .route(
                "/funnel",
                web::get()
                    .to(get_sales_funnel)
                    .wrap(require_permission("statistics:performance:view")),
            )
            // GET /statistics/performance/customer-breakdown - 客户维度拆解
            .route(
                "/customer-breakdown",
                web::get()
                    .to(get_customer_breakdown)
                    .wrap(require_permission("statistics:performance:view")),
            )
            // GET /statistics/performance/product-breakdown - 产品维度拆解
            .route(
                "/product-breakdown",
                web::get()
                    .to(get_product_breakdown)
                    .wrap(require_permission("statistics:performance:view")),
            )
            // GET /statistics/performance/behavior - 行为指标
            .route(
                "/behavior",
                web::get()
                    .to(get_behavior_metrics)
                    .wrap(require_permission("statistics:performance:view")),
            )
            // GET /statistics/performance/region-breakdown - 区域维度拆解
            .route(
                "/region-breakdown",
                web::get()
                    .to(get_region_breakdown)
                    .wrap(require_permission("statistics:performance:view")),
            )
            // GET /statistics/performance/personal-growth - 个人成长档案
            .route(
                "/personal-growth",
                web::get()
                    .to(get_personal_growth)
                    .wrap(require_permission("statistics:performance:view")),
            )
            // GET /statistics/performance/milestone - 业绩里程碑
            .route(
                "/milestone",
                web::get()
                    .to(get_performance_milestone)
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
