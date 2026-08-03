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
use rust_decimal::prelude::ToPrimitive;
use serde::Deserialize;

use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::entity::common::InfoId;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::finance::service::bank_export_service;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileListQuery {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub bank_type: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn list(
    state: web::Data<AppState>,
    query: web::Query<FileListQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.0;
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).max(1);

    match bank_export_service::get_file_list(db, q.year, q.month, q.bank_type, page, page_size).await {
        Ok((list, total)) => {
            HttpResponse::Ok().content_type(MPACK)
                .body(MetaResp::success_with_page(list, "local", page as u32, total as u32))
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateDTO {
    pub year: i32,
    pub month: i32,
    pub bank_type: String,
}

/// 生成结果 DTO（返回给前端）
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateResult {
    pub file_content: String,
    pub file_name: String,
    pub total_count: i32,
    pub total_amount: f64,
    pub file_id: i64,
}

pub async fn generate(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<GenerateDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let creator_id = jwt_token.id.unwrap_or(0);
    let creator_name = jwt_token.username.as_deref().unwrap_or("财务人员");

    // 1. 生成文件内容
    let (file_content, file_name, total_count, total_amount) =
        match bank_export_service::generate_file(db, dto.year, dto.month, &dto.bank_type, creator_id, creator_name).await {
            Ok(data) => data,
            Err(e) => {
                return HttpResponse::Ok().content_type(MPACK)
                    .body(MetaResp::<String>::fail(400, &e, "local"));
            }
        };

    // 2. 保存文件记录到数据库
    let file_id = match bank_export_service::save_file_record(
        db,
        dto.year,
        dto.month,
        &dto.bank_type,
        &file_name,
        &file_content,
        total_count,
        total_amount,
        creator_id,
        creator_name,
    ).await {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Ok().content_type(MPACK)
                .body(MetaResp::<String>::fail(400, &e, "local"));
        }
    };

    let result = GenerateResult {
        file_content,
        file_name,
        total_count,
        total_amount: total_amount.to_f64().unwrap_or_default(),
        file_id,
    };

    HttpResponse::Ok().content_type(MPACK)
        .body(MetaResp::success(result, "local"))
}

pub async fn download(
    state: web::Data<AppState>,
    query: web::Query<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let item = query.0;

    if item.id.is_none() {
        return HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "文件ID不能为空", "local"));
    }

    match bank_export_service::get_file_content(db, item.id.unwrap()).await {
        Ok((file_content, file_name)) => {
            HttpResponse::Ok()
                .content_type("text/plain; charset=utf-8")
                .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", file_name)))
                .body(file_content)
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/finance/bank-export")
            .route("/list", web::get().to(list).wrap(require_permission("finance:bank-export:list")))
            .route("/generate", web::post().to(generate).wrap(require_permission("finance:bank-export:manage")))
            .route("/download", web::get().to(download).wrap(require_permission("finance:bank-export:list")))
            // P2-6: 生成 Excel 格式代发文件
            .route("/generate-excel", web::post().to(generate_excel).wrap(require_permission("finance:bank-export:manage"))),
    );
}

/// P2-6: 生成银行代发 Excel 文件
pub async fn generate_excel(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<GenerateDTO>,
) -> HttpResponse {
    let db = &state.db;
    let dto = form_data.0;

    let jwt_token: JWTToken = get_user(&req).unwrap_or_default();
    let creator_id = jwt_token.id.unwrap_or(0);
    let creator_name = jwt_token.username.as_deref().unwrap_or("财务人员");

    match bank_export_service::generate_excel_file(
        db, dto.year, dto.month, &dto.bank_type, creator_id, creator_name,
    ).await {
        Ok((xlsx_bytes, file_name, total_count, total_amount)) => {
            // 同时保存文件记录到数据库（file_path 存 base64 或留空，仅记录元数据）
            let _ = bank_export_service::save_file_record(
                db, dto.year, dto.month, &dto.bank_type,
                &file_name, "[xlsx-binary]", total_count, total_amount,
                creator_id, creator_name,
            ).await;
            HttpResponse::Ok()
                .content_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
                .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", file_name)))
                .body(xlsx_bytes)
        }
        Err(e) => HttpResponse::Ok().content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}
