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

/// 自定义字段定义表实体（元数据层唯一事实来源）
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_field_def")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 业务模块标识（crm_customer/crm_lead/crm_opportunity/crm_contact/crm_contract/sale_quotation/sale_order）
    pub module: Option<String>,
    /// 字段键：同模块唯一，正则 ^[a-z][a-z0-9_]{1,63}$，创建后不可修改
    pub field_key: Option<String>,
    /// 显示名
    pub field_label: Option<String>,
    /// 类型：1文本 2多行文本 3数字 4日期 5日期时间 6单选 7多选 8布尔 9附件 10成员 11金额；创建后不可修改
    pub field_type: Option<i32>,
    /// 类型配置：单选/多选 {"choices":[{"label","value","active"}]}；数字 {"precision","min","max"}；通用可带 {"defaultValue"}
    pub options: Option<serde_json::Value>,
    /// 必填：0否 1是
    pub required: Option<i32>,
    /// 可见角色 role_key 数组，NULL=全部可见
    pub visible_roles: Option<serde_json::Value>,
    /// 可编辑角色 role_key 数组，NULL=跟随 visible_roles
    pub editable_roles: Option<serde_json::Value>,
    /// 列表显示：0否 1是
    pub list_visible: Option<i32>,
    /// 列表内列顺序
    pub list_sort: Option<i32>,
    /// 参与筛选：0否 1是（P1 生效）
    pub filterable: Option<i32>,
    /// 表达式索引：0未建 1已建（P1 生效）
    pub indexed: Option<i32>,
    /// 存储：0=custom_fields jsonb（默认）1=物化物理列（P2）
    pub storage_type: Option<i32>,
    /// 状态：1启用 0停用
    pub status: Option<i32>,
    /// 表单内排序
    pub sort: Option<i32>,
    /// 备注
    pub remark: Option<String>,
    /// 创建者
    pub create_by: Option<String>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新者
    pub update_by: Option<String>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 删除：0正常 1已删除（逻辑删除，历史数据保留）
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
