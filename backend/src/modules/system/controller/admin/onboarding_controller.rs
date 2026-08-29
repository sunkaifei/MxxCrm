//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use std::time::Instant;

use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use actix_web_grants::authorities::AuthDetails;

use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::system::model::onboarding::OnboardingSaveRequest;
use crate::modules::system::service::{admin_service, onboarding_service, system_log_service};

/// 管理员态判定（13.2-1）：持有 system:onboarding:list 视为管理员，
/// GET /onboarding/config 追加配置页全量数据（version + 含停用步骤），不另设接口
fn is_onboarding_admin(req: &HttpRequest) -> bool {
    req.extensions()
        .get::<AuthDetails<String>>()
        .map(|details| details.authorities.contains("system:onboarding:list"))
        .unwrap_or(false)
}

/// GET /onboarding/config（仅需登录；新用户默认角色无管理权限码，引导卡数据不受影响）
pub async fn config_get(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    let is_admin = is_onboarding_admin(&req);
    onboarding_service::get_config_for_user(db, user_id, is_admin)
        .await
        .map(|data| {
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))
        })
}

/// POST /onboarding/save（system:onboarding:save）：保存成功后落系统操作日志（13.3-4）
pub async fn config_save(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<OnboardingSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let started = Instant::now();
    let form = form_data.0;
    let operator = admin_service::get_by_detail(db, &Some(get_current_user_id(&req)))
        .await?
        .user_name
        .unwrap_or_else(|| "unknown".to_string());

    match onboarding_service::save_config(db, form.clone(), &operator).await {
        Ok(onboarding_service::SaveOutcome::Ok(new_version)) => {
            // 管理操作审计：失败仅告警，不影响主流程
            let ctx = system_log_service::SaveLogContext {
                request: &req,
                title: Some("入职引导配置".to_string()),
                business_type: Some(2),
                method: Some("onboarding_controller::config_save".to_string()),
                request_method: Some("POST".to_string()),
                operator_type: Some(1),
                oper_name: Some(operator.clone()),
                dept_name: None,
                oper_param: Some(serde_json::to_string(&form).unwrap_or_default()),
                json_result: Some(format!("{{\"version\":{}}}", new_version)),
                status: Some(0),
                error_msg: None,
                status_code: Some(200),
                elapsed: Some(started.elapsed().as_millis() as i64),
            };
            if let Err(e) = system_log_service::save_log(db, ctx).await {
                log::warn!("记录入职引导配置审计日志失败: {}", e);
            }
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::success(new_version, "local")))
        }
        Ok(onboarding_service::SaveOutcome::VersionConflict) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(409, "配置已被他人修改，请刷新后重试", "local"))),
        Err(e) => Err(e),
    }
}

/// 注册入职引导路由（/onboarding/config 仅需登录；/onboarding/save 需 system:onboarding:save）
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/onboarding")
            // GET /onboarding/config - 步骤列表 + 配置 + 当前进度（管理员态追加全量数据）
            .route("/config", web::get().to(config_get))
            // POST /onboarding/save - 管理员保存步骤与配置（乐观锁 409）
            .route(
                "/save",
                web::post()
                    .to(config_save)
                    .wrap(require_permission("system:onboarding:save")),
            ),
    );
}
