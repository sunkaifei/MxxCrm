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

/// 标准敏感字段权限表实体（FLS，P2-1；仅开放白名单字段，见 docs/权限体系优化方案.md）
/// 与 mxx_system_field_def 同构的「角色数组 + NULL=全部可见」语义（A8 模式）
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_field_perm")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 业务模块标识（crm_customer/crm_contact/crm_contract，与 field_def 的 module 口径一致）
    pub module: Option<String>,
    /// 标准字段键：与实体源码字段名一致（如 personal_mobile/amount），同模块唯一
    pub field_key: Option<String>,
    /// 显示名：配置 Tab 展示用
    pub field_label: Option<String>,
    /// 可见角色 role_key 数组，NULL=全部可见；后端序列化出口按此裁剪（安全真源）
    pub visible_roles: Option<serde_json::Value>,
    /// 可编辑角色 role_key 数组，NULL=跟随 visible_roles；更新入口白名单过滤（防绕过）
    pub editable_roles: Option<serde_json::Value>,
    /// 创建者
    pub create_by: Option<String>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新者
    pub update_by: Option<String>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 删除：0正常 1已删除（逻辑删除）
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
