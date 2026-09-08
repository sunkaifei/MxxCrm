use crate::core::errors::error::{Error, Result};
use crate::modules::ai::model::company_background_check::{BackgroundCheckDetailVO, BackgroundCheckListVO, BackgroundCheckModel, BackgroundCheckSaveDTO};
use crate::modules::system::service::integration_config_service;
use log::{info, warn, error};
use sea_orm::{DbConn, EntityTrait};
use serde_json::{json, Value};
use std::time::Duration;

pub async fn perform_background_check(
    db: &DbConn,
    company_name: &str,
    lead_id: Option<i64>,
    company_id: Option<i64>,
    user_id: i64,
    user_name: &str,
) -> Result<BackgroundCheckDetailVO> {
    if company_name.trim().is_empty() {
        return Err(Error::from("公司名称不能为空"));
    }

    info!("[背调] 开始执行, company_name={}, lead_id={:?}", company_name, lead_id);

    // ========== 改用 integration_config 统一配置中心 ==========
    // 1. 获取第一个已启用的 AI 提供商（一次性查询，后续取字段零成本）
    let provider = integration_config_service::get_default_ai_provider(db)
        .await
        .map_err(|e| {
            error!("[背调] 加载AI提供商失败: {}", e);
            Error::from("没有可用的AI提供商配置，请先在「系统设置→第三方接口配置→AI配置」中添加并启用")
        })?
        .ok_or_else(|| Error::from("没有已启用的 AI 提供商，请先在「系统设置→第三方接口配置→AI配置」中添加并启用"))?;

    let code = provider.integration_code.clone().unwrap_or_else(|| "unknown".to_string());
    let json = provider.config_json.as_ref().ok_or_else(|| {
        Error::from(format!("AI 提供商 {} 配置为空，请重新保存", code))
    })?;

    // 2. 从 config_json + entity 字段拼接所需参数
    let api_key = json
        .get("api_key")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| Error::from(format!("AI 提供商 {} 缺少 api_key 配置", code)))?;

    let model = json
        .get("model")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| Error::from(format!("AI 提供商 {} 缺少 model 配置", code)))?;

    // api_url 优先级：entity.api_base_url > config_json.api_url > 按 provider code 推定默认值
    let api_url = provider
        .api_base_url
        .clone()
        .filter(|s| !s.is_empty())
        .or_else(|| {
            json.get("api_url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| default_api_url_for_provider(&code));

    if api_url.is_empty() {
        return Err(Error::from(format!("AI 提供商 {} 缺少 API 地址（api_base_url 字段）", code)));
    }

    let temperature = json
        .get("temperature")
        .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok()))
        .or_else(|| json.get("temperature").and_then(|v| v.as_f64()))
        .unwrap_or(0.7);

    // 3. 获取背调提示词
    let prompt =
        integration_config_service::get_ai_prompt_content(db, "background_check").await?;

    info!(
        "[背调] 配置加载完成: provider={}, model={}, api_url={}",
        code, model, api_url
    );

    let full_prompt = format!("{}\n\n请查询以下公司的信息：{}", prompt, company_name);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| Error::from(format!("创建HTTP客户端失败: {}", e)))?;

    info!("[背调] 正在请求AI API...");
    let response = client
        .post(&api_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": "你是一位专业的企业工商信息查询和风险评估专家。请严格按照JSON格式输出结果。"
                },
                {
                    "role": "user",
                    "content": full_prompt
                }
            ],
            "temperature": temperature,
            "max_tokens": 4096
        }))
        .send()
        .await
        .map_err(|e| {
            let msg = if e.is_timeout() {
                "调用AI接口超时，请稍后重试".to_string()
            } else if e.is_connect() {
                format!("无法连接到AI服务 ({}): {}", api_url, e)
            } else {
                format!("调用AI接口失败: {}", e)
            };
            Error::from(msg)
        })?;

    let status = response.status();
    info!("[背调] AI API返回状态: {}", status);
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        warn!("[背调] AI API返回错误: {} - {}", status, body);
        return Err(Error::from(format!("AI接口返回错误 ({}): {}", status, body)));
    }

    let json_response: Value = response.json().await.map_err(|e| {
        error!("[背调] 解析AI响应JSON失败: {}", e);
        Error::from(format!("解析AI响应失败: {}", e))
    })?;

    let raw_content = json_response["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or_default()
        .trim()
        .to_string();

    info!("[背调] AI响应内容长度: {} 字符", raw_content.len());

    let raw_content = raw_content
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let report_data: Value = serde_json::from_str(raw_content)
        .unwrap_or_else(|e| {
            warn!("[背调] 解析AI返回内容为JSON失败: {}，使用降级数据", e);
            json!({
                "raw_text": raw_content,
                "summary": "AI返回内容无法解析为结构化数据，请查看原始文本。"
            })
        });

    let risk_score = report_data["risk_score"].as_i64()
        .or_else(|| report_data["risk_assessment"]["risk_score"].as_i64())
        .or_else(|| report_data["risk_assessment"]["score"].as_i64())
        .or_else(|| report_data["riskAnalysis"]["score"].as_i64())
        .or_else(|| report_data["score"].as_i64())
        .map(|v| v as i32);

    let risk_level = report_data["risk_level"].as_str()
        .or_else(|| report_data["risk_assessment"]["risk_level"].as_str())
        .or_else(|| report_data["risk_assessment"]["level"].as_str())
        .or_else(|| report_data["riskAnalysis"]["level"].as_str())
        .or_else(|| report_data["level"].as_str())
        .map(|s| s.to_string());

    let final_risk_score = risk_score.unwrap_or(50);
    let final_risk_level = risk_level.unwrap_or_else(|| {
        if final_risk_score <= 30 { "高风险".to_string() }
        else if final_risk_score <= 50 { "中风险".to_string() }
        else if final_risk_score <= 70 { "低风险".to_string() }
        else { "安全".to_string() }
    });

    info!("[背调] 评估完成: risk_score={}, risk_level={}", final_risk_score, final_risk_level);

    // 如果仅传 lead_id 未传 company_id，自动检测线索是否已转客户并绑定
    let final_company_id = if company_id.is_none() {
        if let Some(lid) = lead_id {
            use crate::modules::crm::entity::lead;
            match lead::Entity::find_by_id(lid).one(db).await {
                Ok(Some(lm)) => {
                    if let Some(cid) = lm.converted_to_customer_id {
                        info!("[背调] 检测到线索已转客户, 自动关联 company_id={}", cid);
                        Some(cid)
                    } else {
                        None
                    }
                }
                Ok(None) => None,
                Err(e) => {
                    warn!("[背调] 查询线索转换信息失败: {}", e);
                    None
                }
            }
        } else {
            None
        }
    } else {
        company_id
    };

    let save_dto = BackgroundCheckSaveDTO {
        company_name: Some(company_name.to_string()),
        company_id: final_company_id,
        lead_id,
        risk_score: Some(final_risk_score),
        risk_level: Some(final_risk_level.clone()),
        report_data: Some(report_data.clone()),
        ai_model: Some(model),
        prompt_version: Some("1.0".to_string()),
        created_by: Some(user_name.to_string()),
    };

    let check_id = BackgroundCheckModel::insert(db, &save_dto).await?;
    info!("[背调] 结果已保存, id={}", check_id);

    let check = BackgroundCheckModel::find_by_id(db, check_id).await?
        .ok_or_else(|| Error::from("保存背调记录失败"))?;

    Ok(BackgroundCheckDetailVO::from(check))
}

/// 根据 AI provider code 给出默认 API 地址（降级兜底，正常应优先使用 entity.api_base_url）
fn default_api_url_for_provider(code: &str) -> String {
    match code {
        "deepseek" => "https://api.deepseek.com/v1/chat/completions".to_string(),
        "doubao" => "https://ark.cn-beijing.volces.com/api/v3/chat/completions".to_string(),
        "zhipu" => "https://open.bigmodel.cn/api/paas/v4/chat/completions".to_string(),
        "qwen" => "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions".to_string(),
        "moonshot" => "https://api.moonshot.cn/v1/chat/completions".to_string(),
        "wenxin" | "baidu" => {
            "https://aip.baidubce.com/rpc/2.0/ai_custom/v1/wenxinworkshop/chat".to_string()
        }
        _ => String::new(),
    }
}

pub async fn get_by_lead_id(db: &DbConn, lead_id: i64) -> Result<Vec<BackgroundCheckListVO>> {
    let list = BackgroundCheckModel::find_by_lead_id(db, lead_id).await?;
    let result: Vec<BackgroundCheckListVO> = list.into_iter().map(|item| BackgroundCheckListVO::from(item)).collect();
    Ok(result)
}

pub async fn get_latest_by_lead_id(db: &DbConn, lead_id: i64) -> Result<Option<BackgroundCheckDetailVO>> {
    let check = BackgroundCheckModel::find_latest_by_lead_id(db, lead_id).await?;
    if let Some(ref c) = check {
        info!("[背调] 查询最近记录 lead_id={}, 找到id={}", lead_id, c.id);
    } else {
        info!("[背调] 查询最近记录 lead_id={}, 无数据", lead_id);
    }
    let result = check.map(|c| BackgroundCheckDetailVO::from(c));
    Ok(result)
}

pub async fn get_by_id(db: &DbConn, id: i64) -> Result<Option<BackgroundCheckDetailVO>> {
    let check = BackgroundCheckModel::find_by_id(db, id).await?;
    let result = check.map(|c| BackgroundCheckDetailVO::from(c));
    Ok(result)
}

pub async fn get_latest_by_company_id(db: &DbConn, company_id: i64) -> Result<Option<BackgroundCheckDetailVO>> {
    let check = BackgroundCheckModel::find_latest_by_company_id(db, company_id).await?;
    if let Some(ref c) = check {
        info!("[背调] 查询最近记录 company_id={}, 找到id={}", company_id, c.id);
    } else {
        info!("[背调] 查询最近记录 company_id={}, 无数据", company_id);
    }
    let result = check.map(|c| BackgroundCheckDetailVO::from(c));
    Ok(result)
}

pub async fn get_by_company_id(db: &DbConn, company_id: i64) -> Result<Vec<BackgroundCheckListVO>> {
    let list = BackgroundCheckModel::find_by_company_id(db, company_id).await?;
    let result: Vec<BackgroundCheckListVO> = list.into_iter().map(|item| BackgroundCheckListVO::from(item)).collect();
    Ok(result)
}

pub async fn get_timeline_by_company_name(db: &DbConn, company_name: &str) -> Result<Vec<BackgroundCheckListVO>> {
    let list = BackgroundCheckModel::get_timeline_by_company_name(db, company_name).await?;
    let result: Vec<BackgroundCheckListVO> = list.into_iter().map(|item| BackgroundCheckListVO::from(item)).collect();
    Ok(result)
}

pub async fn delete_by_id(db: &DbConn, id: i64) -> Result<i64> {
    let result = BackgroundCheckModel::delete_by_id(db, id).await?;
    Ok(result)
}
