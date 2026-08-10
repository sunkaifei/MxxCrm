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
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::entity::common::BathDeleteIdRequest;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::inventory::model::warehouse_area::{WarehouseAreaListQuery, WarehouseAreaSaveRequest, WarehouseAreaUpdateRequest, WarehouseAreaVO};
use crate::modules::inventory::service::warehouse_area_service;
use crate::modules::inventory::entity::warehouse as warehouse_entity;
use actix_web::{web, HttpRequest, HttpResponse};
use crate::core::web::permission_guard::require_permission;
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};
use std::collections::HashMap;

pub async fn warehouse_area_save(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<WarehouseAreaSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let form_data = form_data.0;

    let result = warehouse_area_service::insert(db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn warehouse_area_update(state: web::Data<AppState>, req: HttpRequest, form_data: web::Json<WarehouseAreaUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let form_data = form_data.0;

    let result = warehouse_area_service::update(db, &form_data, jwt_token.id.unwrap_or_default()).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn batch_delete_warehouse_area(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let ids = item.0.parse_ids();
    if ids.is_empty() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "请选择要删除的记录", "local")));
    }
    let result = warehouse_area_service::batch_delete(db, &ids).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn warehouse_area_info(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let id = req.query_string().split("&").find(|s| s.starts_with("id=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())).unwrap_or(0);
    if id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "ID无效", "local")));
    }

    match warehouse_area_service::get_detail(db, id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn warehouse_area_list(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let query_str = req.query_string();

    fn q<'a>(qs: &'a str, key: &str) -> Option<&'a str> {
        qs.split('&').find(|s| s.starts_with(&format!("{}=", key)))
            .and_then(|s| s.split('=').nth(1))
    }

    let query = WarehouseAreaListQuery {
        page_num: q(query_str, "page").and_then(|s| s.parse().ok()),
        page_size: q(query_str, "pageSize").and_then(|s| s.parse().ok()),
        warehouse_id: q(query_str, "warehouseId").and_then(|s| s.parse().ok()),
        area_name: q(query_str, "areaName").map(|s| s.to_string()),
        area_type: q(query_str, "areaType").map(|s| s.to_string()),
        status: q(query_str, "status").and_then(|s| s.parse().ok()),
    };

    match warehouse_area_service::get_list(db, &query).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

pub async fn warehouse_area_list_by_warehouse(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let warehouse_id = req.query_string().split("&").find(|s| s.starts_with("warehouseId=")).and_then(|s| s.split("=").nth(1).and_then(|s| s.parse::<i64>().ok())).unwrap_or(0);
    if warehouse_id <= 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "warehouseId无效", "local")));
    }

    match warehouse_area_service::list_by_warehouse(db, warehouse_id).await {
        Ok(data) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 按仓库分组的库位树形结构数据
pub async fn warehouse_area_tree(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;

    // 查询所有启用的仓库
    let warehouses = warehouse_entity::Entity::find()
        .filter(warehouse_entity::Column::Deleted.eq(0))
        .all(db)
        .await?;

    // 查询所有未删除的库位
    let areas = warehouse_area_service::list_all(db).await?;

    // 按 warehouse_id 分组
    let mut area_map: HashMap<i64, Vec<WarehouseAreaVO>> = HashMap::new();
    for area in areas {
        let wid = area.warehouse_id.unwrap_or(0);
        area_map.entry(wid).or_default().push(area);
    }

    // 构建树形结构
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct WarehouseTreeNode {
        id: i64,
        warehouse_name: Option<String>,
        code: Option<String>,
        children: Vec<WarehouseAreaVO>,
    }

    let tree: Vec<WarehouseTreeNode> = warehouses
        .into_iter()
        .map(|w| WarehouseTreeNode {
            id: w.id,
            warehouse_name: w.name.clone(),
            code: w.code.clone(),
            children: area_map.remove(&w.id).unwrap_or_default(),
        })
        .collect();

    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(tree, "local")))
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/warehouse_area")
            .route("/save", web::post().to(warehouse_area_save).wrap(require_permission("product:warehouse:save")))
            .route("/update", web::put().to(warehouse_area_update).wrap(require_permission("product:warehouse:save")))
            .route("/batch_delete", web::delete().to(batch_delete_warehouse_area).wrap(require_permission("product:warehouse:delete")))
            .route("/info", web::get().to(warehouse_area_info).wrap(require_permission("product:warehouse:list")))
            // list 类查询为基础参考数据，所有登录用户均可访问
            .route("/list", web::get().to(warehouse_area_list))
            .route("/list_by_warehouse", web::get().to(warehouse_area_list_by_warehouse)),
    );
    cfg.service(
        web::scope("/area")
            .route("/save", web::post().to(warehouse_area_save).wrap(require_permission("product:warehouse:save")))
            .route("/update", web::put().to(warehouse_area_update).wrap(require_permission("product:warehouse:save")))
            .route("/batch_delete", web::delete().to(batch_delete_warehouse_area).wrap(require_permission("product:warehouse:delete")))
            .route("/info", web::get().to(warehouse_area_info).wrap(require_permission("product:warehouse:list")))
            .route("/list", web::get().to(warehouse_area_list))
            .route("/list_by_warehouse", web::get().to(warehouse_area_list_by_warehouse))
            .route("/tree", web::get().to(warehouse_area_tree)),
    );
}
