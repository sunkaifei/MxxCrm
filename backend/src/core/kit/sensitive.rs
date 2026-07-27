//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 敏感数据脱敏工具
//!
//! 提供两种脱敏机制：
//! 1. **基于字段名**：递归遍历 JSON，把 password/token/secret 等字段的值替换为 "***"
//! 2. **基于正则**：识别邮箱、手机号、身份证、银行卡、微信号、QQ号等模式，
//!    对值进行部分脱敏（保留前后若干字符，中间用 * 替换）
//!
//! 调用入口：`mask_body_bytes(bytes)` — 适配后端 msgpack / JSON / form 三种响应格式

use regex::Regex;
use serde_json::Value;
use std::sync::OnceLock;

/// 单字段记录的最大字符数（超出截断）
const MAX_FIELD_LEN: usize = 2000;

/// 所有正则规则的静态缓存（编译一次复用）
static REGEX_RULES: OnceLock<Vec<MaskRule>> = OnceLock::new();

/// 一条脱敏规则：正则 + 替换函数
struct MaskRule {
    /// 编译好的正则
    regex: Regex,
    /// 规则名（用于日志）
    name: &'static str,
    /// 替换回调：传入完整匹配 + 捕获组，返回脱敏后的字符串
    /// 第一个参数是整个 match（含边界），用于保留前后字符
    replace: fn(&str, &regex::Captures) -> String,
}

/// 获取所有正则规则（首次调用时编译并缓存）
///
/// 规则顺序很重要：长数字串（身份证/银行卡/手机号）必须先匹配，
/// 短数字串（QQ号）最后匹配，否则 QQ 规则会先吃掉长数字的前几位
fn regex_rules() -> &'static [MaskRule] {
    REGEX_RULES.get_or_init(|| {
        vec![
            // 邮箱：user@example.com → u***@example.com
            MaskRule {
                regex: Regex::new(r"\b([A-Za-z0-9._%+-])[A-Za-z0-9._%+-]*@([A-Za-z0-9.-]+\.[A-Za-z]{2,})\b").unwrap(),
                name: "email",
                replace: |_full, caps| {
                    format!("{}***@{}", &caps[1], &caps[2])
                },
            },
            // 微信号：前缀 wxid_ + 字母数字（放在前面，避免 QQ 规则误匹配 wxid_xxxx 后面的数字）
            MaskRule {
                regex: Regex::new(r"\b(wxid_[A-Za-z0-9]{4})[A-Za-z0-9_-]*\b").unwrap(),
                name: "wechat_id",
                replace: |_full, caps| {
                    format!("{}***", &caps[1])
                },
            },
            // IP 地址：192.168.1.100 → 192.168.*.* （放在前面，避免被 QQ 规则吃掉）
            MaskRule {
                regex: Regex::new(r"\b(\d{1,3}\.\d{1,3})\.\d{1,3}\.\d{1,3}\b").unwrap(),
                name: "ipv4",
                replace: |_full, caps| {
                    format!("{}.*.*", &caps[1])
                },
            },
            // 18位身份证：110101199001011234 → 110101********1234
            // 总长 18 位 = 6位地址码 + 8位出生日期 + 2位顺序码 + 2位校验码
            // 脱敏：保留前 6 位 + 后 4 位（含校验码），中间 8 位替换为 *
            // 边界用 [^\d*]：跳过紧邻 *（已脱敏内容）的数字串，避免规则间级联误匹配
            MaskRule {
                regex: Regex::new(r"(^|[^\d*])([1-9]\d{5})(\d{8})(\d{4})([^\d*]|$)").unwrap(),
                name: "id_card",
                replace: |_full, caps| {
                    format!("{}{}********{}{}", &caps[1], &caps[2], &caps[4], &caps[5])
                },
            },
            // 15位身份证：110101900101123 → 110101******123
            // 总长 15 位 = 6位地址码 + 6位出生日期 + 3位顺序码
            // 脱敏：保留前 6 位 + 后 3 位，中间 6 位替换为 *
            MaskRule {
                regex: Regex::new(r"(^|[^\d*])([1-9]\d{5})(\d{6})(\d{3})([^\d*]|$)").unwrap(),
                name: "id_card_15",
                replace: |_full, caps| {
                    format!("{}{}******{}{}", &caps[1], &caps[2], &caps[4], &caps[5])
                },
            },
            // 中国手机号：13812345678 → 138****5678
            // 整体匹配格式：[边界符]手机号[边界符]
            // 边界符（前/后）需要保留到结果中
            MaskRule {
                regex: Regex::new(r"(^|[^\d*])(1[3-9]\d)(\d{4})(\d{4})([^\d*]|$)").unwrap(),
                name: "phone",
                replace: |full, caps| {
                    let prefix = &caps[1];
                    let suffix = &caps[5];
                    format!("{}{}****{}{}", prefix, &caps[2], &caps[4], suffix)
                },
            },
            // 银行卡号（16-19位）：6222020200112345678 → 622202*********5678
            // 注意：放在身份证后，因为身份证也是 18 位，身份证正则更严格（[1-9] 开头）
            MaskRule {
                regex: Regex::new(r"(^|[^\d*])(\d{6})(\d{6,9})(\d{4})([^\d*]|$)").unwrap(),
                name: "bank_card",
                replace: |_full, caps| {
                    format!("{}{}*********{}{}", &caps[1], &caps[2], &caps[4], &caps[5])
                },
            },
            // QQ号：5-9 位数字（独立 QQ 号，避免与手机号/身份证/银行卡冲突）
            // 严格限制 5-9 位 + 前后非数字边界（且排除 *，避免误匹配已脱敏内容）
            // 10位及以上的 QQ 号极少见，且容易与长数字冲突，故不处理
            MaskRule {
                regex: Regex::new(r"(^|[^\d*])([1-9]\d{4,8})([^\d*]|$)").unwrap(),
                name: "qq",
                replace: |_full, caps| {
                    let qq = &caps[2];
                    if qq.len() <= 4 {
                        format!("{}{}{}", &caps[1], qq, &caps[3])
                    } else {
                        // 保留前 4 位，后面替换为 ***
                        let prefix = qq.get(..4).unwrap_or(qq);
                        format!("{}{}***{}", &caps[1], prefix, &caps[3])
                    }
                },
            },
        ]
    })
}

/// 把 body bytes 转成可读字符串并脱敏，最后截断到 MAX_FIELD_LEN
///
/// 解析顺序（优先 JSON，避免 msgpack 把 JSON 字节流误识别成 fixmap 等结构）：
/// 1. 空内容 → None
/// 2. 先尝试 UTF-8 文本：
///    a. 若能解析为 JSON → 字段名脱敏 + 正则脱敏 → 转 JSON 字符串
///    b. 非 JSON 文本（如 form-urlencoded）→ 正则脱敏
/// 3. 再尝试 MessagePack（后端响应默认格式）→ 解码为 JSON Value → 脱敏 → 转 JSON 字符串
/// 4. 二进制 → `<binary>`
///
/// 为什么 JSON 优先：
/// - 请求体几乎都是 JSON，msgpack 解码器对任意字节流过于宽松
///   （例如 `{` 在 msgpack 是 fixmap 头，会把 JSON `{...}` 误识别成 map 结构）
/// - 响应体若是 msgpack，前几个字节通常不是合法 UTF-8，会自动跳过 JSON 分支
///
/// 脱敏顺序（双重保险）：
/// - 字段名敏感（password/token/...）→ 值替换为 "***"
/// - 值内正则匹配（邮箱/手机号/...）→ 部分字符替换为 "*"
pub fn mask_body_bytes(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }

    // 1. 优先尝试 UTF-8 文本（涵盖 JSON / form-urlencoded / 普通文本）
    if let Ok(s) = std::str::from_utf8(bytes) {
        if !s.is_empty() {
            // 1a. JSON 文本
            if let Ok(mut value) = serde_json::from_str::<Value>(s) {
                mask_value_in_place(&mut value);
                let json_str = serde_json::to_string(&value).unwrap_or_else(|_| s.to_string());
                return Some(truncate_str(&apply_regex_mask(&json_str), MAX_FIELD_LEN));
            }
            // 1b. 非 JSON 的 UTF-8 文本（如 form-urlencoded）
            return Some(truncate_str(&apply_regex_mask(s), MAX_FIELD_LEN));
        }
    }

    // 2. 尝试 MessagePack（后端响应默认格式；走到这里说明不是合法 UTF-8）
    if let Ok(mut value) = rmp_serde::from_slice::<Value>(bytes) {
        mask_value_in_place(&mut value);
        let json_str = serde_json::to_string(&value).unwrap_or_default();
        if !json_str.is_empty() {
            return Some(truncate_str(&apply_regex_mask(&json_str), MAX_FIELD_LEN));
        }
    }

    // 3. 二进制（如纯二进制文件下载）
    Some("<binary>".to_string())
}

/// 递归遍历 JSON，对敏感字段值替换为 "***"，并对其他字符串值应用正则脱敏
fn mask_value_in_place(value: &mut Value) {
    match value {
        Value::Object(map) => {
            // 先收集敏感字段并替换（避免 iter_mut 时修改 map 结构）
            let keys_to_mask: Vec<String> = map
                .keys()
                .filter(|k| is_sensitive_field(k))
                .cloned()
                .collect();
            for key in keys_to_mask {
                if let Some(val) = map.get_mut(&key) {
                    *val = Value::String("***".to_string());
                }
            }
            // 递归处理所有子值
            for (_, val) in map.iter_mut() {
                mask_value_in_place(val);
            }
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                mask_value_in_place(item);
            }
        }
        Value::String(s) => {
            // 对非敏感字段值内的字符串应用正则脱敏（邮箱、手机号等）
            *s = apply_regex_mask(s);
        }
        _ => {}
    }
}

/// 判断字段名是否为敏感字段（大小写不敏感）
///
/// 包含 camelCase / snake_case 两种命名风格，覆盖常见认证/密钥字段
fn is_sensitive_field(name: &str) -> bool {
    let lower = name.to_lowercase();
    const SENSITIVE: &[&str] = &[
        // 密码类
        "password",
        "pwd",
        "oldpassword",
        "newpassword",
        "oldpwd",
        "newpwd",
        // 令牌类
        "token",
        "accesstoken",
        "refreshtoken",
        "access_token",
        "refresh_token",
        "authorization",
        // 密钥类
        "secret",
        "apikey",
        "apisecret",
        "api_key",
        "api_secret",
        "privatekey",
        "private_key",
        "sessionkey",
        "session_key",
        // 验证码类
        "captchacode",
        "captchakey",
        "captcha_code",
        "captcha_key",
    ];
    SENSITIVE.contains(&lower.as_str())
}

/// 应用所有正则规则到字符串
fn apply_regex_mask(s: &str) -> String {
    let mut result = s.to_string();
    for rule in regex_rules() {
        result = rule
            .regex
            .replace_all(&result, |caps: &regex::Captures| {
                // 把整个 match 字符串也传给 replace 函数，方便保留前后边界字符
                let full_match = caps.get(0).map(|m| m.as_str()).unwrap_or("");
                (rule.replace)(full_match, caps)
            })
            .to_string();
    }
    result
}

/// 截断字符串到 max_len 字符，超出则加 "...(truncated)" 标记
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        return s.to_string();
    }
    let truncated: String = s.chars().take(max_len).collect();
    format!("{}...(truncated)", truncated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_email() {
        let s = "联系我: admin@mxxshop.com 或 test_user@example.com";
        let masked = apply_regex_mask(s);
        assert!(masked.contains("a***@mxxshop.com"));
        assert!(masked.contains("t***@example.com"));
    }

    #[test]
    fn test_mask_phone() {
        let s = "手机号: 13812345678";
        let masked = apply_regex_mask(s);
        assert!(masked.contains("138****5678"));
    }

    #[test]
    fn test_mask_id_card() {
        let s = "身份证: 110101199001011234";
        // 模拟 apply_regex_mask 内部循环，逐步应用规则
        let rules = regex_rules();
        let mut result = s.to_string();
        for rule in rules {
            let before = result.clone();
            result = rule.regex.replace_all(&result, |caps: &regex::Captures| {
                let full_match = caps.get(0).map(|m| m.as_str()).unwrap_or("");
                (rule.replace)(full_match, caps)
            }).to_string();
            if before != result {
                println!("[{}] applied: [{}] -> [{}]", rule.name, before, result);
            }
        }
        println!("Final ID card masked: [{}]", result);
        assert!(result.contains("110101********1234"));
    }

    #[test]
    fn test_mask_json_field() {
        let json = r#"{"username":"admin","password":"secret123","email":"admin@mxxshop.com"}"#;
        let mut value: Value = serde_json::from_str(json).unwrap();
        mask_value_in_place(&mut value);
        assert_eq!(value["password"], "***");
        // email 字段值会被 apply_regex_mask 替换
        assert!(value["email"].as_str().unwrap().contains("***"));
    }

    #[test]
    fn test_mask_msgpack_response() {
        // 模拟后端响应：用 msgpack 编码一个 JSON 对象
        let original = serde_json::json!({
            "code": 200,
            "msg": "success",
            "data": {
                "username": "admin",
                "phone": "13812345678",
                "email": "admin@mxxshop.com"
            }
        });
        let msgpack_bytes = rmp_serde::to_vec_named(&original).unwrap();
        let masked = mask_body_bytes(&msgpack_bytes).unwrap();
        assert!(masked.contains("138****5678"));
        assert!(masked.contains("a***@mxxshop.com"));
        assert!(masked.contains("\"code\":200"));
    }

    #[test]
    fn test_mask_json_request_body() {
        // 模拟前端发来的 JSON 请求体（之前会被 msgpack 误识别成乱码 "123" 之类）
        let json = r#"{"username":"admin","password":"secret123","phone":"13812345678","email":"admin@mxxshop.com"}"#;
        let masked = mask_body_bytes(json.as_bytes()).unwrap();
        // password 字段被字段名脱敏
        assert!(masked.contains("\"password\":\"***\""));
        // username 保留原值
        assert!(masked.contains("\"username\":\"admin\""));
        // 手机号 / 邮箱 走正则脱敏
        assert!(masked.contains("138****5678"));
        assert!(masked.contains("a***@mxxshop.com"));
        // 不应被 msgpack 误识别成短数字
        assert!(!masked.starts_with("123"));
        assert!(masked.starts_with("{"));
    }
}
