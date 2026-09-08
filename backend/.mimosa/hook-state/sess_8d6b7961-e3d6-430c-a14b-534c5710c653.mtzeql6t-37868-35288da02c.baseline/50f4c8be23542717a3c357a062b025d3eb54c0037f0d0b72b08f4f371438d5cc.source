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
use crate::core::web::base_controller::{get_current_user, get_current_user_id};
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK, ResultPage};
use crate::modules::system::entity::pdf_record::{
    self as pdf_record_entity, Entity as PdfRecordEntity,
};
use crate::modules::system::entity::pdf_download_log::{
    self as pdf_download_log, Entity as PdfDownloadLogEntity,
};
use crate::modules::system::model::pdf::{
    PdfGenerateRequest, PdfRecordListQuery, PdfRecordModel, PdfRecordVO, PdfTemplateListQuery,
    PdfTemplateSaveRequest, PdfTemplateUpdateRequest,
};
use crate::modules::system::service::{
    pdf_compiler_service, pdf_generator_service, pdf_template_service,
};
use actix_web::{web, HttpRequest, HttpResponse};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};

// ============================ 请求/响应结构体 ============================

/// PDF 模板选项查询参数
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfDocTypeQuery {
    pub doc_type: Option<String>,
}

/// PDF 预览查询参数
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfPreviewQuery {
    pub doc_type: Option<String>,
    pub doc_id: Option<String>,
    pub template_id: Option<String>,
}

/// PDF 全局记录列表查询参数
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfRecordAllQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub doc_type: Option<String>,
    pub doc_no: Option<String>,
    pub trigger_type: Option<String>,
}

/// PDF 下载日志查询参数
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfDownloadLogQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub doc_type: Option<String>,
    pub doc_no: Option<String>,
}

/// PDF 下载日志 VO
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfDownloadLogVO {
    pub id: i64,
    pub record_id: i64,
    pub doc_type: Option<String>,
    pub doc_id: Option<i64>,
    pub doc_no: Option<String>,
    pub file_name: Option<String>,
    pub operator_id: Option<i64>,
    pub operator_name: Option<String>,
    pub ip_address: Option<String>,
    pub create_time: Option<String>,
}

impl From<pdf_download_log::Model> for PdfDownloadLogVO {
    fn from(m: pdf_download_log::Model) -> Self {
        Self {
            id: m.id,
            record_id: m.record_id,
            doc_type: m.doc_type,
            doc_id: m.doc_id,
            doc_no: m.doc_no,
            file_name: m.file_name,
            operator_id: m.operator_id,
            operator_name: m.operator_name,
            ip_address: m.ip_address,
            create_time: m.create_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// PDF 生成结果 VO
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfGenerateResultVO {
    pub record_id: i64,
    pub file_url: String,
    pub file_path: String,
    pub file_size: i64,
}

/// 文件名 URL 编码（用于 Content-Disposition，处理非 ASCII 字符）
fn url_encode_filename(filename: &str) -> String {
    let mut result = String::new();
    for byte in filename.bytes() {
        if byte.is_ascii_alphanumeric() || byte == b'.' || byte == b'_' || byte == b'-' {
            result.push(byte as char);
        } else {
            result.push_str(&format!("%{:02X}", byte));
        }
    }
    result
}

// ============================ PDF 模板管理 ============================

pub async fn pdf_template_list(
    state: web::Data<AppState>,
    query: web::Query<PdfTemplateListQuery>,
) -> HttpResponse {
    let db = &state.db;
    match pdf_template_service::list(&db, query.into_inner()).await {
        Ok(page_data) => {
            let page = page_data.current_page as u32;
            let total = page_data.total as u32;
            HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::success_with_page(page_data, "local", page, total))
        }
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn pdf_template_info(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let item = item.into_inner();
    if item.id.is_none() {
        return HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "PDF模板ID不能为空", "local"));
    }
    match pdf_template_service::find_by_id(&db, item.id.unwrap()).await {
        Ok(Some(vo)) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(vo, "local")),
        Ok(None) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "PDF模板不存在或已删除", "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn pdf_template_save(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<PdfTemplateSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let result = pdf_template_service::insert(&db, form_data.into_inner(), Some(get_current_user_id(&req))).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

pub async fn pdf_template_update(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<PdfTemplateUpdateRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let result = pdf_template_service::update(&db, form_data.into_inner(), Some(get_current_user_id(&req))).await;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result)))
}

pub async fn pdf_template_bath_delete(
    state: web::Data<AppState>,
    item: web::Json<BathDeleteIdRequest>,
) -> HttpResponse {
    let db = &state.db;
    let delete_item = item.into_inner();
    let ids = delete_item.parse_ids();
    if ids.is_empty() {
        return HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "未获取到删除的PDF模板ID", "local"));
    }
    let result = pdf_template_service::bath_delete(&db, ids).await;
    HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::handle_result(result))
}

pub async fn pdf_template_set_default(
    state: web::Data<AppState>,
    item: web::Query<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let item = item.into_inner();
    if item.id.is_none() {
        return HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "PDF模板ID不能为空", "local"));
    }
    let id = item.id.unwrap();
    // 查询模板以获取 doc_type
    match pdf_template_service::find_by_id(&db, id).await {
        Ok(Some(vo)) => {
            let doc_type = vo.doc_type.unwrap_or_default();
            if doc_type.is_empty() {
                return HttpResponse::Ok()
                    .content_type(MPACK)
                    .body(MetaResp::<String>::fail(400, "模板单据类型为空", "local"));
            }
            let result = pdf_template_service::set_default(&db, id, &doc_type).await;
            HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<i64>::handle_result(result))
        }
        Ok(None) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "PDF模板不存在或已删除", "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn pdf_template_options(
    state: web::Data<AppState>,
    query: web::Query<PdfDocTypeQuery>,
) -> HttpResponse {
    let db = &state.db;
    let doc_type = query.into_inner().doc_type.unwrap_or_default();
    if doc_type.is_empty() {
        return HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "单据类型不能为空", "local"));
    }
    match pdf_template_service::find_options(&db, &doc_type).await {
        Ok(list) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(list, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

// ============================ PDF 生成/下载 ============================

pub async fn pdf_generate(
    state: web::Data<AppState>,
    req: HttpRequest,
    form_data: web::Json<PdfGenerateRequest>,
) -> HttpResponse {
    let db = &state.db;
    let data = form_data.into_inner();
    let doc_type = data.doc_type.unwrap_or_default();
    let doc_id = data.doc_id.unwrap_or_default();
    let template_id = data.template_id;

    if doc_type.is_empty() || doc_id <= 0 {
        return HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "单据类型和单据ID不能为空", "local"));
    }

    match pdf_generator_service::generate_pdf(
        &db,
        &doc_type,
        doc_id,
        template_id,
        "manual",
        Some(get_current_user_id(&req)),
    )
    .await
    {
        Ok(result) => {
            let vo = PdfGenerateResultVO {
                record_id: result.record_id,
                file_url: result.file_url,
                file_path: result.file_path,
                file_size: result.file_size,
            };
            HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(vo, "local"))
        }
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

pub async fn pdf_preview(
    state: web::Data<AppState>,
    query: web::Query<PdfPreviewQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.into_inner();
    let doc_type = q.doc_type.unwrap_or_default();
    let template_id = q.template_id.as_ref().and_then(|s| s.parse::<i64>().ok());

    if doc_type.is_empty() {
        return HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "单据类型不能为空", "local"));
    }

    // 查询模板：优先按 template_id 查询，否则取默认模板
    let template_vo = if let Some(tid) = template_id {
        match pdf_template_service::find_by_id(&db, tid).await {
            Ok(Some(t)) => Some(t),
            Ok(None) => {
                return HttpResponse::Ok()
                    .content_type(MPACK)
                    .body(MetaResp::<String>::fail(400, "模板不存在", "local"))
            }
            Err(e) => {
                return HttpResponse::Ok()
                    .content_type(MPACK)
                    .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))
            }
        }
    } else {
        match pdf_template_service::find_default(&db, &doc_type).await {
            Ok(t) => t,
            Err(e) => {
                return HttpResponse::Ok()
                    .content_type(MPACK)
                    .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))
            }
        }
    };

    let content = template_vo
        .map(|t| t.content.unwrap_or_default())
        .unwrap_or_default();
    HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(content, "local"))
}

pub async fn pdf_download(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Query<InfoId>,
) -> HttpResponse {
    let db = &state.db;
    let item = item.into_inner();
    if item.id.is_none() {
        return HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "PDF记录ID不能为空", "local"));
    }
    let record_id = item.id.unwrap();

    // 查询 PDF 记录
    let record = match PdfRecordEntity::find_by_id(record_id)
        .filter(pdf_record_entity::Column::Deleted.eq(0))
        .one(db)
        .await
    {
        Ok(Some(r)) => r,
        Ok(None) => {
            return HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(400, "PDF记录不存在", "local"))
        }
        Err(e) => {
            return HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))
        }
    };

    let file_path = match record.file_path.as_ref() {
        Some(p) if !p.is_empty() => p.clone(),
        _ => {
            return HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(400, "PDF文件路径为空", "local"))
        }
    };

    // 读取文件字节
    let file_bytes = match std::fs::read(&file_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            return HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(
                400,
                &format!("读取PDF文件失败: {}", e),
                "local",
            ))
        }
    };

    let file_name = record.file_name.clone().unwrap_or_else(|| "download.pdf".to_string());
    let encoded_name = url_encode_filename(&file_name);
    let content_disp = format!(
        "attachment; filename=\"{}\"; filename*=UTF-8''{}",
        encoded_name, encoded_name
    );

    // 写入下载日志（best-effort，不影响下载本身）
    let ip = req
        .peer_addr()
        .map(|a| a.ip().to_string())
        .unwrap_or_default();
    let now = chrono::Local::now().naive_utc();
    let log_entry = pdf_download_log::ActiveModel {
        record_id: sea_orm::Set(record_id),
        doc_type: sea_orm::Set(record.doc_type.clone()),
        doc_id: sea_orm::Set(record.doc_id),
        doc_no: sea_orm::Set(record.doc_no.clone()),
        file_name: sea_orm::Set(record.file_name.clone()),
        operator_id: sea_orm::Set(Some(get_current_user_id(&req))),
        operator_name: sea_orm::Set(Some(get_current_user(&req).1)),
        ip_address: sea_orm::Set(if ip.is_empty() { None } else { Some(ip) }),
        create_time: sea_orm::Set(Some(now)),
        ..Default::default()
    };
    let _ = PdfDownloadLogEntity::insert(log_entry).exec(db).await;

    HttpResponse::Ok()
        .content_type("application/pdf")
        .insert_header(("Content-Disposition", content_disp))
        .body(file_bytes)
}

pub async fn pdf_record_list(
    state: web::Data<AppState>,
    query: web::Query<PdfRecordListQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.into_inner();
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(10);
    let doc_type = q.doc_type.unwrap_or_default();
    let doc_id = q.doc_id.unwrap_or_default();

    if doc_type.is_empty() || doc_id <= 0 {
        return HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, "单据类型和单据ID不能为空", "local"));
    }

    let (list, _) = match PdfRecordModel::select_by_doc(db, &doc_type, doc_id, page, page_size).await {
        Ok(data) => data,
        Err(e) => {
            return HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))
        }
    };

    // 查询总条数
    let total = PdfRecordModel::select_count_by_doc(db, &doc_type, doc_id)
        .await
        .unwrap_or(0);

    let list_data: Vec<PdfRecordVO> = list.into_iter().map(|m| m.into()).collect();
    let result_page = ResultPage::new(list_data, total, page, page_size);
    let page_u32 = page as u32;
    let total_u32 = total as u32;
    HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::success_with_page(
            result_page,
            "local",
            page_u32,
            total_u32,
        ))
}

/// 全局 PDF 记录列表（管理后台用）
pub async fn pdf_record_all(
    state: web::Data<AppState>,
    query: web::Query<PdfRecordAllQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.into_inner();
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(10);
    let doc_type = q.doc_type.as_deref();
    let doc_no = q.doc_no.as_deref();
    let trigger_type = q.trigger_type.as_deref();

    let (list, _) = match PdfRecordModel::select_all(db, doc_type, doc_no, trigger_type, page, page_size).await {
        Ok(data) => data,
        Err(e) => {
            return HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(400, &e.to_string(), "local"));
        }
    };

    let total = PdfRecordModel::select_count_all(db, doc_type, doc_no, trigger_type)
        .await
        .unwrap_or(0);

    let list_data: Vec<PdfRecordVO> = list.into_iter().map(|m| m.into()).collect();
    let result_page = ResultPage::new(list_data, total, page, page_size);
    let page_u32 = page as u32;
    let total_u32 = total as u32;
    HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::success_with_page(
            result_page,
            "local",
            page_u32,
            total_u32,
        ))
}

/// 模板演示：用模拟数据编译模板并返回 PDF
pub async fn pdf_demo(state: web::Data<AppState>, item: web::Query<InfoId>) -> HttpResponse {
    let db = &state.db;
    let template_id = match item.id {
        Some(id) => id,
        None => {
            return HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(400, "模板ID不能为空", "local"));
        }
    };

    let template = match pdf_template_service::find_by_id(db, template_id).await {
        Ok(Some(t)) => t,
        Ok(None) => {
            return HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(400, "模板不存在", "local"));
        }
        Err(e) => {
            return HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(400, &e.to_string(), "local"));
        }
    };

    let doc_type = template.doc_type.clone().unwrap_or_default();
    let content = template.content.clone().unwrap_or_default();
    let header = template.header_content.clone();
    let footer = template.footer_content.clone();

    let opts = pdf_compiler_service::PdfPageOptions {
        paper_size: template.paper_size.clone().unwrap_or_else(|| "a4".to_string()).to_lowercase(),
        orientation: template.orientation.clone().unwrap_or_else(|| "portrait".to_string()).to_lowercase(),
        margin_top: template.margin_top.unwrap_or(20),
        margin_bottom: template.margin_bottom.unwrap_or(20),
        margin_left: template.margin_left.unwrap_or(20),
        margin_right: template.margin_right.unwrap_or(20),
        font_family: template.font_family.clone().unwrap_or_else(|| "Source Han Sans SC".to_string()),
    };

    // 模拟数据
    let mock_context = build_mock_context(&doc_type);

    let pdf_bytes = match pdf_compiler_service::generate_pdf_bytes(&content, &header, &footer, &mock_context, &opts) {
        Ok(bytes) => bytes,
        Err(e) => {
            return HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(400, &format!("模板编译失败: {}", e), "local"));
        }
    };

    HttpResponse::Ok()
        .content_type("application/pdf")
        .insert_header(("Content-Disposition", "inline"))
        .body(pdf_bytes)
}

/// 构建模拟数据用于模板演示
fn build_mock_context(doc_type: &str) -> serde_json::Value {
    let items: Vec<serde_json::Value> = (1..=6)
        .map(|i| {
            serde_json::json!({
                "index": i,
                "product_code": format!("DEMO-{:04}", i),
                "product_name": format!("演示产品{}", i),
                "spec": format!("规格参数{} · 标准版", i),
                "unit": "台",
                "quantity": format!("{}", i * 2),
                "unit_price": format!("{:.2}", 1000.0 * i as f64),
                "subtotal": format!("{:.2}", 1000.0 * i as f64 * (i * 2) as f64),
                "amount": format!("{:.2}", 1000.0 * i as f64 * (i * 2) as f64),
                "tax_rate": "13",
                "discount_amount": "0",
            })
        })
        .collect();

    match doc_type {
        "order" => serde_json::json!({
            "order": {
                "id": 1,
                "order_no": "DEMO-ORD-2026-001",
                "title": "演示订单标题",
                "customer_name": "演示客户有限公司",
                "contact_name": "张经理",
                "order_date": "2026-08-06",
                "delivery_date": "2026-08-20",
                "currency": "人民币",
                "currency_code": "CNY",
                "product_amount": "84000.00",
                "discount_amount": "0",
                "tax_amount": "10920.00",
                "total_amount": "94920.00",
                "buyer_company_name": "演示客户有限公司",
                "buyer_account_name": "演示客户",
                "buyer_bank_name": "中国银行演示支行",
                "buyer_account_number": "62284800000000001",
                "seller_company_name": "演示供方有限公司",
                "seller_bank_name": "工商银行演示支行",
                "seller_account_name": "演示供方",
                "seller_account_number": "6222020000000002",
                "remark": "此为演示数据",
            },
            "items": items,
            "company": {
                "company_name": "演示供方有限公司",
                "register_address": "上海市浦东新区演示路100号",
                "contact_phone": "021-8888-8888",
                "contact_email": "demo@company.com",
                "credit_code": "91310000DEMO001X",
            },
            "customer": {
                "company_name": "演示客户有限公司",
                "person_name": "张经理",
                "address": "北京市海淀区演示大厦",
                "personal_mobile": "138-0000-0001",
            },
            "grand_total_cn": "玖万肆仟玖佰贰拾元整",
        }),
        "contract" => serde_json::json!({
            "contract": {
                "id": 1,
                "contract_no": "DEMO-HT-2026-001",
                "title": "演示购销合同",
                "amount": "94920.00",
                "total_amount": "94920.00",
                "tax_amount": "10920.00",
                "sign_date": "2026-08-06",
                "start_date": "2026-08-06",
                "end_date": "2026-12-31",
                "payment_terms": "月结30天",
                "delivery_terms": "送货上门",
                "payment_method_type": "分期付款",
                "our_signer_name": "李经理",
                "their_signer_name": "张经理",
                "their_signer_phone": "138-0000-0001",
                "remark": "此为演示合同数据",
            },
            "company": {
                "company_name": "演示供方有限公司",
                "register_address": "上海市浦东新区演示路100号",
                "contact_phone": "021-8888-8888",
                "contact_email": "demo@company.com",
                "credit_code": "91310000DEMO001X",
            },
            "customer": {
                "company_name": "演示客户有限公司",
                "person_name": "张经理",
                "address": "北京市海淀区演示大厦",
                "personal_mobile": "138-0000-0001",
            },
            "payment_plans": [
                {"index": 1, "stage_name": "预付款", "plan_amount": "28476.00", "plan_date": "2026-08-10", "actual_date": "", "status": "待付", "remark": ""},
                {"index": 2, "stage_name": "到货款", "plan_amount": "56952.00", "plan_date": "2026-08-25", "actual_date": "", "status": "待付", "remark": ""},
                {"index": 3, "stage_name": "质保金", "plan_amount": "9492.00", "plan_date": "2027-01-10", "actual_date": "", "status": "待付", "remark": ""},
            ],
            "grand_total_cn": "玖万肆仟玖佰贰拾元整",
            "contract_description_typst": "#text(size: 10pt, weight: \"bold\", fill: rgb(\"#96680a\"))[第二条] #text(size: 10pt, weight: \"bold\")[  质量标准]\n#line(length: 100%, stroke: 0.3pt + rgb(\"#cccccc\"))\n#v(2pt)\n#text(size: 8.5pt)[产品质量应符合国家标准及行业技术规格。质保期12个月，自验收合格之日起计算。]\n#v(4pt)\n#text(size: 10pt, weight: \"bold\", fill: rgb(\"#96680a\"))[第三条] #text(size: 10pt, weight: \"bold\")[  交货方式]\n#line(length: 100%, stroke: 0.3pt + rgb(\"#cccccc\"))\n#v(2pt)\n#text(size: 8.5pt)[卖方应于约定交货期内送达指定地点，运费由卖方承担。]",
        }),
        _ => serde_json::json!({
            "quotation": {
                "id": 1,
                "quotation_no": "DEMO-QUO-2026-001",
                "customer_name": "演示客户有限公司",
                "contact_name": "张经理",
                "title": "演示报价单标题",
                "total_amount": "84000.00",
                "currency": "人民币",
                "tax_amount": "10920.00",
                "discount_amount": "",
                "grand_total": "94920.00",
                "valid_until": "2026-09-06",
                "quotation_date": "2026-08-06",
                "payment_terms": "月结30天",
                "delivery_terms": "送货上门",
                "delivery_date": "2026-08-20",
                "remark": "此为演示数据",
            },
            "items": items,
            "company": {
                "company_name": "演示供方有限公司",
                "register_address": "上海市浦东新区演示路100号",
                "contact_phone": "021-8888-8888",
                "contact_email": "demo@company.com",
                "credit_code": "91310000DEMO001X",
            },
            "customer": {
                "company_name": "演示客户有限公司",
                "person_name": "张经理",
                "address": "北京市海淀区演示大厦",
                "personal_mobile": "138-0000-0001",
            },
            "grand_total_cn": "玖万肆仟玖佰贰拾元整",
        }),
    }
}

/// PDF 下载日志列表
pub async fn pdf_download_log_list(
    state: web::Data<AppState>,
    query: web::Query<PdfDownloadLogQuery>,
) -> HttpResponse {
    let db = &state.db;
    let q = query.into_inner();
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(10);
    let doc_type = q.doc_type.as_deref();
    let doc_no = q.doc_no.as_deref();

    let mut qr = PdfDownloadLogEntity::find();
    if let Some(dt) = doc_type {
        if !dt.is_empty() {
            qr = qr.filter(pdf_download_log::Column::DocType.eq(dt));
        }
    }
    if let Some(dn) = doc_no {
        if !dn.is_empty() {
            qr = qr.filter(pdf_download_log::Column::DocNo.contains(dn));
        }
    }

    let total = qr.clone().count(db).await.unwrap_or(0) as u32;
    let paginator = qr
        .order_by_desc(pdf_download_log::Column::Id)
        .paginate(db, page_size as u64);
    let list = match paginator.fetch_page((page - 1) as u64).await {
        Ok(l) => l,
        Err(e) => {
            return HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(400, &e.to_string(), "local"));
        }
    };

    let list_data: Vec<PdfDownloadLogVO> = list.into_iter().map(|m| m.into()).collect();
    let result_page = ResultPage::new(list_data, total as i64, page, page_size);
    HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::success_with_page(result_page, "local", page as u32, total))
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/pdf-template")
            .route(
                "/list",
                web::get()
                    .to(pdf_template_list)
                    .wrap(require_permission("system:pdf-template:list")),
            )
            .route(
                "/info",
                web::get()
                    .to(pdf_template_info)
                    .wrap(require_permission("system:pdf-template:view")),
            )
            .route(
                "/save",
                web::post()
                    .to(pdf_template_save)
                    .wrap(require_permission("system:pdf-template:create")),
            )
            .route(
                "/update",
                web::put()
                    .to(pdf_template_update)
                    .wrap(require_permission("system:pdf-template:update")),
            )
            .route(
                "/bath_delete",
                web::post()
                    .to(pdf_template_bath_delete)
                    .wrap(require_permission("system:pdf-template:delete")),
            )
            .route(
                "/set_default",
                web::put()
                    .to(pdf_template_set_default)
                    .wrap(require_permission("system:pdf-template:update")),
            )
            .route(
                "/options",
                web::get()
                    .to(pdf_template_options)
                    .wrap(require_permission("system:pdf-template:list")),
            ),
    );
    cfg.service(
        web::scope("/pdf")
            .route(
                "/generate",
                web::post()
                    .to(pdf_generate)
                    .wrap(require_permission("system:pdf-template:list")),
            )
            .route(
                "/preview",
                web::get()
                    .to(pdf_preview)
                    .wrap(require_permission("system:pdf-template:list")),
            )
            .route(
                "/download",
                web::get()
                    .to(pdf_download)
                    .wrap(require_permission("system:pdf-template:list")),
            )
            .route(
                "/record-list",
                web::get()
                    .to(pdf_record_list)
                    .wrap(require_permission("system:pdf-record:list")),
            )
            .route(
                "/record-all",
                web::get()
                    .to(pdf_record_all)
                    .wrap(require_permission("system:pdf-record:list")),
            )
            .route(
                "/download-log",
                web::get()
                    .to(pdf_download_log_list)
                    .wrap(require_permission("system:pdf-record:list")),
            )
            .route(
                "/demo",
                web::get()
                    .to(pdf_demo)
                    .wrap(require_permission("system:pdf-template:list")),
            ),
    );
}
