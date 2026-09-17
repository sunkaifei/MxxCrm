//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 模块表单/详情布局管理（自定义模块进阶一期）
//! - 管理侧 4 接口：get/save/reset（权限码 system:form-layout:*）
//! - 无布局返回 data=null，前端回落现用默认渲染（"现用设计不变"）

use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use actix_web::{web, HttpRequest, HttpResponse};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::core::errors::error::Error;
use crate::modules::system::model::form_layout::FormLayoutSaveRequest;
use crate::modules::system::service::{admin_service, audit_service, field_def_service, form_layout_service};

async fn current_operator(db: &DatabaseConnection, req: &HttpRequest) -> Option<String> {
    admin_service::get_by_detail(db, &Some(get_current_user_id(req)))
        .await
        .ok()
        .and_then(|admin| admin.user_name)
}

#[derive(Deserialize)]
pub struct FormLayoutQuery {
    pub module: String,
    /// 1=表单 2=详情页
    pub layout_type: Option<i32>,
    /// 适用角色 key：缺省按当前用户解析（get）/ 默认布局（reset）
    pub role_key: Option<String>,
}

// 取模块布局（无则 data=null）
pub async fn layout_get(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<FormLayoutQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    let layout_type = q.layout_type.unwrap_or(1);
    // D7：按当前用户角色解析布局（角色专属优先，默认布局兜底）
    let (_, role_keys) = field_def_service::load_user_role(db, get_current_user_id(&req))
        .await
        .unwrap_or((false, Vec::new()));
    match form_layout_service::get_for_user(db, &q.module, layout_type, &role_keys).await {
        Ok(vo) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(vo, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<()>::fail(400, &e, "local"))),
    }
}

// 保存布局（upsert + version 乐观锁）
pub async fn layout_save(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<FormLayoutSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let form = form_data.0;
    let operator = current_operator(db, &req).await;

    let result = form_layout_service::save(db, &form, operator).await.map_err(Error::from);
    let saved_id = result.as_ref().copied().unwrap_or_default();
    if result.is_ok() {
        audit_service::record(
            db,
            &req,
            "form_layout",
            "save",
            "form_layout",
            saved_id,
            format!("保存{}布局（{}）", form.module, if form.layout_type == 2 { "详情页" } else { "表单" }),
            None,
            audit_service::snap(vec![
                ("module", serde_json::json!(form.module)),
                ("layout_type", serde_json::json!(form.layout_type)),
            ]),
        ).await;
    }
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

// 恢复默认布局（逻辑删布局行）
pub async fn layout_reset(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<FormLayoutQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    let layout_type = q.layout_type.unwrap_or(1);
    let operator = current_operator(db, &req).await;

    let result = form_layout_service::reset(db, &q.module, layout_type, q.role_key, operator)
        .await
        .map_err(Error::from);
    let reset_ok = result.is_ok();
    if reset_ok {
        audit_service::record(
            db,
            &req,
            "form_layout",
            "reset",
            "form_layout",
            0,
            format!("恢复{}默认布局（{}）", q.module, if layout_type == 2 { "详情页" } else { "表单" }),
            None,
            audit_service::snap(vec![
                ("module", serde_json::json!(q.module)),
                ("layout_type", serde_json::json!(layout_type)),
            ]),
        ).await;
    }
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result.map(|_| 0_i64))))
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/form-layout")
            .route("/get", web::get().to(layout_get).wrap(require_permission("system:form-layout:list")))
            .route("/save", web::post().to(layout_save).wrap(require_permission("system:form-layout:manage")))
            .route("/reset", web::post().to(layout_reset).wrap(require_permission("system:form-layout:manage"))),
    );
}
