//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 卡密池控制器
//!
//! ## 路由表
//!
//! | 方法   | 路径                       | 权限码                  | handler | 说明            |
//! |--------|---------------------------|------------------------|---------|----------------|
//! | GET    | /sale/card-pool/list      | sale:card-pool:list    | list    | 卡密列表        |
//! | POST   | /sale/card-pool/save      | sale:card-pool:save    | save    | 新增卡密        |
//! | POST   | /sale/card-pool/import    | sale:card-pool:save    | import  | 批量导入        |
//! | DELETE | /sale/card-pool/delete    | sale:card-pool:delete  | delete  | 删除/作废卡密   |
//! | GET    | /sale/card-pool/count     | sale:card-pool:list    | count   | 查询可用数量    |
//!

use actix_web::{web, HttpResponse};
use crate::core::kit::global::AppState;
use crate::core::web::entity::common::InfoId;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::sale::model::card_pool::{
    CardPoolImportRequest, CardPoolListQuery, CardPoolSaveRequest,
};
use crate::modules::sale::service::card_pool_service;

/// 卡密列表（脱敏）
pub async fn list(
    state: web::Data<AppState>,
    query: web::Query<CardPoolListQuery>,
) -> HttpResponse {
    let db = &state.db;
    let query = query.0;
    match card_pool_service::get_list(db, &query).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok().content_type(MPACK)
                .body(MetaResp::success_with_page(page_data, "local", page, total))
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 新增单张卡密
pub async fn save(
    state: web::Data<AppState>,
    form_data: web::Json<CardPoolSaveRequest>,
) -> HttpResponse {
    let db = &state.db;
    match card_pool_service::create(db, form_data.0).await {
        Ok(id) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 批量导入卡密
pub async fn import(
    state: web::Data<AppState>,
    form_data: web::Json<CardPoolImportRequest>,
) -> HttpResponse {
    let db = &state.db;
    match card_pool_service::import(db, form_data.0).await {
        Ok(count) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(count, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 删除（软删除 + 作废）
pub async fn delete(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;
    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "ID不能为空", "local"));
    }
    match card_pool_service::delete(db, item.id.unwrap()).await {
        Ok(rows) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(rows, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 查询商品可用卡密数量
pub async fn count(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.0;
    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "商品ID不能为空", "local"));
    }
    match card_pool_service::count_unsold(db, item.id.unwrap()).await {
        Ok(count) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::success(count, "local")),
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// 注册卡密池模块所有路由
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sale/card-pool")
            .route("/list", web::get().to(list).wrap(require_permission("sale:card-pool:list")))
            .route("/save", web::post().to(save).wrap(require_permission("sale:card-pool:save")))
            .route("/import", web::post().to(import).wrap(require_permission("sale:card-pool:save")))
            .route("/delete", web::delete().to(delete).wrap(require_permission("sale:card-pool:delete")))
            .route("/count", web::get().to(count).wrap(require_permission("sale:card-pool:list"))),
    );
}
