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

/// 内容模型字段
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_content_model_field")]
pub struct Model {
    // 字段ID
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    // 模型ID
    pub model_id: Option<i64>,
    // 字段名称
    pub field_name: Option<String>,
    // 字段标签
    pub field_label: Option<String>,
    // 字段类型
    pub field_type: Option<i32>,
    // 字段选项
    pub field_options: Option<String>,
    // 默认值
    pub default_value: Option<String>,
    // 占位提示
    pub placeholder: Option<String>,
    // 是否必填
    pub is_required: Option<i32>,
    // 是否可搜索
    pub is_searchable: Option<i32>,
    // 列表是否显示
    pub is_list_show: Option<i32>,
    // 详情是否显示
    pub is_detail_show: Option<i32>,
    // 排序
    pub sort: Option<i32>,
    // 状态：0停用，1正常
    pub status: Option<i32>,
    // 删除标记：0未删除，1已删除
    pub deleted: Option<i32>,
    // 创建时间
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
