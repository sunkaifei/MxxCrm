//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::system::entity::{
    mail_config, mail_config::Entity as MailConfig, mail_log, mail_log::Entity as MailLog,
    mail_template, mail_template::Entity as MailTemplate,
};
use crate::utils::string_utils::{
    deserialize_string_to_i32, deserialize_string_to_i64, deserialize_string_to_u64,
    serialize_option_u64_to_string,
};
use chrono::Local;
use sea_orm::prelude::DateTime;
use sea_orm::*;

// ============================ 邮箱账号配置 ============================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MailConfigSaveRequest {
    /// 配置名称
    pub name: Option<String>,
    /// SMTP 主机
    pub host: Option<String>,
    /// SMTP 端口
    #[serde(deserialize_with = "deserialize_string_to_i32")]
    pub port: Option<i32>,
    /// 用户名
    pub username: Option<String>,
    /// 密码
    pub password: Option<String>,
    /// 发件人邮箱
    pub from_email: Option<String>,
    /// 发件人名称
    pub from_name: Option<String>,
    /// 是否 SSL（1 是 0 否）
    #[serde(deserialize_with = "deserialize_string_to_i32")]
    pub is_ssl: Option<i32>,
    /// 是否默认（1 是 0 否）
    #[serde(deserialize_with = "deserialize_string_to_i32")]
    pub is_default: Option<i32>,
    /// 状态（1 启用 0 停用）
    #[serde(deserialize_with = "deserialize_string_to_i32")]
    pub status: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MailConfigUpdateRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 配置名称
    pub name: Option<String>,
    /// SMTP 主机
    pub host: Option<String>,
    /// SMTP 端口
    #[serde(deserialize_with = "deserialize_string_to_i32")]
    pub port: Option<i32>,
    /// 用户名
    pub username: Option<String>,
    /// 密码
    pub password: Option<String>,
    /// 发件人邮箱
    pub from_email: Option<String>,
    /// 发件人名称
    pub from_name: Option<String>,
    /// 是否 SSL（1 是 0 否）
    #[serde(deserialize_with = "deserialize_string_to_i32")]
    pub is_ssl: Option<i32>,
    /// 是否默认（1 是 0 否）
    #[serde(deserialize_with = "deserialize_string_to_i32")]
    pub is_default: Option<i32>,
    /// 状态（1 启用 0 停用）
    #[serde(deserialize_with = "deserialize_string_to_i32")]
    pub status: Option<i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MailConfigListQuery {
    #[serde(rename = "page")]
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub name: Option<String>,
    #[serde(deserialize_with = "deserialize_string_to_i32", default)]
    pub status: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct MailConfigVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub name: Option<String>,
    pub host: Option<String>,
    pub port: Option<i32>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub from_email: Option<String>,
    pub from_name: Option<String>,
    pub is_ssl: Option<i32>,
    pub is_default: Option<i32>,
    pub status: Option<i32>,
    pub create_by: Option<i64>,
    pub create_time: Option<String>,
    pub update_by: Option<i64>,
    pub update_time: Option<String>,
}

impl From<mail_config::Model> for MailConfigVO {
    fn from(m: mail_config::Model) -> Self {
        Self {
            id: Option::from(m.id),
            name: m.name,
            host: m.host,
            port: m.port,
            username: m.username,
            password: m.password,
            from_email: m.from_email,
            from_name: m.from_name,
            is_ssl: m.is_ssl,
            is_default: m.is_default,
            status: m.status,
            create_by: m.create_by,
            create_time: m.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_by: m.update_by,
            update_time: m.update_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

// ============================ 邮件模板 ============================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MailTemplateSaveRequest {
    /// 模板名称
    pub name: Option<String>,
    /// 邮件主题
    pub subject: Option<String>,
    /// 邮件正文
    pub body: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MailTemplateUpdateRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 模板名称
    pub name: Option<String>,
    /// 邮件主题
    pub subject: Option<String>,
    /// 邮件正文
    pub body: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MailTemplateListQuery {
    #[serde(rename = "page")]
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct MailTemplateVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub name: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub create_by: Option<i64>,
    pub create_time: Option<String>,
    pub update_by: Option<i64>,
    pub update_time: Option<String>,
}

impl From<mail_template::Model> for MailTemplateVO {
    fn from(m: mail_template::Model) -> Self {
        Self {
            id: Option::from(m.id),
            name: m.name,
            subject: m.subject,
            body: m.body,
            create_by: m.create_by,
            create_time: m.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_by: m.update_by,
            update_time: m.update_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct MailTemplateOption {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub name: Option<String>,
}

// ============================ 发送邮件请求 ============================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SendMailRequest {
    /// 客户ID
    #[serde(deserialize_with = "deserialize_string_to_i64", default)]
    pub customer_id: Option<i64>,
    /// 收件人邮箱集合
    pub to_emails: Vec<String>,
    /// 抄送人邮箱集合
    pub cc_emails: Option<Vec<String>>,
    /// 邮件主题
    pub subject: Option<String>,
    /// 邮件正文
    pub body: Option<String>,
    /// 文档 URL（若提供则异步抓取其内容作为正文）
    pub doc_url: Option<String>,
    /// 联系人ID集合（多个英文逗号分隔，记录到日志）
    pub contact_ids: Option<String>,
}

// ============================ 邮件日志 ============================

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MailLogListQuery {
    #[serde(rename = "page")]
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    #[serde(deserialize_with = "deserialize_string_to_i64", default)]
    pub customer_id: Option<i64>,
    #[serde(deserialize_with = "deserialize_string_to_i32", default)]
    pub status: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct MailLogVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub customer_id: Option<i64>,
    pub contact_ids: Option<String>,
    pub from_email: Option<String>,
    pub to_emails: Option<String>,
    pub cc_emails: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub status: Option<i32>,
    pub error_msg: Option<String>,
    pub smtp_message_id: Option<String>,
    pub sender_id: Option<i64>,
    pub sender_name: Option<String>,
    pub send_time: Option<String>,
    pub create_time: Option<String>,
}

impl From<mail_log::Model> for MailLogVO {
    fn from(m: mail_log::Model) -> Self {
        Self {
            id: Option::from(m.id),
            customer_id: m.customer_id,
            contact_ids: m.contact_ids,
            from_email: m.from_email,
            to_emails: m.to_emails,
            cc_emails: m.cc_emails,
            subject: m.subject,
            body: m.body,
            status: m.status,
            error_msg: m.error_msg,
            smtp_message_id: m.smtp_message_id,
            sender_id: m.sender_id,
            sender_name: m.sender_name,
            send_time: m.send_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            create_time: m.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// 邮件日志写入 DTO
#[derive(Debug, Clone)]
pub struct MailLogSaveDTO {
    pub customer_id: Option<i64>,
    pub contact_ids: Option<String>,
    pub from_email: Option<String>,
    pub to_emails: Option<String>,
    pub cc_emails: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub status: Option<i32>,
    pub error_msg: Option<String>,
    pub smtp_message_id: Option<String>,
    pub sender_id: Option<i64>,
    pub sender_name: Option<String>,
    pub send_time: Option<DateTime>,
}

// ============================ Model 层数据访问 ============================

pub struct MailConfigModel;

impl MailConfigModel {
    /// 新增邮箱配置
    pub async fn insert<C: ConnectionTrait>(
        db: &C,
        req: &MailConfigSaveRequest,
        user_id: Option<i64>,
    ) -> Result<i64, DbErr> {
        let now = Local::now().naive_utc();
        let payload = mail_config::ActiveModel {
            name: Set(req.name.clone()),
            host: Set(req.host.clone()),
            port: Set(req.port),
            username: Set(req.username.clone()),
            password: Set(req.password.clone()),
            from_email: Set(req.from_email.clone()),
            from_name: Set(req.from_name.clone()),
            is_ssl: Set(req.is_ssl),
            is_default: Set(req.is_default),
            status: Set(req.status),
            create_by: Set(user_id),
            create_time: Set(Some(now)),
            update_by: Set(user_id),
            update_time: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        MailConfig::insert(payload).exec(db).await.map(|r| r.last_insert_id)
    }

    /// 修改邮箱配置
    pub async fn update<C: ConnectionTrait>(
        db: &C,
        id: i64,
        req: &MailConfigUpdateRequest,
        user_id: Option<i64>,
    ) -> Result<i64, DbErr> {
        let payload = mail_config::ActiveModel {
            name: Set(req.name.clone()),
            host: Set(req.host.clone()),
            port: Set(req.port),
            username: Set(req.username.clone()),
            password: Set(req.password.clone()),
            from_email: Set(req.from_email.clone()),
            from_name: Set(req.from_name.clone()),
            is_ssl: Set(req.is_ssl),
            is_default: Set(req.is_default),
            status: Set(req.status),
            update_by: Set(user_id),
            update_time: Set(Some(Local::now().naive_utc())),
            ..Default::default()
        };
        let r = MailConfig::update_many()
            .set(payload)
            .filter(mail_config::Column::Id.eq(id))
            .filter(mail_config::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(r.rows_affected as i64)
    }

    /// 软删除
    pub async fn batch_delete_by_ids<C: ConnectionTrait>(
        db: &C,
        ids: Vec<i64>,
    ) -> Result<i64, DbErr> {
        let payload = mail_config::ActiveModel {
            deleted: Set(Some(1)),
            update_time: Set(Some(Local::now().naive_utc())),
            ..Default::default()
        };
        let r = MailConfig::update_many()
            .set(payload)
            .filter(mail_config::Column::Id.is_in(ids))
            .filter(mail_config::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(r.rows_affected as i64)
    }

    /// 重置所有默认配置
    pub async fn update_reset_default<C: ConnectionTrait>(db: &C) -> Result<i64, DbErr> {
        let payload = mail_config::ActiveModel {
            is_default: Set(Some(0)),
            update_time: Set(Some(Local::now().naive_utc())),
            ..Default::default()
        };
        let r = MailConfig::update_many()
            .set(payload)
            .filter(mail_config::Column::IsDefault.eq(1))
            .filter(mail_config::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(r.rows_affected as i64)
    }

    /// 设为默认
    pub async fn update_set_default<C: ConnectionTrait>(
        db: &C,
        id: i64,
        user_id: Option<i64>,
    ) -> Result<i64, DbErr> {
        let payload = mail_config::ActiveModel {
            is_default: Set(Some(1)),
            status: Set(Some(1)),
            update_by: Set(user_id),
            update_time: Set(Some(Local::now().naive_utc())),
            ..Default::default()
        };
        let r = MailConfig::update_many()
            .set(payload)
            .filter(mail_config::Column::Id.eq(id))
            .filter(mail_config::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(r.rows_affected as i64)
    }

    /// 根据 ID 查询
    pub async fn find_by_id<C: ConnectionTrait>(
        db: &C,
        id: i64,
    ) -> Result<Option<mail_config::Model>, DbErr> {
        MailConfig::find_by_id(id)
            .filter(mail_config::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 查询默认邮箱配置
    pub async fn find_default(db: &DbConn) -> Result<Option<mail_config::Model>, DbErr> {
        MailConfig::find()
            .filter(mail_config::Column::IsDefault.eq(1))
            .filter(mail_config::Column::Status.eq(1))
            .filter(mail_config::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 分页查询
    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        page_size: i64,
        name: Option<String>,
        status: Option<i32>,
    ) -> Result<(Vec<mail_config::Model>, i64), DbErr> {
        let paginator = MailConfig::find()
            .filter(mail_config::Column::Deleted.eq(0))
            .apply_if(name, |q, v| {
                q.filter(mail_config::Column::Name.contains(format!("%{}%", v)))
            })
            .apply_if(status, |q, v| q.filter(mail_config::Column::Status.eq(v)))
            .order_by_desc(mail_config::Column::Id)
            .paginate(db, page_size as u64);
        let num_pages = paginator.num_pages().await? as i64;
        let list = paginator.fetch_page((page - 1) as u64).await?;
        Ok((list, num_pages))
    }

    /// 数量统计
    pub async fn select_count(
        db: &DbConn,
        name: Option<String>,
        status: Option<i32>,
    ) -> Result<i64, DbErr> {
        MailConfig::find()
            .filter(mail_config::Column::Deleted.eq(0))
            .apply_if(name, |q, v| {
                q.filter(mail_config::Column::Name.contains(format!("%{}%", v)))
            })
            .apply_if(status, |q, v| q.filter(mail_config::Column::Status.eq(v)))
            .count(db)
            .await
            .map(|c| c as i64)
    }
}

pub struct MailTemplateModel;

impl MailTemplateModel {
    /// 新增模板
    pub async fn insert<C: ConnectionTrait>(
        db: &C,
        req: &MailTemplateSaveRequest,
        user_id: Option<i64>,
    ) -> Result<i64, DbErr> {
        let now = Local::now().naive_utc();
        let payload = mail_template::ActiveModel {
            name: Set(req.name.clone()),
            subject: Set(req.subject.clone()),
            body: Set(req.body.clone()),
            create_by: Set(user_id),
            create_time: Set(Some(now)),
            update_by: Set(user_id),
            update_time: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        MailTemplate::insert(payload).exec(db).await.map(|r| r.last_insert_id)
    }

    /// 修改模板
    pub async fn update<C: ConnectionTrait>(
        db: &C,
        id: i64,
        req: &MailTemplateUpdateRequest,
        user_id: Option<i64>,
    ) -> Result<i64, DbErr> {
        let payload = mail_template::ActiveModel {
            name: Set(req.name.clone()),
            subject: Set(req.subject.clone()),
            body: Set(req.body.clone()),
            update_by: Set(user_id),
            update_time: Set(Some(Local::now().naive_utc())),
            ..Default::default()
        };
        let r = MailTemplate::update_many()
            .set(payload)
            .filter(mail_template::Column::Id.eq(id))
            .filter(mail_template::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(r.rows_affected as i64)
    }

    /// 软删除
    pub async fn batch_delete_by_ids<C: ConnectionTrait>(
        db: &C,
        ids: Vec<i64>,
    ) -> Result<i64, DbErr> {
        let payload = mail_template::ActiveModel {
            deleted: Set(Some(1)),
            update_time: Set(Some(Local::now().naive_utc())),
            ..Default::default()
        };
        let r = MailTemplate::update_many()
            .set(payload)
            .filter(mail_template::Column::Id.is_in(ids))
            .filter(mail_template::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(r.rows_affected as i64)
    }

    /// 根据 ID 查询
    pub async fn find_by_id<C: ConnectionTrait>(
        db: &C,
        id: i64,
    ) -> Result<Option<mail_template::Model>, DbErr> {
        MailTemplate::find_by_id(id)
            .filter(mail_template::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 查询全部模板选项（id + name）
    pub async fn find_all_options(db: &DbConn) -> Result<Vec<MailTemplateOption>, DbErr> {
        let list = MailTemplate::find()
            .filter(mail_template::Column::Deleted.eq(0))
            .order_by_desc(mail_template::Column::Id)
            .all(db)
            .await?;
        Ok(list
            .into_iter()
            .map(|m| MailTemplateOption {
                id: Option::from(m.id),
                name: m.name,
            })
            .collect())
    }

    /// 分页查询
    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        page_size: i64,
        name: Option<String>,
    ) -> Result<(Vec<mail_template::Model>, i64), DbErr> {
        let paginator = MailTemplate::find()
            .filter(mail_template::Column::Deleted.eq(0))
            .apply_if(name, |q, v| {
                q.filter(mail_template::Column::Name.contains(format!("%{}%", v)))
            })
            .order_by_desc(mail_template::Column::Id)
            .paginate(db, page_size as u64);
        let num_pages = paginator.num_pages().await? as i64;
        let list = paginator.fetch_page((page - 1) as u64).await?;
        Ok((list, num_pages))
    }

    /// 数量统计
    pub async fn select_count(db: &DbConn, name: Option<String>) -> Result<i64, DbErr> {
        MailTemplate::find()
            .filter(mail_template::Column::Deleted.eq(0))
            .apply_if(name, |q, v| {
                q.filter(mail_template::Column::Name.contains(format!("%{}%", v)))
            })
            .count(db)
            .await
            .map(|c| c as i64)
    }
}

pub struct MailLogModel;

impl MailLogModel {
    /// 写入邮件日志
    pub async fn insert<C: ConnectionTrait>(
        db: &C,
        dto: &MailLogSaveDTO,
    ) -> Result<i64, DbErr> {
        let now = Local::now().naive_utc();
        let payload = mail_log::ActiveModel {
            customer_id: Set(dto.customer_id),
            contact_ids: Set(dto.contact_ids.clone()),
            from_email: Set(dto.from_email.clone()),
            to_emails: Set(dto.to_emails.clone()),
            cc_emails: Set(dto.cc_emails.clone()),
            subject: Set(dto.subject.clone()),
            body: Set(dto.body.clone()),
            status: Set(dto.status),
            error_msg: Set(dto.error_msg.clone()),
            smtp_message_id: Set(dto.smtp_message_id.clone()),
            sender_id: Set(dto.sender_id),
            sender_name: Set(dto.sender_name.clone()),
            send_time: Set(dto.send_time),
            create_time: Set(Some(now)),
            ..Default::default()
        };
        MailLog::insert(payload).exec(db).await.map(|r| r.last_insert_id)
    }

    /// 分页查询
    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        page_size: i64,
        customer_id: Option<i64>,
        status: Option<i32>,
    ) -> Result<(Vec<mail_log::Model>, i64), DbErr> {
        let paginator = MailLog::find()
            .apply_if(customer_id, |q, v| q.filter(mail_log::Column::CustomerId.eq(v)))
            .apply_if(status, |q, v| q.filter(mail_log::Column::Status.eq(v)))
            .order_by_desc(mail_log::Column::Id)
            .paginate(db, page_size as u64);
        let num_pages = paginator.num_pages().await? as i64;
        let list = paginator.fetch_page((page - 1) as u64).await?;
        Ok((list, num_pages))
    }

    /// 数量统计
    pub async fn select_count(
        db: &DbConn,
        customer_id: Option<i64>,
        status: Option<i32>,
    ) -> Result<i64, DbErr> {
        MailLog::find()
            .apply_if(customer_id, |q, v| q.filter(mail_log::Column::CustomerId.eq(v)))
            .apply_if(status, |q, v| q.filter(mail_log::Column::Status.eq(v)))
            .count(db)
            .await
            .map(|c| c as i64)
    }

    /// 按客户查询全部邮件日志
    pub async fn select_by_customer(
        db: &DbConn,
        customer_id: i64,
    ) -> Result<Vec<mail_log::Model>, DbErr> {
        MailLog::find()
            .filter(mail_log::Column::CustomerId.eq(customer_id))
            .order_by_desc(mail_log::Column::Id)
            .all(db)
            .await
    }
}
