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
use crate::modules::system::model::mail::{MailLogSaveDTO, SendMailRequest};
use crate::modules::system::service::mail_config_service;
use sea_orm::{DbConn, DbErr, TransactionTrait};

use lettre::message::header::ContentType;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

/// 核心邮件发送函数
///
/// 流程：
/// 1. 获取默认邮箱配置，缺失则返回错误"未配置默认邮箱账号"
/// 2. 若 `req.doc_url` 有值，用 reqwest 异步抓取其内容作为正文（失败则回退到 req.body）
/// 3. 用 lettre 构建邮件（from/to/cc/subject/html body）
/// 4. 根据 is_ssl 选择 `relay`（隐式 TLS）或 `starttls_relay`，端口用配置的 port
/// 5. 发送，成功写入 mail_log（status=1），失败写入 mail_log（status=0 + error_msg）
/// 6. 返回 mail_log 的 id
pub async fn send_mail(
    db: &DbConn,
    req: SendMailRequest,
    sender_id: Option<i64>,
    sender_name: Option<String>,
) -> Result<i64> {
    // 1. 获取默认邮箱配置
    let config = mail_config_service::find_default(db)
        .await?
        .ok_or_else(|| Error::from("未配置默认邮箱账号"))?;

    let from_email = config.from_email.clone().unwrap_or_default();
    let from_name = config.from_name.clone().unwrap_or_default();
    let host = config.host.clone().unwrap_or_default();
    let port = config.port.unwrap_or(465);
    let username = config.username.clone().unwrap_or_default();
    let password = config.password.clone().unwrap_or_default();
    let is_ssl = config.is_ssl.unwrap_or(1);
    let subject = req.subject.clone().unwrap_or_default();
    let to_emails = req.to_emails.clone();
    let cc_emails = req.cc_emails.clone();

    // 2. 若 doc_url 有值，异步抓取内容作为正文
    let body = if let Some(doc_url) = &req.doc_url {
        if !doc_url.is_empty() {
            fetch_url_content(doc_url)
                .await
                .unwrap_or_else(|_| req.body.clone().unwrap_or_default())
        } else {
            req.body.clone().unwrap_or_default()
        }
    } else {
        req.body.clone().unwrap_or_default()
    };

    // 3. 构建邮件
    let from_mailbox = format!("{} <{}>", from_name, from_email)
        .parse::<Mailbox>()
        .or_else(|_| from_email.parse::<Mailbox>())
        .map_err(|e| Error::from(format!("发件人邮箱格式错误: {}", e)))?;

    let mut builder = Message::builder()
        .from(from_mailbox)
        .subject(subject.clone())
        .header(ContentType::TEXT_HTML);

    for to_email in &to_emails {
        let to: Mailbox = to_email
            .parse()
            .map_err(|e| Error::from(format!("收件人邮箱格式错误: {}", e)))?;
        builder = builder.to(to);
    }

    if let Some(cc_list) = &cc_emails {
        for cc_email in cc_list {
            let cc: Mailbox = cc_email
                .parse()
                .map_err(|e| Error::from(format!("抄送人邮箱格式错误: {}", e)))?;
            builder = builder.cc(cc);
        }
    }

    // 4. 构建 SMTP transport
    let transport_builder = if is_ssl == 1 {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&host)
            .map_err(|e| Error::from(format!("SMTP 配置错误: {}", e)))?
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)
            .map_err(|e| Error::from(format!("SMTP 配置错误: {}", e)))?
    };
    let transport = transport_builder
        .port(port as u16)
        .credentials(Credentials::new(username, password))
        .build();

    // 5. 发送邮件
    let message = builder
        .body(body.clone())
        .map_err(|e| Error::from(format!("构建邮件失败: {}", e)))?;

    let send_result = transport.send(message).await;

    // 6. 写入邮件日志
    let to_emails_str = to_emails.join(",");
    let cc_emails_str = cc_emails.map(|c| c.join(","));
    let now = chrono::Local::now().naive_utc();

    let (status, error_msg) = match send_result {
        Ok(_) => (1, None),
        Err(e) => (0, Some(e.to_string())),
    };

    let log_dto = MailLogSaveDTO {
        customer_id: req.customer_id,
        contact_ids: req.contact_ids.clone(),
        from_email: Some(from_email),
        to_emails: Some(to_emails_str),
        cc_emails: cc_emails_str,
        subject: Some(subject),
        body: Some(body),
        status: Some(status),
        error_msg,
        smtp_message_id: None,
        sender_id,
        sender_name: sender_name.clone(),
        send_time: Some(now),
    };

    let log_dto = log_dto.clone();
    let log_id = db
        .transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move {
                crate::modules::system::model::mail::MailLogModel::insert(txn, &log_dto).await
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(log_id)
}

/// 异步抓取 URL 内容作为邮件正文
async fn fetch_url_content(url: &str) -> Result<String> {
    let resp = reqwest::get(url)
        .await
        .map_err(|e| Error::from(format!("抓取文档内容失败: {}", e)))?;
    let text = resp
        .text()
        .await
        .map_err(|e| Error::from(format!("读取文档内容失败: {}", e)))?;
    Ok(text)
}
