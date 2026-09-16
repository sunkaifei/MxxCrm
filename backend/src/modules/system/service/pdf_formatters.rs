//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 可视化 PDF 模板设计器 —— 数据格式化器。
//!
//! 设计依据：设计文档 §6.4 与 §21.2。
//!
//! 约定：**格式化一律在后端 context 组装完成后执行**（与既有 `amount_to_chinese` 一致），
//! 前端只存格式化器 key，避免两端格式漂移。

use crate::modules::system::service::pdf_generator_service::amount_to_chinese;
use rust_decimal::Decimal;
use serde_json::Value;

/// 可用格式化器清单（`GET /pdf-designer/formatters`）
pub const FORMATTERS: &[(&str, &str, &str)] = &[
    ("money", "金额(千分位)", "money"),
    ("money_cn", "金额大写(中文)", "money_cn"),
    ("money_en_cn", "金额大写(英文)", "money_en_cn"),
    ("money_currency", "金额(带币种符号)", "money_currency"),
    ("date", "日期(YYYY-MM-DD)", "date"),
    ("date_iso", "日期(ISO)", "date_iso"),
    ("date_en", "日期(英文)", "date_en"),
    ("qty", "数量(去尾零)", "qty"),
    ("rate", "百分比", "rate"),
    ("upper", "大写", "upper"),
    ("lower", "小写", "lower"),
    ("phone", "电话", "phone"),
    ("bank", "银行卡", "bank"),
    ("taxno", "税号", "taxno"),
    ("address_en", "英文地址", "address_en"),
];

/// 按格式化器 key 格式化任意 JSON 值；`fmt` 为 `None` / 未知 key 时走默认渲染。
pub fn format_value(raw: &Value, fmt: Option<&str>) -> String {
    let fmt = fmt.unwrap_or("").trim();
    if fmt.is_empty() {
        return default_render(raw);
    }
    // 支持 `date:YYYY年MM月DD日` 这类带参数的写法
    let (key, arg) = match fmt.split_once(':') {
        Some((k, a)) => (k, Some(a)),
        None => (fmt, None),
    };

    match key {
        "money" => {
            let digits = arg.and_then(|a| a.parse::<usize>().ok()).unwrap_or(2);
            money_str(raw, digits)
        }
        "money_cn" => {
            let d = to_decimal(raw);
            amount_to_chinese(&d)
        }
        "money_en_cn" => {
            let d = to_decimal(raw);
            let words = number_to_english(&d);
            let (_, dec_part) = split_dec(&d);
            let unit = arg.unwrap_or("RMB");
            let cents = format!("{:02}", dec_part);
            format!(
                "SAY {} {} AND CENTS {} ONLY",
                unit.to_uppercase(),
                words,
                cents
            )
            .replace("AND CENTS 00 ONLY", "ONLY")
        }
        "money_currency" => money_str(raw, 2),
        "date" | "date_iso" => match arg {
            Some(a) => format_date(raw, a),
            None => format_date(raw, "YYYY-MM-DD"),
        },
        "date_en" => format_date(raw, "MONTH_DD_YYYY"),
        "qty" => qty_str(raw),
        "rate" => {
            let d = to_decimal(raw);
            format!("{}%", d.round_dp(2).normalize().to_string())
        }
        "upper" => default_render(raw).to_uppercase(),
        "lower" => default_render(raw).to_lowercase(),
        "phone" => default_render(raw),
        "bank" => default_render(raw),
        "taxno" => default_render(raw),
        "address_en" => default_render(raw),
        _ => default_render(raw),
    }
}

/// 无格式化器时的默认渲染：字符串原样、数字去尾零、布尔转为是/否、null 为空串
pub fn default_render(raw: &Value) -> String {
    match raw {
        Value::Null => String::new(),
        Value::Bool(b) => if *b { "是".to_string() } else { "否".to_string() },
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.to_string()
            } else if let Some(f) = n.as_f64() {
                trim_float(f)
            } else {
                n.to_string()
            }
        }
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn trim_float(f: f64) -> String {
    if (f.fract()).abs() < 1e-9 {
        format!("{}", f as i64)
    } else {
        let s = format!("{:.10}", f);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// JSON 值 → Decimal（无法解析时 0）
pub fn to_decimal(raw: &Value) -> Decimal {
    match raw {
        Value::Number(n) => Decimal::try_from(n.as_f64().unwrap_or(0.0)).unwrap_or(Decimal::ZERO),
        Value::String(s) => Decimal::from_str_exact(s.trim()).unwrap_or(Decimal::ZERO),
        _ => Decimal::ZERO,
    }
}

fn money_str(raw: &Value, digits: usize) -> String {
    let d = to_decimal(raw).round_dp(digits as u32);
    thousand_sep(&format!("{:.*}", digits, d))
}

/// 千分位（对 `1234567.89` → `1,234,567.89`）
pub fn thousand_sep(s: &str) -> String {
    let (neg, body) = match s.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, s),
    };
    let (int_part, dec_part) = match body.split_once('.') {
        Some((i, d)) => (i, Some(d)),
        None => (body, None),
    };
    let mut out = String::new();
    let chars: Vec<char> = int_part.chars().collect();
    for (idx, ch) in chars.iter().enumerate() {
        if idx > 0 && (chars.len() - idx) % 3 == 0 {
            out.push(',');
        }
        out.push(*ch);
    }
    let mut result = if neg { format!("-{}", out) } else { out };
    if let Some(d) = dec_part {
        result.push('.');
        result.push_str(d);
    }
    result
}

fn qty_str(raw: &Value) -> String {
    match raw {
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.to_string()
            } else {
                trim_float(n.as_f64().unwrap_or(0.0))
            }
        }
        Value::String(s) => {
            let t = s.trim();
            if let Ok(d) = Decimal::from_str_exact(t) {
                d.normalize().to_string()
            } else {
                t.to_string()
            }
        }
        other => default_render(other),
    }
}

/// 日期格式化。支持：
/// - `YYYY-MM-DD` / `YYYY年MM月DD日` / `YYYY/MM/DD` 等自定义占位
/// - `MONTH_DD_YYYY` → `Sep 15, 2026`
fn format_date(raw: &Value, pattern: &str) -> String {
    let s = match raw {
        Value::String(s) => s.clone(),
        other => default_render(other),
    };
    let s = s.trim();
    if s.is_empty() {
        return String::new();
    }
    // 解析 y-m-d（允许带时间部分）
    let date_part = s.split(['T', ' ']).next().unwrap_or(s);
    let parts: Vec<&str> = date_part.split(['-', '/']).collect();
    if parts.len() < 3 {
        return s.to_string();
    }
    let (y, m, d) = (parts[0], parts[1], parts[2]);
    let mi: usize = m.parse().unwrap_or(0);
    let di: usize = d.parse().unwrap_or(0);

    if pattern == "MONTH_DD_YYYY" {
        const MONTHS: [&str; 12] = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        let mon = if mi >= 1 && mi <= 12 { MONTHS[mi - 1] } else { "" };
        // 英文惯例：Sep 15, 2026（月与日之间无逗号）
        return format!("{} {}, {}", mon, di, y);
    }

    pattern
        .replace("YYYY", y)
        .replace("MM", &format!("{:02}", mi))
        .replace("DD", &format!("{:02}", di))
        .replace('M', &mi.to_string())
        .replace('D', &di.to_string())
}

fn split_dec(d: &Decimal) -> (String, i64) {
    let s = d.abs().round_dp(2).to_string();
    match s.split_once('.') {
        Some((i, dd)) => {
            let cents: i64 = format!("{:0<2}", dd).chars().take(2).collect::<String>().parse().unwrap_or(0);
            (i.to_string(), cents)
        }
        None => (s, 0),
    }
}

/// 金额转英文大写（外贸发票/形式发票必需，设计文档 §21.2）
pub fn number_to_english(d: &Decimal) -> String {
    let (int_str, _) = split_dec(d);
    let n: u128 = int_str.parse().unwrap_or(0);
    int_to_english(n)
}

/// 阿拉伯数字 → 英文单词（支持到万亿级）
pub fn int_to_english(n: u128) -> String {
    const ONES: [&str; 20] = [
        "ZERO", "ONE", "TWO", "THREE", "FOUR", "FIVE", "SIX", "SEVEN", "EIGHT", "NINE", "TEN",
        "ELEVEN", "TWELVE", "THIRTEEN", "FOURTEEN", "FIFTEEN", "SIXTEEN", "SEVENTEEN", "EIGHTEEN",
        "NINETEEN",
    ];
    const TENS: [&str; 10] = [
        "", "", "TWENTY", "THIRTY", "FORTY", "FIFTY", "SIXTY", "SEVENTY", "EIGHTY", "NINETY",
    ];

    fn below_1000(mut n: u128, ones: &[&str; 20], tens: &[&str; 10]) -> String {
        let mut parts: Vec<String> = Vec::new();
        if n >= 100 {
            parts.push(format!("{} HUNDRED", ones[(n / 100) as usize]));
            n %= 100;
        }
        if n >= 20 {
            let t = tens[(n / 10) as usize].to_string();
            let r = n % 10;
            if r > 0 {
                parts.push(format!("{}-{}", t, ones[r as usize]));
            } else {
                parts.push(t);
            }
        } else if n > 0 {
            parts.push(ones[n as usize].to_string());
        }
        parts.join(" ")
    }

    if n == 0 {
        return "ZERO".to_string();
    }

    let groups: [(&str, u128); 4] = [
        ("TRILLION", 1_000_000_000_000),
        ("BILLION", 1_000_000_000),
        ("MILLION", 1_000_000),
        ("THOUSAND", 1_000),
    ];
    let mut out: Vec<String> = Vec::new();
    let mut rest = n;
    for (name, base) in groups {
        if rest >= base {
            let q = rest / base;
            out.push(format!("{} {}", below_1000(q, &ONES, &TENS), name));
            rest %= base;
        }
    }
    if rest > 0 {
        out.push(below_1000(rest, &ONES, &TENS));
    }
    out.join(" ").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_money_thousand_sep() {
        assert_eq!(format_value(&json!(1234567.895), Some("money")), "1,234,567.90");
        assert_eq!(format_value(&json!(200), Some("money")), "200.00");
        assert_eq!(format_value(&json!(-1234.5), Some("money:1")), "-1,234.5");
    }

    #[test]
    fn test_money_cn() {
        assert_eq!(
            format_value(&json!(12345.67), Some("money_cn")),
            "壹万贰仟叁佰肆拾伍元陆角柒分"
        );
    }

    #[test]
    fn test_date_formats() {
        assert_eq!(format_value(&json!("2026-09-15"), Some("date")), "2026-09-15");
        assert_eq!(
            format_value(&json!("2026-09-15"), Some("date:YYYY年MM月DD日")),
            "2026年09月15日"
        );
        assert_eq!(format_value(&json!("2026-09-15"), Some("date_en")), "Sep 15, 2026");
    }

    #[test]
    fn test_qty_trim() {
        assert_eq!(format_value(&json!(2.0), Some("qty")), "2");
        assert_eq!(format_value(&json!(2.5), Some("qty")), "2.5");
        assert_eq!(format_value(&json!("2.500"), Some("qty")), "2.5");
    }

    #[test]
    fn test_default_render_null_and_bool() {
        assert_eq!(default_render(&json!(null)), "");
        assert_eq!(default_render(&json!(true)), "是");
    }

    #[test]
    fn test_int_to_english() {
        assert_eq!(int_to_english(0), "ZERO");
        assert_eq!(int_to_english(21), "TWENTY-ONE");
        assert_eq!(int_to_english(1234), "ONE THOUSAND TWO HUNDRED THIRTY-FOUR");
        assert_eq!(int_to_english(1000000), "ONE MILLION");
    }

    #[test]
    fn test_money_en_cn() {
        let s = format_value(&json!(1234.56), Some("money_en_cn:USD"));
        assert!(s.starts_with("SAY USD"), "实际: {}", s);
        assert!(s.contains("ONE THOUSAND TWO HUNDRED THIRTY-FOUR"), "实际: {}", s);
        assert!(s.contains("AND CENTS 56 ONLY"), "实际: {}", s);
    }
}
