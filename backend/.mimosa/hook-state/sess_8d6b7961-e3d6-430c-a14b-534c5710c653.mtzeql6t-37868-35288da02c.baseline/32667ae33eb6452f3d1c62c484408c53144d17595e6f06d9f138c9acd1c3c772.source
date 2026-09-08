//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{web, HttpRequest, HttpResponse};

use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::system::model::workspace::WorkspaceSaveRequest;
use crate::modules::system::service::{admin_service, workspace_service};

/// 当前用户可见工作台（方案 5.2/5.3-M3：GET /workspace/list，仅需登录；按用户过滤）
pub async fn list(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    workspace_service::list_visible(db, user_id).await.map(|list| {
        HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(list, "local"))
    })
}

/// 保存工作台（方案 5.2：POST /workspace/save，id 空新增否则更新）
pub async fn save(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<WorkspaceSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let form_data = form_data.0;
    let admin = admin_service::get_by_detail(db, &Some(get_current_user_id(&req))).await?;
    let result = workspace_service::save(db, &form_data, &admin.user_name).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

/// 注册工作台维护路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/workspace")
            // GET /workspace/list - 当前用户可见工作台（仅需登录）
            .route("/list", web::get().to(list))
            // POST /workspace/save - 管理员维护工作台
            .route(
                "/save",
                web::post()
                    .to(save)
                    .wrap(require_permission("system:workspace:save")),
            ),
    );
}
