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

/// G-2.1: 文章自定义字段定义表
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_article_field")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 所属栏目ID
    pub category_id: i64,
    /// 字段名（英文标识）
    pub field_name: String,
    /// 字段标签（中文显示名）
    pub field_label: Option<String>,
    /// 字段类型：1文本 2富文本 3图片 4数字 5日期 6下拉 7多选
    pub field_type: Option<i32>,
    /// 下拉/多选选项（JSON 数组）
    pub field_options: Option<String>,
    /// 默认值
    pub default_value: Option<String>,
    /// 是否必填：1是 0否
    pub is_required: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：1启用 0停用
    pub status: Option<i32>,
    /// 是否删除：1是 0否
    pub deleted: Option<i32>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
