//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 第三方接口统一配置中心 Service
//!
//! 提供对各类第三方接口（支付、物流、签约、发票、通知、汇率、AI 等）的统一配置管理。
//! 支持：
//! - 按分类/编码查询配置
//! - 敏感字段 AES 加密存储
//! - 响应脱敏展示
//! - 连接测试
//!

use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DbConn, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set, TransactionTrait,
};

use crate::core::errors::error::{Error, Result};
use crate::modules::system::entity::integration_config::{self, Column, Entity};
use crate::modules::system::model::integration_config::{
    IntegrationConfigSaveRequest, IntegrationConfigVO, IntegrationTestResult,
};
use crate::utils::encryption_utils::{decrypt_card, encrypt_card};

// ==================== 查询 ====================

/// 按分组查询配置列表（脱敏）
pub async fn get_list(db: &DbConn, category: Option<String>) -> Result<Vec<IntegrationConfigVO>> {
    let mut cond = Condition::all().add(Column::Deleted.ne(1));
    if let Some(ref cat) = category {
        if !cat.is_empty() {
            cond = cond.add(Column::Category.eq(cat.clone()));
        }
    }

    let list = Entity::find()
        .filter(cond)
        .order_by_asc(Column::SortOrder)
        .all(db)
        .await?;

    let result: Vec<IntegrationConfigVO> = list.into_iter().map(to_masked_vo).collect();
    Ok(result)
}

/// 查询单个配置详情（脱敏）
pub async fn get_info(db: &DbConn, id: i64) -> Result<Option<IntegrationConfigVO>> {
    let config = Entity::find()
        .filter(Column::Id.eq(id))
        .filter(Column::Deleted.ne(1))
        .one(db)
        .await?;

    Ok(config.map(to_masked_vo))
}

/// 按 integration_code 查询原始配置（不脱敏，内部调用用）
///
/// 如果 `is_encrypted=1`，返回前自动解密 config_json 中的敏感字段。
pub async fn get_by_code(
    db: &DbConn,
    code: &str,
) -> Result<Option<integration_config::Model>> {
    let config = Entity::find()
        .filter(Column::IntegrationCode.eq(code))
        .filter(Column::Deleted.ne(1))
        .one(db)
        .await?;

    match config {
        Some(c) if c.is_encrypted == Some(1) => {
            let mut decrypted = c.clone();
            if let Some(ref json) = decrypted.config_json {
                decrypted.config_json = Some(decrypt_sensitive_fields(json));
            }
            Ok(Some(decrypted))
        }
        other => Ok(other),
    }
}

/// 快捷方法：获取某个接口配置 JSON 中的某个字段值
///
/// 自动处理解密，返回明文字符串。
pub async fn get_config_value(db: &DbConn, code: &str, key: &str) -> Option<String> {
    let config = get_by_code(db, code).await.ok()??;
    let json = config.config_json.as_ref()?;
    json.get(key)?.as_str().map(|s| s.to_string())
}

// ==================== 写入 ====================

/// 保存配置（新增或更新）
///
/// 如果 config_json 中包含敏感字段（key/secret/password/token），
/// 自动对敏感字段做 AES 加密并标记 `is_encrypted=1`。
pub async fn save(db: &DbConn, req: IntegrationConfigSaveRequest) -> Result<i64> {
    // 保留原始提交的明文 JSON（用于更新时与现有明文比对还原未改的敏感字段）
    let submitted_json_plain = req.config_json.clone();

    // 处理 config_json：检测敏感字段并加密
    let (config_json, is_encrypted) = match &req.config_json {
        Some(json) => {
            let encrypted = encrypt_sensitive_fields(json);
            let has_sensitive = has_sensitive_keys(json);
            (Some(encrypted), if has_sensitive { 1 } else { 0 })
        }
        None => (None, 0),
    };

    let now = chrono::Local::now().naive_local();
    let id_opt = req.id;

    let result = db.transaction::<_, i64, sea_orm::DbErr>(|txn| {
        Box::pin(async move {
            if let Some(id) = id_opt {
                // 更新前：查出现有记录，还原用户未修改的敏感字段
                // （前端表单回显的是脱敏值，未修改时提交的仍是脱敏值，需还原成明文避免覆盖）
                let final_json = if let Some(ref new_json) = submitted_json_plain {
                    let existing = Entity::find_by_id(id).one(txn).await?;
                    if let Some(ref ex_model) = existing {
                        let ex_plain = if ex_model.is_encrypted == Some(1) {
                            ex_model.config_json.as_ref()
                                .map(decrypt_sensitive_fields)
                                .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()))
                        } else {
                            ex_model.config_json.clone().unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()))
                        };
                        Some(restore_unchanged_sensitive(new_json, &ex_plain))
                    } else {
                        submitted_json_plain.clone()
                    }
                } else {
                    None
                };

                // 还原后重新检测是否含敏感字段并加密
                let (final_json_encrypted, final_is_encrypted) = match &final_json {
                    Some(json) => {
                        let encrypted = encrypt_sensitive_fields(json);
                        let has_sensitive = has_sensitive_keys(json);
                        (Some(encrypted), if has_sensitive { 1 } else { 0 })
                    }
                    None => (None, 0),
                };

                let active = integration_config::ActiveModel {
                    category: Set(req.category),
                    integration_code: Set(req.integration_code),
                    integration_name: Set(req.integration_name),
                    config_json: Set(final_json_encrypted),
                    api_base_url: Set(req.api_base_url),
                    enabled: Set(req.enabled),
                    is_encrypted: Set(Some(final_is_encrypted)),
                    remark: Set(req.remark),
                    update_time: Set(Some(now)),
                    ..Default::default()
                };
                Entity::update_many()
                    .set(active)
                    .filter(Column::Id.eq(id))
                    .filter(Column::Deleted.ne(1))
                    .exec(txn)
                    .await?;
                Ok(id)
            } else {
                let active = integration_config::ActiveModel {
                    category: Set(req.category),
                    integration_code: Set(req.integration_code),
                    integration_name: Set(req.integration_name),
                    config_json: Set(config_json),
                    api_base_url: Set(req.api_base_url),
                    enabled: Set(req.enabled),
                    is_encrypted: Set(Some(is_encrypted)),
                    remark: Set(req.remark),
                    create_time: Set(Some(now)),
                    update_time: Set(Some(now)),
                    deleted: Set(Some(0)),
                    ..Default::default()
                };
                let model = active.insert(txn).await?;
                Ok(model.id)
            }
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    Ok(result)
}

/// 启用/禁用配置
pub async fn toggle(db: &DbConn, id: i64, enabled: i32) -> Result<i64> {
    let now = chrono::Local::now().naive_local();

    let result = db.transaction::<_, i64, sea_orm::DbErr>(|txn| {
        Box::pin(async move {
            let active = integration_config::ActiveModel {
                enabled: Set(Some(enabled)),
                update_time: Set(Some(now)),
                ..Default::default()
            };
            Entity::update_many()
                .set(active)
                .filter(Column::Id.eq(id))
                .filter(Column::Deleted.ne(1))
                .exec(txn)
                .await?;
            Ok(id)
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    Ok(result)
}

/// 软删除配置
pub async fn delete_by_id(db: &DbConn, id: i64) -> Result<i64> {
    let now = chrono::Local::now().naive_local();

    let result = db.transaction::<_, i64, sea_orm::DbErr>(|txn| {
        Box::pin(async move {
            let active = integration_config::ActiveModel {
                deleted: Set(Some(1)),
                update_time: Set(Some(now)),
                ..Default::default()
            };
            Entity::update_many()
                .set(active)
                .filter(Column::Id.eq(id))
                .filter(Column::Deleted.ne(1))
                .exec(txn)
                .await?;
            Ok(id)
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    Ok(result)
}

// ==================== AI 消费者辅助函数 ====================

/// 获取第一个已启用的 AI 提供商配置（内部调用，自动解密）
///
/// 用于 background_check_service 等业务代码，避免每次硬编码 deepseek。
/// 策略：按 sort_order 取第一个 category='ai' 且 enabled=1 且 integration_code 不是 prompt_ 开头的
pub async fn get_default_ai_provider(
    db: &DbConn,
) -> Result<Option<integration_config::Model>> {
    let list = Entity::find()
        .filter(Column::Deleted.ne(1))
        .filter(Column::Enabled.eq(1))
        .filter(Column::Category.eq("ai"))
        .order_by_asc(Column::SortOrder)
        .order_by_asc(Column::Id)
        .all(db)
        .await?;

    for c in list {
        let code = c.integration_code.clone().unwrap_or_default();
        // 跳过提示词条目（prompt_ 开头的）
        if code.starts_with("prompt_") {
            continue;
        }
        // 解密
        let decrypted = if c.is_encrypted == Some(1) {
            let mut d = c.clone();
            if let Some(ref json) = d.config_json {
                d.config_json = Some(decrypt_sensitive_fields(json));
            }
            d
        } else {
            c
        };
        return Ok(Some(decrypted));
    }
    Ok(None)
}

/// 获取指定 AI 提供商的某个配置字段值
///
/// provider_code: 如 "deepseek"；若传 None 则使用 get_default_ai_provider 自动选取第一个启用的
pub async fn get_ai_provider_value(
    db: &DbConn,
    provider_code: Option<&str>,
    key: &str,
) -> Result<String> {
    let provider = match provider_code {
        Some(code) => get_by_code(db, code)
            .await?
            .ok_or_else(|| Error::from(format!("AI 提供商 {} 未配置", code)))?,
        None => get_default_ai_provider(db)
            .await?
            .ok_or_else(|| Error::from("没有已启用的 AI 提供商，请先在第三方接口配置→AI配置中添加并启用"))?,
    };
    let json = provider
        .config_json
        .as_ref()
        .ok_or_else(|| Error::from("AI 配置 JSON 为空"))?;
    let val = json
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| Error::from(format!("AI 配置字段 {} 不存在", key)))?;
    if val.is_empty() {
        return Err(Error::from(format!("AI 配置字段 {} 为空", key)));
    }
    Ok(val)
}

/// 获取 AI 提示词内容（按 code，如 "prompt_background_check"）
///
/// 先查 category='ai' 且 integration_code=code 的条目，取 config_json.content
pub async fn get_ai_prompt_content(db: &DbConn, prompt_code: &str) -> Result<String> {
    // 保证 code 规范：如果用户传的是 background_check，自动补 prompt_ 前缀
    let code = if prompt_code.starts_with("prompt_") {
        prompt_code.to_string()
    } else {
        format!("prompt_{}", prompt_code)
    };

    let config = Entity::find()
        .filter(Column::Deleted.ne(1))
        .filter(Column::Category.eq("ai"))
        .filter(Column::IntegrationCode.eq(&code))
        .one(db)
        .await?;

    match config {
        Some(c) => {
            let json = c.config_json.as_ref().ok_or_else(|| {
                Error::from(format!("提示词配置 {} 缺少 config_json", code))
            })?;
            // 支持两种格式：{ "content": "xxx" } 或 直接字符串
            let content = json
                .get("content")
                .and_then(|v| v.as_str())
                .or_else(|| json.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| Error::from(format!("提示词配置 {} 没有 content 字段", code)))?;
            Ok(content)
        }
        None => Err(Error::from(format!(
            "提示词配置 {} 不存在，请在第三方接口配置→AI配置→提示词中添加",
            code
        ))),
    }
}

// ==================== 测试连接 ====================

/// 测试单个接口连接
///
/// 根据 category + integration_code 分发到不同的测试逻辑。
/// 测试完成后更新 `last_test_time` / `last_test_result` / `last_test_message`。
pub async fn test_connection(db: &DbConn, id: i64) -> Result<(bool, String)> {
    let config = Entity::find()
        .filter(Column::Id.eq(id))
        .filter(Column::Deleted.ne(1))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("配置不存在"))?;

    let category = config.category.clone().unwrap_or_default();
    let code = config.integration_code.clone().unwrap_or_default();
    let config_json_raw = config.config_json.clone().unwrap_or_default();

    // 解密配置（内部测试需要明文）
    let config_json = if config.is_encrypted == Some(1) {
        decrypt_sensitive_fields(&config_json_raw)
    } else {
        config_json_raw
    };

    let api_base_url = config.api_base_url.clone();
    let (success, message) =
        do_test(&category, &code, &config_json, api_base_url.as_deref()).await;

    // 更新测试结果
    let now = chrono::Local::now().naive_local();
    let message_clone = message.clone();
    let success_val = success;
    db.transaction::<_, (), sea_orm::DbErr>(|txn| {
        Box::pin(async move {
            let active = integration_config::ActiveModel {
                last_test_time: Set(Some(now)),
                last_test_result: Set(Some(if success_val { 1 } else { 0 })),
                last_test_message: Set(Some(message_clone)),
                ..Default::default()
            };
            Entity::update_many()
                .set(active)
                .filter(Column::Id.eq(id))
                .exec(txn)
                .await?;
            Ok(())
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    Ok((success, message))
}

/// 测试所有已启用的接口
pub async fn test_all(db: &DbConn) -> Result<Vec<IntegrationTestResult>> {
    let list = Entity::find()
        .filter(Column::Deleted.ne(1))
        .filter(Column::Enabled.eq(1))
        .all(db)
        .await?;

    let mut results = Vec::new();
    for config in list {
        let code = config.integration_code.clone().unwrap_or_default();
        match test_connection(db, config.id).await {
            Ok((success, message)) => {
                results.push(IntegrationTestResult {
                    integration_code: code,
                    success,
                    message,
                });
            }
            Err(e) => {
                results.push(IntegrationTestResult {
                    integration_code: code,
                    success: false,
                    message: e.to_string(),
                });
            }
        }
    }
    Ok(results)
}

// ==================== 脱敏 & 加密工具 ====================

/// 脱敏函数：遍历 JSON，对 key 包含 "key"/"secret"/"password"/"token" 的值做 `前4位****后4位` 脱敏
pub fn mask_sensitive(config_json: &serde_json::Value) -> serde_json::Value {
    match config_json {
        serde_json::Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (k, v) in map {
                if is_sensitive_key(k) {
                    if let Some(s) = v.as_str() {
                        new_map.insert(k.clone(), serde_json::Value::String(mask_value(s)));
                    } else {
                        new_map.insert(k.clone(), mask_sensitive(v));
                    }
                } else {
                    new_map.insert(k.clone(), mask_sensitive(v));
                }
            }
            serde_json::Value::Object(new_map)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(mask_sensitive).collect())
        }
        _ => config_json.clone(),
    }
}

/// 判断 key 是否为敏感字段（包含 key/secret/password/token，不区分大小写）
fn is_sensitive_key(key: &str) -> bool {
    let lower = key.to_lowercase();
    lower.contains("key")
        || lower.contains("secret")
        || lower.contains("password")
        || lower.contains("token")
}

/// 对字符串值做脱敏：长度 > 8 显示前4位+****+后4位，否则全部 ****
fn mask_value(val: &str) -> String {
    let chars: Vec<char> = val.chars().collect();
    if chars.len() <= 8 {
        return "****".to_string();
    }
    let first4: String = chars.iter().take(4).collect();
    let last4: String = chars[chars.len() - 4..].iter().collect();
    format!("{}****{}", first4, last4)
}

/// 更新时还原未修改的敏感字段
///
/// 遍历提交的 JSON，对每个敏感字段（key/secret/password/token）：
/// - 若提交值等于对现有明文做脱敏后的结果，说明用户未修改（表单里回显的就是脱敏值），用现有明文还原
/// - 否则说明用户输入了新值，保留提交值
fn restore_unchanged_sensitive(
    submitted: &serde_json::Value,
    existing_plain: &serde_json::Value,
) -> serde_json::Value {
    match (submitted, existing_plain) {
        (serde_json::Value::Object(sub_map), serde_json::Value::Object(ex_map)) => {
            let mut new_map = serde_json::Map::new();
            for (k, v) in sub_map {
                if is_sensitive_key(k) {
                    if let (Some(sub_str), Some(ex_str)) = (v.as_str(), ex_map.get(k).and_then(|x| x.as_str())) {
                        // 提交值 == 现有明文的脱敏结果 → 用户没改，还原明文
                        if sub_str == mask_value(ex_str) {
                            new_map.insert(k.clone(), serde_json::Value::String(ex_str.to_string()));
                            continue;
                        }
                    }
                    // 深层对象/数组递归处理
                    if let Some(ex_v) = ex_map.get(k) {
                        new_map.insert(k.clone(), restore_unchanged_sensitive(v, ex_v));
                    } else {
                        new_map.insert(k.clone(), v.clone());
                    }
                } else {
                    if let Some(ex_v) = ex_map.get(k) {
                        new_map.insert(k.clone(), restore_unchanged_sensitive(v, ex_v));
                    } else {
                        new_map.insert(k.clone(), v.clone());
                    }
                }
            }
            serde_json::Value::Object(new_map)
        }
        _ => submitted.clone(),
    }
}

/// 递归检查 JSON 中是否包含敏感字段
fn has_sensitive_keys(json: &serde_json::Value) -> bool {
    match json {
        serde_json::Value::Object(map) => map
            .iter()
            .any(|(k, v)| is_sensitive_key(k) || has_sensitive_keys(v)),
        serde_json::Value::Array(arr) => arr.iter().any(has_sensitive_keys),
        _ => false,
    }
}

/// 递归加密 JSON 中的敏感字符串字段
fn encrypt_sensitive_fields(json: &serde_json::Value) -> serde_json::Value {
    transform_sensitive_strings(json, encrypt_card)
}

/// 递归解密 JSON 中的敏感字符串字段
fn decrypt_sensitive_fields(json: &serde_json::Value) -> serde_json::Value {
    transform_sensitive_strings(json, decrypt_card)
}

/// 通用：递归遍历 JSON，对敏感 key 的字符串值应用转换函数
fn transform_sensitive_strings<F>(json: &serde_json::Value, transform: F) -> serde_json::Value
where
    F: Fn(&str) -> String + Copy,
{
    match json {
        serde_json::Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (k, v) in map {
                if is_sensitive_key(k) {
                    if let Some(s) = v.as_str() {
                        new_map.insert(k.clone(), serde_json::Value::String(transform(s)));
                    } else {
                        new_map.insert(k.clone(), transform_sensitive_strings(v, transform));
                    }
                } else {
                    new_map.insert(k.clone(), transform_sensitive_strings(v, transform));
                }
            }
            serde_json::Value::Object(new_map)
        }
        serde_json::Value::Array(arr) => serde_json::Value::Array(
            arr.iter()
                .map(|v| transform_sensitive_strings(v, transform))
                .collect(),
        ),
        _ => json.clone(),
    }
}

// ==================== 内部工具 ====================

/// 将 entity Model 转换为脱敏后的 VO
fn to_masked_vo(m: integration_config::Model) -> IntegrationConfigVO {
    let config_json = m.config_json.as_ref().map(|json| {
        let json = if m.is_encrypted == Some(1) {
            decrypt_sensitive_fields(json)
        } else {
            json.clone()
        };
        mask_sensitive(&json)
    });

    let test_status_name = match m.last_test_result {
        Some(1) => Some("成功".to_string()),
        Some(0) => Some("失败".to_string()),
        _ => Some("未测试".to_string()),
    };

    IntegrationConfigVO {
        id: Some(m.id),
        category: m.category,
        integration_code: m.integration_code,
        integration_name: m.integration_name,
        config_json,
        api_base_url: m.api_base_url,
        enabled: m.enabled,
        sort_order: m.sort_order,
        last_test_time: m.last_test_time,
        last_test_result: m.last_test_result,
        last_test_message: m.last_test_message,
        is_encrypted: m.is_encrypted,
        remark: m.remark,
        create_time: m.create_time,
        update_time: m.update_time,
        deleted: m.deleted,
        test_status_name,
    }
}

/// 根据 category + code 执行接口测试
async fn do_test(
    category: &str,
    code: &str,
    config_json: &serde_json::Value,
    api_base_url: Option<&str>,
) -> (bool, String) {
    use crate::core::kit::json_util;
    // AI 类：检查 api_key
    if category == "ai" {
        let api_key = json_util::get_str(config_json, "api_key");
        return match api_key {
            Some(k) if !k.is_empty() => (true, "AI 接口参数校验通过".to_string()),
            _ => (false, "缺少必要参数: api_key".to_string()),
        };
    }

    // 通知类 webhook：尝试 POST 测试消息
    if category == "notification" && code.ends_with("webhook") {
        let webhook_url = match json_util::get_str(config_json, "webhook_url") {
            Some(u) if !u.is_empty() => u,
            _ => return (false, "缺少必要参数: webhook_url".to_string()),
        };
        let client = reqwest::Client::new();
        let test_msg =
            serde_json::json!({"text": "测试消息 - 来自MxxCRM集成配置中心", "msgtype": "text"});
        match client.post(&webhook_url).json(&test_msg).send().await {
            Ok(resp) if resp.status().is_success() => {
                (true, "Webhook 测试消息发送成功".to_string())
            }
            Ok(resp) => (false, format!("Webhook 返回状态码: {}", resp.status())),
            Err(e) => (false, format!("Webhook 请求失败: {}", e)),
        }
    } else {
        match (category, code) {
            ("payment", "wechat_pay") => {
                let app_id = json_util::get_str(config_json, "app_id");
                let mchid = json_util::get_str(config_json, "mchid");
                match (app_id, mchid) {
                    (Some(a), Some(m)) if !a.is_empty() && !m.is_empty() => {
                        (true, "参数校验通过".to_string())
                    }
                    _ => (false, "缺少必要参数: app_id 或 mchid".to_string()),
                }
            }
            ("logistics", "kuaidi100") => {
                let customer = json_util::get_str(config_json, "customer");
                let key = json_util::get_str(config_json, "key");
                match (customer, key) {
                    (Some(c), Some(k)) if !c.is_empty() && !k.is_empty() => {
                        (true, "参数校验通过".to_string())
                    }
                    _ => (false, "缺少必要参数: customer 或 key".to_string()),
                }
            }
            ("esign", "esign_cn") => {
                let app_id = json_util::get_str(config_json, "app_id");
                let app_secret = json_util::get_str(config_json, "app_secret");
                match (app_id, app_secret) {
                    (Some(a), Some(s)) if !a.is_empty() && !s.is_empty() => {
                        (true, "参数校验通过".to_string())
                    }
                    _ => (false, "缺少必要参数: app_id 或 app_secret".to_string()),
                }
            }
            ("invoice", "baiwang") => {
                let device_no = json_util::get_str(config_json, "device_no");
                let tax_no = json_util::get_str(config_json, "tax_no");
                match (device_no, tax_no) {
                    (Some(d), Some(t)) if !d.is_empty() && !t.is_empty() => {
                        (true, "参数校验通过".to_string())
                    }
                    _ => (false, "缺少必要参数: device_no 或 tax_no".to_string()),
                }
            }
            ("notification", "smtp_email") => {
                let host = json_util::get_str(config_json, "host");
                let username = json_util::get_str(config_json, "username");
                let password = json_util::get_str(config_json, "password");
                match (host, username, password) {
                    (Some(h), Some(u), Some(p))
                        if !h.is_empty() && !u.is_empty() && !p.is_empty() =>
                    {
                        (true, "SMTP 参数校验通过".to_string())
                    }
                    _ => (false, "缺少必要参数: host/username/password".to_string()),
                }
            }
            ("notification", "sms_aliyun") => {
                let access_key = json_util::get_str(config_json, "access_key");
                let secret_key = json_util::get_str(config_json, "secret_key");
                match (access_key, secret_key) {
                    (Some(a), Some(s)) if !a.is_empty() && !s.is_empty() => {
                        (true, "参数校验通过".to_string())
                    }
                    _ => (
                        false,
                        "缺少必要参数: access_key 或 secret_key".to_string(),
                    ),
                }
            }
            ("exchange_rate", "ecb") => {
                let url = api_base_url.unwrap_or(
                    "https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml",
                );
                match reqwest::get(url).await {
                    Ok(resp) if resp.status().is_success() => {
                        (true, "ECB 接口连接正常".to_string())
                    }
                    Ok(resp) => (
                        false,
                        format!("ECB 接口返回状态码: {}", resp.status()),
                    ),
                    Err(e) => (false, format!("ECB 接口请求失败: {}", e)),
                }
            }
            _ => (false, "接口测试功能待实现".to_string()),
        }
    }
}

// ==================== 一次性数据迁移：ai_config / mail_config → integration_config ====================

///
/// 一次性数据迁移：把旧配置表的数据搬迁到统一配置表 `mxx_system_integration_config`。
///
/// 幂等设计：
/// - 迁移前先按 `integration_code` 检查目标是否已存在，已存在则跳过。
/// - 迁移直接走本模块的 `save`（自动加密敏感字段），无需手写 ActiveModel。
///
/// 迁移规则：
/// 1. `mxx_crm_ai_config`（旧 AI KV 表，无软删除）
///    - 同一 `provider`（由 config_key 前缀 `ai_<provider>_*` 解析得到）下的多条 KV
///      合并为一条 `category='ai'`、`integration_code=<provider>` 的记录；
///      config_json 形如 `{api_key, model, temperature, api_url}`。
///    - config_key 含 `prompt` 的条目合并为 `integration_code='prompt_<biz>'`，
///      config_json 形如 `{content: <config_value>}`。
/// 2. `mxx_system_mail_config`（旧邮箱表，有软删除）
///    - 取 `is_default=1 && deleted=0 && status=1` 的那条（取不到则取第一条未删除的），
///      迁移为 `category='notification'`、`integration_code='smtp_email'` 的一条记录，
///      config_json = `{host, port, username, password, from_email, from_name, is_ssl}`，
///      `enabled=1`。
///
pub async fn migrate_legacy_configs(db: &DbConn) -> Result<()> {
    use crate::modules::ai::entity::ai_config as ai_entity;
    use crate::modules::system::entity::mail_config as mail_entity;

    // ---------- 1) AI 配置迁移 ----------
    // 读全部 AI KV（表无软删除字段）
    let ai_rows = match ai_entity::Entity::find().all(db).await {
        Ok(rows) => rows,
        Err(e) => {
            // 旧表可能已不存在（干净环境），按"无可迁移"处理
            log::warn!("[配置迁移] 读取 mxx_crm_ai_config 失败（可能表不存在），跳过 AI 迁移: {}", e);
            vec![]
        }
    };

    if !ai_rows.is_empty() {
        // 按 provider 分组：config_key 形如 ai_deepseek_api_key / ai_deepseek_model / ai_prompt_xxx_content
        let mut provider_groups: std::collections::HashMap<String, Vec<ai_entity::Model>> =
            std::collections::HashMap::new();
        let mut prompt_groups: std::collections::HashMap<String, Vec<ai_entity::Model>> =
            std::collections::HashMap::new();

        for row in ai_rows {
            let key = row.config_key.clone().unwrap_or_default();
            // 提示词：ai_<biz>_prompt 或 ai_prompt_<biz>
            if key.contains("prompt") {
                // 约定 code：prompt_<biz>，biz = 去掉 ai_ 前缀、去掉 _prompt/_content 后缀
                let biz = key
                    .trim_start_matches("ai_")
                    .trim_start_matches("prompt_")
                    .trim_end_matches("_prompt")
                    .trim_end_matches("_content")
                    .to_string();
                let biz = if biz.is_empty() { "default".to_string() } else { biz };
                let code = format!("prompt_{}", biz);
                prompt_groups.entry(code).or_default().push(row);
            } else if let Some(provider) = key
                .strip_prefix("ai_")
                .and_then(|s| s.split('_').next())
            {
                provider_groups
                    .entry(provider.to_string())
                    .or_default()
                    .push(row);
            }
        }

        // 合并 provider KV → 一条 integration_config
        for (provider, items) in provider_groups.into_iter() {
            let code = provider.clone();
            // 目标已存在则跳过
            if code_exists(db, &code).await? {
                continue;
            }
            let mut json = serde_json::json!({});
            let mut name = provider.clone();
            let mut api_url = String::new();
            for it in &items {
                let k = it.config_key.clone().unwrap_or_default();
                let v = it.config_value.clone().unwrap_or_default();
                if k.ends_with("api_key") {
                    json["api_key"] = serde_json::Value::String(v);
                } else if k.ends_with("model") {
                    json["model"] = serde_json::Value::String(v);
                } else if k.ends_with("temperature") {
                    json["temperature"] = serde_json::Value::String(v);
                } else if k.ends_with("api_url") || k.ends_with("base_url") {
                    api_url = v.clone();
                }
                if let Some(n) = &it.config_name {
                    if !n.is_empty() {
                        name = n.clone();
                    }
                }
            }
            // 没有任何有效字段则跳过
            if json.get("api_key").is_none() && json.get("model").is_none() {
                continue;
            }
            let req = IntegrationConfigSaveRequest {
                id: None,
                category: Some("ai".to_string()),
                integration_code: Some(code),
                integration_name: Some(name),
                config_json: Some(json),
                api_base_url: if api_url.is_empty() { None } else { Some(api_url) },
                enabled: Some(1),
                remark: Some("由 ai_config 表迁移".to_string()),
            };
            if let Err(e) = save(db, req).await {
                log::warn!("[配置迁移] AI 提供商 {} 迁移失败: {}", provider, e);
            }
        }

        // 合并 prompt KV → 一条 integration_config
        for (code, items) in prompt_groups.into_iter() {
            if code_exists(db, &code).await? {
                continue;
            }
            // 取第一个非空 content
            let content = items
                .iter()
                .find_map(|it| {
                    let v = it.config_value.clone().unwrap_or_default();
                    if v.is_empty() { None } else { Some(v) }
                })
                .unwrap_or_default();
            let name = items
                .iter()
                .find_map(|it| it.config_name.clone())
                .unwrap_or_else(|| code.clone());
            let req = IntegrationConfigSaveRequest {
                id: None,
                category: Some("ai".to_string()),
                integration_code: Some(code.clone()),
                integration_name: Some(name),
                config_json: Some(serde_json::json!({ "content": content })),
                api_base_url: None,
                enabled: Some(1),
                remark: Some("由 ai_config 表迁移".to_string()),
            };
            if let Err(e) = save(db, req).await {
                log::warn!("[配置迁移] 提示词 {} 迁移失败: {}", code, e);
            }
        }
    }

    // ---------- 2) 邮箱配置迁移 ----------
    // 取未删除记录，优先 is_default=1 && status=1
    let mail_rows = match mail_entity::Entity::find()
        .filter(mail_entity::Column::Deleted.ne(1))
        .all(db)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            log::warn!(
                "[配置迁移] 读取 mxx_system_mail_config 失败（可能表不存在），跳过邮箱迁移: {}",
                e
            );
            vec![]
        }
    };

    if !mail_rows.is_empty() {
        // 目标 smtp_email 已存在则跳过
        if !code_exists(db, "smtp_email").await? {
            let chosen = mail_rows
                .iter()
                .find(|m| m.is_default == Some(1) && m.status == Some(1))
                .or_else(|| mail_rows.iter().find(|m| m.is_default == Some(1)))
                .or_else(|| mail_rows.first());

            if let Some(m) = chosen {
                let json = serde_json::json!({
                    "host": m.host.clone().unwrap_or_default(),
                    "port": m.port.unwrap_or(465),
                    "username": m.username.clone().unwrap_or_default(),
                    "password": m.password.clone().unwrap_or_default(),
                    "from_email": m.from_email.clone().unwrap_or_default(),
                    "from_name": m.from_name.clone().unwrap_or_default(),
                    "is_ssl": m.is_ssl.unwrap_or(1),
                });
                let req = IntegrationConfigSaveRequest {
                    id: None,
                    category: Some("notification".to_string()),
                    integration_code: Some("smtp_email".to_string()),
                    integration_name: Some(
                        m.name.clone().unwrap_or_else(|| "SMTP 邮件".to_string()),
                    ),
                    config_json: Some(json),
                    api_base_url: None,
                    enabled: Some(1),
                    remark: Some("由 mail_config 表迁移".to_string()),
                };
                if let Err(e) = save(db, req).await {
                    log::warn!("[配置迁移] SMTP 邮箱配置迁移失败: {}", e);
                }
            }
        }
    }

    Ok(())
}

/// 判断某个 integration_code 在目标表（未删除）中是否已存在 —— 用于迁移幂等
async fn code_exists(db: &DbConn, code: &str) -> Result<bool> {
    let cnt = Entity::find()
        .filter(Column::IntegrationCode.eq(code))
        .filter(Column::Deleted.ne(1))
        .count(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(cnt > 0)
}
