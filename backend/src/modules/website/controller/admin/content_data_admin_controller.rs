//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 内容模型 - 动态表内容 CRUD 控制器（T-P0.4）
//!
//! 面向「通用内容页」：按模型编码 `model_code` 对 `mxx_model_{code}` 动态表
//! 做内容的新增/查询/修改/删除，是所有内容模型共用的数据出入口。
//!
//! 路由（上游 scope `/api/system`）：
//! - GET    /content/data/{model_code}/list          分页列表
//! - GET    /content/data/{model_code}/detail/{id}   详情
//! - POST   /content/data/{model_code}/add           新增（原始 JSON 对象体）
//! - PUT    /content/data/{model_code}/update/{id}   修改（原始 JSON 对象体）
//! - DELETE /content/data/{model_code}/batch_delete  批量软删除

use crate::core::errors::error::Result;
use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use crate::core::kit::global::AppState;
use crate::core::web::entity::common::BathDeleteIdRequest;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, ResultPage, MPACK};
use crate::modules::website::service::dynamic_table_service::DynamicTableService;

/// 动态表内容列表查询参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub category_id: Option<i64>,
    pub keywords: Option<String>,
}

/// 分页列表
pub async fn get_list(
    state: web::Data<AppState>,
    path: web::Path<String>,
    query: web::Query<DataListQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let model_code = path.into_inner();
    DynamicTableService::ensure_table(db, &model_code).await?;
    let q = query.into_inner();
    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(10);

    let (list, total) = DynamicTableService::paginate(
        db,
        &model_code,
        page,
        page_size,
        q.category_id,
        q.keywords.as_deref(),
    )
    .await?;

    let result = ResultPage::new(list, total as i64, page as i64, page_size as i64);
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(result, "local")))
}

/// 详情
pub async fn get_detail(
    state: web::Data<AppState>,
    path: web::Path<(String, i64)>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let (model_code, id) = path.into_inner();
    DynamicTableService::ensure_table(db, &model_code).await?;
    match DynamicTableService::find_by_id(db, &model_code, id).await? {
        Some(v) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(v, "local"))),
        None => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(404, "记录不存在", "local"))),
    }
}

/// 新增
pub async fn add(
    state: web::Data<AppState>,
    _req: HttpRequest,
    path: web::Path<String>,
    item: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let model_code = path.into_inner();
    DynamicTableService::ensure_table(db, &model_code).await?;
    let payload = item.into_inner();
    let id = DynamicTableService::insert(db, &model_code, &payload).await?;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(id, "local")))
}

/// 修改
pub async fn update_by_id(
    state: web::Data<AppState>,
    _req: HttpRequest,
    path: web::Path<(String, i64)>,
    item: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let (model_code, id) = path.into_inner();
    DynamicTableService::ensure_table(db, &model_code).await?;
    let payload = item.into_inner();
    let n = DynamicTableService::update(db, &model_code, id, &payload).await?;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(Ok(n))))
}

/// 批量软删除
pub async fn batch_delete(
    state: web::Data<AppState>,
    path: web::Path<String>,
    item: web::Json<BathDeleteIdRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let model_code = path.into_inner();
    DynamicTableService::ensure_table(db, &model_code).await?;
    let ids_vec = match item.ids.clone() {
        Some(v) if !v.is_empty() => v,
        _ => {
            return Ok(HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }
    };
    let ids: Vec<i64> = ids_vec
        .iter()
        .filter_map(|s| s.as_ref().and_then(|x| x.parse::<i64>().ok()))
        .collect();

    let mut affected = 0i64;
    for id in ids {
        affected += DynamicTableService::soft_delete(db, &model_code, id).await.unwrap_or(0);
    }
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(Ok(affected))))
}

// ==================== 路由注册（单点维护）====================

/// 注册内容模型「动态表内容」模块所有路由
///
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(content_data_admin_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/content/data")
            .route(
                "/{model_code}/list",
                web::get().to(get_list).wrap(require_permission("content:data:list")),
            )
            .route(
                "/{model_code}/detail/{id}",
                web::get().to(get_detail).wrap(require_permission("content:data:view")),
            )
            .route(
                "/{model_code}/add",
                web::post().to(add).wrap(require_permission("content:data:add")),
            )
            .route(
                "/{model_code}/update/{id}",
                web::put().to(update_by_id).wrap(require_permission("content:data:update")),
            )
            .route(
                "/{model_code}/batch_delete",
                web::delete().to(batch_delete).wrap(require_permission("content:data:delete")),
            ),
    );
}
