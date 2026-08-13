use crate::core::errors::error::{Error, Result};
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::statistics::model::performance_target::{PerformanceTargetQuery, PerformanceTargetBatchSaveRequest, PerformanceRankingQuery};
use crate::modules::statistics::model::performance_overview::PerformanceOverviewQuery;
use crate::modules::statistics::model::customer_stats::CustomerStatsQuery;
use crate::modules::statistics::model::employee_stats::EmployeeStatsQuery;
use crate::modules::statistics::model::contract_stats::ContractStatsQuery;
use crate::modules::statistics::model::payment_stats::PaymentStatsQuery;
use crate::modules::statistics::model::stats_agg::{AggBatchQuery, AggRefreshRequest};
use crate::modules::statistics::service::stats_range::{StatsRange, StatsScope};
use crate::modules::statistics::service::{performance_target_service, customer_stats_service, employee_stats_service, contract_stats_service, payment_stats_service, performance_overview_service, stats_agg_query};
use crate::modules::statistics::service::{stats_agg_service, stats_cache};
use crate::modules::system::service::data_scope_service;
use actix_web::{web, HttpRequest, HttpResponse};
use sea_orm::{ConnectionTrait, DbBackend, Statement};

/// 解析统计时间范围：
/// 1. 优先使用 start_date/end_date（YYYY-MM-DD，支持季度/自定义范围）
/// 2. 否则按 year（可选 month）转换为月/年范围
/// 3. 均未传时返回空范围（表示全部，不过滤）
fn resolve_range(start_date: Option<String>, end_date: Option<String>, year: Option<i32>, month: Option<i32>) -> StatsRange {
    let parse = |s: &str| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok();
    if let (Some(s), Some(e)) = (&start_date, &end_date) {
        if let (Some(s), Some(e)) = (parse(s), parse(e)) {
            return StatsRange { start: Some(s), end: Some(e) };
        }
    }
    if let Some(y) = year {
        if let Some(m) = month {
            let start = chrono::NaiveDate::from_ymd_opt(y, m as u32, 1);
            let next_month = if m == 12 {
                chrono::NaiveDate::from_ymd_opt(y + 1, 1, 1)
            } else {
                chrono::NaiveDate::from_ymd_opt(y, (m + 1) as u32, 1)
            };
            let end = next_month.map(|n| n - chrono::Duration::days(1));
            return StatsRange { start, end };
        }
        return StatsRange {
            start: chrono::NaiveDate::from_ymd_opt(y, 1, 1),
            end: chrono::NaiveDate::from_ymd_opt(y, 12, 31),
        };
    }
    StatsRange { start: None, end: None }
}

/// 解析当前用户数据权限范围（错误时返回错误而非降级为超管，避免越权）
async fn resolve_scope(db: &sea_orm::DbConn, req: &HttpRequest) -> Result<StatsScope> {
    let current_user_id = get_current_user_id(req);
    data_scope_service::get_accessible_user_ids(db, current_user_id)
        .await
        .map_err(|e| Error::from(format!("数据权限解析失败: {}", e)))
}

/// 汇总表快路径启用条件：
/// - 有明确起止日期（无下界的"全部"查询无法确认汇总覆盖范围，走实时）
/// - 区间不含今天（含当日需实时合并，走实时路径）
fn can_use_agg(range: &StatsRange) -> bool {
    range.start.is_some() && range.end.is_some() && !range.covers_today()
}

pub async fn get_performance_target(state: web::Data<AppState>, query: web::Query<PerformanceTargetQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    
    match performance_target_service::get_targets(db, query.employee_id, query.year, query.month).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn save_performance_target(state: web::Data<AppState>, form_data: web::Json<PerformanceTargetBatchSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    
    if form_data.targets.is_none() || form_data.targets.as_ref().unwrap().is_empty() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "目标数据不能为空", "local")));
    }
    
    match performance_target_service::save_targets(db, form_data.targets.as_ref().unwrap()).await {
        Ok((saved_count, updated_count)) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success({
            serde_json::json!({
                "saved_count": saved_count,
                "updated_count": updated_count
            })
        }, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_monthly_performance(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PerformanceRankingQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();

    // 获取当前用户可访问的用户ID列表（按 data_scope 过滤）
    let current_user_id = get_current_user_id(&req);
    let accessible_user_ids = data_scope_service::get_accessible_user_ids(db, current_user_id).await.unwrap_or(None);

    match performance_target_service::get_monthly_performance(db, query.year, query.department_id, accessible_user_ids).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_performance_ranking(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PerformanceRankingQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();

    // 获取当前用户可访问的用户ID列表（按 data_scope 过滤）
    let current_user_id = get_current_user_id(&req);
    let accessible_user_ids = data_scope_service::get_accessible_user_ids(db, current_user_id).await.unwrap_or(None);

    match performance_target_service::get_performance_ranking(db, query.year, query.month, query.order_by, query.department_id, accessible_user_ids).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

// ==================== 业绩概览扩展接口（15项补充功能）====================

/// GET /statistics/performance/comparison - 业绩对比（同比/环比）
pub async fn get_performance_comparison(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PerformanceOverviewQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();

    // 获取当前用户可访问的用户ID列表（按 data_scope 过滤）
    let current_user_id = get_current_user_id(&req);
    let accessible_user_ids = data_scope_service::get_accessible_user_ids(db, current_user_id).await.unwrap_or(None);

    match performance_overview_service::get_comparison(db, q.year, q.month, q.time_dimension, accessible_user_ids).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// GET /statistics/performance/forecast - 业绩预测
pub async fn get_performance_forecast(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PerformanceOverviewQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();

    // 获取当前用户可访问的用户ID列表（按 data_scope 过滤）
    let current_user_id = get_current_user_id(&req);
    let accessible_user_ids = data_scope_service::get_accessible_user_ids(db, current_user_id).await.unwrap_or(None);

    match performance_overview_service::get_forecast(db, q.year, q.month, q.time_dimension, accessible_user_ids).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// GET /statistics/performance/funnel - 销售漏斗
pub async fn get_sales_funnel(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PerformanceOverviewQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();

    // 获取当前用户可访问的用户ID列表（按 data_scope 过滤）
    let current_user_id = get_current_user_id(&req);
    let accessible_user_ids = data_scope_service::get_accessible_user_ids(db, current_user_id).await.unwrap_or(None);

    match performance_overview_service::get_funnel(db, q.year, q.month, q.time_dimension, accessible_user_ids).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// GET /statistics/performance/customer-breakdown - 客户维度拆解
pub async fn get_customer_breakdown(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PerformanceOverviewQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();

    // 获取当前用户可访问的用户ID列表（按 data_scope 过滤）
    let current_user_id = get_current_user_id(&req);
    let accessible_user_ids = data_scope_service::get_accessible_user_ids(db, current_user_id).await.unwrap_or(None);

    match performance_overview_service::get_customer_breakdown(db, q.year, q.month, q.time_dimension, accessible_user_ids).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// GET /statistics/performance/product-breakdown - 产品维度拆解
pub async fn get_product_breakdown(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PerformanceOverviewQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();

    // 获取当前用户可访问的用户ID列表（按 data_scope 过滤）
    let current_user_id = get_current_user_id(&req);
    let accessible_user_ids = data_scope_service::get_accessible_user_ids(db, current_user_id).await.unwrap_or(None);

    match performance_overview_service::get_product_breakdown(db, q.year, q.month, q.time_dimension, accessible_user_ids).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// GET /statistics/performance/behavior - 行为指标
pub async fn get_behavior_metrics(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PerformanceOverviewQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();

    // 获取当前用户可访问的用户ID列表（按 data_scope 过滤）
    let current_user_id = get_current_user_id(&req);
    let accessible_user_ids = data_scope_service::get_accessible_user_ids(db, current_user_id).await.unwrap_or(None);

    match performance_overview_service::get_behavior_metrics(db, q.year, q.month, q.time_dimension, accessible_user_ids).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// GET /statistics/performance/region-breakdown - 区域维度拆解
pub async fn get_region_breakdown(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PerformanceOverviewQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();

    // 获取当前用户可访问的用户ID列表（按 data_scope 过滤）
    let current_user_id = get_current_user_id(&req);
    let accessible_user_ids = data_scope_service::get_accessible_user_ids(db, current_user_id).await.unwrap_or(None);

    match performance_overview_service::get_region_breakdown(db, q.year, q.month, q.time_dimension, accessible_user_ids).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// GET /statistics/performance/personal-growth - 个人成长档案
pub async fn get_personal_growth(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PerformanceOverviewQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();

    // 个人成长档案：未指定 employee_id 时使用当前用户ID
    let current_user_id = get_current_user_id(&req);
    let employee_id = q.employee_id.unwrap_or(current_user_id);

    match performance_overview_service::get_personal_growth(db, Some(employee_id)).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// GET /statistics/performance/milestone - 业绩里程碑
pub async fn get_performance_milestone(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PerformanceOverviewQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();

    // 业绩里程碑：未指定 employee_id 时使用当前用户ID
    let current_user_id = get_current_user_id(&req);
    let employee_id = q.employee_id.unwrap_or(current_user_id);

    match performance_overview_service::get_milestone(db, q.year, Some(employee_id), current_user_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_customer_type_stats(state: web::Data<AppState>, req: HttpRequest, query: web::Query<CustomerStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    let range = resolve_range(q.start_date, q.end_date, q.year, q.month);
    let scope = resolve_scope(db, &req).await?;
    let key = stats_cache::stats_cache_key("customer-type", &scope, &range, "");
    match stats_cache::get_or_set(&key, stats_cache::stats_ttl(&range), || async {
        customer_stats_service::get_customer_type_stats(db, &range, &scope).await
    }).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_customer_source_stats(state: web::Data<AppState>, req: HttpRequest, query: web::Query<CustomerStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    let range = resolve_range(q.start_date, q.end_date, q.year, q.month);
    let scope = resolve_scope(db, &req).await?;
    let key = stats_cache::stats_cache_key("customer-source", &scope, &range, "");
    match stats_cache::get_or_set(&key, stats_cache::stats_ttl(&range), || async {
        customer_stats_service::get_customer_source_stats(db, &range, &scope).await
    }).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_customer_industry_stats(state: web::Data<AppState>, req: HttpRequest, query: web::Query<CustomerStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    let range = resolve_range(q.start_date, q.end_date, q.year, q.month);
    let scope = resolve_scope(db, &req).await?;
    let key = stats_cache::stats_cache_key("customer-industry", &scope, &range, "");
    match stats_cache::get_or_set(&key, stats_cache::stats_ttl(&range), || async {
        customer_stats_service::get_customer_industry_stats(db, &range, &scope).await
    }).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_customer_funnel(state: web::Data<AppState>, req: HttpRequest, query: web::Query<CustomerStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    let range = resolve_range(q.start_date, q.end_date, q.year, q.month);
    let scope = resolve_scope(db, &req).await?;
    let key = stats_cache::stats_cache_key("customer-funnel", &scope, &range, "");
    match stats_cache::get_or_set(&key, stats_cache::stats_ttl(&range), || async {
        customer_stats_service::get_customer_funnel(db, &range, &scope).await
    }).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_employee_customer_count(state: web::Data<AppState>, req: HttpRequest, query: web::Query<EmployeeStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    let range = resolve_range(q.start_date, q.end_date, q.year, q.month);
    let scope = resolve_scope(db, &req).await?;
    let key = stats_cache::stats_cache_key("employee-customer-count", &scope, &range, "");
    match stats_cache::get_or_set(&key, stats_cache::stats_ttl(&range), || async {
        // 公共输入（双路径共用：员工列表 / 总客户存量 / 未跟进客户）
        let admins = employee_stats_service::load_admins(db, &scope).await?;
        let total_map = employee_stats_service::load_total_customer_map(db, &scope).await?;
        if can_use_agg(&range) && stats_agg_service::agg_fresh(db, stats_agg_service::TOPIC_EMPLOYEE).await {
            stats_agg_query::employee_customer_count(db, &range, &scope, admins, &total_map).await
        } else {
            employee_stats_service::get_employee_customer_count_realtime(db, &range, &scope, q.department_id, admins, total_map).await
        }
    }).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_employee_follow_up(state: web::Data<AppState>, req: HttpRequest, query: web::Query<EmployeeStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    let range = resolve_range(q.start_date, q.end_date, q.year, q.month);
    let scope = resolve_scope(db, &req).await?;
    let key = stats_cache::stats_cache_key("employee-follow-up", &scope, &range, "");
    match stats_cache::get_or_set(&key, stats_cache::stats_ttl(&range), || async {
        let admins = employee_stats_service::load_admins(db, &scope).await?;
        let no_follow_map = employee_stats_service::load_no_follow_map(db, &scope).await?;
        if can_use_agg(&range) && stats_agg_service::agg_fresh(db, stats_agg_service::TOPIC_EMPLOYEE).await {
            stats_agg_query::employee_follow_up(db, &range, &scope, admins, &no_follow_map).await
        } else {
            employee_stats_service::get_employee_follow_up_realtime(db, &range, &scope, q.department_id, admins, no_follow_map).await
        }
    }).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_employee_conversion(state: web::Data<AppState>, req: HttpRequest, query: web::Query<EmployeeStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    let range = resolve_range(q.start_date, q.end_date, q.year, q.month);
    let scope = resolve_scope(db, &req).await?;
    let key = stats_cache::stats_cache_key("employee-conversion", &scope, &range, "");
    match stats_cache::get_or_set(&key, stats_cache::stats_ttl(&range), || async {
        let admins = employee_stats_service::load_admins(db, &scope).await?;
        if can_use_agg(&range) && stats_agg_service::agg_fresh(db, stats_agg_service::TOPIC_EMPLOYEE).await {
            stats_agg_query::employee_conversion(db, &range, &scope, admins).await
        } else {
            employee_stats_service::get_employee_conversion_realtime(db, &range, &scope, q.department_id, admins).await
        }
    }).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_contract_ranking(state: web::Data<AppState>, req: HttpRequest, query: web::Query<ContractStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    let range = resolve_range(q.start_date, q.end_date, q.year, q.month);
    let scope = resolve_scope(db, &req).await?;
    let extra = format!("{}-{}", q.order_by.clone().unwrap_or_default(), q.limit.unwrap_or(10));
    let key = stats_cache::stats_cache_key("contract-ranking", &scope, &range, &extra);
    let (order_by, order_type, limit) = (q.order_by, q.order_type, q.limit);
    match stats_cache::get_or_set(&key, stats_cache::stats_ttl(&range), || async {
        if can_use_agg(&range) && stats_agg_service::agg_fresh(db, stats_agg_service::TOPIC_CONTRACT).await {
            stats_agg_query::contract_ranking(db, &range, &scope, order_by, limit).await
        } else {
            contract_stats_service::get_contract_ranking(db, &range, &scope, order_by, order_type, limit).await
        }
    }).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_contract_type_distribution(state: web::Data<AppState>, req: HttpRequest, query: web::Query<CustomerStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    let range = resolve_range(q.start_date, q.end_date, q.year, q.month);
    let scope = resolve_scope(db, &req).await?;
    let key = stats_cache::stats_cache_key("contract-type-dist", &scope, &range, "");
    match stats_cache::get_or_set(&key, stats_cache::stats_ttl(&range), || async {
        if can_use_agg(&range) && stats_agg_service::agg_fresh(db, stats_agg_service::TOPIC_CONTRACT).await {
            stats_agg_query::contract_type_distribution(db, &range, &scope).await
        } else {
            contract_stats_service::get_contract_type_distribution(db, &range, &scope).await
        }
    }).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_contract_status_analysis(state: web::Data<AppState>, req: HttpRequest, query: web::Query<CustomerStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    let range = resolve_range(q.start_date, q.end_date, q.year, q.month);
    let scope = resolve_scope(db, &req).await?;
    let key = stats_cache::stats_cache_key("contract-status", &scope, &range, "");
    match stats_cache::get_or_set(&key, stats_cache::stats_ttl(&range), || async {
        if can_use_agg(&range) && stats_agg_service::agg_fresh(db, stats_agg_service::TOPIC_CONTRACT).await {
            stats_agg_query::contract_status_analysis(db, &range, &scope).await
        } else {
            contract_stats_service::get_contract_status_analysis(db, &range, &scope).await
        }
    }).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_payment_completion(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PaymentStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    let range = resolve_range(q.start_date, q.end_date, q.year, q.month);
    let scope = resolve_scope(db, &req).await?;
    let key = stats_cache::stats_cache_key("payment-completion", &scope, &range, "");
    match stats_cache::get_or_set(&key, stats_cache::stats_ttl(&range), || async {
        if can_use_agg(&range)
            && stats_agg_service::agg_fresh(db, stats_agg_service::TOPIC_CONTRACT).await
            && stats_agg_service::agg_fresh(db, stats_agg_service::TOPIC_PAYMENT).await
        {
            stats_agg_query::payment_completion(db, &range, &scope).await
        } else {
            payment_stats_service::get_payment_completion(db, &range, &scope).await
        }
    }).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_payment_monthly_trend(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PaymentStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    let scope = resolve_scope(db, &req).await?;
    let default_year = chrono::Local::now().format("%Y").to_string().parse::<i32>().unwrap_or(2026);
    let extra = format!("y{}", q.year.unwrap_or(default_year));
    let full_range = StatsRange { start: None, end: None };
    let key = stats_cache::stats_cache_key("payment-monthly-trend", &scope, &full_range, &extra);
    let year = q.year;
    match stats_cache::get_or_set(&key, std::time::Duration::from_secs(300), || async {
        payment_stats_service::get_payment_monthly_trend(db, year, &scope).await
    }).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_payment_status_analysis(state: web::Data<AppState>, req: HttpRequest, query: web::Query<CustomerStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    let range = resolve_range(q.start_date, q.end_date, q.year, q.month);
    let scope = resolve_scope(db, &req).await?;
    // 状态分桶依赖明细（汇总表无分桶维度）→ 始终实时（有索引）+ 缓存
    let key = stats_cache::stats_cache_key("payment-status", &scope, &range, "");
    match stats_cache::get_or_set(&key, stats_cache::stats_ttl(&range), || async {
        payment_stats_service::get_payment_status_analysis(db, &range, &scope).await
    }).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn get_payment_ranking(state: web::Data<AppState>, req: HttpRequest, query: web::Query<PaymentStatsQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    let range = resolve_range(q.start_date, q.end_date, q.year, q.month);
    let scope = resolve_scope(db, &req).await?;
    let extra = format!("{}-{}", q.order_by.clone().unwrap_or_default(), q.limit.unwrap_or(10));
    let key = stats_cache::stats_cache_key("payment-ranking", &scope, &range, &extra);
    let (order_by, limit) = (q.order_by, q.limit);
    match stats_cache::get_or_set(&key, stats_cache::stats_ttl(&range), || async {
        if can_use_agg(&range)
            && stats_agg_service::agg_fresh(db, stats_agg_service::TOPIC_CONTRACT).await
            && stats_agg_service::agg_fresh(db, stats_agg_service::TOPIC_PAYMENT).await
        {
            stats_agg_query::payment_ranking(db, &range, &scope, order_by, limit).await
        } else {
            payment_stats_service::get_payment_ranking(db, &range, &scope, order_by, limit).await
        }
    }).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

// ==================== 汇总管理接口 ====================

/// POST /statistics/agg/refresh - 手动重算
pub async fn agg_refresh(state: web::Data<AppState>, req: HttpRequest, item: web::Json<AggRefreshRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let body = item.into_inner();

    let start = chrono::NaiveDate::parse_from_str(&body.start_date, "%Y-%m-%d")
        .map_err(|_| Error::from("start_date 格式错误，应为 YYYY-MM-DD"))?;
    let end = chrono::NaiveDate::parse_from_str(&body.end_date, "%Y-%m-%d")
        .map_err(|_| Error::from("end_date 格式错误，应为 YYYY-MM-DD"))?;
    let trigger_by = get_current_user_id(&req);

    let result = if body.topic == "all" {
        stats_agg_service::refresh_all_with(db, start, end, 2, trigger_by).await
    } else if stats_agg_service::ALL_TOPICS.contains(&body.topic.as_str()) {
        stats_agg_service::refresh_topic(db, &body.topic, start, end, 2, trigger_by).await.map(|(_, n)| n)
    } else {
        Err(Error::from("topic 无效，应为 contract/payment/employee/customer/all"))
    };

    match result {
        Ok(count) => {
            // 重算成功后清除全部统计缓存（下次查询重新加载）
            stats_cache::invalidate_all_stats_cache().await;
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(
                serde_json::json!({"affected_rows": count}), "local")))
        }
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// GET /statistics/agg/batches - 批次列表（追溯）
pub async fn agg_batches(state: web::Data<AppState>, query: web::Query<AggBatchQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 100);

    let mut sql = String::from(
        "SELECT id, topic, start_date, end_date, row_count, trigger_type, trigger_by, status, message, create_time
         FROM mxx_statistics_agg_batch",
    );
    let mut values: Vec<sea_orm::Value> = Vec::new();
    if let Some(topic) = &q.topic {
        values.push(topic.clone().into());
        sql.push_str(&format!(" WHERE topic = ${}", values.len()));
    }
    // 总数
    let count_sql = format!("SELECT COUNT(*)::int8 AS cnt FROM ({}) t", sql);
    let total = db
        .query_one_raw(Statement::from_sql_and_values(DbBackend::Postgres, count_sql, values.clone()))
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .and_then(|r| r.try_get::<i64>("", "cnt").ok())
        .unwrap_or(0);
    // 分页
    let offset = ((page - 1) * page_size) as i64;
    values.push(page_size.into());
    let limit_idx = values.len();
    values.push(offset.into());
    sql.push_str(&format!(" ORDER BY create_time DESC LIMIT ${} OFFSET ${}", limit_idx, limit_idx + 1));

    let rows = db
        .query_all_raw(Statement::from_sql_and_values(DbBackend::Postgres, sql, values))
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let list: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.try_get::<i64>("", "id").unwrap_or(0),
                "topic": r.try_get::<String>("", "topic").unwrap_or_default(),
                "start_date": r.try_get::<chrono::NaiveDate>("", "start_date").map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default(),
                "end_date": r.try_get::<chrono::NaiveDate>("", "end_date").map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default(),
                "row_count": r.try_get::<i32>("", "row_count").unwrap_or(0),
                "trigger_type": r.try_get::<i16>("", "trigger_type").unwrap_or(0),
                "trigger_by": r.try_get::<i64>("", "trigger_by").unwrap_or(0),
                "status": r.try_get::<i16>("", "status").unwrap_or(0),
                "message": r.try_get::<String>("", "message").ok(),
                "create_time": r.try_get::<chrono::NaiveDateTime>("", "create_time").map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_default(),
            })
        })
        .collect();

    Ok(HttpResponse::Ok().content_type(MPACK).body(
        MetaResp::success_with_page(list, "local", page, total as u32),
    ))
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
    // 汇总管理
    cfg.service(
        web::scope("/statistics/agg")
            // POST /statistics/agg/refresh - 手动重算
            .route(
                "/refresh",
                web::post()
                    .to(agg_refresh)
                    .wrap(require_permission("statistics:agg:manage")),
            )
            // GET /statistics/agg/batches - 批次列表
            .route(
                "/batches",
                web::get()
                    .to(agg_batches)
                    .wrap(require_permission("statistics:agg:list")),
            ),
    );
}
