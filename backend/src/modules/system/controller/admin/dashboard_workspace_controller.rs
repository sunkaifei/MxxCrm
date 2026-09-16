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
use serde::Deserialize;

use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::system::service::dashboard_workspace_service;

/// summary 查询参数（方案 v2.1 §7.2.1）
///
/// `scope` 可选，取值与项目既有口径一致：`my`（默认）/ `subordinate` / `all`。
/// **不传时行为与改造前完全一致**（仅本人），保证既有验收不破。
#[derive(Debug, Deserialize)]
pub struct SummaryQuery {
    #[serde(default)]
    pub scope: Option<String>,
    /// 统计时间范围：month（默认）/ quarter / year
    #[serde(default)]
    pub time_range: Option<String>,
}

/// 工作台聚合摘要（三期 #4：一次返回各卡摘要，防首屏请求线性膨胀，仅需登录）
///
/// v2.1：新增可选 `scope` 参数，接通既有数据权限统一层。
/// 越权视角**静默降级**（返回 `scopeApplied:false`），不返回 403。
/// 卡片自身配置（card_config.timeRange）优先于全局 time_range 参数。
pub async fn summary(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<SummaryQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    let scope = query.scope.as_deref();
    let time_range = query.time_range.as_deref();
    dashboard_workspace_service::get_workspace_summary(db, user_id, scope, time_range).await.map(|vo| {
        HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(vo, "local"))
    })
}

/// 注册工作台聚合路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/dashboard/workspace")
            // GET /dashboard/workspace/summary - 工作台各卡聚合摘要（仅需登录）
            //   可选参数 scope=my|subordinate|all
            .route("/summary", web::get().to(summary)),
    );
}
