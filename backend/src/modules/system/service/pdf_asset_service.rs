//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 可视化 PDF 模板设计器 —— 素材服务。
//!
//! 设计依据：设计文档 §4.2（素材表）、§18（TypstWorld 虚拟 FS）、
//!          §18.3（素材物化流程）、§18.5（缓存）。
//!
//! 素材物化原则：
//! - 编译前把素材读入**内存虚拟 FS**，不落临时目录（§18.4）；
//! - 条码 / 二维码由服务端**即时生成 SVG**，不落库；
//! - 进程内 LRU 缓存素材字节与条码结果（§18.5）。

use crate::core::errors::error::{Error, Result};
use crate::modules::system::entity::pdf_asset::{self, Entity as PdfAsset};
use crate::modules::system::service::pdf_layout_to_typst::AssetCatalog;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DbConn, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

/// 素材分类
pub const CATEGORIES: &[(&str, &str)] = &[
    ("logo", "公司 Logo"),
    ("seal", "电子签章"),
    ("background", "底图 / 预印纸"),
    ("font", "字体"),
    ("barcode", "条码图片"),
];

// ============================================================================
// DTO / VO
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfAssetSaveRequest {
    pub id: Option<i64>,
    pub name: String,
    pub category: String,
    pub file_url: String,
    pub file_path: Option<String>,
    pub file_size: Option<i64>,
    pub width_px: Option<i32>,
    pub height_px: Option<i32>,
    pub sort: Option<i32>,
    pub status: Option<i32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfAssetVO {
    pub id: String,
    pub name: String,
    pub category: String,
    pub file_url: String,
    pub file_path: Option<String>,
    pub file_size: Option<i64>,
    pub md5: Option<String>,
    pub width_px: Option<i32>,
    pub height_px: Option<i32>,
    pub sort: i32,
    pub status: i32,
    pub create_time: Option<String>,
}

impl From<pdf_asset::Model> for PdfAssetVO {
    fn from(m: pdf_asset::Model) -> Self {
        Self {
            id: m.id.to_string(),
            name: m.name,
            category: m.category,
            file_url: m.file_url,
            file_path: m.file_path,
            file_size: m.file_size,
            md5: m.md5,
            width_px: m.width_px,
            height_px: m.height_px,
            sort: m.sort,
            status: m.status,
            create_time: m.create_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

// ============================================================================
// CRUD
// ============================================================================

pub async fn list(
    db: &DbConn,
    category: Option<&str>,
    name: Option<&str>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<PdfAssetVO>, i64)> {
    let mut q = PdfAsset::find().filter(pdf_asset::Column::Deleted.eq(0));
    if let Some(c) = category.filter(|c| !c.is_empty()) {
        q = q.filter(pdf_asset::Column::Category.eq(c));
    }
    if let Some(n) = name.filter(|n| !n.is_empty()) {
        q = q.filter(pdf_asset::Column::Name.contains(n));
    }
    let total = q.clone().count(db).await.map_err(|e| Error::from(e.to_string()))? as i64;
    let rows = q
        .order_by_asc(pdf_asset::Column::Category)
        .order_by_asc(pdf_asset::Column::Sort)
        .order_by_desc(pdf_asset::Column::Id)
        .paginate(db, page_size.max(1) as u64)
        .fetch_page((page.max(1) - 1) as u64)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok((rows.into_iter().map(|m| m.into()).collect(), total))
}

pub async fn save(db: &DbConn, req: PdfAssetSaveRequest, operator_id: Option<i64>) -> Result<i64> {
    if req.name.trim().is_empty() {
        return Err(Error::from("素材名称不能为空"));
    }
    if req.file_url.trim().is_empty() {
        return Err(Error::from("素材文件不能为空"));
    }
    if !CATEGORIES.iter().any(|(c, _)| *c == req.category) {
        return Err(Error::from(format!("未知素材分类: {}", req.category)));
    }

    // 计算 md5（导入去重 / 素材包命名用）
    let md5 = read_asset_bytes(req.file_path.as_deref(), &req.file_url)
        .map(|b| format!("{:x}", md5::compute(&b)));

    if let Some(id) = req.id {
        let existing = PdfAsset::find_by_id(id)
            .filter(pdf_asset::Column::Deleted.eq(0))
            .one(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?
            .ok_or_else(|| Error::from("素材不存在"))?;
        let am = pdf_asset::ActiveModel {
            id: Set(existing.id),
            name: Set(req.name),
            category: Set(req.category),
            file_url: Set(req.file_url),
            file_path: Set(req.file_path),
            file_size: Set(req.file_size),
            md5: Set(md5),
            width_px: Set(req.width_px),
            height_px: Set(req.height_px),
            sort: Set(req.sort.unwrap_or(existing.sort)),
            status: Set(req.status.unwrap_or(existing.status)),
            update_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        PdfAsset::update(am)
            .exec(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        return Ok(id);
    }

    let am = pdf_asset::ActiveModel {
        name: Set(req.name),
        category: Set(req.category),
        file_url: Set(req.file_url),
        file_path: Set(req.file_path),
        file_size: Set(req.file_size),
        md5: Set(md5),
        width_px: Set(req.width_px),
        height_px: Set(req.height_px),
        sort: Set(req.sort.unwrap_or(0)),
        status: Set(req.status.unwrap_or(1)),
        create_by: Set(operator_id),
        create_time: Set(Some(chrono::Local::now().naive_local())),
        update_time: Set(Some(chrono::Local::now().naive_local())),
        ..Default::default()
    };
    let r = PdfAsset::insert(am)
        .exec(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(r.last_insert_id)
}

pub async fn delete(db: &DbConn, ids: &[i64]) -> Result<u64> {
    if ids.is_empty() {
        return Ok(0);
    }
    let r = PdfAsset::update_many()
        .col_expr(pdf_asset::Column::Deleted, Expr::value(1))
        .filter(pdf_asset::Column::Id.is_in(ids.to_vec()))
        .exec(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(r.rows_affected)
}

/// 按 md5 查重（模板包导入用，§33.2）
pub async fn find_by_md5(db: &DbConn, md5: &str) -> Result<Option<i64>> {
    let row = PdfAsset::find()
        .filter(pdf_asset::Column::Deleted.eq(0))
        .filter(pdf_asset::Column::Md5.eq(md5))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(row.map(|r| r.id))
}

// ============================================================================
// 素材物化（§18.3）
// ============================================================================

/// 从磁盘读取素材字节。
///
/// 优先用 `file_path`；否则由 `/upload/xxx` 反推 `storage/upload/xxx`。
pub fn read_asset_bytes(file_path: Option<&str>, file_url: &str) -> Option<Vec<u8>> {
    let candidates: Vec<String> = {
        let mut v = Vec::new();
        if let Some(p) = file_path.filter(|p| !p.is_empty()) {
            v.push(p.to_string());
            if !p.starts_with("./") {
                v.push(format!("./{}", p));
            }
        }
        if !file_url.is_empty() {
            let rel = file_url.trim_start_matches('/');
            let stripped = rel.strip_prefix("upload/").unwrap_or(rel);
            v.push(format!("storage/upload/{}", stripped));
            v.push(format!("./storage/upload/{}", stripped));
        }
        v
    };
    for c in candidates {
        if let Ok(bytes) = std::fs::read(&c) {
            return Some(bytes);
        }
    }
    None
}

/// 归一化扩展名（决定虚拟路径后缀）
fn ext_of(path: &str) -> String {
    let p = path.split('?').next().unwrap_or(path);
    let e = p.rsplit('.').next().unwrap_or("png").to_lowercase();
    match e.as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" | "pdf" => e,
        _ => "png".to_string(),
    }
}

/// 素材字节缓存（§18.5：按 assetId 做进程内 LRU，容量 64MB）
fn bytes_cache() -> &'static moka::sync::Cache<String, Vec<u8>> {
    static CACHE: OnceLock<moka::sync::Cache<String, Vec<u8>>> = OnceLock::new();
    CACHE.get_or_init(|| {
        moka::sync::Cache::builder()
            .max_capacity(64 * 1024 * 1024)
            .weigher(|_k: &String, v: &Vec<u8>| v.len().min(u32::MAX as usize) as u32)
            .build()
    })
}

/// 把模板引用的素材物化到虚拟 FS，并返回 `AssetCatalog`（assetId / elementId → 虚拟路径）
///
/// `barcode_items`: (element_id, kind, payload) —— kind = `code128` | `qrcode`
pub async fn materialize(
    db: &DbConn,
    asset_ids: &[i64],
    barcode_items: &[(String, String, String)],
) -> Result<(HashMap<String, Vec<u8>>, AssetCatalog)> {
    let mut fs: HashMap<String, Vec<u8>> = HashMap::new();
    let mut catalog = AssetCatalog::default();

    if !asset_ids.is_empty() {
        let rows = PdfAsset::find()
            .filter(pdf_asset::Column::Deleted.eq(0))
            .filter(pdf_asset::Column::Id.is_in(asset_ids.to_vec()))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        for row in rows {
            let key = format!("asset:{}", row.id);
            let bytes = match bytes_cache().get(&key) {
                Some(b) => b,
                None => {
                    let path_hint = row.file_path.clone().unwrap_or_else(|| row.file_url.clone());
                    match read_asset_bytes(row.file_path.as_deref(), &row.file_url) {
                        Some(b) => {
                            if b.len() <= 8 * 1024 * 1024 {
                                bytes_cache().insert(key.clone(), b.clone());
                            }
                            b
                        }
                        None => {
                            log::warn!("PDF 素材 {} 文件缺失: {}", row.id, path_hint);
                            continue;
                        }
                    }
                }
            };
            let ext = ext_of(row.file_path.as_deref().unwrap_or(&row.file_url));
            let vpath = format!("assets/{}.{}", row.id, ext);
            fs.insert(vpath.clone(), bytes);
            catalog.by_asset_id.insert(row.id, format!("/{}", vpath));
        }
    }

    // 条码 / 二维码：服务端即时生成 SVG，不落库
    for (element_id, kind, payload) in barcode_items {
        if payload.trim().is_empty() {
            continue;
        }
        let cache_key = format!("bc:{}:{}", kind, payload);
        let svg = match barcode_cache().get(&cache_key) {
            Some(s) => s,
            None => {
                let s = match kind.as_str() {
                    "qrcode" => qrcode_svg(payload),
                    _ => code128_svg(payload, 60.0),
                };
                barcode_cache().insert(cache_key, s.clone());
                s
            }
        };
        let safe_id = element_id.replace(|c: char| !c.is_ascii_alphanumeric() && c != '_', "_");
        let vpath = format!("assets/bc_{}.svg", safe_id);
        fs.insert(vpath.clone(), svg.into_bytes());
        catalog
            .by_element
            .insert(element_id.clone(), format!("/{}", vpath));
    }

    Ok((fs, catalog))
}

fn barcode_cache() -> &'static moka::sync::Cache<String, String> {
    static CACHE: OnceLock<moka::sync::Cache<String, String>> = OnceLock::new();
    CACHE.get_or_init(|| moka::sync::Cache::builder().max_capacity(256).build())
}

// ============================================================================
// 一维码 Code128（纯 Rust 实现，离线可用）
// ============================================================================

/// Code128 编码表（每项 6 位数字表示条/空交替宽度，Stop 为 7 位）
const CODE128_PATTERNS: [&str; 107] = [
    "212222", "222122", "222221", "121223", "121322", "131222", "122213", "122312", "132212",
    "221213", "221312", "231212", "112232", "122132", "122231", "113222", "123122", "123221",
    "223211", "221132", "221231", "213212", "223112", "312131", "311222", "321122", "321221",
    "312212", "322112", "322211", "212123", "212321", "232121", "111323", "131123", "131321",
    "112313", "132113", "132311", "211313", "231113", "231311", "112133", "112331", "132131",
    "113123", "113321", "133121", "313121", "211331", "231131", "213113", "213311", "213131",
    "311123", "311321", "331121", "312113", "312311", "332111", "314111", "221411", "431111",
    "111224", "111422", "121124", "121421", "141122", "141221", "112214", "112412", "122114",
    "122411", "142112", "142211", "241211", "221114", "413111", "241112", "134111", "111242",
    "121142", "121241", "114212", "124112", "124211", "411212", "421112", "421211", "212141",
    "214121", "412121", "111143", "111341", "131141", "114113", "114311", "411113", "411311",
    "113141", "114131", "311141", "411131", "211412", "211214", "211232", "2331112",
];

/// 生成 Code128B 条码 SVG。
///
/// 非 ASCII 字符会被忽略（Code128B 仅覆盖 ASCII 32-126）。
pub fn code128_svg(data: &str, height_px: f64) -> String {
    let module = 2.0f64;
    let mut codes: Vec<usize> = vec![104]; // Start B
    let mut sum = 104i64;
    let mut pos = 1i64;
    for ch in data.chars() {
        let c = ch as u32;
        if (32..=126).contains(&c) {
            let v = (c - 32) as usize;
            codes.push(v);
            sum += v as i64 * pos;
            pos += 1;
        }
    }
    codes.push((sum % 103) as usize); // 校验位
    codes.push(106); // Stop

    let mut x = 0.0f64;
    let mut bars = String::new();
    for code in codes {
        let pattern = CODE128_PATTERNS[code];
        let mut is_bar = true;
        for ch in pattern.chars() {
            let w = ch.to_digit(10).unwrap_or(0) as f64 * module;
            if is_bar {
                bars.push_str(&format!(
                    "<rect x=\"{}\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"#000000\"/>",
                    x, w, height_px
                ));
            }
            x += w;
            is_bar = !is_bar;
        }
    }
    let total_w = x.max(1.0);
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" shape-rendering=\"crispEdges\"><rect x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"#ffffff\"/>{}</svg>",
        total_w, height_px, total_w, height_px, total_w, height_px, bars
    )
}

/// 生成二维码 SVG（`qrcode` crate，纠错等级 M，静默区 2 模块）
pub fn qrcode_svg(data: &str) -> String {
    use qrcode::QrCode;
    match QrCode::new(data.as_bytes()) {
        Ok(code) => {
            let svg = code
                .render::<qrcode::render::svg::Color>()
                .min_dimensions(120, 120)
                .quiet_zone(true)
                .build();
            // 去掉 XML 声明与 DOCTYPE：qrcode crate 会把它们与 <svg 输出在**同一行**，
            // 逐行过滤无效，必须从首个 `<svg` 处截断（Typst 的 SVG 解析器不接受前导声明）。
            match svg.find("<svg") {
                Some(i) => svg[i..].to_string(),
                None => svg,
            }
        }
        Err(e) => {
            log::warn!("二维码生成失败: {}", e);
            String::new()
        }
    }
}

/// 从元素 props 汇总需要生成的条码项
pub fn collect_barcode_items(layout: &crate::modules::system::service::pdf_layout_schema::LayoutJson, ctx: &serde_json::Value) -> Vec<(String, String, String)> {
    use crate::modules::system::service::pdf_layout_to_typst::resolve_path;
    let mut out = Vec::new();
    for el in layout.all_elements() {
        if el.element_type != "barcode" && el.element_type != "qrcode" {
            continue;
        }
        let raw = el
            .prop_str("value")
            .or_else(|| el.bind.path.clone())
            .unwrap_or_default();
        // 支持 `{{order.order_no}}` 形式或裸路径
        let payload = if let Some(inner) = raw
            .trim()
            .strip_prefix("{{")
            .and_then(|s| s.strip_suffix("}}"))
        {
            resolve_path(ctx, inner.trim())
                .map(|v| match v {
                    serde_json::Value::String(s) => s,
                    other => other.to_string(),
                })
                .unwrap_or_default()
        } else if raw.starts_with("http") || raw.contains(' ') {
            raw
        } else {
            resolve_path(ctx, &raw)
                .map(|v| match v {
                    serde_json::Value::String(s) => s,
                    other => other.to_string(),
                })
                .unwrap_or(raw)
        };
        let kind = if el.element_type == "qrcode" {
            "qrcode".to_string()
        } else {
            el.prop_str("format")
                .unwrap_or_else(|| "code128".to_string())
                .to_lowercase()
        };
        out.push((el.id.clone(), kind, payload));
    }
    out
}

/// 素材上传（multipart）——落盘 + 登记。
///
/// 存储路径约定：`{attach.upload_path}/pdf-asset/{YYYY/MM/DD}/{md5}.{ext}`
/// 访问 URL 约定：`{attach.upload_url}/pdf-asset/{YYYY/MM/DD}/{md5}.{ext}`
///
/// 用 md5 命名 → 同图重复上传天然覆盖，不留垃圾文件；并据此做跨记录去重。
pub async fn upload(
    db: &DbConn,
    file: actix_multipart::form::tempfile::TempFile,
    name: Option<String>,
    category: String,
    operator_id: Option<i64>,
) -> Result<PdfAssetVO> {
    use crate::core::kit::config;

    if !CATEGORIES.iter().any(|(c, _)| *c == category) {
        return Err(Error::from(format!("未知素材分类: {}", category)));
    }

    let original_name = file.file_name.clone().unwrap_or_default();
    let bytes = std::fs::read(&file.file)
        .map_err(|e| Error::from(format!("读取上传文件失败: {}", e)))?;
    // 无论成功失败都清理临时文件
    let _ = std::fs::remove_file(&file.file);

    if bytes.is_empty() {
        return Err(Error::from("上传文件为空"));
    }
    // 单文件 20MB 上限（底图 + Logo 足够）
    if bytes.len() > 20 * 1024 * 1024 {
        return Err(Error::from("素材文件过大（上限 20MB）"));
    }

    let md5_hex = format!("{:x}", md5::compute(&bytes));
    let ext = ext_of(&original_name);
    let date_dir = chrono::Local::now().format("%Y/%m/%d").to_string();

    // 已存在同 md5 素材 → 直接复用，不重复落盘
    if let Some(existing_id) = find_by_md5(db, &md5_hex).await? {
        let row = PdfAsset::find_by_id(existing_id)
            .one(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        if let Some(r) = row {
            return Ok(r.into());
        }
    }

    let base = config::section::<String>("attach", "upload_path", "./storage/upload".to_string());
    let base = base.trim_end_matches('/').to_string();
    let url_base = config::section::<String>("attach", "upload_url", "/upload/".to_string());
    let url_base = url_base
        .trim_start_matches('/')
        .trim_end_matches('/')
        .to_string();

    let disk_name = format!("{}.{}", md5_hex, ext);
    let dir = format!("{}/pdf-asset/{}", base, date_dir);
    std::fs::create_dir_all(&dir).map_err(|e| Error::from(format!("创建目录失败: {}", e)))?;
    let disk_path = format!("{}/{}", dir, disk_name);
    std::fs::write(&disk_path, &bytes).map_err(|e| Error::from(format!("文件写入失败: {}", e)))?;

    let file_url = format!("/{}/pdf-asset/{}/{}", url_base, date_dir, disk_name);

    // 图片尺寸（失败不阻断，底图尺寸可由前端补传）
    let (w, h) = image::load_from_memory(&bytes)
        .map(|img| (Some(img.width() as i32), Some(img.height() as i32)))
        .unwrap_or((None, None));

    let display_name = name
        .filter(|n| !n.trim().is_empty())
        .or_else(|| {
            let n = original_name.trim();
            if n.is_empty() {
                None
            } else {
                Some(n.to_string())
            }
        })
        .unwrap_or_else(|| format!("素材 {}", &md5_hex[..8.min(md5_hex.len())]));

    let new_id = save(
        db,
        PdfAssetSaveRequest {
            id: None,
            name: display_name,
            category,
            file_url,
            file_path: Some(disk_path),
            file_size: Some(bytes.len() as i64),
            width_px: w,
            height_px: h,
            sort: Some(0),
            status: Some(1),
        },
        operator_id,
    )
    .await?;

    let row = PdfAsset::find_by_id(new_id)
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("素材登记失败"))?;
    Ok(row.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code128_svg_non_empty() {
        let svg = code128_svg("SO20260915001", 60.0);
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("<rect"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_code128_length_scales_with_content() {
        let a = code128_svg("A", 60.0);
        let b = code128_svg("ABCDEFGHIJ", 60.0);
        let wa: f64 = a
            .split("width=\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let wb: f64 = b
            .split("width=\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        assert!(wb > wa, "更长内容应生成更宽条码: {} vs {}", wb, wa);
    }

    #[test]
    fn test_qrcode_svg_valid() {
        let svg = qrcode_svg("https://example.com/verify/1");
        assert!(svg.starts_with("<svg"), "svg 头异常: {}", &svg[..svg.len().min(60)]);
        assert!(!svg.contains("<?xml"), "应剔除 XML 声明");
    }

    #[test]
    fn test_ext_of() {
        assert_eq!(ext_of("/upload/logo.png"), "png");
        assert_eq!(ext_of("a/b/c.JPG"), "jpg");
        assert_eq!(ext_of("noext"), "png");
        assert_eq!(ext_of("x.svg?v=1"), "svg");
    }
}
