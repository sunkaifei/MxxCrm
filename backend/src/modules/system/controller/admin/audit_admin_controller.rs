//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::{Error, Result};
use crate::core::kit::global::AppState;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::system::model::audit::AuditEventQuery;
use actix_web::{web, HttpResponse};
use sea_orm::{ConnectionTrait, DbBackend, Statement};

/// GET /audit/list - 审计事件分页查询（append-only 表，只读）
pub async fn audit_list(state: web::Data<AppState>, query: web::Query<AuditEventQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 100);

    // 动态构建 WHERE（全部参数化绑定）
    let mut conditions: Vec<String> = Vec::new();
    let mut values: Vec<sea_orm::Value> = Vec::new();

    if let Some(uid) = q.user_id {
        values.push(uid.into());
        conditions.push(format!("user_id = ${}", values.len()));
    }
    if let Some(module) = &q.module {
        values.push(module.clone().into());
        conditions.push(format!("module = ${}", values.len()));
    }
    if let Some(action) = &q.action {
        values.push(action.clone().into());
        conditions.push(format!("action = ${}", values.len()));
    }
    if let Some(s) = &q.start_date {
        if let Some(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok() {
            values.push(crate::modules::statistics::service::stats_range::date_param(Some(d)));
            conditions.push(format!("create_time >= ${}::timestamp", values.len()));
        }
    }
    if let Some(e) = &q.end_date {
        if let Some(d) = chrono::NaiveDate::parse_from_str(e, "%Y-%m-%d").ok() {
            values.push(crate::modules::statistics::service::stats_range::date_param(Some(d)));
            conditions.push(format!("create_time < (${}::date + INTERVAL '1 day')", values.len()));
        }
    }
    if let Some(kw) = &q.keyword {
        if !kw.is_empty() {
            values.push(format!("%{}%", kw).into());
            conditions.push(format!("summary LIKE ${}", values.len()));
        }
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    // 总数
    let count_sql = format!("SELECT COUNT(*)::int8 AS cnt FROM mxx_system_audit_event{}", where_clause);
    let total = db
        .query_one_raw(Statement::from_sql_and_values(DbBackend::Postgres, count_sql, values.clone()))
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .and_then(|r| r.try_get::<i64>("", "cnt").ok())
        .unwrap_or(0);

    // 分页数据
    let offset = ((page - 1) * page_size) as i64;
    values.push(page_size.into());
    let limit_idx = values.len();
    values.push(offset.into());
    let list_sql = format!(
        "SELECT id, user_id, user_name, module, action, target_type, target_id, summary, before_json, after_json, ip, create_time \
         FROM mxx_system_audit_event{} ORDER BY create_time DESC, id DESC LIMIT ${} OFFSET ${}",
        where_clause, limit_idx, limit_idx + 1
    );
    let rows = db
        .query_all_raw(Statement::from_sql_and_values(DbBackend::Postgres, list_sql, values))
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let list: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.try_get::<i64>("", "id").unwrap_or(0),
                "user_id": r.try_get::<i64>("", "user_id").unwrap_or(0),
                "user_name": r.try_get::<String>("", "user_name").unwrap_or_default(),
                "module": r.try_get::<String>("", "module").unwrap_or_default(),
                "action": r.try_get::<String>("", "action").unwrap_or_default(),
                "target_type": r.try_get::<String>("", "target_type").unwrap_or_default(),
                "target_id": r.try_get::<i64>("", "target_id").unwrap_or(0),
                "summary": r.try_get::<String>("", "summary").unwrap_or_default(),
                "before_json": r.try_get::<serde_json::Value>("", "before_json").ok(),
                "after_json": r.try_get::<serde_json::Value>("", "after_json").ok(),
                "ip": r.try_get::<String>("", "ip").unwrap_or_default(),
                "create_time": r.try_get::<chrono::NaiveDateTime>("", "create_time").map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_default(),
            })
        })
        .collect();

    Ok(HttpResponse::Ok().content_type(MPACK).body(
        MetaResp::success_with_page(list, "local", page, total as u32),
    ))
}

/// 注册路由（单点维护）
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/audit")
            // GET /audit/list - 审计事件查询
            .route(
                "/list",
                web::get()
                    .to(audit_list)
                    .wrap(require_permission("system:audit:list")),
            ),
    );
}
