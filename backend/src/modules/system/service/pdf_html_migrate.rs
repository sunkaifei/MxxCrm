//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! P2-5：存量 HTML 模板 → 可视化设计器 layout 草稿迁移器
//!
//! 背景：9 个存量模板的 `content` 实为「Typst + Jinja 混合格式」
//! （`#grid/#table/{{路径}}/{% for %}`，见 §35 前旧出图链路），
//! 非真正 HTML。本模块把该格式**行级解析**为 `layout_json` 草稿：
//!   - `= 标题`                       → 居中大字标题
//!   - `*标签*: {{ a.b }}`            → 绑定字段元素（标签记入元素名）
//!   - `{% for item in items %}...{% endfor %}` → table 明细（列取自 item.* 占位）
//!   - `#image("x")` / 其余文本行      → 图片 / 静态文本
//! 布局为纵向流式（x=左边距，y 依次累加），用户在画布上校正位置后保存。

use serde_json::{json, Value};

use crate::modules::system::service::pdf_layout_schema::{
    Element, LayoutJson, Margin, PageConfig,
};

/// 明细字段中文名（bind 尾段 → 列标题）
fn item_field_title(suffix: &str) -> &'static str {
    match suffix {
        "index" => "序号",
        "product_code" => "商品编码",
        "product_name" => "商品名称",
        "spec" => "规格型号",
        "unit" => "单位",
        "quantity" => "数量",
        "price" | "unit_price" => "单价",
        "amount" | "subtotal" => "金额",
        "tax_rate" => "税率",
        "tax_amount" => "税额",
        "remark" => "备注",
        "hs_code" => "HS Code",
        _ => "明细",
    }
}

fn strip_jinja(s: &str) -> String {
    s.trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim()
        .to_string()
}

/// 抽取行内所有 {{ a.b }} 绑定路径
/// （必须用 `str::find` 定位——按字节下标切 UTF-8 中文字符串会 panic：
///   "end byte index is not a char boundary"，v1.8 实测踩坑）
fn extract_paths(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = s;
    while let Some(start) = rest.find("{{") {
        let after = &rest[start + 2..];
        match after.find("}}") {
            Some(end) => {
                let path = after[..end].trim().to_string();
                if !path.is_empty() {
                    out.push(path);
                }
                rest = &after[end + 2..];
            }
            None => break,
        }
    }
    out
}

/// 解析单张模板 content → layout_json 草稿（页宽/高由调用方按模板配置写入）
pub fn migrate_content_to_layout(content: &str, page: PageConfig) -> Value {
    let m_left = page.margin.left.max(12.0);
    let content_w = (page.width - m_left - page.margin.right.max(12.0)).max(80.0);
    let mut y = page.margin.top.max(12.0);
    let mut elements: Vec<Value> = Vec::new();
    let mut seq = 0usize;
    let mut el = |etype: &str, name: String, x: f64, yy: &mut f64, w: f64, h: f64, props: Value, style: Value| {
        seq += 1;
        let e = json!({
            "id": format!("mig_{}", seq),
            "type": etype,
            "name": name,
            "x": (x * 10.0).round() / 10.0,
            "y": ((*yy) * 10.0).round() / 10.0,
            "w": w,
            "h": h,
            "props": props,
            "style": style,
        });
        *yy += h + 2.0;
        e
    };

    let mut lines = content.lines().peekable();
    while let Some(line) = lines.next() {
        let t = line.trim();
        // 跳过 Typst 指令与空行（#grid/#table/#v/#set/#align 等旧排版指令）
        if t.is_empty() || t.starts_with('#') || t.starts_with("{%") || t.starts_with("//") {
            // #grid( ... ) 多行键值块：提取块内 [*label*: {{ path }}] 字段对 → field 元素
            if t.starts_with("#grid(") {
                let mut depth: i32 = 0;
                let mut block = String::new();
                let mut src: &str = line;
                // 括号平衡收集（跨行），中英文括号内容均按字符计数
                loop {
                    for ch in src.chars() {
                        block.push(ch);
                        match ch {
                            '(' => depth += 1,
                            ')' => depth -= 1,
                            _ => {}
                        }
                    }
                    block.push('\n');
                    if depth <= 0 {
                        break;
                    }
                    match lines.next() {
                        Some(next) => src = next,
                        None => break,
                    }
                }
                // 提取 [*label*: {{ path }}] 模式
                let mut rest: &str = &block;
                while let Some(pos) = rest.find("[*") {
                    let after = &rest[pos + 1..];
                    match after.find(']') {
                        Some(end) => {
                            let cell = &after[..end];
                            let paths = extract_paths(cell);
                            for p in paths {
                                let label = cell
                                    .replace(&format!("{{{{ {} }}}}", p), "")
                                    .trim_matches(|c| "*: ".contains(c))
                                    .replace(':', "")
                                    .trim()
                                    .trim_matches(|c| "* ".contains(c))
                                    .to_string();
                                let name = if label.is_empty() {
                                    p.split('.').next_back().unwrap_or("字段").to_string()
                                } else {
                                    label
                                };
                                elements.push(el(
                                    "field",
                                    name,
                                    m_left,
                                    &mut y,
                                    content_w / 2.0,
                                    6.0,
                                    json!({ "bind": { "path": p, "format": "" } }),
                                    json!({ "fontSize": 10 }),
                                ));
                            }
                            rest = &after[end + 1..];
                        }
                        None => break,
                    }
                }
                continue;
            }
            // for 块单独处理
            if t.starts_with("{% for") {
                // 收集 endfor 前的所有行
                let mut block: Vec<String> = Vec::new();
                for inner in lines.by_ref() {
                    let it = inner.trim();
                    if it.starts_with("{% endfor") {
                        break;
                    }
                    if !it.is_empty() && !it.starts_with('#') {
                        block.push(it.to_string());
                    }
                }
                // 提取 item.* 列
                let mut cols: Vec<Value> = Vec::new();
                for b in &block {
                    for p in extract_paths(b) {
                        if let Some(suf) = p.strip_prefix("item.") {
                            if !cols
                                .iter()
                                .any(|c: &Value| c["bind"] == format!("item.{}", suf))
                            {
                                cols.push(json!({
                                    "title": item_field_title(suf),
                                    "bind": format!("item.{}", suf),
                                    "width": "auto",
                                    "align": if suf == "product_name" { "left" } else { "center" },
                                }));
                            }
                        }
                    }
                }
                if !cols.is_empty() {
                    let table = json!({
                        "dataSource": "items",
                        "columns": cols,
                        "showIndex": false,
                        "headerRepeat": true,
                        "rowHeight": 8,
                        "emptyText": "无明细记录",
                    });
                    let h = 40.0f64.min(120.0).max(cols.len() as f64 * 3.0 + 16.0);
                    elements.push(el(
                        "table",
                        "明细表".to_string(),
                        m_left,
                        &mut y,
                        content_w,
                        h,
                        table,
                        json!({}),
                    ));
                }
            }
            continue;
        }

        // 标题：= / ==
        if let Some(rest) = t.strip_prefix("== ") {
            let title = rest.trim().trim_matches('=').trim();
            elements.push(el(
                "text",
                "标题".to_string(),
                m_left,
                &mut y,
                content_w,
                10.0,
                json!({ "content": title }),
                json!({ "fontSize": 18, "bold": true, "align": "center" }),
            ));
            continue;
        }
        if let Some(rest) = t.strip_prefix("= ") {
            let title = rest.trim().trim_matches('=').trim();
            elements.push(el(
                "text",
                "标题".to_string(),
                m_left,
                &mut y,
                content_w,
                10.0,
                json!({ "content": title }),
                json!({ "fontSize": 16, "bold": true, "align": "center" }),
            ));
            continue;
        }

        // `*标签*: {{ a.b }}` 或含绑定的普通行 → field 元素（标签记入元素名）
        let paths = extract_paths(t);
        if !paths.is_empty() {
            for p in &paths {
                let label = t
                    .replace(&format!("{{{{ {} }}}}", p), "")
                    .trim_matches(|c| "[]*,: ".contains(c))
                    .replace('*', "")
                    .replace(':', "")
                    .trim()
                    .trim_matches(|c| "[]*, ".contains(c))
                    .to_string();
                let name = if label.is_empty() {
                    p.split('.').next_back().unwrap_or("字段").to_string()
                } else {
                    label
                };
                let style = json!({ "fontSize": 10, "bold": t.contains('*') });
                elements.push(el(
                    "field",
                    name.clone(),
                    m_left,
                    &mut y,
                    content_w / 2.0,
                    6.0,
                    json!({ "bind": { "path": p, "format": "" } }),
                    style,
                ));
                let _ = name;
            }
            continue;
        }

        // 图片
        if t.starts_with("#image(") {
            elements.push(el(
                "image",
                "图片".to_string(),
                m_left,
                &mut y,
                40.0,
                20.0,
                json!({}),
                json!({}),
            ));
            continue;
        }

        // 纯文本（去 Typst 修饰：*粗体*、#align 包裹等简单处理）
        let plain = t.replace('*', "");
        if plain.chars().all(|c| !c.is_alphanumeric() && !('\u{4e00}'..='\u{9fff}').contains(&c)) {
            continue;
        }
        let h = 6.0;
        let name = plain.chars().take(10).collect::<String>();
        elements.push(el(
            "text",
            name,
            m_left,
            &mut y,
            content_w,
            h,
            json!({ "content": plain }),
            json!({ "fontSize": 10 }),
        ));
    }

    json!({
        "version": 1,
        "page": {
            "size": page.size,
            "width": page.width,
            "height": page.height,
            "orientation": page.orientation,
            "margin": {
                "top": page.margin.top,
                "right": page.margin.right,
                "bottom": page.margin.bottom,
                "left": page.margin.left,
            },
            "background": { "type": "none", "previewOnly": false },
            "watermark": { "text": null, "opacity": 0.08, "rotate": 45 },
        },
        "elements": elements,
    })
}
