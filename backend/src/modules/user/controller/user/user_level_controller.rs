//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{get, post, put, delete, web, HttpResponse, Responder};
use sea_orm::{DbConn};
use chrono::Local;
use crate::core::kit::global::AppState;
use crate::modules::user::entity::user_level;
use crate::modules::user::model::user_level::UserLevelModel;
use crate::core::web::response::MetaResp;

#[derive(Debug, serde::Deserialize)]
pub struct UserLevelCreateRequest {
    pub level_code: String,
    pub level_name: String,
    pub level_desc: Option<String>,
    pub turtle_quota: i32,
    pub qrcode_quota: i32,
    pub tag_quota: i32,
    pub permissions: Option<String>,
    pub sort_order: i32,
    pub status: i32,
}

#[derive(Debug, serde::Deserialize)]
pub struct UserLevelUpdateRequest {
    pub id: i64,
    pub level_code: String,
    pub level_name: String,
    pub level_desc: Option<String>,
    pub turtle_quota: i32,
    pub qrcode_quota: i32,
    pub tag_quota: i32,
    pub permissions: Option<String>,
    pub sort_order: i32,
    pub status: i32,
}

/// 获取所有会员等级列表
#[get("/admin/member/level/list")]
pub async fn list_all_levels(data: web::Data<AppState>) -> impl Responder {
    let db = &data.db;

    match UserLevelModel::find_all_enabled(db).await {
        Ok(levels) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(levels, "local")),
        Err(e) => HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<serde_json::Value>::fail(400, &format!("查询会员等级失败: {}", e), "local")),
    }
}

/// 获取所有会员等级（包含禁用的）
#[get("/admin/member/level/all")]
pub async fn list_all(data: web::Data<AppState>) -> impl Responder {
    let db = &data.db;

    match UserLevelModel::find_all(db).await {
        Ok(levels) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(levels, "local")),
        Err(e) => HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<serde_json::Value>::fail(400, &format!("查询会员等级失败: {}", e), "local")),
    }
}

/// 根据ID获取会员等级详情
#[get("/admin/member/level/{id}")]
pub async fn get_by_id(path: web::Path<i64>, data: web::Data<AppState>) -> impl Responder {
    let db = &data.db;
    let id = path.into_inner();

    match UserLevelModel::find_by_id(db, id).await {
        Ok(Some(level)) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(level, "local")),
        Ok(None) => HttpResponse::NotFound().content_type("application/msgpack").body(MetaResp::<serde_json::Value>::fail(400, "会员等级不存在", "local")),
        Err(e) => HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<serde_json::Value>::fail(400, &format!("查询会员等级失败: {}", e), "local")),
    }
}

/// 根据等级标识获取会员等级
#[get("/admin/member/level/code/{code}")]
pub async fn get_by_code(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let db = &data.db;
    let code = path.into_inner();

    match UserLevelModel::find_by_code(db, &code).await {
        Ok(Some(level)) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(level, "local")),
        Ok(None) => HttpResponse::NotFound().content_type("application/msgpack").body(MetaResp::<serde_json::Value>::fail(400, "会员等级不存在", "local")),
        Err(e) => HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<serde_json::Value>::fail(400, &format!("查询会员等级失败: {}", e), "local")),
    }
}

/// 创建会员等级
#[post("/admin/member/level")]
pub async fn create_level(req: web::Json<UserLevelCreateRequest>, data: web::Data<AppState>) -> impl Responder {
    let db = &data.db;

    let model = user_level::Model {
        id: 0,
        level_code: req.level_code.clone(),
        level_name: req.level_name.clone(),
        level_desc: req.level_desc.clone(),
        turtle_quota: req.turtle_quota,
        qrcode_quota: req.qrcode_quota,
        tag_quota: req.tag_quota,
        permissions: req.permissions.as_ref().map(|p| serde_json::json!(p)),
        sort_order: req.sort_order,
        status: req.status,
        create_time: Some(Local::now().naive_local()),
        update_time: Some(Local::now().naive_local()),
    };

    match UserLevelModel::create(db, &model).await {
        Ok(id) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<serde_json::Value>::fail(400, &format!("创建会员等级失败: {}", e), "local")),
    }
}

/// 更新会员等级
#[put("/admin/member/level")]
pub async fn update_level(req: web::Json<UserLevelUpdateRequest>, data: web::Data<AppState>) -> impl Responder {
    let db = &data.db;

    let model = user_level::Model {
        id: req.id,
        level_code: req.level_code.clone(),
        level_name: req.level_name.clone(),
        level_desc: req.level_desc.clone(),
        turtle_quota: req.turtle_quota,
        qrcode_quota: req.qrcode_quota,
        tag_quota: req.tag_quota,
        permissions: req.permissions.as_ref().map(|p| serde_json::json!(p)),
        sort_order: req.sort_order,
        status: req.status,
        create_time: None,
        update_time: Some(Local::now().naive_local()),
    };

    match UserLevelModel::update(db, req.id, &model).await {
        Ok(_) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<serde_json::Value>::fail(200, "更新成功", "local")),
        Err(e) => HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<serde_json::Value>::fail(400, &format!("更新会员等级失败: {}", e), "local")),
    }
}

/// 删除会员等级（软删除）
#[delete("/admin/member/level/{id}")]
pub async fn delete_level(path: web::Path<i64>, data: web::Data<AppState>) -> impl Responder {
    let db = &data.db;
    let id = path.into_inner();

    match UserLevelModel::delete(db, id).await {
        Ok(_) => HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<serde_json::Value>::fail(200, "删除成功", "local")),
        Err(e) => HttpResponse::InternalServerError().content_type("application/msgpack").body(MetaResp::<serde_json::Value>::fail(400, &format!("删除会员等级失败: {}", e), "local")),
    }
}