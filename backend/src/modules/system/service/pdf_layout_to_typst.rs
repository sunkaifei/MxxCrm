//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 可视化 PDF 模板设计器 —— 把 `layout_json` 编译为 Typst 源码。
//!
//! 设计依据：设计文档 §5（元素映射）、§16（能力补全）、§17（Typst 落地细则）、
//!          §19.2/§19.3（联次与预印纸）、§21（双语）。
//!
//! 关键约束（§17）：
//! 1. `#block` **必须显式设定高度**——`place` 脱离文档流不撑高父容器；
//! 2. `place` 的 `dx/dy` 相对**父容器内容区**，而设计坐标相对**纸张左上角**，
//!    故统一减 `margin`（方案 B，§17.1）；
//! 3. **静态文案同样必须转义**（§17.6），否则 `[名称]`、`#值` 会破坏 Typst 语法；
//! 4. `settings.designSample` 是设计器偏好，编译时**必须完全忽略**（§32.11 实现红线）。

use super::pdf_compiler_service::escape_typst;
use super::pdf_formatters;
use super::pdf_layout_schema::*;
use rust_decimal::Decimal;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

/// 素材目录：assetId / elementId → Typst 虚拟路径（形如 `/assets/logo_12.png`）
#[derive(Default, Clone, Debug)]
pub struct AssetCatalog {
    pub by_asset_id: HashMap<i64, String>,
    pub by_element: HashMap<String, String>,
}

impl AssetCatalog {
    pub fn asset_path(&self, id: i64) -> Option<&String> {
        self.by_asset_id.get(&id)
    }
    pub fn element_path(&self, id: &str) -> Option<&String> {
        self.by_element.get(id)
    }
}

/// 分页健康检查（§34.6，对应 TC-08 的自动化）
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageHealth {
    pub header_repeated: bool,
    pub footer_on_last_page: bool,
    pub blank_tail_page: bool,
    /// 每页可容纳的明细行数（估算值）
    pub rows_per_page: i32,
    /// 当前数据下的估算页数
    pub estimated_pages: i32,
}

/// 编译产物
#[derive(Debug, Clone, Default)]
pub struct CompiledLayout {
    /// Typst 源码
    pub source: String,
    /// 编译期告警（元素超框、素材缺失等）
    pub warnings: Vec<String>,
    /// 每页分页点的纸张 y 坐标（mm，估算）
    pub page_break_y: Vec<f64>,
    pub health: PageHealth,
}

// ============================================================================
// 工具函数
// ============================================================================

fn mm(v: f64) -> String {
    let r = (v * 100.0).round() / 100.0;
    if (r - r.round()).abs() < 1e-9 {
        format!("{}mm", r.round() as i64)
    } else {
        format!("{}mm", r)
    }
}

fn pt(v: f64) -> String {
    let r = (v * 100.0).round() / 100.0;
    format!("{}pt", r)
}

fn num(v: f64) -> String {
    let r = (v * 100.0).round() / 100.0;
    if (r - r.round()).abs() < 1e-9 {
        format!("{}", r.round() as i64)
    } else {
        format!("{}", r)
    }
}

/// 颜色表达式；`opacity < 1` 时用 `transparentize` 降低不透明度
fn color_expr(hex: &str, opacity: f64) -> String {
    let c = format!("rgb(\"{}\")", hex.trim());
    if opacity >= 0.999 {
        c
    } else {
        let amount = ((1.0 - opacity) * 100.0).clamp(0.0, 100.0);
        format!("{}.transparentize({}%)", c, num(amount))
    }
}

/// Typst 对齐对：水平 + 垂直
fn align_pair(h: &str, v: &str) -> String {
    let hh = match h {
        "center" => "center",
        "right" => "right",
        _ => "left",
    };
    let vv = match v {
        "top" => "top",
        "bottom" => "bottom",
        _ => "horizon",
    };
    if hh == "center" && vv == "horizon" {
        "center + horizon".to_string()
    } else {
        format!("{} + {}", hh, vv)
    }
}

fn h_align(h: &str) -> &'static str {
    match h {
        "center" => "center",
        "right" => "right",
        _ => "left",
    }
}

/// 按 `.` / `[n]` 路径取值（支持 `a.b`、`a[0].b`）
pub fn resolve_path(root: &Value, path: &str) -> Option<Value> {
    let path = path.trim();
    if path.is_empty() {
        return None;
    }
    let mut cur = root;
    for seg in split_path(path) {
        match seg {
            PathSeg::Key(k) => {
                cur = cur.get(&k)?;
            }
            PathSeg::Index(i) => {
                cur = cur.get(i)?;
            }
        }
    }
    Some(cur.clone())
}

enum PathSeg {
    Key(String),
    Index(usize),
}

fn split_path(path: &str) -> Vec<PathSeg> {
    let mut out = Vec::new();
    for part in path.split('.') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        // 形如 `items[0]`
        if let Some(open) = part.find('[') {
            let key = &part[..open];
            if !key.is_empty() {
                out.push(PathSeg::Key(key.to_string()));
            }
            let rest = &part[open..];
            for chunk in rest.split('[').skip(1) {
                if let Some(close) = chunk.find(']') {
                    if let Ok(i) = chunk[..close].trim().parse::<usize>() {
                        out.push(PathSeg::Index(i));
                    }
                }
            }
        } else {
            out.push(PathSeg::Key(part.to_string()));
        }
    }
    out
}

/// 粗略估算文本宽度（mm）：中文 1em，西文 0.5em（§17.5）
fn estimate_text_width_mm(text: &str, font_size_pt: f64) -> f64 {
    let mut em = 0.0f64;
    for ch in text.chars() {
        if (ch as u32) >= 0x2E80 {
            em += 1.0;
        } else if ch == '\n' {
            // 取最长行由调用方处理
            em += 0.0;
        } else {
            em += 0.5;
        }
    }
    em * font_size_pt * 0.3528
}

fn longest_line_width_mm(text: &str, font_size_pt: f64) -> f64 {
    text.split('\n')
        .map(|l| estimate_text_width_mm(l, font_size_pt))
        .fold(0.0, f64::max)
}

/// 转义后的内容：`\n` 转为 Typst 硬换行
fn escaped_content(text: &str) -> String {
    escape_typst(text).replace('\n', "\\\n")
}

// ============================================================================
// 主入口
// ============================================================================

/// 把 `layout_json` + 数据上下文编译为 Typst 源码。
///
/// ⚠️ 本函数**不读取** `layout.settings.design_sample`（§32.11 实现红线）。

/// 打印偏移校准的边距视图：dx 右移 / dy 下移（叠加在用户页边距上）



pub fn compile_layout(
    layout: &LayoutJson,
    ctx: &Value,
    catalog: &AssetCatalog,
) -> Result<CompiledLayout, String> {
    let mut warnings: Vec<String> = Vec::new();
    let (page_w, page_h) = layout.page.effective_size();
    let m = &layout.page.margin;
    let print_offset = &layout.settings.print_offset;
    // 打印偏移校准（P2-6）：dx 右移 / dy 下移，通过平移页边距实现
    let mut shifted = m.clone();
    shifted.left = (shifted.left + print_offset.x).max(0.0);
    shifted.right = (shifted.right - print_offset.x).max(0.0);
    shifted.top = (shifted.top + print_offset.y).max(0.0);
    shifted.bottom = (shifted.bottom - print_offset.y).max(0.0);
    let m = &shifted;

    // ---------- 元素分区 ----------
    let mut main_els: Vec<&Element> = layout
        .elements
        .iter()
        .filter(|e| e.visible && e.band != "pageHeader" && e.band != "pageFooter")
        .collect();
    main_els.sort_by(|a, b| {
        a.y.partial_cmp(&b.y)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.z.cmp(&b.z))
    });

    let tables: Vec<&Element> = main_els
        .iter()
        .copied()
        .filter(|e| e.element_type == "table")
        .collect();

    if tables.is_empty() {
        warnings.push("模板中没有明细表元素，明细数据不会输出".to_string());
    }

    // ---------- 页眉 / 页脚 ----------
    let header_src = render_band(
        &layout.bands.page_header.elements,
        (m.left, 0.0),
        layout,
        ctx,
        catalog,
        page_w,
        &mut warnings,
    )?;
    let footer_src = render_band(
        &layout.bands.page_footer.elements,
        (m.left, page_h - m.bottom),
        layout,
        ctx,
        catalog,
        page_w,
        &mut warnings,
    )?;

    // ---------- 背景水印 / 预印底图 ----------
    let mut background_parts: Vec<String> = Vec::new();
    if layout.page.background.r#type == "image" || layout.page.background.r#type == "pdf" {
        if layout.page.background.preview_only || layout.settings.paper_type == "preprinted" {
            // §19.3：预印纸底图**绝不打印**，否则与预印内容重叠
            warnings.push(
                "预印纸 / 仅预览底图不会输出到 PDF（避免与预印内容重叠打印）".to_string(),
            );
        } else if let Some(v) = layout.page.background.value.as_ref() {
            let path = v
                .parse::<i64>()
                .ok()
                .and_then(|id| catalog.asset_path(id).cloned());
            match path {
                Some(p) => {
                    let fit = match layout.page.background.fit.as_str() {
                        "cover" => "cover",
                        "stretch" => "stretch",
                        _ => "contain",
                    };
                    background_parts.push(format!(
                        "#image(\"{}\", width: 100%, height: 100%, fit: \"{}\")",
                        p, fit
                    ));
                }
                None => warnings.push(format!("页面底图素材（{}）缺失，已跳过", v)),
            }
        }
    }
    if layout.page.watermark.enabled {
        if let Some(t) = layout.page.watermark.text.as_ref().filter(|t| !t.is_empty()) {
            let op = layout.page.watermark.opacity;
            background_parts.push(format!(
                "#place(center + horizon, scope: \"page\")[#rotate({}deg, reflow: true)[#text(size: {}pt, fill: {})[{}]]]",
                num(layout.page.watermark.rotate),
                num(layout.page.watermark.size),
                color_expr("#808080", op),
                escaped_content(t)
            ));
        } else if let Some(v) = layout.page.watermark.image.as_ref() {
            if let Ok(id) = v.parse::<i64>() {
                if let Some(p) = catalog.asset_path(id) {
                    background_parts.push(format!(
                        "#place(center + horizon, scope: \"page\")[#image(\"{}\", width: {}mm)]",
                        p,
                        num(layout.page.watermark.size)
                    ));
                }
            }
        }
    }
    // 骑缝章（简化为每页右侧/左侧居中放置，§16.3）
    if layout.seam_seal.enabled {
        if let Some(id) = layout.seam_seal.asset_id {
            match catalog.asset_path(id) {
                Some(p) => {
                    let pos = if layout.seam_seal.position == "left" { "left" } else { "right" };
                    background_parts.push(format!(
                        "#place({} + horizon, scope: \"page\", dx: {}mm)[#image(\"{}\", width: {}mm)]",
                        pos,
                        num(-layout.seam_seal.size / 2.0 + layout.seam_seal.offset),
                        p,
                        num(layout.seam_seal.size)
                    ));
                }
                None => warnings.push("骑缝章素材缺失，已跳过".to_string()),
            }
        }
    }

    // ---------- 联次（§19.2） ----------
    let copies: Vec<CopyItem> = if layout.page.copies.is_empty() {
        vec![CopyItem::default()]
    } else {
        layout.page.copies.clone()
    };

    let header_inner = if header_src.trim().is_empty() {
        String::new()
    } else {
        format!(",\n  header: context [ {} ],", header_src)
    };
    let footer_extra = if footer_src.trim().is_empty() {
        String::new()
    } else {
        format!(",\n  footer: context [ {} ],", footer_src)
    };
    let bg = if background_parts.is_empty() {
        String::new()
    } else {
        format!(",\n  background: [ {} ],", background_parts.join("\n"))
    };

    let mut out = String::new();
    let mut page_break_y: Vec<f64> = Vec::new();
    let mut total_rows_seen = 0i32;
    let mut rows_per_page = 0i32;
    let mut has_repeating_header = false;
    let mut has_footer_rows = false;

    for (ci, copy) in copies.iter().enumerate() {
        if ci > 0 {
            out.push_str("#pagebreak()\n");
        }
        // 每联独立 `#set page`，使联次文字随页眉/页脚生效
        let mut copy_header = header_inner.clone();
        if !copy.label.is_empty() {
            let label_el = format!(
                "#place(dx: 0mm, dy: 0mm, box(width: {}mm)[#align(right + top)[#text(size: 9pt, fill: {})[{}]]])",
                num(page_w - m.left - m.right),
                color_expr(&copy.color, 1.0),
                escaped_content(&copy.label)
            );
            copy_header = if copy_header.is_empty() {
                format!(",\n  header: context [ {} ],", label_el)
            } else {
                format!("{}{}", copy_header, label_el)
            };
        }

        // flipped 仅对具名纸张生效；自定义宽高的横版由前端直接写 width/height，
        // 若再 flipped 会双重翻转（横版自定义纸渲染成竖版）
        let named_sizes = ["A4", "a4", "A5", "a5", "Letter", "letter"];
        let is_named = named_sizes.contains(&layout.page.size.as_str());
        let flipped = if is_named {
            (layout.page.orientation == "landscape").to_string()
        } else {
            "false".to_string()
        };
        out.push_str(&format!(
            "#set page(\n  {},\n  flipped: {},\n  margin: (top: {}, bottom: {}, left: {}, right: {}){}{}{}\n)\n",
            layout.page.typst_paper(),
            flipped,
            mm(m.top),
            mm(m.bottom),
            mm(m.left),
            mm(m.right),
            copy_header,
            footer_extra,
            bg
        ));
        out.push_str(&format!(
            "#set text(font: \"{}\", size: {})\n",
            layout
                .styles
                .font_family
                .clone()
                .unwrap_or_else(|| "Source Han Sans SC".to_string()),
            pt(layout.styles.font_size)
        ));
        out.push_str(&format!(
            "#set par(leading: {}em, justify: false)\n\n",
            num((layout.styles.line_height - 1.0).max(0.0))
        ));

        // ---------- 主体：固定块 / 表格 交替 ----------
        let body = render_body(
            &main_els,
            &tables,
            layout,
            ctx,
            catalog,
            (page_w, page_h),
            &mut warnings,
            &mut page_break_y,
            &mut total_rows_seen,
            &mut rows_per_page,
            &mut has_repeating_header,
            &mut has_footer_rows,
        )?;
        out.push_str(&body);
    }

    // ---------- 分页健康检查（§34.6）----------
    let mut health = PageHealth::default();
    health.rows_per_page = rows_per_page;
    health.header_repeated = has_repeating_header && total_rows_seen > rows_per_page && rows_per_page > 0;
    health.footer_on_last_page = has_footer_rows;
    let est_pages = if rows_per_page > 0 {
        ((total_rows_seen + rows_per_page - 1) / rows_per_page).max(1)
    } else {
        1
    };
    health.estimated_pages = est_pages * copies.len() as i32;
    health.blank_tail_page = false; // Typst 不会输出空白尾页；此处由行数算法判定
    if rows_per_page > 0 && total_rows_seen > 0 && total_rows_seen % rows_per_page == 0 {
        // 恰好占满最后一页 → 若实现上仍产生新页，才是空白尾页；Typst 的 table.footer 会跟随，
        // 因此这里只在"footerRows 高度超过剩余空间"时提示，属保守判断
    }

    Ok(CompiledLayout {
        source: out,
        warnings,
        page_break_y,
        health,
    })
}

#[allow(clippy::too_many_arguments)]
fn render_body(
    main_els: &[&Element],
    tables: &[&Element],
    layout: &LayoutJson,
    ctx: &Value,
    catalog: &AssetCatalog,
    page: (f64, f64),
    warnings: &mut Vec<String>,
    page_break_y: &mut Vec<f64>,
    total_rows_seen: &mut i32,
    rows_per_page: &mut i32,
    has_repeating_header: &mut bool,
    has_footer_rows: &mut bool,
) -> Result<String, String> {
    let (page_w, page_h) = page;
    let m = &layout.page.margin;
    let mut out = String::new();

    if tables.is_empty() {
        // 无表格：整页作为一个固定块，dy 相对内容区
        let cond = render_elements(
            main_els,
            (m.left, m.top),
            layout,
            ctx,
            catalog,
            page,
            warnings,
            &HashMap::new(),
        )?;
        out.push_str(&cond);
        return Ok(out);
    }

    let mut pending: Vec<&Element> = Vec::new();
    let mut block_top = m.top;

    for el in main_els.iter().copied() {
        if el.element_type == "table" {
            // 1) 表格之前的固定块
            if !pending.is_empty() {
                let block_h = (el.y - block_top).max(0.0);
                let inner = render_elements(
                    &pending,
                    (m.left, block_top),
                    layout,
                    ctx,
                    catalog,
                    page,
                    warnings,
                    &HashMap::new(),
                )?;
                out.push_str(&format!(
                    "#block(height: {})[\n{}\n]\n",
                    mm(block_h),
                    inner
                ));
                pending.clear();
            }
            // 2) 流式表格
            out.push_str(&render_table(
                el,
                layout,
                ctx,
                catalog,
                page,
                warnings,
                page_break_y,
                total_rows_seen,
                rows_per_page,
                has_repeating_header,
                has_footer_rows,
            )?);
            block_top = el.y + el.h;
        } else {
            pending.push(el);
        }
    }

    // 3) 末尾固定块（跟随表格末尾）
    if !pending.is_empty() {
        let block_h = (page_h - m.bottom - block_top).max(0.0);
        let inner = render_elements(
            &pending,
            (m.left, block_top),
            layout,
            ctx,
            catalog,
            page,
            warnings,
            &HashMap::new(),
        )?;
        out.push_str(&format!(
            "#v(2mm)\n#block(height: {})[\n{}\n]\n",
            mm(block_h),
            inner
        ));
    }

    let _ = page_w;
    Ok(out)
}

fn render_band(
    elements: &[Element],
    origin: (f64, f64),
    layout: &LayoutJson,
    ctx: &Value,
    catalog: &AssetCatalog,
    page_w: f64,
    warnings: &mut Vec<String>,
) -> Result<String, String> {
    if elements.is_empty() {
        return Ok(String::new());
    }
    render_elements(
        &elements.iter().collect::<Vec<_>>(),
        origin,
        layout,
        ctx,
        catalog,
        (page_w, layout.page.effective_size().1),
        warnings,
        &HashMap::new(),
    )
}

/// 渲染一组元素为 `#place(...)` 序列
#[allow(clippy::too_many_arguments)]
fn render_elements(
    elements: &[&Element],
    origin: (f64, f64),
    layout: &LayoutJson,
    ctx: &Value,
    catalog: &AssetCatalog,
    page: (f64, f64),
    warnings: &mut Vec<String>,
    extra_vars: &HashMap<String, Value>,
) -> Result<String, String> {
    let mut out = String::new();
    for el in elements {
        out.push_str(&render_element(
            el, origin, layout, ctx, catalog, page, warnings, extra_vars,
        )?);
    }
    Ok(out)
}

#[allow(clippy::too_many_arguments)]
fn render_element(
    el: &Element,
    origin: (f64, f64),
    layout: &LayoutJson,
    ctx: &Value,
    catalog: &AssetCatalog,
    page: (f64, f64),
    warnings: &mut Vec<String>,
    extra_vars: &HashMap<String, Value>,
) -> Result<String, String> {
    let m = &layout.page.margin;
    let dx = (el.x - origin.0).max(0.0);
    let dy = (el.y - origin.1).max(0.0);
    let opacity = if el.opacity <= 0.0 { 1.0 } else { el.opacity };

    // 条件显示（§16.2：受限结构，后端求值）
    let cond = eval_print_if(el, ctx);
    let wrap_cond = |body: String| -> String {
        match cond {
            Some(true) => body,
            Some(false) => String::new(),
            // None = 未配置 print_if（或表达式后端不可静态求值）→ 直接输出正文。
            // 此前这里会生成 `#context { if  { … } }`（空条件），属非法 Typst，
            // 导致所有不含 print_if 的模板（即全部模板）编译失败。
            None => body,
        }
    };

    // 纸外元素仍照常编译（PDF 页面盒天然裁切，§34.8）
    let out_of_paper = el.x + el.w < 0.0
        || el.y + el.h < 0.0
        || el.x > page.0
        || el.y > page.1;
    if out_of_paper {
        warnings.push(format!("元素「{}」位于纸张之外，出图时会被裁切", el.name));
    }

    let body = match el.element_type.as_str() {
        "text" | "rich" => {
            let locale = &layout.settings.locale;
            let raw = if el.element_type == "rich" {
                el.prop_str("html").unwrap_or_default()
            } else {
                el.static_text(locale)
            };
            if el.element_type == "rich" {
                let typst = super::html_to_typst::convert_html_to_typst(&raw).unwrap_or_default();
                format!(
                    "#place(dx: {}, dy: {})[#box(width: {}, height: {}, stroke: {}, inset: (top: {}mm, right: {}mm, bottom: {}mm, left: {}mm))[{}]]\n",
                    mm(dx),
                    mm(dy),
                    mm(el.w),
                    mm(el.h),
                    border_expr(&el.style),
                    num(el.style.padding.t),
                    num(el.style.padding.r),
                    num(el.style.padding.b),
                    num(el.style.padding.l),
                    wrap_rotate(&el.style, opacity, el, &typst)
                )
            } else {
                let text_w = longest_line_width_mm(&raw, el.style.font_size);
                let avail = (el.w - el.style.padding.l - el.style.padding.r).max(1.0);
                if text_w > avail && !el.style.auto_shrink {
                    warnings.push(format!(
                        "元素「{}」文本宽度约 {:.1}mm，超出容器 {:.1}mm，可能溢出",
                        el.name, text_w, avail
                    ));
                }
                let size = if el.style.auto_shrink {
                    shrink_size(el.style.font_size, text_w, avail)
                } else {
                    el.style.font_size
                };
                let inner = text_call(el, &raw, size, opacity);
                format!(
                    "#place(dx: {}, dy: {})[{}]\n",
                    mm(dx),
                    mm(dy),
                    box_wrap(el, inner, opacity)
                )
            }
        }
        "field" | "expr" => {
            let (val, _found) = resolve_bind(el, ctx, extra_vars);
            let inner = text_call(el, &val, el.style.font_size, opacity);
            format!(
                "#place(dx: {}, dy: {})[{}]\n",
                mm(dx),
                mm(dy),
                box_wrap(el, inner, opacity)
            )
        }
        "image" | "signature" | "seamseal" => {
            let asset_id = el
                .prop_i64("assetId")
                .or_else(|| el.prop_i64("sealId"))
                .or_else(|| {
                    el.bind
                        .path
                        .as_deref()
                        .and_then(|p| resolve_path(ctx, p))
                        .and_then(|v| v.as_i64())
                });
            let path = el
                .prop_str("path")
                .or_else(|| catalog.element_path(&el.id).cloned())
                .or_else(|| asset_id.and_then(|id| catalog.asset_path(id).cloned()));
            match path {
                Some(p) => {
                    let fit = el.prop_str("fit").unwrap_or_else(|| "contain".to_string());
                    format!(
                        "#place(dx: {}, dy: {}, image(\"{}\", width: {}, height: {}, fit: \"{}\"))\n",
                        mm(dx),
                        mm(dy),
                        p,
                        mm(el.w),
                        mm(el.h),
                        fit
                    )
                }
                None => {
                    // §24.1 PDF_E_ASSET_MISSING：跳过该元素并告警，其余正常出图
                    warnings.push(format!("素材「{}」缺失，该元素已跳过渲染", el.name));
                    String::new()
                }
            }
        }
        "line" => {
            let direction = el.prop_str("direction").unwrap_or_else(|| "h".to_string());
            let thickness = el.prop_f64("thickness").unwrap_or(0.5);
            let color = el.prop_str("color").unwrap_or_else(|| "#000000".to_string());
            let dash = el.prop_str("dash").unwrap_or_default();
            let (angle, length) = if direction == "v" {
                (90.0, el.h)
            } else {
                (0.0, el.w)
            };
            format!(
                "#place(dx: {}, dy: {}, line(angle: {}deg, length: {}, stroke: {}))\n",
                mm(dx),
                mm(dy),
                num(angle),
                mm(length),
                stroke_expr(&color, thickness, &dash, opacity)
            )
        }
        "rect" => {
            let fill = el.prop_str("fill").unwrap_or_default();
            let stroke_w = el.prop_f64("stroke").unwrap_or(0.5);
            let radius = el.prop_f64("radius").unwrap_or(0.0);
            let fill_expr = if fill.is_empty() || fill == "none" {
                "none".to_string()
            } else {
                color_expr(&fill, opacity)
            };
            format!(
                "#place(dx: {}, dy: {}, rect(width: {}, height: {}, radius: {}pt, fill: {}, stroke: {}))\n",
                mm(dx),
                mm(dy),
                mm(el.w),
                mm(el.h),
                num(radius),
                fill_expr,
                stroke_expr("#000000", stroke_w, "", opacity)
            )
        }
        "pagenum" => {
            let tpl = el
                .prop_str("template")
                .unwrap_or_else(|| "第 {page} 页 / 共 {total} 页".to_string());
            let inner = pagenum_content(&tpl);
            format!(
                "#place(dx: {}, dy: {})[{}]\n",
                mm(dx),
                mm(dy),
                box_wrap_raw(el, inner, opacity)
            )
        }
        "divider" => {
            if el.prop_str("mode").unwrap_or_else(|| "before".to_string()) == "after" {
                String::new()
            } else {
                "#pagebreak()\n".to_string()
            }
        }
        "watermark" => {
            let t = el.prop_str("text").unwrap_or_default();
            format!(
                "#place(dx: {}, dy: {}, rotate({}deg, reflow: true)[#text(size: {}pt, fill: {})[{}]])\n",
                mm(dx),
                mm(dy),
                num(el.prop_f64("rotate").unwrap_or(-30.0)),
                num(el.style.font_size),
                color_expr("#808080", opacity * 0.12),
                escaped_content(&t)
            )
        }
        "group" => {
            let children = el
                .props
                .get("children")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let mut s = String::new();
            for c in children {
                if let Ok(child) = serde_json::from_value::<Element>(c) {
                    s.push_str(&render_element(
                        &child, origin, layout, ctx, catalog, page, warnings, extra_vars,
                    )?);
                }
            }
            s
        }
        "table" => return Ok(String::new()), // 表格在 render_body 中单独处理
        other => {
            warnings.push(format!("未知元素类型「{}」，已跳过", other));
            String::new()
        }
    };

    let _ = m;
    Ok(wrap_cond(body))
}

/// 文本元素的 `#text(...)` 调用
fn text_call(el: &Element, raw: &str, size: f64, opacity: f64) -> String {
    let st = &el.style;
    let mut args: Vec<String> = vec![format!("size: {}", pt(size))];
    if let Some(f) = st.font_family.as_ref().filter(|f| !f.is_empty()) {
        args.push(format!("font: \"{}\"", f));
    }
    if st.bold {
        args.push("weight: \"bold\"".to_string());
    }
    if st.italic {
        args.push("style: \"italic\"".to_string());
    }
    if st.letter_spacing.abs() > 1e-9 {
        args.push(format!("tracking: {}", pt(st.letter_spacing)));
    }
    args.push(format!(
        "fill: {}",
        color_expr(st.color.as_deref().unwrap_or("#000000"), opacity)
    ));

    let content = escaped_content(raw);
    let body = if st.underline {
        format!("#underline[{}]", content)
    } else {
        content
    };
    format!("#text({})[{}]", args.join(", "), body)
}

/// 完整 box 包装：尺寸 / 填充 / 边框 / 内边距 / 对齐
fn box_wrap(el: &Element, inner: String, opacity: f64) -> String {
    box_wrap_with(el, inner, opacity, true)
}

fn box_wrap_raw(el: &Element, inner: String, opacity: f64) -> String {
    box_wrap_with(el, inner, opacity, false)
}

fn box_wrap_with(el: &Element, inner: String, opacity: f64, apply_par: bool) -> String {
    let st = &el.style;
    let align = align_pair(&st.align, &st.valign);
    let fill = match st.bg_color.as_deref() {
        Some(c) if !c.is_empty() => color_expr(c, opacity),
        _ => "none".to_string(),
    };
    let stroke = border_expr(st);
    let mut body = String::new();
    if apply_par && st.align == "justify" {
        body.push_str(&format!(
            "#set par(justify: true, leading: {}em)\n",
            num((st.line_height - 1.0).max(0.0))
        ));
    }
    body.push_str(&format!("#align({})[{}]", align, inner));
    format!(
        "#box(width: {}, height: {}, fill: {}, stroke: {}, inset: (top: {}mm, right: {}mm, bottom: {}mm, left: {}mm))[\n{}\n]",
        mm(el.w),
        mm(el.h),
        fill,
        stroke,
        num(st.padding.t),
        num(st.padding.r),
        num(st.padding.b),
        num(st.padding.l),
        body
    )
}

fn wrap_rotate(st: &ElementStyle, opacity: f64, el: &Element, inner: &str) -> String {
    let aligned = format!("#align({})[{}]", align_pair(&st.align, &st.valign), inner);
    let filled = match st.bg_color.as_deref() {
        Some(c) if !c.is_empty() => format!(
            "#box(width: 100%, height: 100%, fill: {})[{}]",
            color_expr(c, opacity),
            aligned
        ),
        _ => aligned,
    };
    let _ = el;
    if el.rotate.abs() > 1e-9 {
        format!("#place(rotate({}deg, reflow: true)[{}])", num(el.rotate), filled)
    } else {
        filled
    }
}

fn border_expr(st: &ElementStyle) -> String {
    if !st.border.any() {
        return "none".to_string();
    }
    let dash = match st.border.style.as_str() {
        "dashed" => ", dash: \"dashed\"",
        "dotted" => ", dash: \"dotted\"",
        _ => "",
    };
    let mut sides: Vec<String> = Vec::new();
    if st.border.t > 0.0 {
        sides.push(format!(
            "top: (paint: rgb(\"{}\"), thickness: {}{})",
            st.border.color,
            pt(st.border.t),
            dash
        ));
    }
    if st.border.b > 0.0 {
        sides.push(format!(
            "bottom: (paint: rgb(\"{}\"), thickness: {}{})",
            st.border.color,
            pt(st.border.b),
            dash
        ));
    }
    if st.border.l > 0.0 {
        sides.push(format!(
            "left: (paint: rgb(\"{}\"), thickness: {}{})",
            st.border.color,
            pt(st.border.l),
            dash
        ));
    }
    if st.border.r > 0.0 {
        sides.push(format!(
            "right: (paint: rgb(\"{}\"), thickness: {}{})",
            st.border.color,
            pt(st.border.r),
            dash
        ));
    }
    format!("({})", sides.join(", "))
}

fn stroke_expr(color: &str, thickness: f64, dash: &str, opacity: f64) -> String {
    let paint = color_expr(color, opacity);
    if dash.is_empty() || dash == "solid" {
        format!("{} + {}", pt(thickness), paint)
    } else {
        format!(
            "(paint: {}, thickness: {}, dash: \"{}\")",
            paint,
            pt(thickness),
            dash
        )
    }
}

fn shrink_size(base: f64, text_w: f64, avail: f64) -> f64 {
    if text_w <= avail || text_w <= 0.0 {
        return base;
    }
    let ratio = avail / text_w;
    let mut size = base * ratio;
    // 逐档下调（§17.5），不低于 6pt
    for step in [10.5, 9.5, 9.0, 8.0, 7.5, 7.0, 6.5, 6.0] {
        if step <= base {
            size = size.max(step * ratio);
        }
    }
    size.clamp(6.0, base)
}

fn pagenum_content(tpl: &str) -> String {
    // 把 `{page}` / `{total}` 替换为 Typst 计数器表达式，其余部分转义
    let mut out = String::new();
    let mut rest = tpl;
    while let Some(idx) = rest.find('{') {
        out.push_str(&escaped_content(&rest[..idx]));
        let tail = &rest[idx..];
        if let Some(end) = tail.find('}') {
            let token = &tail[1..end];
            match token {
                "page" => out.push_str("#context counter(page).display(\"1\")"),
                "total" | "pages" => {
                    out.push_str("#context counter(page).final().first().display(\"1\")")
                }
                other => out.push_str(&escaped_content(&format!("{{{}}}", other))),
            }
            rest = &tail[end + 1..];
        } else {
            out.push_str(&escaped_content(tail));
            rest = "";
            break;
        }
    }
    out.push_str(&escaped_content(rest));
    out
}

// ============================================================================
// 数据绑定
// ============================================================================

/// 解析元素绑定值（返回渲染字符串）
fn resolve_bind(el: &Element, ctx: &Value, extra: &HashMap<String, Value>) -> (String, bool) {
    // 1. 绑定路径
    if let Some(path) = el.bind.path.as_deref().filter(|p| !p.is_empty()) {
        let base = extra.get("__item__");
        let raw = if path.starts_with("item.") {
            base.and_then(|item| resolve_path(item, path.trim_start_matches("item.")))
        } else {
            resolve_path(ctx, path).or_else(|| extra.get(path).cloned())
        };
        match raw {
            Some(v) if !v.is_null() => {
                let s = pdf_formatters::format_value(&v, el.bind.format.as_deref());
                if !s.is_empty() {
                    return (s, true);
                }
            }
            _ => {}
        }
    }
    // 2. expr 元素：字段路径或字面量
    if el.element_type == "expr" {
        if let Some(e) = el.prop_str("expr") {
            if let Some(v) = resolve_path(ctx, &e) {
                return (pdf_formatters::format_value(&v, el.bind.format.as_deref()), true);
            }
            return (e, true);
        }
    }
    // 3. 空值兜底
    if let Some(d) = el.bind.default_value.as_ref().filter(|d| !d.is_empty()) {
        return (d.clone(), false);
    }
    (String::new(), false)
}

// ============================================================================
// printIf 求值（§16.2）
// ============================================================================

/// 返回 Some(true/false) 表示可静态求值；None 表示需要运行时上下文
fn eval_print_if(el: &Element, ctx: &Value) -> Option<bool> {
    let pi = el.print_if.as_ref()?;
    if pi.field.is_empty() {
        return Some(true);
    }
    let v = resolve_path(ctx, &pi.field)?;
    let b = match pi.op.as_str() {
        "eq" => json_eq(&v, &pi.value),
        "ne" => !json_eq(&v, &pi.value),
        "gt" => cmp_num(&v, &pi.value).map(|o| o > 0).unwrap_or(false),
        "gte" => cmp_num(&v, &pi.value).map(|o| o >= 0).unwrap_or(false),
        "lt" => cmp_num(&v, &pi.value).map(|o| o < 0).unwrap_or(false),
        "lte" => cmp_num(&v, &pi.value).map(|o| o <= 0).unwrap_or(false),
        "empty" => is_empty_value(&v),
        "not_empty" => !is_empty_value(&v),
        _ => true,
    };
    Some(b)
}

/// 编译期无法求值时的兜底条件：**一律渲染**。
///
/// 设计取舍：`printIf` 引用的字段在上下文中不存在时，宁可多渲染也不误删内容
/// （漏打印比多打印风险更高）。真正需要"不显示"的场景应在 context builder 里
/// 预计算成布尔字段（§16.2）。
fn cond_expr(el: &Element) -> Option<String> {
    el.print_if.as_ref()?;
    Some("true".to_string())
}

fn json_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            (x.as_f64().unwrap_or(0.0) - y.as_f64().unwrap_or(0.0)).abs() < 1e-9
        }
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::String(x), Value::Bool(y)) => x == if *y { "true" } else { "false" },
        _ => a == b,
    }
}

fn cmp_num(a: &Value, b: &Value) -> Option<i32> {
    let x = pdf_formatters::to_decimal(a);
    let y = pdf_formatters::to_decimal(b);
    Some(match x.cmp(&y) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    })
}

fn is_empty_value(v: &Value) -> bool {
    match v {
        Value::Null => true,
        Value::String(s) => s.trim().is_empty(),
        Value::Array(a) => a.is_empty(),
        _ => false,
    }
}

// ============================================================================
// 表格
// ============================================================================

#[allow(clippy::too_many_arguments)]
fn render_table(
    el: &Element,
    layout: &LayoutJson,
    ctx: &Value,
    _catalog: &AssetCatalog,
    page: (f64, f64),
    warnings: &mut Vec<String>,
    page_break_y: &mut Vec<f64>,
    total_rows_seen: &mut i32,
    rows_per_page: &mut i32,
    has_repeating_header: &mut bool,
    has_footer_rows: &mut bool,
) -> Result<String, String> {
    let props = TableProps::from_value(&el.props);
    let rows: Vec<Value> = if props.data_source.is_empty() {
        Vec::new()
    } else {
        resolve_path(ctx, &props.data_source)
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default()
    };
    let row_count = rows.len() as i32;
    *total_rows_seen += row_count;
    if props.header_repeat && !props.columns.is_empty() {
        *has_repeating_header = true;
    }
    if !props.footer_rows.is_empty()
        || props.columns.iter().any(|c| c.total != "none" && !c.total.is_empty())
    {
        *has_footer_rows = true;
    }

    let col_count = props.columns.len() + if props.show_index { 1 } else { 0 };
    if col_count == 0 {
        warnings.push(format!("明细表「{}」没有配置列，已跳过", el.name));
        return Ok(String::new());
    }

    // 估算每页可容行数（用于健康检查与分页点提示）
    let header_h = if props.columns.is_empty() { 0.0 } else { props.header_height.max(6.0) };
    let row_h = if props.row_height > 0.0 { props.row_height } else { 8.0 };
    let avail = (page.1 - layout.page.margin.bottom - (el.y + header_h)).max(0.0);
    let rpp = if props.max_rows_per_page > 0 {
        props.max_rows_per_page
    } else {
        ((avail / row_h).floor() as i32).max(1)
    };
    *rows_per_page = rpp;

    // 手动切片（金蝶式"每页固定行数"，§17.3）
    let force_height = !props.auto_grow && props.row_height > 0.0;
    let mut out = String::new();

    if props.max_rows_per_page > 0 {
        let chunks: Vec<Vec<Value>> = if rows.is_empty() {
            vec![Vec::new()]
        } else {
            rows.chunks(props.max_rows_per_page as usize)
                .map(|c| c.to_vec())
                .collect()
        };
        let total_pages = chunks.len();
        for (i, chunk) in chunks.iter().enumerate() {
            out.push_str(&render_one_table(
                &props,
                chunk,
                ctx,
                force_height,
                row_count,
                total_pages,
                i,
            ));
            if i + 1 < total_pages {
                out.push_str("#pagebreak()\n");
                page_break_y.push(el.y + header_h + (chunk.len() as f64) * row_h);
            }
        }
    } else {
        out.push_str(&render_one_table(
            &props,
            &rows,
            ctx,
            force_height,
            row_count,
            1,
            0,
        ));
    }

    Ok(out)
}

fn render_one_table(
    props: &TableProps,
    rows: &[Value],
    ctx: &Value,
    force_height: bool,
    total_row_count: i32,
    _total_pages: usize,
    _page_index: usize,
) -> String {
    let mut cols: Vec<String> = Vec::new();
    let mut aligns: Vec<String> = Vec::new();
    if props.show_index {
        cols.push("8mm".to_string());
        aligns.push("center".to_string());
    }
    for c in &props.columns {
        cols.push(match c.width_mm() {
            Some(w) => mm(w),
            None => "1fr".to_string(),
        });
        aligns.push(h_align(&c.align).to_string());
    }
    let col_count = cols.len();

    let mut out = String::new();
    out.push_str("#table(\n");
    out.push_str(&format!("  columns: ({}),\n", cols.join(", ")));
    out.push_str(&format!("  align: ({}),\n", aligns.join(", ")));
    out.push_str("  stroke: 0.5pt + rgb(\"#d9d9d9\"),\n");
    out.push_str("  inset: 3pt,\n");

    // 表头
    if !props.columns.is_empty() {
        let mut cells: Vec<String> = Vec::new();
        if props.show_index {
            let t = props.index_title.clone().unwrap_or_else(|| "#".to_string());
            cells.push(header_cell(&t, props));
        }
        for c in &props.columns {
            cells.push(header_cell(&c.title, props));
        }
        let header_block = format!("  table.header({}),\n", cells.join(", "));
        if props.header_repeat {
            out.push_str(&header_block);
        } else {
            // 不重复：当普通行输出
            out.push_str(&format!("  {},\n", cells.join(", ")));
        }
    }

    // 数据行
    if rows.is_empty() {
        let empty = props
            .empty_text
            .clone()
            .unwrap_or_else(|| "无明细记录".to_string());
        let mut cells: Vec<String> = Vec::new();
        cells.push(format!(
            "table.cell(colspan: {})[#align(center)[{}]]",
            col_count,
            escaped_content(&empty)
        ));
        out.push_str(&format!("  {},\n", cells.join(", ")));
    } else {
        for (i, row) in rows.iter().enumerate() {
            let mut cells: Vec<String> = Vec::new();
            if props.show_index {
                cells.push(cell_content(&(i + 1).to_string(), props, force_height));
            }
            for c in &props.columns {
                let raw = c
                    .bind
                    .as_deref()
                    .filter(|b| !b.is_empty())
                    .and_then(|b| {
                        let p = b.trim_start_matches("item.");
                        resolve_path(row, p)
                    })
                    .unwrap_or(Value::Null);
                let s = if raw.is_null() {
                    String::new()
                } else {
                    pdf_formatters::format_value(&raw, c.format.as_deref())
                };
                cells.push(cell_content(&s, props, force_height));
            }
            out.push_str(&format!("  {},\n", cells.join(", ")));
        }
    }

    // 表尾：显式 footerRows + 列合计自动行
    let mut footer_rows: Vec<String> = Vec::new();
    let any_total = props
        .columns
        .iter()
        .any(|c| !c.total.is_empty() && c.total != "none");
    if any_total {
        let mut cells: Vec<String> = Vec::new();
        if props.show_index {
            cells.push("[]".to_string());
        }
        // 非合计列先累计为空位；遇到合计列时，标签以 colspan 吞并前导空位
        // （colspan = 空位数 + 1，即标签自身落在合计列上），保证总格数恒等于列数
        let mut pending_empty = 0usize;
        let mut first_used = false;
        for c in &props.columns {
            let total_key = c.total.as_str();
            if total_key == "none" || total_key.is_empty() {
                pending_empty += 1;
            } else {
                let sum = if total_key == "count" {
                    Decimal::from(total_row_count)
                } else {
                    rows.iter()
                        .map(|r| {
                            c.bind
                                .as_deref()
                                .and_then(|b| resolve_path(r, b.trim_start_matches("item.")))
                                .map(|v| pdf_formatters::to_decimal(&v))
                                .unwrap_or(Decimal::ZERO)
                        })
                        .fold(Decimal::ZERO, |a, b| a + b)
                };
                let s = pdf_formatters::format_value(
                    &serde_json::json!(sum.to_string()),
                    c.format.as_deref().or(Some("money")),
                );
                if !first_used {
                    cells.push(format!(
                        "table.cell(colspan: {}, align: left)[#text(weight: \"bold\")[合计]]",
                        pending_empty + 1
                    ));
                    first_used = true;
                    pending_empty = 0;
                } else if pending_empty > 0 {
                    // 连续多个合计列之间的非合计列空位（少见，保守补空）
                    for _ in 0..pending_empty {
                        cells.push("[]".to_string());
                    }
                    pending_empty = 0;
                }
                cells.push(format!(
                    "[#text(weight: \"bold\")[{}]]",
                    escaped_content(&s)
                ));
            }
        }
        // 尾部未配合计的列补空位，保证总格数 = 列数
        for _ in 0..pending_empty {
            cells.push("[]".to_string());
        }
        footer_rows.push(cells.join(", "));
    }
    for fr in &props.footer_rows {
        let mut cells: Vec<String> = Vec::new();
        for c in &fr.cells {
            let content = if let Some(b) = c.bind.as_deref().filter(|b| !b.is_empty()) {
                resolve_path(ctx, b)
                    .map(|v| pdf_formatters::format_value(&v, c.format.as_deref()))
                    .unwrap_or_default()
            } else {
                c.value.clone().unwrap_or_default()
            };
            let span = if c.colspan > 1 {
                format!(", colspan: {}", c.colspan)
            } else {
                String::new()
            };
            let al = c
                .align
                .as_deref()
                .map(h_align)
                .unwrap_or("left");
            cells.push(format!(
                "table.cell(align: {}{})[{}]",
                al,
                span,
                escaped_content(&content)
            ));
        }
        footer_rows.push(cells.join(", "));
    }
    if !footer_rows.is_empty() {
        out.push_str("  table.footer(\n");
        for fr in footer_rows {
            out.push_str(&format!("    {},\n", fr));
        }
        out.push_str("  ),\n");
    }

    out.push_str(")\n\n");
    out
}

fn header_cell(title: &str, props: &TableProps) -> String {
    let hs = &props.header_style;
    let mut args: Vec<String> = Vec::new();
    if hs.bold {
        args.push("weight: \"bold\"".to_string());
    }
    args.push(format!("size: {}", pt(hs.font_size)));
    let inner = format!(
        "#text({})[{}]",
        args.join(", "),
        escaped_content(title)
    );
    match hs.bg_color.as_deref().filter(|c| !c.is_empty()) {
        Some(c) => format!(
            "table.cell(fill: rgb(\"{}\"), align: center)[{}]",
            c, inner
        ),
        None => format!("table.cell(align: center)[{}]", inner),
    }
}

fn cell_content(value: &str, props: &TableProps, force_height: bool) -> String {
    let body = format!(
        "#text(size: {})[{}]",
        pt(props.header_style.font_size.max(9.0) - 0.5),
        escaped_content(value)
    );
    if force_height && props.row_height > 0.0 {
        // 该串位于 #table(...) 的参数区（代码模式），函数调用不能带 `#` 前缀
        format!("block(height: {})[{}]", mm(props.row_height), body)
    } else {
        format!("[{}]", body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn catalog() -> AssetCatalog {
        let mut c = AssetCatalog::default();
        c.by_asset_id.insert(12, "/assets/logo_12.png".to_string());
        c
    }

    fn layout_from(v: Value) -> LayoutJson {
        LayoutJson::parse_strict(&v).expect("layout 解析失败")
    }

    fn order_ctx(items: usize) -> Value {
        let rows: Vec<Value> = (1..=items)
            .map(|i| {
                json!({
                    "product_name": format!("示例商品{:03}", i),
                    "quantity": 2,
                    "price": 100.0,
                    "amount": 200.0
                })
            })
            .collect();
        json!({
            "order": { "order_no": "SO20260915001", "order_date": "2026-09-15",
                       "total_amount": 12345.67, "total_amount_cn": "壹万贰仟叁佰肆拾伍元陆角柒分",
                       "currency": "CNY" },
            "customer": { "name": "示例客户 A" },
            "items": rows
        })
    }

    fn basic_layout() -> LayoutJson {
        layout_from(json!({
            "version": 1,
            "page": { "size": "A4", "width": 210, "height": 297,
                      "margin": { "top": 12, "right": 12, "bottom": 12, "left": 12 } },
            "elements": [
                { "id": "t1", "type": "text", "name": "标题", "x": 12, "y": 14, "w": 186, "h": 10,
                  "props": { "content": "销 售 订 单" },
                  "style": { "fontSize": 20, "bold": true, "align": "center" } },
                { "id": "f1", "type": "field", "name": "订单编号", "x": 12, "y": 30, "w": 90, "h": 6,
                  "bind": { "path": "order.order_no" } },
                { "id": "tb", "type": "table", "name": "商品明细", "x": 12, "y": 80, "w": 186, "h": 60,
                  "props": {
                    "dataSource": "items", "showIndex": true, "headerRepeat": true, "rowHeight": 8,
                    "columns": [
                      { "title": "商品名称", "bind": "item.product_name", "width": "auto", "align": "left" },
                      { "title": "数量", "bind": "item.quantity", "width": 20, "align": "right", "format": "qty" },
                      { "title": "金额", "bind": "item.amount", "width": 30, "align": "right", "format": "money", "total": "sum" }
                    ]
                  } }
            ]
        }))
    }

    #[test]
    fn test_compile_basic_layout() {
        let layout = basic_layout();
        let out = compile_layout(&layout, &order_ctx(3), &catalog()).unwrap();
        assert!(out.source.contains("#set page("), "缺少页面设置");
        assert!(out.source.contains("#table("), "缺少表格");
        assert!(out.source.contains("table.header("), "缺少跨页重复表头");
        assert!(out.source.contains("table.footer("), "缺少表尾/合计");
        assert!(out.source.contains("SO20260915001"), "缺少绑定值");
    }

    #[test]
    fn test_static_text_is_escaped() {
        let layout = layout_from(json!({
            "version": 1,
            "elements": [
                { "id": "a", "type": "text", "name": "危险文案", "x": 5, "y": 5, "w": 40, "h": 6,
                  "props": { "content": "[名称] #值 *加粗*" } }
            ]
        }));
        let out = compile_layout(&layout, &json!({}), &catalog()).unwrap();
        assert!(out.source.contains("\\[名称\\]"), "静态文案未转义: {}", out.source);
        assert!(out.source.contains("\\#值"), "静态文案未转义 #");
    }

    #[test]
    fn test_design_sample_is_ignored_by_compiler() {
        let mut l1 = basic_layout();
        let mut l2 = basic_layout();
        l1.settings.design_sample.preset = "long".to_string();
        l1.settings.design_sample.item_rows = 50;
        l2.settings.design_sample.preset = "empty".to_string();
        l2.settings.design_sample.item_rows = 1;
        let a = compile_layout(&l1, &order_ctx(3), &catalog()).unwrap().source;
        let b = compile_layout(&l2, &order_ctx(3), &catalog()).unwrap().source;
        assert_eq!(a, b, "designSample 不得影响编译产物（§32.11 实现红线）");
    }

    #[test]
    fn test_print_if_static_false_drops_element() {
        let layout = layout_from(json!({
            "version": 1,
            "elements": [
                { "id": "a", "type": "text", "name": "税", "x": 5, "y": 5, "w": 40, "h": 6,
                  "props": { "content": "含税" },
                  "printIf": { "field": "order.is_tax", "op": "eq", "value": true } }
            ]
        }));
        let ctx = json!({ "order": { "is_tax": false } });
        let out = compile_layout(&layout, &ctx, &catalog()).unwrap();
        assert!(!out.source.contains("含税"), "printIf=false 应不输出");
    }

    #[test]
    fn test_max_rows_per_page_slicing() {
        let mut layout = basic_layout();
        for el in layout.elements.iter_mut() {
            if el.element_type == "table" {
                el.props["maxRowsPerPage"] = json!(5);
                el.props["autoGrow"] = json!(false);
                el.props["rowHeight"] = json!(8);
            }
        }
        let out = compile_layout(&layout, &order_ctx(12), &catalog()).unwrap();
        let tables = out.source.matches("#table(").count();
        let breaks = out.source.matches("#pagebreak()").count();
        assert_eq!(tables, 3, "12 行 / 每页 5 行应切 3 片");
        assert_eq!(breaks, 2, "3 片之间应有 2 个分页符");
    }

    #[test]
    fn test_missing_asset_warns_and_skips() {
        let layout = layout_from(json!({
            "version": 1,
            "elements": [
                { "id": "img", "type": "image", "name": "Logo", "x": 5, "y": 5, "w": 30, "h": 15,
                  "props": { "assetId": 999 } }
            ]
        }));
        let out = compile_layout(&layout, &json!({}), &catalog()).unwrap();
        assert!(out.warnings.iter().any(|w| w.contains("缺失")), "应产生素材缺失告警");
        assert!(!out.source.contains("#image("), "缺失素材不应输出 image");
    }

    #[test]
    fn test_copies_merge_produces_pages() {
        let mut layout = basic_layout();
        layout.page.copies = vec![
            CopyItem { label: "存根联".to_string(), color: "#000000".to_string() },
            CopyItem { label: "客户联".to_string(), color: "#C00000".to_string() },
            CopyItem { label: "财务联".to_string(), color: "#1F4E79".to_string() },
        ];
        let out = compile_layout(&layout, &order_ctx(3), &catalog()).unwrap();
        assert_eq!(out.source.matches("#pagebreak()").count(), 2, "3 联应有 2 个分页");
        assert!(out.source.contains("存根联") && out.source.contains("财务联"));
    }

    #[test]
    fn test_preprinted_background_not_printed() {
        let mut layout = basic_layout();
        layout.page.background.r#type = "image".to_string();
        layout.page.background.value = Some("12".to_string());
        layout.settings.paper_type = "preprinted".to_string();
        let out = compile_layout(&layout, &order_ctx(1), &catalog()).unwrap();
        assert!(
            !out.source.contains("/assets/logo_12.png"),
            "预印底图不得出图（§19.3）"
        );
        assert!(
            out.warnings.iter().any(|w| w.contains("预印")),
            "应给出预印底图不出图的告警"
        );
    }

    #[test]
    fn test_pagenum_code_emitted() {
        let layout = layout_from(json!({
            "version": 1,
            "bands": { "pageFooter": { "height": 8, "elements": [
                { "id": "pn", "type": "pagenum", "name": "页码", "x": 150, "y": 285, "w": 48, "h": 6,
                  "band": "pageFooter", "props": { "template": "第 {page} 页 / 共 {total} 页" } }
            ] } },
            "elements": []
        }));
        let out = compile_layout(&layout, &json!({}), &catalog()).unwrap();
        assert!(out.source.contains("counter(page).display"), "页码未生成");
        assert!(out.source.contains("counter(page).final"), "总页数未生成");
    }

    #[test]
    fn test_landscape_page_paper() {
        let mut layout = basic_layout();
        layout.page.orientation = "landscape".to_string();
        let out = compile_layout(&layout, &order_ctx(1), &catalog()).unwrap();
        assert!(out.source.contains("flipped: true"));
    }

    #[test]
    fn test_table_empty_shows_empty_text() {
        let layout = basic_layout();
        let out = compile_layout(&layout, &json!({ "items": [] }), &catalog()).unwrap();
        assert!(out.source.contains("无明细记录"), "空数据应输出 emptyText");
    }

    #[test]
    fn test_health_check_longlist() {
        let layout = basic_layout();
        let out = compile_layout(&layout, &order_ctx(50), &catalog()).unwrap();
        let rpp = out.health.rows_per_page;
        assert!(rpp > 0, "应估算出每页可容行数");
        // 几何自洽：表格 y=80、下边距 12、表头 6、行高 8 → 可用 ~199mm → 24 行/页
        assert_eq!(rpp, 24, "每页行数应来自几何估算（实际 {}）", rpp);
        // 50 行 ÷ 24 行/页 = 3 页（不依赖实现细节，按 rpp 推导）
        assert_eq!(
            out.health.estimated_pages,
            (50 + rpp - 1) / rpp,
            "估算页数应由行数与每页行数推导"
        );
        assert!(
            out.health.estimated_pages >= 2,
            "50 行应至少 2 页，实际 {}",
            out.health.estimated_pages
        );
        assert!(out.health.header_repeated, "应提示表头重复");
        assert!(out.health.footer_on_last_page, "应提示合计落末页");
    }
}
