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
use serde::{Deserialize, Serialize};

use crate::modules::system::entity::field_perm;

/// 标准字段权限保存请求（管理侧 save，逐行配置；P2-1）
/// visible_roles/editable_roles 传 null=全部可见/跟随可见，传数组=限定角色（空数组会被归一为 null）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct FieldPermSaveRequest {
    /// 业务模块标识（crm_customer/crm_contact/crm_contract）
    pub module: Option<String>,
    /// 标准字段键（白名单内，如 personal_mobile）
    pub field_key: Option<String>,
    /// 可见角色 role_key 数组，null=全部可见
    pub visible_roles: Option<serde_json::Value>,
    /// 可编辑角色 role_key 数组，null=跟随 visible_roles
    pub editable_roles: Option<serde_json::Value>,
}

/// 标准字段权限查询条件（管理侧 list）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct FieldPermQuery {
    /// 业务模块标识，空=全部模块
    pub module: Option<String>,
}

/// 标准字段权限管理侧 VO（列表共用）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct FieldPermListVO {
    pub id: i64,
    pub module: Option<String>,
    pub field_key: Option<String>,
    pub field_label: Option<String>,
    pub visible_roles: Option<serde_json::Value>,
    pub editable_roles: Option<serde_json::Value>,
    pub update_by: Option<String>,
    pub update_time: Option<String>,
}

impl From<field_perm::Model> for FieldPermListVO {
    fn from(item: field_perm::Model) -> Self {
        FieldPermListVO {
            id: item.id,
            module: item.module,
            field_key: item.field_key,
            field_label: item.field_label,
            visible_roles: item.visible_roles,
            editable_roles: item.editable_roles,
            update_by: item.update_by,
            update_time: item.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}
