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
use crate::core::web::base_controller::get_current_user_id;
use crate::core::web::entity::common::BathDeleteIdRequest;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, ResultPage, MPACK};
use crate::modules::website::service::dynamic_table_service::{DateRange, DynamicTableService, FieldFilter};

/// 动态表内容列表查询参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub category_id: Option<i64>,
    pub keywords: Option<String>,
    // 创建时间范围（YYYY-MM-DD 或完整时间戳）
    pub create_time_from: Option<String>,
    pub create_time_to: Option<String>,
}

/// 分页列表
///
/// 搜索卡片：除关键词/创建时间外，支持模型字段设置里勾选「可搜索」的自定义字段——
/// - 文本类参数前缀 `sf_{field}`：文本模糊、数字/用户/下拉精确
/// - 日期字段参数前缀 `df_{field}`：值 [起, 止]（_from/_to 两段），时间段过滤
/// 字段白名单以模型设置的 isSearchable 字段为准，防止越权列名注入。
pub async fn get_list(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
    query: web::Query<DataListQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let model_code = path.into_inner();
    DynamicTableService::ensure_table(db, &model_code).await?;
    let q = query.into_inner();
    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(10);

    // 时间段筛选：创建时间 + 日期类型自定义字段（列名经标识符校验，后端再做实存列校验）
    let mut date_ranges: Vec<DateRange> = Vec::new();
    if q.create_time_from.is_some() || q.create_time_to.is_some() {
        date_ranges.push(DateRange::new(
            "create_time",
            q.create_time_from.clone(),
            q.create_time_to.clone(),
        ));
    }

    // 解析 sf_/df_ 动态搜索参数：白名单 = 该模型 is_searchable=1 的字段
    let mut field_filters: Vec<FieldFilter> = Vec::new();
    let qs = req.query_string();
    log::info!("[content_search] qs={}", qs);
    if qs.contains("sf_") || qs.contains("df_") {
        let searchable: Vec<(String, i32)> =
            search_field_white_list(db, &model_code).await?;
        log::info!("[content_search] model={} searchable={:?}", model_code, searchable);
        let pairs = parse_query_pairs(qs);
        if !pairs.is_empty() {
            for (key, value) in pairs {                if value.is_empty() {
                    continue;
                }
                if let Some(name) = key.strip_prefix("sf_") {
                    if let Some((_, ft)) = searchable.iter().find(|(n, _)| n == name) {
                        field_filters.push(FieldFilter {
                            column: name.to_string(),
                            field_type: *ft,
                            value,
                        });
                    }
                } else if let Some(rest) = key.strip_prefix("df_") {
                    if let Some(name) = rest.strip_suffix("_from") {
                        if searchable.iter().any(|(n, _)| n == name) {
                            date_ranges.push(DateRange::new(name, Some(value), None));
                        }
                    } else if let Some(name) = rest.strip_suffix("_to") {
                        if searchable.iter().any(|(n, _)| n == name) {
                            date_ranges.push(DateRange::new(name, None, Some(value)));
                        }
                    }
                }
            }
        }
    }

    log::info!(
        "[content_search] date_ranges={} field_filters={}",
        date_ranges.len(),
        field_filters.len()
    );
    let (list, total) = DynamicTableService::paginate(
        db,
        &model_code,
        page,
        page_size,
        q.category_id,
        q.keywords.as_deref(),
        &date_ranges,
        &field_filters,
    )
    .await?;

    let result = ResultPage::new(list, total as i64, page as i64, page_size as i64);
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(result, "local")))
}

/// 最小 query string 解析：a=1&b=2 → [(a,1), (b,2)]，含 %XX 与 + 的解码。
/// 仅用于搜索参数提取（数据校验靠字段白名单 + 标识符规则）。
fn parse_query_pairs(qs: &str) -> Vec<(String, String)> {
    fn percent_decode(s: &str) -> String {
        let bytes = s.as_bytes();
        let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'%' if i + 2 < bytes.len() => {
                    let hex = &s[i + 1..i + 3];
                    match u8::from_str_radix(hex, 16) {
                        Ok(b) => {
                            out.push(b);
                            i += 3;
                        }
                        Err(_) => {
                            out.push(b'%');
                            i += 1;
                        }
                    }
                }
                b'+' => {
                    out.push(b' ');
                    i += 1;
                }
                b => {
                    out.push(b);
                    i += 1;
                }
            }
        }
        String::from_utf8_lossy(&out).into_owned()
    }

    let mut out = Vec::new();
    for pair in qs.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (k, v) = match pair.split_once('=') {
            Some((k, v)) => (k, v),
            None => (pair, ""),
        };
        out.push((percent_decode(k), percent_decode(v)));
    }
    out
}

/// 可搜索字段白名单：(field_name, field_type)，仅 is_searchable=1 且启用的字段
async fn search_field_white_list(
    db: &crate::core::kit::db::DbConn,
    model_code: &str,
) -> Result<Vec<(String, i32)>> {    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    use crate::modules::website::entity::{content_model, content_model_field};
    let Some(model) = content_model::Entity::find()
        .filter(content_model::Column::ModelCode.eq(model_code))
        .filter(content_model::Column::Deleted.eq(0))
        .one(db)
        .await?
    else {
        return Ok(Vec::new());
    };
    let rows = content_model_field::Entity::find()
        .filter(content_model_field::Column::ModelId.eq(model.id))
        .filter(content_model_field::Column::Deleted.eq(0))
        .filter(
            sea_orm::Condition::any()
                .add(content_model_field::Column::Status.eq(1))
                .add(content_model_field::Column::Status.is_null()),
        )
        .filter(content_model_field::Column::IsSearchable.eq(1))
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .filter_map(|f| f.field_name.zip(f.field_type))
        .collect())
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
    req: HttpRequest,
    path: web::Path<String>,
    item: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let model_code = path.into_inner();
    DynamicTableService::ensure_table(db, &model_code).await?;
    let mut payload = item.into_inner();
    // 创建者（关联用户 id）由 insert 服务端按 creator_id 注入固定列 create_user_id
    let id = DynamicTableService::insert(db, &model_code, &payload, Some(get_current_user_id(&req))).await?;
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
