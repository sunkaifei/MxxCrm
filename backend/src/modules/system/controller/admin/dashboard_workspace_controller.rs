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
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::system::service::dashboard_workspace_service;

/// 工作台聚合摘要（三期 #4：一次返回各卡摘要，防首屏请求线性膨胀，仅需登录）
pub async fn summary(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    dashboard_workspace_service::get_workspace_summary(db, user_id).await.map(|vo| {
        HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(vo, "local"))
    })
}

/// 注册工作台聚合路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/dashboard/workspace")
            // GET /dashboard/workspace/summary - 工作台各卡聚合摘要（仅需登录）
            .route("/summary", web::get().to(summary)),
    );
}
