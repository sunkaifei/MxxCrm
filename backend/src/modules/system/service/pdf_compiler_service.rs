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
use minijinja::Environment;

/// 转义 typst 内容模式中的特殊字符
///
/// typst 在 `[...]` 内容模式下，以下字符有特殊含义，需用 `\` 转义：
/// - `#` 进入代码模式
/// - `$` 进入数学公式模式
/// - `*` 粗体定界符
/// - `_` 强调（斜体）定界符
/// - `[` / `]` 内容块定界符
/// - `` ` `` 原始文本定界符
/// - `~` 不换行空格
/// - `\` 转义字符本身
pub fn escape_typst(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => result.push_str("\\\\"),
            '#' => result.push_str("\\#"),
            '$' => result.push_str("\\$"),
            '*' => result.push_str("\\*"),
            '_' => result.push_str("\\_"),
            '[' => result.push_str("\\["),
            ']' => result.push_str("\\]"),
            '`' => result.push_str("\\`"),
            '~' => result.push_str("\\~"),
            '<' => result.push_str("\\<"),
            '>' => result.push_str("\\>"),
            '@' => result.push_str("\\@"),
            '=' => result.push_str("\\="),
            _ => result.push(ch),
        }
    }
    result
}

/// 递归转义 JSON 值中的所有字符串
///
/// 遍历 `serde_json::Value`，对每个字符串字段调用 `escape_typst` 进行转义。
/// 用于在模板渲染前对业务数据进行安全处理，防止数据中的特殊字符破坏 typst 语法。
pub fn escape_typst_in_json(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::String(s) => serde_json::Value::String(escape_typst(&s)),
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(escape_typst_in_json).collect())
        }
        serde_json::Value::Object(map) => {
            let new_map: serde_json::Map<String, serde_json::Value> = map
                .into_iter()
                .map(|(k, v)| (k, escape_typst_in_json(v)))
                .collect();
            serde_json::Value::Object(new_map)
        }
        other => other,
    }
}

/// PDF 页面配置
pub struct PdfPageOptions {
    /// 纸张大小，如 "a4"、"a5"、"letter"
    pub paper_size: String,
    /// 页面方向："portrait"（纵向）或 "landscape"（横向）
    pub orientation: String,
    /// 上边距（pt）
    pub margin_top: i32,
    /// 下边距（pt）
    pub margin_bottom: i32,
    /// 左边距（pt）
    pub margin_left: i32,
    /// 右边距（pt）
    pub margin_right: i32,
    /// 正文字体族名称
    pub font_family: String,
}

impl Default for PdfPageOptions {
    fn default() -> Self {
        Self {
            paper_size: "a4".to_string(),
            orientation: "portrait".to_string(),
            margin_top: 20,
            margin_bottom: 20,
            margin_left: 40,
            margin_right: 40,
            font_family: "Source Han Sans SC".to_string(),
        }
    }
}

/// 用 minijinja 渲染模板（变量插值）
///
/// 将模板内容中的 `{{ variable }}` 占位符替换为 context 中对应的值。
pub fn render_template(template_content: &str, context: &serde_json::Value) -> Result<String> {
    let env = Environment::new();
    let template = env
        .template_from_str(template_content)
        .map_err(|e| Error::from(format!("模板解析失败: {}", e)))?;
    let rendered = template
        .render(context)
        .map_err(|e| Error::from(format!("模板渲染失败: {}", e)))?;
    Ok(rendered)
}

/// 组装完整 typst 源码（页面设置 + 页眉页脚 + 正文）
///
/// 根据页面配置生成 `#set page(...)`、`#set text(...)` 等 typst 指令，
/// 并将页眉、页脚、正文拼接为一段完整的 typst 源码字符串。
pub fn assemble_typst_source(
    body: &str,
    header: &Option<String>,
    footer: &Option<String>,
    opts: &PdfPageOptions,
) -> String {
    let mut src = String::new();

    // 1. 页面设置
    let flipped = if opts.orientation == "landscape" {
        ", flipped: true"
    } else {
        ""
    };
    src.push_str(&format!(
        "#set page(\n  paper: \"{}\",\n  margin: (top: {}pt, bottom: {}pt, left: {}pt, right: {}pt){},\n",
        opts.paper_size,
        opts.margin_top,
        opts.margin_bottom,
        opts.margin_left,
        opts.margin_right,
        flipped
    ));

    // 2. 页眉
    if let Some(h) = header {
        if !h.trim().is_empty() {
            src.push_str(&format!("  header: [ {} ],\n", h));
        }
    }

    // 3. 页脚（默认页码）
    if let Some(f) = footer {
        if !f.trim().is_empty() {
            src.push_str(&format!("  footer: context [ {} ],\n", f));
        }
    } else {
        // typst 0.15: 正确的总页数语法 counter(page).final()
        src.push_str("  footer: context [\n    #set align(center)\n    第 ");
        src.push_str("#counter(page).display(\"1\")");
        src.push_str(" 页 / 共 ");
        src.push_str("#context counter(page).final().first().display(\"1\")");
        src.push_str(" 页\n  ],\n");
    }
    src.push_str(")\n");

    // 4. 字体设置
    src.push_str(&format!(
        "#set text(font: \"{}\", size: 10.5pt)\n",
        opts.font_family
    ));
    src.push_str("#set par(leading: 0.8em, justify: true)\n\n");

    // 5. 正文
    src.push_str(body);

    src
}

/// 完整的 PDF 生成流程：渲染模板 → 组装源码 → 编译PDF
///
/// 依次完成：
/// 1. 用 minijinja 渲染正文模板（变量插值）
/// 2. 用 minijinja 渲染页眉、页脚（如果存在）
/// 3. 组装完整 typst 源码（页面设置 + 页眉页脚 + 正文）
/// 4. 调用 typst 编译为 PDF 字节数组
pub fn generate_pdf_bytes(
    template_content: &str,
    header_content: &Option<String>,
    footer_content: &Option<String>,
    context: &serde_json::Value,
    opts: &PdfPageOptions,
) -> Result<Vec<u8>> {
    // 0. 转义业务数据中的 typst 特殊字符，防止数据内容破坏模板语法
    let escaped_context = escape_typst_in_json(context.clone());

    // 1. minijinja 渲染正文
    let rendered_body = render_template(template_content, &escaped_context)?;

    // 2. minijinja 渲染页眉页脚（如果存在）
    let rendered_header = match header_content {
        Some(h) if !h.trim().is_empty() => Some(render_template(h, &escaped_context)?),
        _ => None,
    };
    let rendered_footer = match footer_content {
        Some(f) if !f.trim().is_empty() => Some(render_template(f, &escaped_context)?),
        _ => None,
    };

    // 3. 组装完整 typst 源码
    // 若模板已自控 #set page（高级模板，含续页header/页码等），则只补充字体设置，不再重复添加page设置
    let typst_source = if rendered_body.contains("#set page(") || rendered_body.contains("#set page (") {
        let mut s = String::new();
        s.push_str(&format!(
            "#set text(font: \"{}\", size: 10.5pt)\n",
            opts.font_family
        ));
        s.push_str("#set par(leading: 0.8em, justify: true)\n\n");
        s.push_str(&rendered_body);
        s
    } else {
        assemble_typst_source(&rendered_body, &rendered_header, &rendered_footer, opts)
    };

    // 4. 编译为 PDF
    let pdf_bytes = crate::modules::system::service::typst_world::compile_to_pdf(&typst_source)
        .map_err(Error::from)?;

    Ok(pdf_bytes)
}
