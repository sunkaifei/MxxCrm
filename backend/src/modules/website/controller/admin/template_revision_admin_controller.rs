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
use actix_web::{web, HttpResponse};
use crate::core::kit::global::AppState;
use crate::core::web::entity::common::InfoId;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::website::service::{template_revision_service};

/// 获取某模板数据的版本历史列表
pub async fn get_by_template_data_id(state: web::Data<AppState>, template_data_id: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let template_data_id = Some(template_data_id.into_inner());
    let result = template_revision_service::get_revisions(db, &template_data_id).await?;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(result, "local")))
}

/// 获取版本详情
pub async fn get_by_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    let result = template_revision_service::get_by_detail(db, &item.id).await?;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(result, "local")))
}

// ==================== 路由注册（单点维护）====================

/// 注册模板版本历史模块所有路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/template/revision")
            // GET /template/revision/list/{template_data_id} - 获取某模板的版本历史
            .route(
                "/list/{template_data_id}",
                web::get()
                    .to(get_by_template_data_id)
                    .wrap(require_permission("template:revision:list")),
            )
            // GET /template/revision/detail/{id} - 获取版本详情
            .route(
                "/detail/{id}",
                web::get()
                    .to(get_by_detail)
                    .wrap(require_permission("template:revision:view")),
            ),
    );
}
