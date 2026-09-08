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

/// 网站留言数据模型（mxx_website_leave_msg）
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_website_leave_msg")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(skip_deserializing)]
    pub id: i64,

    /// 站点ID
    pub website_id: Option<i64>,

    /// 栏目ID
    pub category_id: Option<i64>,

    /// 联系人姓名
    pub contact_name: Option<String>,

    /// 联系电话
    pub contact_phone: Option<String>,

    /// 联系邮箱
    pub contact_email: Option<String>,

    /// 留言内容
    pub content: Option<String>,

    /// 状态：0待处理 1已转线索 2已处理 3已忽略
    #[serde(default)]
    pub status: Option<i32>,

    /// 转线索后的线索ID（旧字段，保留兼容）
    pub convert_lead_id: Option<i64>,

    /// 关联产品ID（来自产品页咨询）
    pub product_id: Option<i64>,

    /// 来源页URL
    pub source_url: Option<String>,

    /// 来源标识（默认 website）
    #[serde(default = "default_source")]
    pub source: Option<String>,

    /// 提交者IP
    pub ip_address: Option<String>,

    /// 提交者User-Agent
    pub user_agent: Option<String>,

    /// 转线索后的线索ID（新字段，与 convert_lead_id 同步）
    pub lead_id: Option<i64>,

    /// 是否已转线索：0未转 1已转
    #[serde(default)]
    pub converted_to_lead: Option<i32>,

    /// 备注
    pub remark: Option<String>,

    /// 创建时间
    pub create_time: Option<DateTime>,

    /// 更新时间
    pub update_time: Option<DateTime>,

    /// 软删除：0未删除 1已删除
    #[serde(default)]
    pub deleted: Option<i32>,
}

fn default_source() -> Option<String> {
    Some("website".to_string())
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
