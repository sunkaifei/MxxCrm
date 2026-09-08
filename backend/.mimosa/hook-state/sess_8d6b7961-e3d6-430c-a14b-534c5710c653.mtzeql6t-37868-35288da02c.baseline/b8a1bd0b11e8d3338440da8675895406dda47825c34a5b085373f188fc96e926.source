//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 第三方接口统一配置实体
//!

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_integration_config")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 配置分类（payment/logistics/esign/invoice/notification/exchange_rate/ai）
    pub category: Option<String>,
    /// 接口编码
    pub integration_code: Option<String>,
    /// 接口名称
    pub integration_name: Option<String>,
    /// 配置 JSON
    pub config_json: Option<serde_json::Value>,
    /// 接口基础地址
    pub api_base_url: Option<String>,
    /// 是否启用（1启用 0禁用）
    pub enabled: Option<i32>,
    /// 排序
    pub sort_order: Option<i32>,
    /// 最后测试时间
    pub last_test_time: Option<DateTime>,
    /// 最后测试结果（1成功 0失败）
    pub last_test_result: Option<i32>,
    /// 最后测试消息
    pub last_test_message: Option<String>,
    /// 是否加密存储敏感字段（1是 0否）
    pub is_encrypted: Option<i32>,
    /// 备注
    pub remark: Option<String>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 软删除（0未删除 1已删除）
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
