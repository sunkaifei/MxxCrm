//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use std::num::ParseIntError;
use actix_multipart::form::text::Text;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serializer};
use serde::de::Error;

/// 字符串中汉字的个数
pub fn chinese_string_count(chinese_count: String) -> i32 {
    let mut count = 0;
    for c in chinese_count.chars() {
        if is_chinese_character(c) {
            count += 1;
        }
    }
    return count;
}

/// 判断一个字符是否为有效的汉字
fn is_chinese_character(c: char) -> bool {
    ('\u{4E00}' <= c && c <= '\u{9FFF}') || ('\u{3400}' <= c && c <= '\u{4DBF}')
}

///u64转为字符
pub fn u64_to_string<S>(value: &i64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

///i64转为字符
pub fn i64_to_string<S>(value: &i64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

///字符转为u64
pub fn string_to_u64<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<i64>().map_err(serde::de::Error::custom)
}

///字符串或数字转为Option<i32>（兼容空字符串）
pub fn deserialize_string_to_i32<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
    where
        D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrI32 {
        String(String),
        I32(i32),
        Null,
    }

    let value: Option<StringOrI32> = Option::deserialize(deserializer)?;
    
    match value {
        Some(StringOrI32::String(s)) => {
            if s.trim().is_empty() {
                Ok(None)
            } else {
                Ok(s.parse::<i32>().ok())
            }
        }
        Some(StringOrI32::I32(v)) => Ok(Some(v)),
        Some(StringOrI32::Null) => Ok(None),
        None => Ok(None),
    }
}

///字符串或数字转为Option<f64>（兼容空字符串）
pub fn deserialize_string_to_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
    where
        D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrF64 {
        String(String),
        F64(f64),
        Null,
    }

    let value: Option<StringOrF64> = Option::deserialize(deserializer)?;
    
    match value {
        Some(StringOrF64::String(s)) => {
            if s.trim().is_empty() {
                Ok(None)
            } else {
                Ok(s.parse::<f64>().ok())
            }
        }
        Some(StringOrF64::F64(v)) => Ok(Some(v)),
        Some(StringOrF64::Null) => Ok(None),
        None => Ok(None),
    }
}

pub fn serialize_option_i32_to_string<S>(
    value: &Option<i32>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match value {
        Some(val) => serializer.serialize_str(&val.to_string()),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize_string_to_u32<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt_string: Option<String> = Option::deserialize(deserializer)?;
    let opt_u64: Option<u32> = match opt_string {
        Some(s) => Some(s.parse().unwrap_or_default()),
        None => None,
    };
    Ok(opt_u64)
}

///字符串/数字转为<Option<i64>
pub fn deserialize_string_to_u64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;

    match Option::<Value>::deserialize(deserializer)? {
        Some(Value::String(s)) => match s.parse::<i64>() {
            Ok(num) => Ok(Some(num)),
            Err(_) => Ok(None),
        },
        Some(Value::Number(n)) => Ok(n.as_i64()),
        Some(_) => Err(D::Error::custom("expected string or number")),
        None => Ok(None),
    }
}

///字符串转为<Option<i64>
pub fn deserialize_string_to_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(val) => match val.parse::<i64>() {
            Ok(num) => Ok(Some(num)),
            Err(_) => Ok(None), // 或者返回默认值 Some(0)
        },
        None => Ok(None),
    }
}

pub fn serialize_option_u64_to_string<S>(
    value: &Option<i64>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match value {
        Some(val) => serializer.serialize_str(&val.to_string()),
        None => serializer.serialize_none(),
    }
}

pub fn serialize_option_i64_to_string<S>(
    value: &Option<i64>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match value {
        Some(val) => serializer.serialize_str(&val.to_string()),
        None => serializer.serialize_none(),
    }
}

pub fn serialize_naive_date_to_string<S>(
    value: &chrono::NaiveDate,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.format("%Y-%m-%d").to_string())
}

pub fn serialize_option_naive_date_to_string<S>(
    value: &Option<chrono::NaiveDate>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match value {
        Some(val) => serializer.serialize_str(&val.format("%Y-%m-%d").to_string()),
        None => serializer.serialize_none(),
    }
}

/// 反序列化 NaiveDateTime，兼容 ISO 8601（T分隔）和 %Y-%m-%d %H:%M:%S（空格分隔）格式
pub fn deserialize_naive_date_time<'de, D>(deserializer: D) -> Result<chrono::NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S") {
        return Ok(dt);
    }
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S") {
        return Ok(dt);
    }
    Err(D::Error::custom(format!("无效的日期时间格式: {}", s)))
}

/// 反序列化 Option<NaiveDateTime>，兼容多种格式
pub fn deserialize_option_naive_date_time<'de, D>(deserializer: D) -> Result<Option<chrono::NaiveDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(val) => {
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&val, "%Y-%m-%d %H:%M:%S") {
                return Ok(Some(dt));
            }
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&val, "%Y-%m-%dT%H:%M:%S") {
                return Ok(Some(dt));
            }
            Ok(None)
        }
        None => Ok(None),
    }
}

pub fn deserialize_string_vec_to_u64_vec<'de, D>(deserializer: D) -> Result<Option<Vec<i64>>, D::Error>
where
    D: Deserializer<'de>,
{
    let vec: Option<Vec<String>> = Option::deserialize(deserializer)?;
    if let Some(v) = vec {
        let result: Result<Vec<i64>, _> = v.into_iter().map(|s| s.parse::<i64>().map_err(D::Error::custom)).collect();
        result.map(Some)
    } else {
        Ok(None)
    }
}


/// 将 Vec<Option<String>> 转换为 Vec<i64>
pub fn convert_vec_option_string_to_vec_u64(input: Vec<Option<String>>) -> Vec<i64> {
    input.into_iter()
        .filter_map(|opt| opt) // 过滤掉 None 并提取 Some(String)
        .filter_map(|s| s.parse::<i64>().ok()) // 尝试将 String 转换为 i64，过滤掉解析失败的情况
        .collect()
}


///获取第一个汉字
pub fn get_first_chinese_character(s: &str) -> Option<char> {
    for c in s.chars() {
        if is_chinese_character(c) {
            return Some(c);
        }
    }
    None
}

///判断字符串不能有除汉字以外的字符，字符串数必须大于0小于2
pub fn is_valid_string(s: &str) -> bool {
    let re = Regex::new(r"^[\u{4e00}-\u{9fff}]{1,2}$").unwrap();
    re.is_match(s)
}

///数字的五行分类
///按照数理进行五行分类：尾数为1，2五行为木；尾数为3，4五行为火；尾数为5，6五行为土；尾数为7，8五行为金；尾数为9，0五行为水；
pub fn wuxing_classification(num: i32) -> &'static str {
    let last_digit = num % 10;
    match last_digit {
        1 | 2 => "木",
        3 | 4 => "火",
        5 | 6 => "土",
        7 | 8 => "金",
        9 | 0 => "水",
        _ => "未知五行", // 处理其他情况，例如超出范围的数字
    }
}

// 这个函数将被调用来序列化 Option<String> 类型的值。
pub fn serialize_option_string<S>(value: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
{
    match value {
        Some(s) => serializer.serialize_str(s),
        None => serializer.serialize_str(""),
    }
}

///分析客户端操作系统
pub fn user_agent_os(user_agent: &str) -> Option<&str> {
    // 使用模式匹配来检查包含的操作系统信息
    match user_agent {
        ua if ua.contains("Windows NT 10.0") => Some("Windows 10"),
        ua if ua.contains("Windows NT 6.2") => Some("Windows 8"),
        ua if ua.contains("Windows NT 6.1") => Some("Windows 7"),
        ua if ua.contains("Windows") => Some("Windows"),
        ua if ua.contains("iPhone OS 16") => Some("iOS 16"),
        ua if ua.contains("iPhone OS 15") => Some("iOS 15"),
        ua if ua.contains("iPhone OS 14") => Some("iOS 14"),
        ua if ua.contains("iPhone OS 13") => Some("iOS 13"),
        ua if ua.contains("iPhone OS 12") => Some("iOS 12"),
        ua if ua.contains("iPhone OS 11") => Some("iOS 11"),
        ua if ua.contains("iPhone OS 10") => Some("iOS 10"),
        ua if ua.contains("Android 14") => Some("Android 14"),
        ua if ua.contains("Android 13") => Some("Android 13"),
        ua if ua.contains("Android 12") => Some("Android 12"),
        ua if ua.contains("Android 11") => Some("Android 11"),
        ua if ua.contains("Android 10") => Some("Android 10"),
        ua if ua.contains("Android 9") => Some("Android 9"),
        ua if ua.contains("Android 8") => Some("Android 8"),
        ua if ua.contains("Mac") || ua.contains("macOS") => Some("macOS"),
        ua if ua.contains("Linux") => Some("Linux"),
        _ => Some("OthersOS"),
    }
}

///获取浏览器和版本号
pub fn user_agent_browser(user_agent: &str) -> Option<String> {
    let re = Regex::new(r#".*(Edg|Safari|Chrome|Firefox)/(\d+(\.\d+)+)$"#).unwrap();

    if let Some(captures) = re.captures(user_agent) {
        let browser = captures.get(1).unwrap().as_str();
        let version = captures.get(2).unwrap().as_str();
        Some(format!("{} {}", browser, version))
    } else {
        Some("Others".to_string())
    }
}

///将下划线转换为点
/// 例如："hello_world" => "hello.world"
pub fn convert_to_dot_notation(input: Option<&str>) -> Option<String> {
    if let Some(value) = input {
        let result = value.replace("_", ".");
        return Some(result);
    } else {
        None
    }
}

///过滤HTML标签
pub fn filter_html_tags(html: Option<String>) -> String {
    if let Some(input) = html {
        // 去除前后空格
        let trimmed_input = input.trim();
        // 过滤HTML标签
        let re = Regex::new(r"<[^>]*>").expect("过滤html标签正则错误");
        let filtered = re.replace_all(trimmed_input, "").into_owned();
        filtered
    } else {
        String::new()
    }
}

///过滤HTML标签
pub fn filter_html(html: Option<String>) -> String {
    if let Some(input) = html {
        // 去除前后空格
        let trimmed_input = input.trim();
        // 过滤HTML标签
        let re = Regex::new(r"<[^>]*>| |\n|\(|\)|（|）").expect("过滤html标签正则错误");
        let filtered = re.replace_all(trimmed_input, "").into_owned();
        filtered
    } else {
        String::new()
    }
}


/// 截取字符串到指定长度
/// 如果字符串长度小于指定长度，则返回原字符串
/// 否则返回截取后的字符串
///
/// # Examples
///
/// 
pub fn truncate_string(s: String, length: usize) -> String {
    s.chars()
        .take(length)
        .collect()
}




/// 检查用逗号分隔的路径列表中是否包含与通配符路径匹配的路径。
///
/// # Arguments
///
/// * `csv_string` - 用逗号分隔的路径字符串，例如 "/user/login,/user/logout,/user/forgot,/user/supplier-join,/admin/*"
/// * `query_path` - 要查找的路径
///
/// # Returns
///
/// * `true` 如果路径在列表中（包括匹配通配符的路径）否则 `false`
pub fn contains_path(csv_string: &str, query_path: &str) -> bool {
    // 将字符串分割成 Vec
    let paths: Vec<&str> = csv_string.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    // 检查 Vec 中是否包含指定的路径或匹配通配符的路径
    paths.iter().any(|&path| {
        if path.ends_with("/*") {
            // 去掉通配符部分
            let prefix = &path[..path.len() - 2]; // 去掉 "/*"
            query_path.starts_with(prefix) && query_path.len() > prefix.len()
        } else {
            path == query_path
        }
    })
}


pub fn text_to_u64(text: &Text<String>) -> Result<i64, ParseIntError> {
    // 1. 提取内部的字符串
    let value: &str = &*text;

    // 2. 解析字符串为 i64
    value.parse::<i64>()
}

