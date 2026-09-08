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
#[sea_orm(table_name = "mxx_system_notification_channel_config")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 通道编码: sms/wecom/dingtalk/feishu
    pub channel_code: Option<String>,

    /// 通道名称
    pub channel_name: Option<String>,

    /// 是否启用: 0=禁用 1=启用
    pub enabled: Option<i16>,

    /// 配置JSON（webhook_url / 短信网关账号等）
    pub config_json: Option<String>,

    /// 备注
    pub remark: Option<String>,

    /// 创建时间
    pub create_time: Option<DateTime>,

    /// 更新时间
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
