//! 合同正文 HTML → Typst 转换器：将富文本编辑器输出的 HTML 转换为 typst 语法。
//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use scraper::node::Node;
use scraper::{ElementRef, Html};

/// 将 HTML 片段转换为 typst 语法，用于合同 PDF 正文渲染。
///
/// 支持 h1-h3、p、strong/b、em/i、ul/ol、br、img、table 等常见富文本标签。
/// 输入内容为空时返回 `Err`。
pub fn convert_html_to_typst(html: &str) -> Result<String, String> {
    if html.trim().is_empty() {
        return Err("HTML 内容为空".to_string());
    }
    let fragment = Html::parse_fragment(html);
    let mut output = String::new();
    let root = fragment.root_element();
    process_block_children(&root, &mut output);
    Ok(output)
}

/// 遍历块级子节点并写入 typst 输出。
///
/// 通过 `ElementRef` 的 `Deref` 目标 `NodeRef<Node>` 调用 `children()`，
/// 可同时拿到元素子节点与文本子节点，从而保留内联文本与标签的先后顺序。
fn process_block_children(element: &ElementRef, output: &mut String) {
    for child in element.children() {
        match child.value() {
            Node::Text(text) => {
                let collapsed = collapse_ws(&**text);
                let trimmed = collapsed.trim();
                if !trimmed.is_empty() {
                    output.push_str(&escape_typst(trimmed));
                    output.push_str("\n\n");
                }
            }
            Node::Element(el) => match el.name() {
                "h1" => {
                    if let Some(er) = ElementRef::wrap(child) {
                        write_heading(&er, output, 1);
                    }
                }
                "h2" => {
                    if let Some(er) = ElementRef::wrap(child) {
                        write_heading(&er, output, 2);
                    }
                }
                "h3" => {
                    if let Some(er) = ElementRef::wrap(child) {
                        write_heading(&er, output, 3);
                    }
                }
                "p" => {
                    if let Some(er) = ElementRef::wrap(child) {
                        write_paragraph(&er, output);
                    }
                }
                "ul" => {
                    if let Some(er) = ElementRef::wrap(child) {
                        write_list(&er, output, false);
                    }
                }
                "ol" => {
                    if let Some(er) = ElementRef::wrap(child) {
                        write_list(&er, output, true);
                    }
                }
                "br" => {
                    output.push('\n');
                }
                "img" => {
                    if let Some(src) = el.attr("src") {
                        output.push_str(&format!("#image(\"{}\")\n\n", src));
                    }
                }
                "table" => {
                    if let Some(er) = ElementRef::wrap(child) {
                        convert_table(&er, output);
                    }
                }
                _ => {
                    if let Some(er) = ElementRef::wrap(child) {
                        process_block_children(&er, output);
                    }
                }
            },
            _ => {}
        }
    }
}

/// 收集元素内的内联内容（文本 + 加粗 + 斜体 + 换行 + 图片），返回 typst 片段。
fn collect_inline(element: &ElementRef) -> String {
    let mut out = String::new();
    collect_inline_children(element, &mut out);
    out
}

fn collect_inline_children(element: &ElementRef, out: &mut String) {
    for child in element.children() {
        match child.value() {
            Node::Text(text) => {
                let collapsed = collapse_ws(&**text);
                if !collapsed.is_empty() {
                    out.push_str(&escape_typst(&collapsed));
                }
            }
            Node::Element(el) => match el.name() {
                "strong" | "b" => {
                    if let Some(er) = ElementRef::wrap(child) {
                        out.push('*');
                        collect_inline_children(&er, out);
                        out.push('*');
                    }
                }
                "em" | "i" => {
                    if let Some(er) = ElementRef::wrap(child) {
                        out.push('_');
                        collect_inline_children(&er, out);
                        out.push('_');
                    }
                }
                "br" => {
                    out.push('\n');
                }
                "img" => {
                    if let Some(src) = el.attr("src") {
                        out.push_str(&format!("#image(\"{}\")", src));
                    }
                }
                _ => {
                    if let Some(er) = ElementRef::wrap(child) {
                        collect_inline_children(&er, out);
                    }
                }
            },
            _ => {}
        }
    }
}

/// 写入标题：`= 标题`、`== 标题`、`=== 标题`。
fn write_heading(element: &ElementRef, output: &mut String, level: usize) {
    let text = collect_inline(element).trim().to_string();
    if text.is_empty() {
        return;
    }
    let prefix = "=".repeat(level);
    output.push_str(&prefix);
    output.push(' ');
    output.push_str(&text);
    output.push_str("\n\n");
}

/// 写入段落：`段落\n\n`。空段落不输出。
fn write_paragraph(element: &ElementRef, output: &mut String) {
    let text = collect_inline(element).trim().to_string();
    if text.is_empty() {
        return;
    }
    output.push_str(&text);
    output.push_str("\n\n");
}

/// 写入列表：无序列表用 `- `，有序列表用 `+ `。
fn write_list(element: &ElementRef, output: &mut String, ordered: bool) {
    let marker = if ordered { '+' } else { '-' };
    for li in element.child_elements() {
        if li.value().name() == "li" {
            let text = collect_inline(&li).trim().to_string();
            if !text.is_empty() {
                output.push(marker);
                output.push(' ');
                output.push_str(&text);
                output.push('\n');
            }
        }
    }
    output.push('\n');
}

/// 将 `<table>` 转换为 typst table 语法，含 `table.header` 自动重复表头。
fn convert_table(table: &ElementRef, output: &mut String) {
    let mut header_cells: Vec<String> = Vec::new();
    let mut data_rows: Vec<Vec<String>> = Vec::new();

    // 遍历所有 <tr>（兼容 thead/tbody 包裹与裸 tr 两种结构）
    for tr in table.descendent_elements() {
        if tr.value().name() != "tr" {
            continue;
        }
        let mut ths: Vec<String> = Vec::new();
        let mut tds: Vec<String> = Vec::new();
        for cell in tr.child_elements() {
            let text = collect_inline(&cell).trim().to_string();
            match cell.value().name() {
                "th" => ths.push(text),
                "td" => tds.push(text),
                _ => {}
            }
        }
        // 取第一行 th 作为表头
        if !ths.is_empty() && header_cells.is_empty() {
            header_cells = ths;
        }
        if !tds.is_empty() {
            data_rows.push(tds);
        }
    }

    let max_data_cols = data_rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let col_count = header_cells.len().max(max_data_cols);
    if col_count == 0 {
        return;
    }

    output.push_str("#table(\n");
    output.push_str(&format!("  columns: {},\n", col_count));
    output.push_str("  align: center,\n");
    output.push_str("  stroke: 0.5pt,\n");

    if !header_cells.is_empty() {
        let header_inner = header_cells
            .iter()
            .map(|c| format!("[*{}*]", c))
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str("  table.header(\n");
        output.push_str(&format!("    {},\n", header_inner));
        output.push_str("  ),\n");
    }

    for row in &data_rows {
        let row_inner = row
            .iter()
            .map(|c| format!("[{}]", c))
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!("  {},\n", row_inner));
    }

    output.push_str(")\n\n");
}

/// 折叠连续空白（含换行/缩进）为单个空格，保留首尾单个空格。
fn collapse_ws(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_ws = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            prev_ws = true;
        } else {
            if prev_ws {
                out.push(' ');
            }
            out.push(ch);
            prev_ws = false;
        }
    }
    if prev_ws {
        out.push(' ');
    }
    out
}

/// 转义 typst 标记模式下的特殊字符，避免正文内容破坏排版。
fn escape_typst(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' | '#' | '$' | '*' | '_' | '[' | ']' | '`' | '~' | '@' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}