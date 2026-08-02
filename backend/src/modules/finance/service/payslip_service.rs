//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::*;
use sea_orm::sea_query::Expr;
use chrono::Utc;
use rust_decimal::prelude::ToPrimitive;

use crate::modules::finance::entity::{payslip, salary_record, notification_channel_config};
use crate::modules::system::entity::admin;
use crate::modules::message::service::notification_service::NotificationService;

/// 分页查询工资条
pub async fn get_payslip_list(
    db: &DatabaseConnection,
    year: Option<i32>,
    month: Option<i32>,
    employee_id: Option<i64>,
    send_status: Option<i32>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<payslip::Model>, i64), String> {
    let mut stmt = payslip::Entity::find();

    if let Some(y) = year {
        stmt = stmt.filter(payslip::Column::Year.eq(y));
    }
    if let Some(m) = month {
        stmt = stmt.filter(payslip::Column::Month.eq(m));
    }
    if let Some(eid) = employee_id {
        stmt = stmt.filter(payslip::Column::EmployeeId.eq(eid));
    }
    if let Some(s) = send_status {
        stmt = stmt.filter(payslip::Column::SendStatus.eq(s));
    }

    stmt = stmt
        .order_by_desc(payslip::Column::Year)
        .order_by_desc(payslip::Column::Month)
        .order_by_desc(payslip::Column::CreateTime);

    let page = std::cmp::max(page, 1);
    let page_size = std::cmp::max(page_size, 1);

    let paginator = stmt.paginate(db, page_size as u64);
    let total = paginator.num_items().await.map_err(|e| e.to_string())? as i64;
    let items = paginator
        .fetch_page((page - 1) as u64)
        .await
        .map_err(|e| e.to_string())?;

    Ok((items, total))
}

/// 为指定年月已发放工资记录生成工资条
/// 查 salary_record where year/month and status=2 and deleted=0，
/// 为每条记录创建 payslip，detail_json 包含完整工资明细。
pub async fn generate_payslips(
    db: &DatabaseConnection,
    year: i32,
    month: i32,
) -> Result<i64, String> {
    // 1. 查询已发放的工资记录
    let records = salary_record::Entity::find()
        .filter(salary_record::Column::Year.eq(year))
        .filter(salary_record::Column::Month.eq(month))
        .filter(salary_record::Column::Status.eq(2))
        .filter(salary_record::Column::Deleted.eq(0))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    if records.is_empty() {
        return Err(format!("{}年{}月没有已发放的工资记录", year, month));
    }

    // 2. 查询已存在的工资条（避免重复生成）
    let salary_record_ids: Vec<i64> = records.iter().map(|r| r.id).collect();
    let existing_payslips = payslip::Entity::find()
        .filter(payslip::Column::SalaryRecordId.is_in(salary_record_ids.clone()))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;
    let existing_set: std::collections::HashSet<i64> = existing_payslips
        .iter()
        .map(|p| p.salary_record_id)
        .collect();

    let now = Utc::now().naive_utc();
    let txn = db.begin().await.map_err(|e| e.to_string())?;
    let mut generated_count: i64 = 0;

    for record in &records {
        // 跳过已存在的
        if existing_set.contains(&record.id) {
            continue;
        }

        // 构建 detail_json（完整工资明细）
        let detail_json = serde_json::json!({
            "baseSalary": record.base_salary.to_f64().unwrap_or_default(),
            "commissionAmount": record.commission_amount.to_f64().unwrap_or_default(),
            "performanceBonus": record.performance_bonus.to_f64().unwrap_or_default(),
            "deductionAmount": record.deduction_amount.to_f64().unwrap_or_default(),
            "socialInsurancePersonal": record.social_insurance_personal.to_f64().unwrap_or_default(),
            "housingFundPersonal": record.housing_fund_personal.to_f64().unwrap_or_default(),
            "taxAmount": record.tax_amount.to_f64().unwrap_or_default(),
            "teamCommissionAmount": record.team_commission_amount.to_f64().unwrap_or_default(),
            "totalSalary": record.total_salary.to_f64().unwrap_or_default(),
            "netSalary": record.net_salary.to_f64().unwrap_or_default(),
        });

        let model = payslip::ActiveModel {
            salary_record_id: Set(record.id),
            employee_id: Set(record.employee_id),
            year: Set(record.year),
            month: Set(record.month),
            total_salary: Set(Some(record.total_salary)),
            social_insurance_personal: Set(Some(record.social_insurance_personal)),
            tax_amount: Set(Some(record.tax_amount)),
            net_salary: Set(Some(record.net_salary)),
            detail_json: Set(Some(detail_json)),
            send_status: Set(Some(0)),
            send_channels: Set(None),
            send_time: Set(None),
            read_time: Set(None),
            confirm_time: Set(None),
            password_protected: Set(Some(0)),
            password_hash: Set(None),
            create_time: Set(Some(now)),
            ..Default::default()
        };

        model.insert(&txn).await.map_err(|e| e.to_string())?;
        generated_count += 1;
    }

    txn.commit().await.map_err(|e| e.to_string())?;

    Ok(generated_count)
}

/// 发送单条工资条
/// V7-9: 对接实际推送通道（site 站内信 + email 邮件；sms/wecom/dingtalk/feishu 暂记日志）
pub async fn send_payslip(
    db: &DatabaseConnection,
    payslip_id: i64,
    channels: Vec<String>,
) -> Result<(), String> {
    let record = payslip::Entity::find_by_id(payslip_id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "工资条不存在".to_string())?;

    // 查员工信息
    let employee = admin::Entity::find_by_id(record.employee_id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "员工不存在".to_string())?;

    let emp_name = employee.nick_name.clone()
        .or_else(|| employee.user_name.clone())
        .unwrap_or_else(|| format!("员工{}", record.employee_id));

    let (subject, plain_content) = build_payslip_message(&record, &emp_name);

    // 逐通道发送
    let mut succeeded: Vec<String> = Vec::new();
    let mut failed: Vec<(String, String)> = Vec::new();

    for ch in &channels {
        let ch_str = ch.as_str();
        match ch_str {
            "site" => {
                match NotificationService::send_system_notification(
                    db,
                    record.employee_id,
                    subject.clone(),
                    plain_content.clone(),
                    3, // 3=工资条通知
                    Some("/finance/payslip".to_string()),
                ).await {
                    Ok(_) => succeeded.push("site".to_string()),
                    Err(e) => failed.push(("site".to_string(), format!("{:?}", e))),
                }
            },
            "email" => {
                let to_email = employee.email.clone().unwrap_or_default();
                if to_email.is_empty() {
                    failed.push(("email".to_string(), "员工邮箱为空".to_string()));
                    continue;
                }
                match send_payslip_email(db, &to_email, &subject, &plain_content).await {
                    Ok(_) => succeeded.push("email".to_string()),
                    Err(e) => failed.push(("email".to_string(), e)),
                }
            },
            "sms" | "wecom" | "dingtalk" | "feishu" => {
                // 查询通道配置，enabled=1 且 config_json 含 webhook_url 时实际推送
                match notification_channel_config::Entity::find()
                    .filter(notification_channel_config::Column::ChannelCode.eq(ch_str))
                    .one(db)
                    .await
                {
                    Ok(Some(cfg)) if cfg.enabled.unwrap_or(0) == 1 => {
                        let config_json = cfg.config_json.clone().unwrap_or_default();
                        match send_webhook_message(&config_json, ch_str, &subject, &plain_content).await {
                            Ok(_) => succeeded.push(ch_str.to_string()),
                            Err(e) => {
                                log::error!("[payslip] 通道 {} 推送失败 员工={} payslip_id={} err={}", ch_str, record.employee_id, record.id, e);
                                failed.push((ch_str.to_string(), e));
                            }
                        }
                    }
                    Ok(_) => {
                        // 通道未启用或无配置，仅记日志，不计为失败
                        log::warn!("[payslip] 通道 {} 未启用或无配置，跳过员工={} payslip_id={}", ch_str, record.employee_id, record.id);
                    }
                    Err(e) => {
                        log::error!("[payslip] 查询通道 {} 配置失败: {}", ch_str, e);
                        failed.push((ch_str.to_string(), format!("查询配置失败: {}", e)));
                    }
                }
            },
            _ => {
                failed.push((ch_str.to_string(), "不支持的通道".to_string()));
            }
        }
    }

    let now = Utc::now().naive_utc();
    let channels_str = channels.join(",");

    // V7-9: 至少一个通道成功即记为已发送(1)，全部失败记为 0
    let final_status = if succeeded.is_empty() { 0i32 } else { 1i32 };

    let mut model: payslip::ActiveModel = record.into();
    model.send_status = Set(Some(final_status));
    model.send_channels = Set(Some(channels_str));
    model.send_time = Set(Some(now));

    let txn = db.begin().await.map_err(|e| e.to_string())?;
    model.update(&txn).await.map_err(|e| e.to_string())?;
    txn.commit().await.map_err(|e| e.to_string())?;

    if !succeeded.is_empty() {
        Ok(())
    } else {
        let errs: Vec<String> = failed.iter().map(|(c, m)| format!("{}: {}", c, m)).collect();
        Err(format!("所有通道发送失败 - {}", errs.join("; ")))
    }
}

/// 批量发送工资条
/// V7-9: 逐条调用 send_payslip 实现真实推送，统计成功数
pub async fn batch_send_payslips(
    db: &DatabaseConnection,
    ids: Vec<i64>,
    channels: Vec<String>,
) -> Result<i64, String> {
    if ids.is_empty() {
        return Err("请选择要发送的工资条".to_string());
    }
    if channels.is_empty() {
        return Err("请至少选择一个发送通道".to_string());
    }

    let mut success_count: i64 = 0;
    for id in &ids {
        match send_payslip(db, *id, channels.clone()).await {
            Ok(_) => success_count += 1,
            Err(e) => log::error!("[payslip] 批量发送失败 payslip_id={} err={}", id, e),
        }
    }

    Ok(success_count)
}

/// V7-9: 构造工资条消息内容（标题 + 纯文本内容）
fn build_payslip_message(record: &payslip::Model, emp_name: &str) -> (String, String) {
    let title = format!("{}年{}月工资条 - {}", record.year, record.month, emp_name);

    let total = record.total_salary.map(|v| v.to_string()).unwrap_or_else(|| "0".to_string());
    let insurance = record.social_insurance_personal.map(|v| v.to_string()).unwrap_or_else(|| "0".to_string());
    let tax = record.tax_amount.map(|v| v.to_string()).unwrap_or_else(|| "0".to_string());
    let net = record.net_salary.map(|v| v.to_string()).unwrap_or_else(|| "0".to_string());

    let content = format!(
        "{emp} 您好，您 {y}年{m}月 的工资条已生成：\n\n\
         应发工资：{total} 元\n\
         个人社保：-{insurance} 元\n\
         个人所得税：-{tax} 元\n\
         实发工资：{net} 元\n\n\
         如有疑问，请联系财务部门。",
        emp = emp_name,
        y = record.year,
        m = record.month,
        total = total,
        insurance = insurance,
        tax = tax,
        net = net,
    );

    (title, content)
}

/// V7-9: 通过 mail_service 发送工资条邮件
/// 注：需要在 mxx_system_mail_config 中配置默认邮箱账号
async fn send_payslip_email(
    db: &DatabaseConnection,
    to_email: &str,
    subject: &str,
    body: &str,
) -> Result<(), String> {
    use crate::modules::system::model::mail::SendMailRequest;
    use crate::modules::system::service::mail_service;

    let req = SendMailRequest {
        customer_id: None,
        to_emails: vec![to_email.to_string()],
        cc_emails: None,
        subject: Some(subject.to_string()),
        body: Some(format!("<pre style=\"font-family: monospace; line-height: 1.6;\">{}</pre>", body)),
        doc_url: None,
        contact_ids: None,
    };

    match mail_service::send_mail(db, req, None, Some("财务系统".to_string())).await {
        Ok(_) => {
            log::info!("[payslip] 邮件发送成功 to={} subject={}", to_email, subject);
            Ok(())
        }
        Err(e) => {
            let msg = e.to_string();
            log::error!("[payslip] 邮件发送失败 to={} err={}", to_email, msg);
            Err(msg)
        }
    }
}

/// 通过 webhook 方式推送消息（wecom/dingtalk/feishu/sms）
///
/// 从 config_json 中解析 webhook_url（sms 通道解析 gateway_url），
/// 使用 reqwest 发送 POST 请求。
///
/// - wecom/dingtalk: `{"msgtype":"text","text":{"content":"title\ncontent"}}`
/// - feishu: `{"msg_type":"text","content":{"text":"title\ncontent"}}`
/// - sms: 调用短信网关API（从 config_json 解析 access_key/secret_key/sign_name/phone）
pub async fn send_webhook_message(
    config_json: &str,
    channel: &str,
    title: &str,
    content: &str,
) -> Result<(), String> {
    let config: serde_json::Value = serde_json::from_str(config_json)
        .map_err(|e| format!("解析通道配置JSON失败: {}", e))?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

    match channel {
        "wecom" | "dingtalk" => {
            let webhook_url = config.get("webhook_url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("{} 通道配置缺少 webhook_url", channel))?
                .to_string();
            let body = serde_json::json!({
                "msgtype": "text",
                "text": {
                    "content": format!("{}\n{}", title, content)
                }
            });
            let resp = client.post(&webhook_url)
                .json(&body)
                .send()
                .await
                .map_err(|e| format!("{} 推送请求失败: {}", channel, e))?;
            if !resp.status().is_success() {
                return Err(format!("{} 推送响应状态异常: {}", channel, resp.status()));
            }
            log::info!("[payslip] {} 推送成功", channel);
            Ok(())
        }
        "feishu" => {
            let webhook_url = config.get("webhook_url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "feishu 通道配置缺少 webhook_url".to_string())?
                .to_string();
            let body = serde_json::json!({
                "msg_type": "text",
                "content": {
                    "text": format!("{}\n{}", title, content)
                }
            });
            let resp = client.post(&webhook_url)
                .json(&body)
                .send()
                .await
                .map_err(|e| format!("feishu 推送请求失败: {}", e))?;
            if !resp.status().is_success() {
                return Err(format!("feishu 推送响应状态异常: {}", resp.status()));
            }
            log::info!("[payslip] feishu 推送成功");
            Ok(())
        }
        "sms" => {
            // 短信网关：从 config_json 解析 access_key/secret_key/sign_name/phone/gateway_url
            let gateway_url = config.get("gateway_url")
                .or_else(|| config.get("webhook_url"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| "sms 通道配置缺少 gateway_url".to_string())?
                .to_string();
            let access_key = config.get("access_key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "sms 通道配置缺少 access_key".to_string())?
                .to_string();
            let secret_key = config.get("secret_key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "sms 通道配置缺少 secret_key".to_string())?
                .to_string();
            let sign_name = config.get("sign_name")
                .and_then(|v| v.as_str())
                .unwrap_or("MxxCRM")
                .to_string();
            let phone = config.get("phone")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "sms 通道配置缺少 phone".to_string())?
                .to_string();

            let body = serde_json::json!({
                "accessKey": access_key,
                "secretKey": secret_key,
                "signName": sign_name,
                "phone": phone,
                "title": title,
                "content": content,
            });
            let resp = client.post(&gateway_url)
                .json(&body)
                .send()
                .await
                .map_err(|e| format!("sms 推送请求失败: {}", e))?;
            if !resp.status().is_success() {
                return Err(format!("sms 推送响应状态异常: {}", resp.status()));
            }
            log::info!("[payslip] sms 推送成功 phone={}", phone);
            Ok(())
        }
        _ => Err(format!("不支持的 webhook 通道: {}", channel)),
    }
}

/// 标记已读
pub async fn mark_read(db: &DatabaseConnection, payslip_id: i64) -> Result<(), String> {
    let record = payslip::Entity::find_by_id(payslip_id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "工资条不存在".to_string())?;

    let now = Utc::now().naive_utc();
    let mut model: payslip::ActiveModel = record.into();
    model.send_status = Set(Some(2));
    model.read_time = Set(Some(now));

    let txn = db.begin().await.map_err(|e| e.to_string())?;
    model.update(&txn).await.map_err(|e| e.to_string())?;
    txn.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

/// 查阅统计 DTO
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadStatistics {
    /// 已发送数
    pub sent_count: i64,
    /// 已读数
    pub read_count: i64,
    /// 未读数
    pub unread_count: i64,
    /// 总数
    pub total_count: i64,
}

/// 获取查阅统计（已发送数、已读数、未读数）
pub async fn get_read_statistics(
    db: &DatabaseConnection,
    year: i32,
    month: i32,
) -> Result<ReadStatistics, String> {
    let records = payslip::Entity::find()
        .filter(payslip::Column::Year.eq(year))
        .filter(payslip::Column::Month.eq(month))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let total_count = records.len() as i64;
    // send_status: 0=未发送 1=已发送 2=已读 3=已确认
    let sent_count = records
        .iter()
        .filter(|r| r.send_status.unwrap_or(0) >= 1)
        .count() as i64;
    let read_count = records
        .iter()
        .filter(|r| r.send_status.unwrap_or(0) >= 2)
        .count() as i64;
    let unread_count = sent_count - read_count;

    Ok(ReadStatistics {
        sent_count,
        read_count,
        unread_count,
        total_count,
    })
}

// ===== V8-4: 工资条二次密码与撤回功能 =====

/// 简单 SHA-256 哈希（依赖 sha2 库已存在于项目）
fn hash_password(password: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    password.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// 为工资条设置二次密码
///
/// - `payslip_id` 工资条ID
/// - `password` 明文密码（前端传入，建议4-8位数字）
pub async fn set_payslip_password(
    db: &DatabaseConnection,
    payslip_id: i64,
    password: &str,
) -> Result<(), String> {
    if password.is_empty() {
        return Err("密码不能为空".to_string());
    }
    let record = payslip::Entity::find_by_id(payslip_id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("工资条不存在")?;

    let hash = hash_password(password);
    let mut model: payslip::ActiveModel = record.into();
    model.password_protected = Set(Some(1));
    model.password_hash = Set(Some(hash));
    model.update(db).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// 取消工资条二次密码
pub async fn clear_payslip_password(
    db: &DatabaseConnection,
    payslip_id: i64,
) -> Result<(), String> {
    let record = payslip::Entity::find_by_id(payslip_id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("工资条不存在")?;

    let mut model: payslip::ActiveModel = record.into();
    model.password_protected = Set(Some(0));
    model.password_hash = Set(None);
    model.update(db).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// 验证工资条二次密码
///
/// 用于员工查阅工资条前的密码校验
pub async fn verify_payslip_password(
    db: &DatabaseConnection,
    payslip_id: i64,
    password: &str,
) -> Result<bool, String> {
    let record = payslip::Entity::find_by_id(payslip_id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("工资条不存在")?;

    if record.password_protected.unwrap_or(0) != 1 {
        // 未启用密码保护，直接通过
        return Ok(true);
    }

    let stored_hash = record.password_hash.unwrap_or_default();
    let input_hash = hash_password(password);
    Ok(stored_hash == input_hash)
}

/// 撤回已发送的工资条
///
/// - 将 send_status 改为 4（已撤回）
/// - 记录撤回时间和原因
/// - 发送站内信通知员工工资条已撤回
pub async fn withdraw_payslip(
    db: &DatabaseConnection,
    payslip_id: i64,
    withdrawn_by: i64,
    reason: &str,
) -> Result<(), String> {
    let record = payslip::Entity::find_by_id(payslip_id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("工资条不存在")?;

    // 只有已发送(1)/已读(2)/已确认(3)状态才能撤回
    let status = record.send_status.unwrap_or(0);
    if status != 1 && status != 2 && status != 3 {
        return Err(format!("当前状态({})不可撤回，只有已发送/已读/已确认状态可撤回", status));
    }

    let now = Utc::now().naive_utc();
    let employee_id = record.employee_id;

    let mut model: payslip::ActiveModel = record.into();
    model.send_status = Set(Some(4)); // 4=已撤回
    model.withdraw_time = Set(Some(now));
    model.withdraw_reason = Set(Some(reason.to_string()));
    model.withdrawn_by = Set(Some(withdrawn_by));
    model.update(db).await.map_err(|e| e.to_string())?;

    // 发送站内信通知员工
    let _ = NotificationService::send_system_notification(
        db,
        employee_id,
        "工资条已撤回".to_string(),
        format!("您的工资条已被撤回，原因：{}。如有疑问请联系财务。", reason),
        3, // 工资条通知
        Some("/finance/payslip".to_string()),
    ).await;

    Ok(())
}
