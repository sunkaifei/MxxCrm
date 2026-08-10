//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 第三方接口统一配置管理控制器
//!
//! ## 路由表
//!
//! | 方法 | 路径                      | 权限码                        | handler  | 说明           |
//! |------|---------------------------|-------------------------------|----------|----------------|
//! | GET  | /system/integration/list  | system:integration:list       | get_list | 配置列表       |
//! | GET  | /system/integration/info  | system:integration:list       | get_info | 配置详情       |
//! | POST | /system/integration/save  | system:integration:save       | save     | 保存配置       |
//! | POST | /system/integration/toggle| system:integration:save       | toggle   | 启用/禁用      |
//! | POST | /system/integration/test  | system:integration:list       | test     | 测试单个连接   |
//! | POST | /system/integration/test-all | system:integration:list    | test_all | 测试全部连接   |
//! | DELETE| /system/integration/delete   | system:integration:save      | delete   | 删除配置       |
//!

use actix_web::{web, HttpResponse};
use crate::core::kit::global::AppState;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::system::model::integration_config::IntegrationConfigSaveRequest;
use crate::modules::system::service::integration_config_service;

/// 列表查询参数
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub category: Option<String>,
}

/// 详情查询参数
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoQuery {
    pub id: Option<i64>,
}

/// 启用/禁用请求
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggleRequest {
    #[serde(deserialize_with = "crate::utils::string_utils::deserialize_string_to_u64")]
    pub id: Option<i64>,
    pub enabled: Option<i32>,
}

/// 测试请求参数
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestQuery {
    #[serde(deserialize_with = "crate::utils::string_utils::deserialize_string_to_u64")]
    pub id: Option<i64>,
}

/// 删除请求路径参数
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteQuery {
    #[serde(deserialize_with = "crate::utils::string_utils::deserialize_string_to_u64")]
    pub id: Option<i64>,
}

/// GET /system/integration/list — 配置列表（按分类筛选）
pub async fn get_list(
    state: web::Data<AppState>,
    query: web::Query<ListQuery>,
) -> HttpResponse {
    let db = &state.db;
    match integration_config_service::get_list(db, query.category.clone()).await {
        Ok(list) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(list, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// GET /system/integration/info — 配置详情
pub async fn get_info(
    state: web::Data<AppState>,
    query: web::Query<InfoQuery>,
) -> HttpResponse {
    let db = &state.db;
    let id = match query.id {
        Some(id) => id,
        None => {
            return HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(400, "id 不能为空", "local"))
        }
    };
    match integration_config_service::get_info(db, id).await {
        Ok(Some(vo)) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(vo, "local")),
        Ok(None) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "配置不存在", "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// POST /system/integration/save — 保存配置
pub async fn save(
    state: web::Data<AppState>,
    item: web::Json<IntegrationConfigSaveRequest>,
) -> HttpResponse {
    let db = &state.db;
    if item.integration_code.is_none() {
        return HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "接口编码不能为空", "local"));
    }
    match integration_config_service::save(db, item.into_inner()).await {
        Ok(id) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// POST /system/integration/toggle — 启用/禁用
pub async fn toggle(
    state: web::Data<AppState>,
    item: web::Json<ToggleRequest>,
) -> HttpResponse {
    let db = &state.db;
    let id = match item.id {
        Some(id) => id,
        None => {
            return HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(400, "id 不能为空", "local"))
        }
    };
    let enabled = item.enabled.unwrap_or(0);
    match integration_config_service::toggle(db, id, enabled).await {
        Ok(result_id) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(result_id, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// POST /system/integration/test — 测试单个接口连接
pub async fn test(state: web::Data<AppState>, query: web::Query<TestQuery>) -> HttpResponse {
    let db = &state.db;
    let id = match query.id {
        Some(id) => id,
        None => {
            return HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(400, "id 不能为空", "local"))
        }
    };
    match integration_config_service::test_connection(db, id).await {
        Ok((success, message)) => HttpResponse::Ok().content_type(MPACK).body(
            MetaResp::success(
                serde_json::json!({"success": success, "message": message}),
                "local",
            ),
        ),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// POST /system/integration/test-all — 测试所有已启用接口
pub async fn test_all(state: web::Data<AppState>) -> HttpResponse {
    let db = &state.db;
    match integration_config_service::test_all(db).await {
        Ok(results) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(results, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// DELETE /system/integration/delete — 删除配置（软删除）
pub async fn delete(
    state: web::Data<AppState>,
    query: web::Query<DeleteQuery>,
) -> HttpResponse {
    let db = &state.db;
    let id = match query.id {
        Some(id) => id,
        None => {
            return HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(400, "id 不能为空", "local"))
        }
    };
    match integration_config_service::delete_by_id(db, id).await {
        Ok(result_id) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(result_id, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 注册第三方接口配置管理模块所有路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/system/integration")
            .route(
                "/list",
                web::get()
                    .to(get_list)
                    .wrap(require_permission("system:integration:list")),
            )
            .route(
                "/info",
                web::get()
                    .to(get_info)
                    .wrap(require_permission("system:integration:list")),
            )
            .route(
                "/save",
                web::post()
                    .to(save)
                    .wrap(require_permission("system:integration:save")),
            )
            .route(
                "/toggle",
                web::post()
                    .to(toggle)
                    .wrap(require_permission("system:integration:save")),
            )
            .route(
                "/test",
                web::post()
                    .to(test)
                    .wrap(require_permission("system:integration:list")),
            )
            .route(
                "/test-all",
                web::post()
                    .to(test_all)
                    .wrap(require_permission("system:integration:list")),
            )
            .route(
                "/delete",
                web::delete()
                    .to(delete)
                    .wrap(require_permission("system:integration:save")),
            ),
    );
}
