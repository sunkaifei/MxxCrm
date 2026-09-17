//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 可视化 PDF 模板设计器 —— 业务服务层。
//!
//! 设计依据：设计文档 §7（接口）、§25（报文）、§32（设计时数据）、
//!          §33（软锁 / 模板包 / schema 演进）、§34.6（分页预览）。

use crate::core::errors::error::{Error, Result};
use crate::modules::system::entity::{
    pdf_asset, pdf_field_meta, pdf_template, pdf_template_version,
};
use crate::modules::system::entity::pdf_template::Entity as PdfTemplate;
use crate::modules::system::service::{
    pdf_asset_service, pdf_doc_meta, pdf_formatters, pdf_layout_schema, pdf_layout_to_typst,
};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DbConn, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

// ============================================================================
// 请求 / 响应结构
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveLayoutRequest {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub template_code: Option<String>,
    pub doc_type: Option<String>,
    pub layout_json: Value,
    pub based_on_template_id: Option<i64>,
    pub remark: Option<String>,
    /// 乐观锁：客户端持有的版本号
    pub version: Option<i32>,
    /// 是否强制覆盖（他人已修改时）
    pub force: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewRequest {
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_to_u64")]
    pub template_id: Option<i64>,
    pub layout_json: Option<Value>,
    pub doc_type: Option<String>,
    pub doc_id: Option<i64>,
    /// placeholder | sample | real
    pub data_mode: Option<String>,
    pub preset: Option<String>,
    pub item_rows: Option<i32>,
    /// all = 分页预览；1 | 2 … = 精确预览
    pub pages: Option<Value>,
    pub format: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateBindingsRequest {
    pub doc_type: String,
    pub layout_json: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockRequest {
    pub template_id: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditLock {
    pub user_id: i64,
    pub user_name: String,
    pub expire_at: String,
    #[serde(skip)]
    pub expire_ts: i64,
}

// ============================================================================
// 编辑期软锁（§33.1）
// ============================================================================

/// 进程内锁表；TTL 30s，惰性过期
fn edit_locks() -> &'static parking_lot::RwLock<HashMap<i64, EditLock>> {
    static LOCKS: OnceLock<parking_lot::RwLock<HashMap<i64, EditLock>>> = OnceLock::new();
    LOCKS.get_or_init(|| parking_lot::RwLock::new(HashMap::new()))
}

const LOCK_TTL_SECS: i64 = 30;

fn now_ts() -> i64 {
    chrono::Local::now().timestamp()
}

fn fmt_ts(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|d| {
            d.with_timezone(&chrono::Local)
                .format("%H:%M:%S")
                .to_string()
        })
        .unwrap_or_default()
}

/// 抢占锁。返回 Ok(Some(lock)) 表示已被他人占用。
pub fn acquire_lock(template_id: i64, user_id: i64, user_name: &str) -> Option<EditLock> {
    let mut locks = edit_locks().write();
    let now = now_ts();
    if let Some(existing) = locks.get(&template_id) {
        if existing.expire_ts > now && existing.user_id != user_id {
            return Some(existing.clone());
        }
    }
    let lock = EditLock {
        user_id,
        user_name: user_name.to_string(),
        expire_at: fmt_ts(now + LOCK_TTL_SECS),
        expire_ts: now + LOCK_TTL_SECS,
    };
    locks.insert(template_id, lock.clone());
    None
}

/// 心跳续期。返回 true 表示仍持有；false 表示已被他人接管。
pub fn heartbeat_lock(template_id: i64, user_id: i64) -> bool {
    let mut locks = edit_locks().write();
    let now = now_ts();
    match locks.get_mut(&template_id) {
        Some(l) => {
            if l.user_id != user_id {
                return false;
            }
            l.expire_ts = now + LOCK_TTL_SECS;
            l.expire_at = fmt_ts(l.expire_ts);
            true
        }
        None => {
            // 锁已过期被清理：允许当前用户重新持有
            locks.insert(
                template_id,
                EditLock {
                    user_id,
                    user_name: String::new(),
                    expire_at: fmt_ts(now + LOCK_TTL_SECS),
                    expire_ts: now + LOCK_TTL_SECS,
                },
            );
            true
        }
    }
}

pub fn release_lock(template_id: i64, user_id: i64) {
    let mut locks = edit_locks().write();
    if let Some(l) = locks.get(&template_id) {
        if l.user_id == user_id {
            locks.remove(&template_id);
        }
    }
}

pub fn current_lock(template_id: i64) -> Option<EditLock> {
    let locks = edit_locks().read();
    let now = now_ts();
    locks
        .get(&template_id)
        .filter(|l| l.expire_ts > now)
        .cloned()
}

// ============================================================================
// 元数据接口
// ============================================================================

pub fn doc_types() -> Value {
    let list: Vec<Value> = pdf_doc_meta::DOC_TYPES
        .iter()
        .map(|(code, name, real)| {
            json!({
                "code": code,
                "name": name,
                "hasRealData": real,
                // §20.1：法定发票票面版式不可自定义
                "hint": if *code == "invoice" {
                    Value::String("仅支持打印已开具发票，票面版式不可修改".to_string())
                } else {
                    Value::Null
                }
            })
        })
        .collect();
    json!(list)
}

pub fn formatters() -> Value {
    let list: Vec<Value> = pdf_formatters::FORMATTERS
        .iter()
        .map(|(k, n, _)| json!({ "key": k, "name": n }))
        .collect();
    json!(list)
}

pub fn presets() -> Value {
    let list: Vec<Value> = pdf_doc_meta::PRESETS
        .iter()
        .map(|(k, n, d)| json!({ "key": k, "name": n, "desc": d }))
        .collect();
    json!(list)
}

/// 字段树（§25.1），DB 元数据表作为覆盖层
pub async fn field_tree(db: &DbConn, doc_type: &str) -> Result<Value> {
    let rows: Vec<pdf_field_meta::Model> = pdf_field_meta::Entity::find()
        .filter(pdf_field_meta::Column::Deleted.eq(0))
        .filter(pdf_field_meta::Column::Status.eq(1))
        .filter(pdf_field_meta::Column::DocType.eq(doc_type))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let overrides: Vec<(String, String, String, String, bool)> = rows
        .into_iter()
        .map(|r| {
            (
                r.field_path,
                r.field_name,
                r.data_type,
                r.sample.unwrap_or_default(),
                r.nullable,
            )
        })
        .collect();

    let groups = pdf_doc_meta::build_field_tree(doc_type, &overrides);
    Ok(json!({
        "docType": doc_type,
        "docName": pdf_doc_meta::doc_name(doc_type),
        "hasRealData": pdf_doc_meta::doc_has_real_context(doc_type),
        "groups": groups,
        "formatters": formatters(),
        "presets": presets()
    }))
}

/// 样例数据（§32.12）
pub async fn sample_data(
    db: &DbConn,
    doc_type: &str,
    preset: Option<&str>,
    item_rows: Option<i32>,
    template_id: Option<i64>,
) -> Result<Value> {
    let mut preset = preset.map(|s| s.to_string());
    let mut rows = item_rows;
    // 模板未显式传参时，取模板内的 designSample 作为初始值（§32.11）
    if (preset.is_none() || rows.is_none()) && template_id.is_some() {
        if let Some(t) = PdfTemplate::find_by_id(template_id.unwrap())
            .one(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?
        {
            if let Some(lj) = t.layout_json.as_ref() {
                if let Ok(layout) = pdf_layout_schema::LayoutJson::parse_strict(lj) {
                    preset = preset.or(Some(layout.settings.design_sample.preset.clone()));
                    rows = rows.or(Some(layout.settings.design_sample.item_rows));
                }
            }
        }
    }
    let preset = preset.unwrap_or_else(|| "typical".to_string());
    let rows = rows.unwrap_or(3);
    Ok(json!({
        "docType": doc_type,
        "preset": preset,
        "itemRows": rows,
        "context": pdf_doc_meta::sample_context(doc_type, &preset, rows)
    }))
}

/// 真实单据上下文（§32.12）。数据权限由控制器层校验。
pub async fn real_data(db: &DbConn, doc_type: &str, doc_id: i64) -> Result<Value> {
    use crate::modules::system::service::pdf_generator_service as gen;
    if !pdf_doc_meta::doc_has_real_context(doc_type) {
        return Err(Error::from(format!(
            "单据类型「{}」的真实数据上下文尚未接入（当前仅支持设计态样例数据）",
            pdf_doc_meta::doc_name(doc_type)
        )));
    }
    if doc_id <= 0 {
        return Err(Error::from("请选择一条真实单据"));
    }
    match doc_type {
        "quotation" => gen::build_quotation_context(db, doc_id).await,
        "order" => gen::build_order_context(db, doc_id).await,
        "contract" => gen::build_contract_context(db, doc_id).await,
        // P1-1 扩展层（发货/出库/入库/采购/收款）
        other => {
            crate::modules::system::service::pdf_context_extra::build_real_context(db, other, doc_id)
                .await?
                .ok_or_else(|| {
                    Error::from(format!("未接入的真实数据上下文: {}", doc_type))
                })
        }
    }
}

// ============================================================================
// 绑定健康度检查（§32.8）
// ============================================================================

pub fn validate_bindings(doc_type: &str, layout_value: &Value) -> Result<Value> {
    let layout = pdf_layout_schema::LayoutJson::parse_strict(layout_value)
        .map_err(Error::from)?;

    let defs = pdf_doc_meta::builtin_fields(doc_type);
    let known: HashSet<String> = defs.iter().map(|d| d.field_path.clone()).collect();
    let mut used: HashSet<String> = HashSet::new();
    let mut issues: Vec<Value> = Vec::new();

    let empty_ctx = pdf_doc_meta::sample_context(doc_type, "empty", 3);
    let long_ctx = pdf_doc_meta::sample_context(doc_type, "long", 3);

    for el in layout.all_elements() {
        // 1. 未绑定
        if el.element_type == "field" {
            let path = el.bind.path.clone().unwrap_or_default();
            if path.trim().is_empty() {
                issues.push(json!({
                    "level": "error", "code": "unbound",
                    "message": format!("元素「{}」未绑定字段", el.name),
                    "elementId": el.id
                }));
            } else if !known.contains(&path) {
                issues.push(json!({
                    "level": "error", "code": "dangling",
                    "message": format!("元素「{}」绑定了不存在的字段：{}", el.name, path),
                    "elementId": el.id, "fieldPath": path
                }));
            } else {
                used.insert(path.clone());
                // 7. 全空字段
                let v = pdf_layout_to_typst::resolve_path(&empty_ctx, &path);
                let is_empty = match &v {
                    None | Some(Value::Null) => true,
                    Some(Value::String(s)) => s.trim().is_empty(),
                    _ => false,
                };
                if is_empty && el.bind.default_value.as_deref().unwrap_or("").is_empty() {
                    issues.push(json!({
                        "level": "info", "code": "empty_value",
                        "message": format!("元素「{}」绑定的字段「{}」在缺省样本下无值且未设置兜底文案", el.name, path),
                        "elementId": el.id, "fieldPath": path
                    }));
                }
                // 3. 溢出风险：超长样本下的文本宽度
                if el.w > 0.0 {
                    let long_v = pdf_layout_to_typst::resolve_path(&long_ctx, &path);
                    let text = long_v
                        .map(|v| pdf_formatters::format_value(&v, el.bind.format.as_deref()))
                        .unwrap_or_default();
                    let est = estimate_width_mm(&text, el.style.font_size);
                    let avail = el.w - el.style.padding.l - el.style.padding.r;
                    if est > avail {
                        issues.push(json!({
                            "level": "warn", "code": "overflow",
                            "message": format!("元素「{}」在「超长」样本下文本宽约 {:.1}mm，超出容器 {:.1}mm", el.name, est, avail),
                            "elementId": el.id, "fieldPath": path
                        }));
                    }
                }
            }
        }

        // 4. 条件永不成立
        if let Some(pi) = el.print_if.as_ref() {
            if !pi.field.is_empty() && !known.contains(&pi.field) {
                issues.push(json!({
                    "level": "warn", "code": "dead_condition",
                    "message": format!("元素「{}」的显示条件引用了不存在的字段：{}", el.name, pi.field),
                    "elementId": el.id, "fieldPath": pi.field
                }));
            }
        }

        // 5. 疑似漏绑
        if el.element_type == "text" {
            let content = el
                .props
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if looks_like_data(content) {
                issues.push(json!({
                    "level": "info", "code": "suspected_unbound",
                    "message": format!("文本元素「{}」内容形如单据数据（{}），可能本应绑定字段", el.name, content),
                    "elementId": el.id
                }));
            }
        }

        // 表格列绑定检查
        if el.element_type == "table" {
            let props = pdf_layout_to_typst::resolve_path(&json!({}), "").is_none();
            let _ = props;
            let tp = serde_json::from_value::<
                crate::modules::system::service::pdf_layout_schema::TableProps,
            >(el.props.clone())
            .unwrap_or_default();
            for c in &tp.columns {
                if let Some(b) = c.bind.as_deref().filter(|b| !b.is_empty()) {
                    let norm = format!("{}", b);
                    if !known.contains(&norm) {
                        issues.push(json!({
                            "level": "error", "code": "dangling",
                            "message": format!("明细列「{}」绑定了不存在的字段：{}", c.title, b),
                            "elementId": el.id, "fieldPath": b
                        }));
                    } else {
                        used.insert(norm);
                    }
                }
            }
        }
    }

    // 6. 未使用字段
    for d in defs.iter() {
        if d.group_code == "item" || d.group_code == "sys" || d.group_code == "custom" {
            continue; // 明细列与系统字段不参与"未使用"提示，避免噪音
        }
        if !used.contains(&d.field_path) {
            issues.push(json!({
                "level": "info", "code": "unused_field",
                "message": format!("字段「{}」未被任何元素使用", d.field_name),
                "fieldPath": d.field_path
            }));
        }
    }

    let errors = issues.iter().filter(|i| i["level"] == "error").count();
    let warns = issues.iter().filter(|i| i["level"] == "warn").count();
    let infos = issues.iter().filter(|i| i["level"] == "info").count();

    Ok(json!({
        "summary": { "error": errors, "warn": warns, "info": infos, "total": issues.len() },
        "issues": issues
    }))
}

fn estimate_width_mm(text: &str, font_size_pt: f64) -> f64 {
    let mut em = 0.0f64;
    for ch in text.chars() {
        if ch == '\n' {
            continue;
        }
        if (ch as u32) >= 0x2E80 {
            em += 1.0;
        } else {
            em += 0.5;
        }
    }
    em * font_size_pt * 0.3528
}

fn looks_like_data(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() {
        return false;
    }
    let re_no = regex::Regex::new(r"^[A-Z]{2,4}\d{8,}").unwrap();
    let re_date = regex::Regex::new(r"^\d{4}[-/年]\d{1,2}[-/月]\d{1,2}").unwrap();
    let re_money = regex::Regex::new(r"^\d{1,3}(,\d{3})*(\.\d+)?$").unwrap();
    re_no.is_match(t) || re_date.is_match(t) || (re_money.is_match(t) && t.len() > 5)
}

// ============================================================================
// 模板详情 / 保存 / 复制
// ============================================================================

pub async fn template_detail(db: &DbConn, id: i64) -> Result<Value> {
    let t = PdfTemplate::find_by_id(id)
        .filter(pdf_template::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("PDF 模板不存在"))?;

    let mut layout = t.layout_json.clone();
    // §33.3：加载即迁移（三条路径：设计器打开 / 精确预览 / 正式出图）
    if let Some(lj) = layout.as_ref() {
        let mut v = lj.clone();
        pdf_layout_schema::migrate_layout(&mut v, None).map_err(Error::from)?;
        layout = Some(v);
    }
    // P2-5：html 引擎模板无 layout_json → 从 content 迁移出 layout **草稿**
    // （仅随本次响应下发，不落库；用户在设计器校正后「另存为」才生成 layout 模板）
    let migrated_from_html;
    if layout.is_none() && t.engine.as_deref() == Some("html") {
        let page = pdf_layout_schema::PageConfig {
            size: t.paper_size.clone().unwrap_or_else(|| "A4".to_string()),
            width: 210.0,
            height: 297.0,
            orientation: t.orientation.clone().unwrap_or_else(|| "portrait".to_string()),
            margin: pdf_layout_schema::Margin {
                top: t.margin_top.map(|v| v as f64).unwrap_or(12.0),
                right: t.margin_right.map(|v| v as f64).unwrap_or(12.0),
                bottom: t.margin_bottom.map(|v| v as f64).unwrap_or(12.0),
                left: t.margin_left.map(|v| v as f64).unwrap_or(12.0),
            },
            ..Default::default()
        };
        let draft = crate::modules::system::service::pdf_html_migrate::migrate_content_to_layout(
            t.content.as_deref().unwrap_or(""),
            page,
        );
        migrated_from_html = Some(draft);
        layout = migrated_from_html.clone();
    } else {
        migrated_from_html = None;
    }

    Ok(json!({
        "id": t.id.to_string(),
        "name": t.name,
        "templateCode": t.template_code,
        "docType": t.doc_type,
        "engine": t.engine.unwrap_or_else(|| "html".to_string()),
        "layoutJson": layout,
        "content": t.content,
        "headerContent": t.header_content,
        "footerContent": t.footer_content,
        "paperSize": t.paper_size,
        "orientation": t.orientation,
        "marginTop": t.margin_top,
        "marginBottom": t.margin_bottom,
        "marginLeft": t.margin_left,
        "marginRight": t.margin_right,
        "fontFamily": t.font_family,
        "isDefault": t.is_default,
        "status": t.status,
        "version": t.version,
        "remark": t.remark,
        "previewUrl": t.preview_url,
        "basePdfId": t.base_pdf_id,
        "updateTime": t.update_time.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
    }))
}

pub async fn save_layout(
    db: &DbConn,
    req: SaveLayoutRequest,
    operator_id: Option<i64>,
) -> Result<Value> {
    let normalized = pdf_layout_schema::normalize_layout(&req.layout_json).map_err(Error::from)?;
    let layout = pdf_layout_schema::LayoutJson::parse_strict(&normalized).map_err(Error::from)?;
    let doc_type = req
        .doc_type
        .clone()
        .or_else(|| {
            layout
                .page
                .background
                .value
                .clone()
                .filter(|_| false)
        })
        .unwrap_or_else(|| "order".to_string());
    if !pdf_doc_meta::DOC_TYPES.iter().any(|(c, _, _)| *c == doc_type) {
        return Err(Error::from(format!("不支持的单据类型: {}", doc_type)));
    }

    let now = chrono::Local::now().naive_local();
    let paper = if layout.page.orientation == "landscape" {
        "a4"
    } else {
        "a4"
    };
    let _ = paper;

    if let Some(id) = req.id {
        let existing = PdfTemplate::find_by_id(id)
            .filter(pdf_template::Column::Deleted.eq(0))
            .one(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?
            .ok_or_else(|| Error::from("PDF 模板不存在"))?;

        // 乐观锁（§24.3）
        if let Some(client_ver) = req.version {
            if client_ver != existing.version && !req.force.unwrap_or(false) {
                return Err(Error::from(format!(
                    "__CONFLICT__模板已被他人修改（服务器版本 {}，你的版本 {}），是否覆盖？",
                    existing.version, client_ver
                )));
            }
        }

        let new_version = existing.version + 1;
        // 版本快照（§7.1 versions/rollback）
        snapshot(db, &existing, existing.version, "保存前快照", operator_id).await?;

        let am = pdf_template::ActiveModel {
            id: Set(existing.id),
            name: Set(req.name.clone().or(existing.name.clone())),
            template_code: Set(req
                .template_code
                .clone()
                .or(existing.template_code.clone())),
            doc_type: Set(Some(doc_type.clone())),
            layout_json: Set(Some(normalized.clone())),
            engine: Set(Some("layout".to_string())),
            version: Set(new_version),
            remark: Set(req.remark.clone().or(existing.remark.clone())),
            update_by: Set(operator_id),
            update_time: Set(Some(now)),
            ..Default::default()
        };
        PdfTemplate::update(am)
            .exec(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        return Ok(json!({
            "id": id.to_string(),
            "version": new_version,
            "previewUrl": existing.preview_url
        }));
    }

    // 新建
    let name = req
        .name
        .clone()
        .filter(|n| !n.trim().is_empty())
        .ok_or_else(|| Error::from("模板名称不能为空"))?;
    let code = req
        .template_code
        .clone()
        .filter(|c| !c.trim().is_empty())
        .unwrap_or_else(|| {
            format!(
                "{}_layout_{}",
                doc_type,
                chrono::Local::now().format("%Y%m%d%H%M%S")
            )
        });

    let exists = PdfTemplate::find()
        .filter(pdf_template::Column::TemplateCode.eq(code.as_str()))
        .filter(pdf_template::Column::Deleted.eq(0))
        .count(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    if exists > 0 {
        return Err(Error::from(format!("模板编码已存在: {}", code)));
    }

    let am = pdf_template::ActiveModel {
        name: Set(Some(name)),
        template_code: Set(Some(code)),
        doc_type: Set(Some(doc_type.clone())),
        layout_json: Set(Some(normalized)),
        engine: Set(Some("layout".to_string())),
        // content 列（HTML 正文）为 NOT NULL；layout 引擎不出图走 layout_json，
        // 落一个占位串满足约束（历史 HTML 链路不会读它）
        content: Set(Some(String::new())),
        paper_size: Set(Some("a4".to_string())),
        orientation: Set(Some(layout.page.orientation.clone())),
        is_default: Set(Some(0)),
        status: Set(Some(1)),
        version: Set(1),
        parent_id: Set(req.based_on_template_id),
        remark: Set(req.remark),
        create_by: Set(operator_id),
        create_time: Set(Some(now)),
        update_by: Set(operator_id),
        update_time: Set(Some(now)),
        ..Default::default()
    };
    let r = PdfTemplate::insert(am)
        .exec(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(json!({ "id": r.last_insert_id.to_string(), "version": 1 }))
}

async fn snapshot(
    db: &DbConn,
    t: &pdf_template::Model,
    version: i32,
    note: &str,
    operator_id: Option<i64>,
) -> Result<()> {
    let dup = pdf_template_version::Entity::find()
        .filter(pdf_template_version::Column::TemplateId.eq(t.id))
        .filter(pdf_template_version::Column::Version.eq(version))
        .count(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    if dup > 0 {
        return Ok(());
    }
    let am = pdf_template_version::ActiveModel {
        template_id: Set(t.id),
        version: Set(version),
        name: Set(t.name.clone()),
        doc_type: Set(t.doc_type.clone()),
        engine: Set(t.engine.clone()),
        layout_json: Set(t.layout_json.clone()),
        content: Set(t.content.clone()),
        change_note: Set(Some(note.to_string())),
        create_by: Set(operator_id),
        create_time: Set(Some(chrono::Local::now().naive_local())),
        ..Default::default()
    };
    pdf_template_version::Entity::insert(am)
        .exec(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(())
}

pub async fn versions(db: &DbConn, template_id: i64) -> Result<Value> {
    let rows = pdf_template_version::Entity::find()
        .filter(pdf_template_version::Column::TemplateId.eq(template_id))
        .order_by_desc(pdf_template_version::Column::Version)
        .limit(50)
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    let list: Vec<Value> = rows
        .into_iter()
        .map(|r| {
            json!({
                "id": r.id.to_string(),
                "version": r.version,
                "name": r.name,
                "engine": r.engine,
                "changeNote": r.change_note,
                "createTime": r.create_time.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
            })
        })
        .collect();
    Ok(json!(list))
}

pub async fn rollback(
    db: &DbConn,
    template_id: i64,
    version: i32,
    operator_id: Option<i64>,
) -> Result<Value> {
    let cur = PdfTemplate::find_by_id(template_id)
        .filter(pdf_template::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("PDF 模板不存在"))?;
    let target = pdf_template_version::Entity::find()
        .filter(pdf_template_version::Column::TemplateId.eq(template_id))
        .filter(pdf_template_version::Column::Version.eq(version))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("目标版本不存在"))?;

    snapshot(db, &cur, cur.version, "回滚前快照", operator_id).await?;

    let am = pdf_template::ActiveModel {
        id: Set(cur.id),
        layout_json: Set(target.layout_json.clone()),
        content: Set(target.content.clone()),
        engine: Set(target.engine.clone().or(cur.engine.clone())),
        version: Set(cur.version + 1),
        update_by: Set(operator_id),
        update_time: Set(Some(chrono::Local::now().naive_local())),
        ..Default::default()
    };
    PdfTemplate::update(am)
        .exec(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(json!({ "id": template_id.to_string(), "version": cur.version + 1 }))
}

pub async fn duplicate(db: &DbConn, id: i64, operator_id: Option<i64>) -> Result<Value> {
    let t = PdfTemplate::find_by_id(id)
        .filter(pdf_template::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("PDF 模板不存在"))?;
    let now = chrono::Local::now().naive_local();
    let am = pdf_template::ActiveModel {
        name: Set(Some(format!("{}（副本）", t.name.clone().unwrap_or_default()))),
        template_code: Set(Some(format!(
            "{}_copy_{}",
            t.template_code.clone().unwrap_or_else(|| "tpl".to_string()),
            chrono::Local::now().format("%Y%m%d%H%M%S")
        ))),
        doc_type: Set(t.doc_type.clone()),
        content: Set(t.content.clone()),
        header_content: Set(t.header_content.clone()),
        footer_content: Set(t.footer_content.clone()),
        paper_size: Set(t.paper_size.clone()),
        orientation: Set(t.orientation.clone()),
        margin_top: Set(t.margin_top),
        margin_bottom: Set(t.margin_bottom),
        margin_left: Set(t.margin_left),
        margin_right: Set(t.margin_right),
        font_family: Set(t.font_family.clone()),
        engine: Set(t.engine.clone()),
        layout_json: Set(t.layout_json.clone()),
        base_pdf_id: Set(t.base_pdf_id),
        width_mm: Set(t.width_mm),
        height_mm: Set(t.height_mm),
        version: Set(1),
        parent_id: Set(Some(t.id)),
        is_default: Set(Some(0)),
        status: Set(Some(1)),
        remark: Set(t.remark.clone()),
        create_by: Set(operator_id),
        create_time: Set(Some(now)),
        update_by: Set(operator_id),
        update_time: Set(Some(now)),
        ..Default::default()
    };
    let r = PdfTemplate::insert(am)
        .exec(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(json!({ "id": r.last_insert_id.to_string() }))
}

/// 设为默认模板（独立权限 + C 档元素二次确认，§22.1 / §34.8）
pub async fn set_default(
    db: &DbConn,
    id: i64,
    confirm_out_of_paper: bool,
) -> Result<Value> {
    let t = PdfTemplate::find_by_id(id)
        .filter(pdf_template::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("PDF 模板不存在"))?;
    let doc_type = t.doc_type.clone().unwrap_or_default();

    // C 档（纸外元素）检查
    let mut out_of_paper: Vec<String> = Vec::new();
    if let Some(lj) = t.layout_json.as_ref() {
        if let Ok(layout) = pdf_layout_schema::LayoutJson::parse_strict(lj) {
            let (w, h) = layout.page.effective_size();
            for el in layout.all_elements() {
                let outside = el.x < 0.0 || el.y < 0.0 || el.x + el.w > w || el.y + el.h > h;
                if outside {
                    out_of_paper.push(el.name.clone());
                }
            }
        }
    }
    if !out_of_paper.is_empty() && !confirm_out_of_paper {
        return Err(Error::from(format!(
            "__NEED_CONFIRM__模板存在 {} 个超出纸张的元素（出图时会被裁切）：{}",
            out_of_paper.len(),
            out_of_paper.join("、")
        )));
    }

    PdfTemplate::update_many()
        .col_expr(pdf_template::Column::IsDefault, Expr::value(0))
        .filter(pdf_template::Column::DocType.eq(doc_type.as_str()))
        .filter(pdf_template::Column::Deleted.eq(0))
        .exec(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let am = pdf_template::ActiveModel {
        id: Set(id),
        is_default: Set(Some(1)),
        status: Set(Some(1)),
        ..Default::default()
    };
    PdfTemplate::update(am)
        .exec(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(json!({ "id": id.to_string(), "outOfPaper": out_of_paper }))
}

// ============================================================================
// 精确预览 / 分页预览（§25.3、§34.6）
// ============================================================================

/// 预览页缓存：token → 每页 SVG（分页预览改为临时 URL 返回，避免 msgpack 体积过大）
fn preview_cache() -> &'static moka::sync::Cache<String, Vec<String>> {
    static CACHE: OnceLock<moka::sync::Cache<String, Vec<String>>> = OnceLock::new();
    CACHE.get_or_init(|| {
        moka::sync::Cache::builder()
            .max_capacity(64)
            .time_to_live(std::time::Duration::from_secs(600))
            .build()
    })
}

pub fn take_preview_page(token: &str, page: usize) -> Option<String> {
    preview_cache()
        .get(token)
        .and_then(|pages| pages.get(page.saturating_sub(1)).cloned())
}

pub async fn precise_preview(db: &DbConn, req: PreviewRequest) -> Result<Value> {
    // 1. 取 layout：优先显式传入（未保存也可预览），否则按 templateId 读库
    let (layout_value, doc_type) = match req.layout_json.clone() {
        Some(v) if !v.is_null() => {
            let dt = req
                .doc_type
                .clone()
                .unwrap_or_else(|| "order".to_string());
            (v, dt)
        }
        _ => {
            let id = req
                .template_id
                .ok_or_else(|| Error::from("缺少 templateId 或 layoutJson"))?;
            let t = PdfTemplate::find_by_id(id)
                .filter(pdf_template::Column::Deleted.eq(0))
                .one(db)
                .await
                .map_err(|e| Error::from(e.to_string()))?
                .ok_or_else(|| Error::from("PDF 模板不存在"))?;
            let lj = t
                .layout_json
                .clone()
                .ok_or_else(|| Error::from("该模板不是可视化设计器模板（缺少 layout_json）"))?;
            let dt = req
                .doc_type
                .clone()
                .or(t.doc_type.clone())
                .unwrap_or_else(|| "order".to_string());
            (lj, dt)
        }
    };

    // 2. 迁移 + 校验（§33.3 第二条路径）
    let mut lv = layout_value.clone();
    pdf_layout_schema::migrate_layout(&mut lv, None).map_err(Error::from)?;
    let layout = pdf_layout_schema::LayoutJson::parse_strict(&lv).map_err(Error::from)?;

    // 3. 按数据视图取上下文（§32）
    let data_mode = req.data_mode.clone().unwrap_or_else(|| "sample".to_string());
    let ctx = match data_mode.as_str() {
        "real" => {
            let doc_id = req.doc_id.unwrap_or(0);
            real_data(db, &doc_type, doc_id).await?
        }
        // placeholder / outline 只用于结构检查，直接用样例数据渲染（前端自行叠加占位样式）
        _ => {
            let preset = req.preset.clone().unwrap_or_else(|| "typical".to_string());
            let rows = req.item_rows.unwrap_or(3);
            pdf_doc_meta::sample_context(&doc_type, &preset, rows)
        }
    };

    // 4. 素材物化
    let asset_ids = collect_asset_ids(&layout);
    let barcode_items = pdf_asset_service::collect_barcode_items(&layout, &ctx);
    let (fs, catalog) = pdf_asset_service::materialize(db, &asset_ids, &barcode_items).await?;

    // 5. 编译
    let started = std::time::Instant::now();
    let compiled = pdf_layout_to_typst::compile_layout(&layout, &ctx, &catalog)
        .map_err(Error::from)?;
    // 诊断开关：PDF_DUMP_TYPST=1 时把生成的 Typst 源码落到工作目录，便于排查编译错误
    if std::env::var("PDF_DUMP_TYPST").as_deref() == Ok("1") {
        let _ = std::fs::write("typst_dump.txt", &compiled.source);
    }
    let svgs = crate::modules::system::service::typst_world::compile_to_svg_pages(
        &compiled.source,
        fs,
    )
    .map_err(|e| Error::from(format!("模板编译失败: {}", e)))?;
    let compile_ms = started.elapsed().as_millis() as i64;

    let pages_param = req.pages.clone().unwrap_or_else(|| json!("all"));
    let want_all = pages_param == json!("all");

    if want_all {
        let token = uuid::Uuid::new_v4().to_string();
        // 逐页计算尺寸（pt → px @96dpi）
        let page_list: Vec<Value> = svgs
            .iter()
            .enumerate()
            .map(|(i, _)| {
                json!({
                    "index": i + 1,
                    "url": format!("/api/system/pdf-designer/preview-svg?token={}&page={}", token, i + 1)
                })
            })
            .collect();
        let count = svgs.len();
        preview_cache().insert(token.clone(), svgs);
        Ok(json!({
            "format": "url",
            "token": token,
            "pages": page_list,
            "pageCount": count,
            "pageBreakY": compiled.page_break_y,
            "health": compiled.health,
            "warnings": compiled.warnings,
            "compileMs": compile_ms
        }))
    } else {
        let idx = match &pages_param {
            Value::Number(n) => n.as_u64().unwrap_or(1).max(1) as usize,
            Value::String(s) => s.parse::<usize>().unwrap_or(1).max(1),
            _ => 1,
        };
        let svg = svgs.get(idx - 1).cloned().ok_or_else(|| {
            Error::from(format!("第 {} 页不存在（共 {} 页）", idx, svgs.len()))
        })?;
        Ok(json!({
            "format": "svg",
            "svg": svg,
            "index": idx,
            "pageCount": svgs.len(),
            "pageBreakY": compiled.page_break_y,
            "health": compiled.health,
            "warnings": compiled.warnings,
            "compileMs": compile_ms
        }))
    }
}

/// 从 layout 中收集引用的 assetId
pub fn collect_asset_ids(layout: &pdf_layout_schema::LayoutJson) -> Vec<i64> {
    let mut ids: Vec<i64> = Vec::new();
    let mut push = |v: Option<i64>| {
        if let Some(id) = v {
            if id > 0 && !ids.contains(&id) {
                ids.push(id);
            }
        }
    };
    push(layout.page.background.value.as_deref().and_then(|v| v.parse().ok()));
    push(layout.page.watermark.image.as_deref().and_then(|v| v.parse().ok()));
    push(layout.seam_seal.asset_id);
    for el in layout.all_elements() {
        push(el.prop_i64("assetId"));
        push(el.prop_i64("sealId"));
    }
    ids
}

// ============================================================================
// 上传 PDF 反设计（阶段 A：底图模式，§9）
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsePdfRequest {
    pub asset_id: i64,
    pub doc_type: String,
    pub mode: Option<String>,
    pub page_width_mm: Option<f64>,
    pub page_height_mm: Option<f64>,
}

/// 底图模式：把上传的底图（前端用 pdfjs 渲染出的 PNG）铺满 A4，生成可继续拖拽的草稿。
pub async fn parse_pdf(db: &DbConn, req: ParsePdfRequest) -> Result<Value> {
    let asset = pdf_asset::Entity::find_by_id(req.asset_id)
        .filter(pdf_asset::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("底图素材不存在，请先上传"))?;

    let doc_type = req.doc_type.clone();
    let w = req.page_width_mm.unwrap_or(210.0);
    let h = req.page_height_mm.unwrap_or(297.0);

    let mut layout = pdf_layout_schema::LayoutJson::default();
    layout.page.width = w;
    layout.page.height = h;
    layout.page.background.r#type = "image".to_string();
    layout.page.background.value = Some(asset.id.to_string());
    layout.page.background.fit = "contain".to_string();
    layout.page.background.opacity = 1.0;
    layout.page.background.preview_only = false;
    layout.page.margin = pdf_layout_schema::Margin::default();

    // 预置一个明细表骨架，便于用户直接在底图上对齐表头
    let mut table = pdf_layout_schema::Element::default();
    table.id = "tb_1".to_string();
    table.element_type = "table".to_string();
    table.name = "明细表".to_string();
    table.x = 12.0;
    table.y = h / 2.0;
    table.w = w - 24.0;
    table.h = 60.0;
    table.props = json!({
        "dataSource": "items",
        "showIndex": true,
        "headerRepeat": true,
        "rowHeight": 8,
        "autoGrow": true,
        "headerHeight": 8,
        "emptyText": "无明细记录",
        "columns": [
            { "title": "商品名称", "bind": "item.product_name", "width": "auto", "align": "left" },
            { "title": "数量", "bind": "item.quantity", "width": 20, "align": "right", "format": "qty" },
            { "title": "单价", "bind": "item.price", "width": 28, "align": "right", "format": "money" },
            { "title": "金额", "bind": "item.amount", "width": 30, "align": "right", "format": "money", "total": "sum" }
        ]
    });
    layout.elements.push(table);

    Ok(json!({
        "layoutJson": serde_json::to_value(&layout).unwrap_or(Value::Null),
        "docType": doc_type,
        "suggestions": [],
        "warnings": [
            "当前为底图模式：请直接在底图上拖拽摆放字段并对齐文字。",
            "元素自动识别（文本/线条/表格）为 v2 能力，本期不提供。"
        ]
    }))
}

// ============================================================================
// 模板包导出 / 导入（§33.2）
// ============================================================================

/// 导出：zip(manifest.json + layout.json + assets/{md5}.{ext})
pub async fn export_bundle(db: &DbConn, id: i64) -> Result<(String, Vec<u8>)> {
    use std::io::Write;
    let t = PdfTemplate::find_by_id(id)
        .filter(pdf_template::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("PDF 模板不存在"))?;

    let layout = t
        .layout_json
        .clone()
        .ok_or_else(|| Error::from("该模板没有 layout_json，无法导出模板包"))?;

    let schema_version = layout.get("version").and_then(|v| v.as_i64()).unwrap_or(1);

    let manifest = json!({
        "bundleVersion": 1,
        "templateId": t.id.to_string(),
        "name": t.name,
        "templateCode": t.template_code,
        "docType": t.doc_type,
        "schemaVersion": schema_version,
        "engine": t.engine,
        "exportedAt": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        "appVersion": env!("CARGO_PKG_VERSION")
    });

    // 收集素材
    let mut asset_ids: Vec<i64> = Vec::new();
    if let Ok(layout_parsed) = pdf_layout_schema::LayoutJson::parse_strict(&layout) {
        asset_ids = collect_asset_ids(&layout_parsed);
    }
    let rows = if asset_ids.is_empty() {
        Vec::new()
    } else {
        pdf_asset::Entity::find()
            .filter(pdf_asset::Column::Deleted.eq(0))
            .filter(pdf_asset::Column::Id.is_in(asset_ids))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?
    };

    let mut buf: Vec<u8> = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let opts: zip::write::FileOptions =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("manifest.json", opts)
            .map_err(|e| Error::from(e.to_string()))?;
        zip.write_all(manifest.to_string().as_bytes())
            .map_err(|e| Error::from(e.to_string()))?;

        zip.start_file("layout.json", opts)
            .map_err(|e| Error::from(e.to_string()))?;
        zip.write_all(layout.to_string().as_bytes())
            .map_err(|e| Error::from(e.to_string()))?;

        for row in rows {
            let bytes = pdf_asset_service::read_asset_bytes(row.file_path.as_deref(), &row.file_url);
            if let Some(b) = bytes {
                let md5 = row
                    .md5
                    .clone()
                    .unwrap_or_else(|| format!("{:x}", md5::compute(&b)));
                let ext = row
                    .file_path
                    .as_deref()
                    .or(Some(row.file_url.as_str()))
                    .map(|p| p.rsplit('.').next().unwrap_or("png").to_lowercase())
                    .unwrap_or_else(|| "png".to_string());
                zip.start_file(format!("assets/{}.{}", md5, ext), opts)
                    .map_err(|e| Error::from(e.to_string()))?;
                zip.write_all(&b).map_err(|e| Error::from(e.to_string()))?;
            }
        }
        zip.finish().map_err(|e| Error::from(e.to_string()))?;
    }

    let file_name = format!(
        "{}_{}.pdfdesigner.zip",
        t.template_code.unwrap_or_else(|| format!("tpl{}", t.id)),
        chrono::Local::now().format("%Y%m%d")
    );
    Ok((file_name, buf))
}

/// 导入模板包（multipart 传入 zip 字节）
pub async fn import_bundle(
    db: &DbConn,
    bytes: &[u8],
    operator_id: Option<i64>,
) -> Result<Value> {
    use std::io::Read;
    let reader = std::io::Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(reader).map_err(|e| Error::from(format!("不是合法的 zip: {}", e)))?;

    let mut manifest: Value = Value::Null;
    let mut layout: Value = Value::Null;
    let mut assets: HashMap<String, Vec<u8>> = HashMap::new();

    for i in 0..zip.len() {
        let mut f = zip.by_index(i).map_err(|e| Error::from(e.to_string()))?;
        let name = f.name().to_string();
        let mut content = Vec::new();
        f.read_to_end(&mut content)
            .map_err(|e| Error::from(e.to_string()))?;
        if name == "manifest.json" {
            manifest = serde_json::from_slice(&content).unwrap_or(Value::Null);
        } else if name == "layout.json" {
            layout = serde_json::from_slice(&content)
                .map_err(|e| Error::from(format!("layout.json 解析失败: {}", e)))?;
        } else if let Some(rest) = name.strip_prefix("assets/") {
            assets.insert(rest.to_string(), content);
        }
    }

    if layout.is_null() {
        return Err(Error::from("模板包缺少 layout.json"));
    }
    let bundle_version = manifest
        .get("bundleVersion")
        .and_then(|v| v.as_i64())
        .unwrap_or(1);
    if bundle_version > 1 {
        return Err(Error::from(format!(
            "模板包版本 {} 高于当前系统支持的 1，请先升级系统",
            bundle_version
        )));
    }

    // 素材：md5 去重 + 旧 assetId → 新 id 映射
    let mut id_map: HashMap<String, i64> = HashMap::new();
    let mut fs_dir = "storage/upload/pdf-asset".to_string();
    for (file_name, content) in assets.iter() {
        let md5 = format!("{:x}", md5::compute(content));
        if let Some(existing) = pdf_asset_service::find_by_md5(db, &md5).await? {
            id_map.insert(file_name.clone(), existing);
            continue;
        }
        let ext = file_name.rsplit('.').next().unwrap_or("png").to_string();
        std::fs::create_dir_all(&fs_dir).map_err(|e| Error::from(e.to_string()))?;
        let disk_name = format!("{}.{}", md5, ext);
        let disk_path = format!("{}/{}", fs_dir, disk_name);
        std::fs::write(&disk_path, content).map_err(|e| Error::from(e.to_string()))?;
        let url = format!("/upload/pdf-asset/{}", disk_name);
        let req = pdf_asset_service::PdfAssetSaveRequest {
            id: None,
            name: format!("导入素材 {}", &md5[..8.min(md5.len())]),
            category: "background".to_string(),
            file_url: url,
            file_path: Some(disk_path),
            file_size: Some(content.len() as i64),
            width_px: None,
            height_px: None,
            sort: Some(0),
            status: Some(1),
        };
        let new_id = pdf_asset_service::save(db, req, operator_id).await?;
        id_map.insert(file_name.clone(), new_id);
    }
    let _ = fs_dir;

    // 重写 layout 中的 assetId 引用
    let mut rewritten = layout.clone();
    rewrite_asset_refs(&mut rewritten, &id_map, &assets);

    // 一律生成新模板
    let manifest_name = manifest
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("导入模板")
        .to_string();
    let doc_type = manifest
        .get("docType")
        .and_then(|v| v.as_str())
        .unwrap_or("order")
        .to_string();

    let req = SaveLayoutRequest {
        id: None,
        name: Some(format!("{}（导入）", manifest_name)),
        template_code: None,
        doc_type: Some(doc_type),
        layout_json: rewritten,
        based_on_template_id: None,
        remark: Some(format!(
            "从模板包导入（原 templateId={}）",
            manifest
                .get("templateId")
                .and_then(|v| v.as_str())
                .unwrap_or("-")
        )),
        version: None,
        force: None,
    };
    let r = save_layout(db, req, operator_id).await?;

    let mut warnings: Vec<String> = Vec::new();
    if !pdf_doc_meta::DOC_TYPES
        .iter()
        .any(|(c, _, _)| c == &r["id"].as_str().unwrap_or("").to_string())
    {
        // 占位，真正的 docType 校验在上面
    }
    if assets.is_empty() {
        warnings.push("模板包内不含素材".to_string());
    }
    if id_map.is_empty() && !assets.is_empty() {
        warnings.push("素材已按 md5 去重复用现有素材".to_string());
    }

    Ok(json!({
        "id": r["id"],
        "version": r["version"],
        "assetMap": id_map.iter().map(|(k, v)| (k.clone(), v.to_string())).collect::<HashMap<_, _>>(),
        "warnings": warnings
    }))
}

/// 把 layout 中的 assetId 按 `assets/{md5}.{ext}` 文件名重映射
fn rewrite_asset_refs(layout: &mut Value, id_map: &HashMap<String, i64>, assets: &HashMap<String, Vec<u8>>) {
    // md5 → 新 id
    let mut md5_to_new: HashMap<String, i64> = HashMap::new();
    for (file_name, content) in assets.iter() {
        let md5 = format!("{:x}", md5::compute(content));
        if let Some(new_id) = id_map.get(file_name) {
            md5_to_new.insert(md5, *new_id);
        }
    }

    // 原 assetId 无法直接反查 md5（包里不带原 id），改为：若 layout 引用 id 在当前环境不存在，
    // 则按顺序与包内素材一一对应（导出时保持插入顺序）。
    let mut old_ids: Vec<i64> = Vec::new();
    let mut collect = |v: &Value, ids: &mut Vec<i64>| {
        if let Some(n) = v.as_i64() {
            if n > 0 && !ids.contains(&n) {
                ids.push(n);
            }
        }
    };
    if let Some(obj) = layout.as_object() {
        if let Some(bg) = obj.get("page").and_then(|p| p.get("background")).and_then(|b| b.get("value")) {
            if let Some(s) = bg.as_str() {
                if let Ok(n) = s.parse::<i64>() {
                    collect(&json!(n), &mut old_ids);
                }
            }
        }
    }
    if let Some(els) = layout.get("elements").and_then(|e| e.as_array()) {
        for el in els {
            if let Some(p) = el.get("props") {
                collect(&p.get("assetId").cloned().unwrap_or(Value::Null), &mut old_ids);
                collect(&p.get("sealId").cloned().unwrap_or(Value::Null), &mut old_ids);
            }
        }
    }
    if let Some(ss) = layout.get("seamSeal") {
        collect(&ss.get("assetId").cloned().unwrap_or(Value::Null), &mut old_ids);
    }

    let new_ids: Vec<i64> = md5_to_new.values().copied().collect();
    let mapping: HashMap<i64, i64> = old_ids
        .iter()
        .zip(new_ids.iter())
        .map(|(o, n)| (*o, *n))
        .collect();

    if mapping.is_empty() {
        return;
    }
    let mut apply = |v: &mut Value| {
        if let Some(n) = v.as_i64() {
            if let Some(new_id) = mapping.get(&n) {
                *v = json!(new_id);
            }
        }
    };
    if let Some(bg) = layout
        .get_mut("page")
        .and_then(|p| p.get_mut("background"))
        .and_then(|b| b.get_mut("value"))
    {
        if let Some(s) = bg.as_str() {
            if let Ok(n) = s.parse::<i64>() {
                if let Some(new_id) = mapping.get(&n) {
                    *bg = json!(new_id.to_string());
                }
            }
        }
    }
    if let Some(els) = layout.get_mut("elements").and_then(|e| e.as_array_mut()) {
        for el in els {
            if let Some(p) = el.get_mut("props").and_then(|p| p.as_object_mut()) {
                let mut changed: Vec<(String, Value)> = Vec::new();
                for key in ["assetId", "sealId"] {
                    if let Some(v) = p.get_mut(key) {
                        let before = v.clone();
                        apply(v);
                        if *v != before {
                            changed.push((key.to_string(), v.clone()));
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// 单测
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn layout_with_table() -> Value {
        json!({
            "version": 1,
            "elements": [
                { "id": "t1", "type": "text", "name": "标题", "x": 12, "y": 14, "w": 186, "h": 10,
                  "props": { "content": "销售订单" } },
                { "id": "f1", "type": "field", "name": "订单编号", "x": 12, "y": 30, "w": 90, "h": 6,
                  "bind": { "path": "order.order_no" } },
                { "id": "tb", "type": "table", "name": "明细", "x": 12, "y": 80, "w": 186, "h": 60,
                  "props": { "dataSource": "items", "columns": [
                      { "title": "商品名称", "bind": "item.product_name", "width": "auto" }
                  ] } }
            ]
        })
    }

    #[test]
    fn test_validate_bindings_detects_dangling() {
        let mut l = layout_with_table();
        l["elements"][1]["bind"]["path"] = json!("order.not_exist_field");
        let r = validate_bindings("order", &l).unwrap();
        let issues = r["issues"].as_array().unwrap();
        assert!(issues.iter().any(|i| i["code"] == "dangling"), "{:?}", issues);
    }

    #[test]
    fn test_validate_bindings_detects_unbound() {
        let mut l = layout_with_table();
        l["elements"][1]["bind"]["path"] = json!("");
        let r = validate_bindings("order", &l).unwrap();
        let issues = r["issues"].as_array().unwrap();
        assert!(issues.iter().any(|i| i["code"] == "unbound"));
    }

    #[test]
    fn test_validate_bindings_detects_overflow_in_long_preset() {
        let mut l = layout_with_table();
        l["elements"][1]["w"] = json!(20);
        let r = validate_bindings("order", &l).unwrap();
        let issues = r["issues"].as_array().unwrap();
        // 订单编号在超长样本下远超 20mm
        assert!(
            issues.iter().any(|i| i["code"] == "overflow"),
            "应检出溢出风险: {:?}",
            issues
        );
    }

    #[test]
    fn test_validate_bindings_clean_layout_has_no_error() {
        let l = layout_with_table();
        let r = validate_bindings("order", &l).unwrap();
        assert_eq!(r["summary"]["error"], json!(0), "{:?}", r["issues"]);
    }

    #[test]
    fn test_looks_like_data() {
        assert!(looks_like_data("SO20260915001"));
        assert!(looks_like_data("2026-09-15"));
        assert!(looks_like_data("1,234,567.89"));
        assert!(!looks_like_data("客户名称："));
        assert!(!looks_like_data("销售订单"));
    }

    #[test]
    fn test_collect_asset_ids() {
        let lv = json!({
            "version": 1,
            "page": { "background": { "type": "image", "value": "12" } },
            "elements": [ { "id": "im", "type": "image", "props": { "assetId": 34 } } ]
        });
        let layout = pdf_layout_schema::LayoutJson::parse_strict(&lv).unwrap();
        let ids = collect_asset_ids(&layout);
        assert!(ids.contains(&12));
        assert!(ids.contains(&34));
    }

    #[test]
    fn test_soft_lock_lifecycle() {
        let tid = 900_001;
        assert!(acquire_lock(tid, 1, "张三").is_none(), "首次应抢占成功");
        let conflict = acquire_lock(tid, 2, "李四");
        assert!(conflict.is_some(), "他人应被拒绝");
        assert_eq!(conflict.unwrap().user_name, "张三");
        assert!(heartbeat_lock(tid, 1), "持有者心跳应成功");
        assert!(!heartbeat_lock(tid, 2), "非持有者心跳应失败");
        // 强制接管
        release_lock(tid, 1);
        assert!(acquire_lock(tid, 3, "王五").is_none(), "释放后应可抢占");
    }

    #[test]
    fn test_doc_types_excludes_invoice_layout_design() {
        let v = doc_types();
        let invoice = v
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["code"] == "invoice")
            .unwrap();
        assert!(
            invoice["hint"].as_str().unwrap().contains("票面版式不可修改"),
            "发票必须带合规提示（§20.1）"
        );
    }
}
