//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::{DbConn, DbErr, TransactionTrait};
use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::system::model::mail::SendMailRequest;
use crate::modules::system::service::mail_service;
use crate::modules::website::model::website_notification_config::{
    NotificationConfigListQuery, NotificationConfigModel, NotificationConfigSaveDTO, NotificationConfigVO,
};

/// 查询当前（默认）站点的全部通知配置
///
/// 单站模式下不需要传 website_id，后端自动定位默认站点。
/// 供前端"网站设置 → 通知配置"Tab 一次性加载。
pub async fn find_current_all(db: &DbConn, website_id: i64) -> Result<Vec<NotificationConfigVO>> {
    NotificationConfigModel::find_all_by_website(db, website_id)
        .await
        .map_err(|e| Error::from(e.to_string()))
}

/// 批量保存（upsert）当前站点的通知配置
///
/// 供前端"网站设置 → 通知配置"Tab 一次性提交所有场景配置。
/// 对每条配置：若已存在（按 website_id + scene_code）则更新，否则新建。
/// 整体在事务内执行，任一条失败回滚全部。
pub async fn bulk_upsert(db: &DbConn, website_id: i64, configs: Vec<NotificationConfigSaveDTO>) -> Result<i64> {
    let mut affected: i64 = 0;
    db.transaction::<_, (), DbErr>(|txn| {
        let configs_clone = configs.clone();
        Box::pin(async move {
            for req in configs_clone {
                // 必填校验
                if req.scene_code.is_empty() {
                    return Err(DbErr::Custom("场景编码不能为空".into()));
                }
                // 查已有
                let existing = NotificationConfigModel::find_by_website_and_scene(
                    txn,
                    website_id,
                    &req.scene_code,
                )
                .await?;
                let mut req_with_site = req.clone();
                req_with_site.website_id = Some(website_id);
                if let Some(model) = existing {
                    // 更新
                    NotificationConfigModel::update(txn, model.id, &req_with_site).await?;
                } else {
                    // 新建
                    NotificationConfigModel::insert(txn, &req_with_site).await?;
                }
            }
            Ok(())
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;
    affected += configs.len() as i64;
    Ok(affected)
}

/// 分页查询通知配置列表
pub async fn get_by_page(
    db: &DbConn,
    query: NotificationConfigListQuery,
) -> Result<ResultPage<Vec<NotificationConfigVO>>> {
    NotificationConfigModel::find_by_page(db, &query)
        .await
        .map_err(|e| Error::from(e.to_string()))
}

/// 根据ID查询通知配置详情
pub async fn get_by_id(db: &DbConn, id: i64) -> Result<NotificationConfigVO> {
    NotificationConfigModel::find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("通知配置不存在"))
}

/// 新增通知配置
pub async fn create(db: &DbConn, req: NotificationConfigSaveDTO) -> Result<i64> {
    if req.scene_code.is_empty() {
        return Err(Error::from("场景编码不能为空"));
    }
    // 唯一性校验（website_id + scene_code）
    if let Some(wid) = req.website_id {
        if NotificationConfigModel::find_by_website_and_scene(db, wid, &req.scene_code)
            .await?
            .is_some()
        {
            return Err(Error::from("该网站下已存在相同场景编码的通知配置"));
        }
    }

    let req_clone = req.clone();
    let id = db
        .transaction::<_, i64, DbErr>(|txn| {
            let req_clone2 = req_clone.clone();
            Box::pin(async move { NotificationConfigModel::insert(txn, &req_clone2).await })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(id)
}

/// 更新通知配置
pub async fn update(db: &DbConn, id: i64, req: NotificationConfigSaveDTO) -> Result<i64> {
    if req.scene_code.is_empty() {
        return Err(Error::from("场景编码不能为空"));
    }

    let req_clone = req.clone();
    db.transaction::<_, i64, DbErr>(|txn| {
        let req_clone2 = req_clone.clone();
        Box::pin(async move { NotificationConfigModel::update(txn, id, &req_clone2).await })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;
    Ok(id)
}

/// 切换启用状态
pub async fn toggle_enabled(db: &DbConn, id: i64, enabled: i32) -> Result<i64> {
    NotificationConfigModel::update_enabled(db, id, enabled)
        .await
        .map_err(|e| Error::from(e.to_string()))
}

/// 批量软删除
pub async fn batch_delete(db: &DbConn, ids: Vec<i64>) -> Result<i64> {
    db.transaction::<_, i64, DbErr>(|txn| {
        let ids_clone = ids.clone();
        Box::pin(async move { NotificationConfigModel::batch_delete(txn, ids_clone).await })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))
}

/// 发送通知（触发器）
///
/// 根据网站ID和场景编码查找通知配置，若已启用且渠道包含 email，
/// 则使用 context 渲染邮件主题和正文（简单变量替换 {{ var_name }}），
/// 通过系统邮件服务（mail_service::send_mail）实际发送邮件，
/// 复用 mxx_mail_config 默认邮箱账号 + mxx_mail_log 日志记录。
///
/// 失败不抛错（仅记录日志），避免影响主业务流程。
pub async fn send_notification(
    db: &DbConn,
    website_id: i64,
    scene_code: &str,
    context: serde_json::Value,
) -> Result<()> {
    let config = NotificationConfigModel::find_by_website_and_scene(db, website_id, scene_code)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from(format!("未找到场景编码 {} 的通知配置", scene_code)))?;

    // 未启用则跳过
    if config.enabled.unwrap_or(0) != 1 {
        return Ok(());
    }

    // 检查渠道是否包含 email
    let channels = config
        .channels
        .clone()
        .unwrap_or_else(|| "email".to_string());
    if !channels.split(',').any(|c| c.trim() == "email") {
        return Ok(());
    }

    // 渲染邮件主题和正文（简单变量替换 {{ var_name }}）
    let subject = render_template(config.email_subject.as_deref().unwrap_or(""), &context);
    let body = render_template(config.email_body.as_deref().unwrap_or(""), &context);
    let recipients_str = config.recipient_emails.clone().unwrap_or_default();

    // 解析收件人邮箱（支持逗号/分号分隔）
    let to_emails: Vec<String> = recipients_str
        .split([',', ';'])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if to_emails.is_empty() {
        log::warn!(
            "[通知触发] website_id={}, scene_code={}, 收件人为空，跳过发送",
            website_id,
            scene_code
        );
        return Ok(());
    }

    log::info!(
        "[通知触发] website_id={}, scene_code={}, recipients={:?}, subject={}",
        website_id,
        scene_code,
        to_emails,
        subject
    );

    // 构造邮件请求，复用系统邮件服务发送
    let mail_req = SendMailRequest {
        customer_id: None,
        to_emails,
        cc_emails: None,
        subject: Some(subject),
        body: Some(body),
        doc_url: None,
        contact_ids: None,
    };

    // 调用系统邮件服务发送，失败仅记录日志不抛错
    match mail_service::send_mail(db, mail_req, None, Some(format!("cms-notification:{}", scene_code))).await {
        Ok(log_id) => {
            log::info!(
                "[通知发送成功] website_id={}, scene_code={}, mail_log_id={}",
                website_id,
                scene_code,
                log_id
            );
        }
        Err(e) => {
            log::warn!(
                "[通知发送失败] website_id={}, scene_code={}, error={}",
                website_id,
                scene_code,
                e
            );
        }
    }

    Ok(())
}

/// 简单模板渲染：将 {{ var_name }} 替换为 context 中对应的值
fn render_template(tpl: &str, context: &serde_json::Value) -> String {
    let mut result = tpl.to_string();
    if let Some(obj) = context.as_object() {
        for (key, val) in obj {
            let placeholder = format!("{{{{ {} }}}}", key);
            let replacement = match val {
                serde_json::Value::String(s) => s.clone(),
                _ => val.to_string(),
            };
            result = result.replace(&placeholder, &replacement);
        }
    }
    result
}
