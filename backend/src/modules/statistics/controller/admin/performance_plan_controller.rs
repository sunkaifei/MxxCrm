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
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::statistics::model::performance_plan::{
    CreatePlanRequest, SubmitPlanRequest, ReviewPlanRequest, ModifyPlanRequest, PlanQuery,
    UpdatePlanTargetsRequest, PlanCoverageQuery,
};
use crate::modules::statistics::service::performance_plan_service;
use crate::modules::system::service::data_scope_service;
use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Datelike;

/// 从Admin JWT中获取当前用户信息
fn get_admin_info(req: &HttpRequest) -> (i64, String) {
    let jwt_secret = crate::config::section::<String>("server", "jwt_secret_admin", "".to_string());
    let token_str = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .trim_start_matches("Bearer ")
        .to_string();

    match JWTToken::verify(&jwt_secret, &token_str) {
        Ok(data) => (
            data.id.unwrap_or(0),
            data.username.unwrap_or_default(),
        ),
        Err(_) => (0, String::new()),
    }
}

/// 创建草稿计划
/// 支持「代建」：请求指定 employeeId 时为目标员工创建（管理员集中管理入口），
/// 目标必须是本人或数据权限范围内的下属；缺省仍是本人自助创建。
pub async fn create_plan(state: web::Data<AppState>, req: web::Json<CreatePlanRequest>, http_req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let req = req.into_inner();
    let (current_user_id, _) = get_admin_info(&http_req);

    // 目标解析：0/缺省 = 本人
    let target_id = req.employee_id.filter(|id| *id > 0).unwrap_or(current_user_id);
    let allowed = target_id == current_user_id || match data_scope_service::get_accessible_user_ids(db, current_user_id).await {
        Ok(None) => true,
        Ok(Some(ids)) => ids.contains(&target_id),
        Err(_) => false,
    };

    let result = if allowed {
        performance_plan_service::create_plan(db, target_id, &req).await
    } else {
        Err(crate::core::errors::error::Error::BadRequest("目标员工不在您的数据权限范围内，无法代建年度计划".to_string()))
    };

    match result {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 提交计划（草稿→待审批）
pub async fn submit_plan(state: web::Data<AppState>, req: web::Json<SubmitPlanRequest>, http_req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let req = req.into_inner();
    let (user_id, user_name) = get_admin_info(&http_req);

    match performance_plan_service::submit_plan(db, req.plan_id, user_id, &user_name).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 审批通过
pub async fn approve_plan(state: web::Data<AppState>, req: web::Json<ReviewPlanRequest>, http_req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let req = req.into_inner();
    let (user_id, user_name) = get_admin_info(&http_req);

    match performance_plan_service::approve_plan(db, &req, user_id, &user_name).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 驳回
pub async fn reject_plan(state: web::Data<AppState>, req: web::Json<ReviewPlanRequest>, http_req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let req = req.into_inner();
    let (user_id, user_name) = get_admin_info(&http_req);

    match performance_plan_service::reject_plan(db, &req, user_id, &user_name).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 申请修改
pub async fn modify_plan(state: web::Data<AppState>, req: web::Json<ModifyPlanRequest>, http_req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let req = req.into_inner();
    let (user_id, user_name) = get_admin_info(&http_req);

    match performance_plan_service::modify_plan(db, &req, user_id, &user_name).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 查询计划列表
pub async fn get_plan_list(state: web::Data<AppState>, query: web::Query<PlanQuery>, http_req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let query = query.into_inner();
    let (current_user_id, _) = get_admin_info(&http_req);

    match performance_plan_service::get_plan_list(
        db, query.employee_id, query.year, query.status, query.pending_my_approval, current_user_id
    ).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 查询计划详情（含月度目标和审批记录）
pub async fn get_plan_detail(state: web::Data<AppState>, query: web::Query<SubmitPlanRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let plan_id = query.plan_id;

    match performance_plan_service::get_plan_detail(db, plan_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 年度计划覆盖度（集中管理视角：数据权限范围内谁还缺当年销售计划）
pub async fn get_plan_coverage(state: web::Data<AppState>, http_req: HttpRequest, query: web::Query<PlanCoverageQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    let (current_user_id, _) = get_admin_info(&http_req);
    let year = q.year.unwrap_or_else(|| chrono::Local::now().year());

    match data_scope_service::get_accessible_user_ids(db, current_user_id).await {
        Ok(scope) => match performance_plan_service::get_plan_coverage(db, &scope, year).await {
            Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
            Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
        },
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &format!("数据权限解析失败: {}", e), "local"))),
    }
}

/// 获取计划修改详情（编辑回显）
pub async fn get_plan_modify_detail(state: web::Data<AppState>, query: web::Query<SubmitPlanRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let plan_id = query.plan_id;

    match performance_plan_service::get_plan_modify_detail(db, plan_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 更新草稿/驳回状态的月度目标（不走审批流）
pub async fn update_plan_targets(state: web::Data<AppState>, req: web::Json<UpdatePlanTargetsRequest>, http_req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let req = req.into_inner();
    let (user_id, user_name) = get_admin_info(&http_req);

    match performance_plan_service::update_plan_targets(db, &req, user_id, &user_name).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 获取进度汇总（个人 + 团队，自下而上汇总）
pub async fn get_plan_progress_summary(state: web::Data<AppState>, http_req: HttpRequest, query: web::Query<PlanQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let (user_id, _) = get_admin_info(&http_req);
    let year = query.year.unwrap_or_else(|| chrono::Local::now().year());

    match performance_plan_service::get_plan_progress_summary(db, user_id, year).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册业绩计划模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(performance_plan_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/statistics/performance/plan")
            // POST /statistics/performance/plan/create - 创建草稿计划
            .route(
                "/create",
                web::post()
                    .to(create_plan)
                    .wrap(require_permission("statistics:performance-plan:manage")),
            )
            // POST /statistics/performance/plan/submit - 提交计划
            .route(
                "/submit",
                web::post()
                    .to(submit_plan)
                    .wrap(require_permission("statistics:performance-plan:manage")),
            )
            // POST /statistics/performance/plan/approve - 审批通过
            .route(
                "/approve",
                web::post()
                    .to(approve_plan)
                    .wrap(require_permission("statistics:performance-plan:audit")),
            )
            // POST /statistics/performance/plan/reject - 驳回
            .route(
                "/reject",
                web::post()
                    .to(reject_plan)
                    .wrap(require_permission("statistics:performance-plan:audit")),
            )
            // POST /statistics/performance/plan/modify - 申请修改
            .route(
                "/modify",
                web::post()
                    .to(modify_plan)
                    .wrap(require_permission("statistics:performance-plan:manage")),
            )
            // GET /statistics/performance/plan/list - 计划列表
            .route(
                "/list",
                web::get()
                    .to(get_plan_list)
                    .wrap(require_permission("statistics:performance-plan:view")),
            )
            // GET /statistics/performance/plan/detail - 计划详情
            .route(
                "/detail",
                web::get()
                    .to(get_plan_detail)
                    .wrap(require_permission("statistics:performance-plan:view")),
            )
            // GET /statistics/performance/plan/modify-detail - 计划修改详情
            .route(
                "/modify-detail",
                web::get()
                    .to(get_plan_modify_detail)
                    .wrap(require_permission("statistics:performance-plan:view")),
            )
            // POST /statistics/performance/plan/update-targets - 更新草稿/驳回状态的月度目标
            .route(
                "/update-targets",
                web::post()
                    .to(update_plan_targets)
                    .wrap(require_permission("statistics:performance-plan:manage")),
            )
            // GET /statistics/performance/plan/progress-summary - 进度汇总（个人+团队）
            .route(
                "/progress-summary",
                web::get()
                    .to(get_plan_progress_summary)
                    .wrap(require_permission("statistics:performance-plan:view")),
            )
            // GET /statistics/performance/plan/coverage - 年度计划覆盖度（集中管理视角）
            .route(
                "/coverage",
                web::get()
                    .to(get_plan_coverage)
                    .wrap(require_permission("statistics:performance-plan:view")),
            ),
    );
}
