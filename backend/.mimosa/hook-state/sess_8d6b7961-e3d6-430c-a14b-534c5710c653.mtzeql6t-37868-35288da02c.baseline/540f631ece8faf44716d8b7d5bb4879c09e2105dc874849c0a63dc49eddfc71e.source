//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_crm_mail_log")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 客户ID
    pub customer_id: Option<i64>,
    /// 联系人ID集合（多个英文逗号分隔）
    pub contact_ids: Option<String>,
    /// 发件人邮箱
    pub from_email: Option<String>,
    /// 收件人邮箱集合（多个英文逗号分隔）
    pub to_emails: Option<String>,
    /// 抄送人邮箱集合（多个英文逗号分隔）
    pub cc_emails: Option<String>,
    /// 邮件主题
    pub subject: Option<String>,
    /// 邮件正文
    pub body: Option<String>,
    /// 发送状态（0 失败 1 成功）
    pub status: Option<i32>,
    /// 错误信息
    pub error_msg: Option<String>,
    /// SMTP Message-ID
    pub smtp_message_id: Option<String>,
    /// 发送人ID
    pub sender_id: Option<i64>,
    /// 发送人名称
    pub sender_name: Option<String>,
    /// 发送时间
    pub send_time: Option<DateTime>,
    /// 创建时间
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
