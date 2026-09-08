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

/// 编号规则配置实体
/// 表 mxx_company_code_rule：按模块存储编号生成规则
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_company_code_rule")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 模块编码，如 customer/order/contract/tech_doc
    pub module_code: Option<String>,
    pub module_name: Option<String>,
    pub rule_name: Option<String>,
    /// 公司简称冗余字段，取自 mxx_company_info.company_abbr
    pub company_abbr: Option<String>,
    /// 部门编码，管理员自定义
    pub dept_code: Option<String>,
    pub biz_type_code: Option<String>,
    /// 分隔符，默认 -
    pub separator: Option<String>,
    /// 段位配置 JSON 数组，按 sort 排序拼接
    pub segments: Option<Json>,
    /// 流水号位数，默认 4
    pub seq_length: Option<i16>,
    /// 是否启用，1=是 0=否
    pub enabled: Option<i16>,
    pub remark: Option<String>,
    pub deleted: Option<i16>,
    pub created_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub updated_by: Option<i64>,
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
