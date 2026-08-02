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

/// 模板变量实体
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_template_var")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 变量key
    pub var_key: Option<String>,
    /// 变量标签
    pub var_label: Option<String>,
    /// 变量值
    pub var_value: Option<String>,
    /// 变量类型：1=文本, 2=数字, 3=布尔, 4=HTML, 5=图片
    pub var_type: Option<i32>,
    /// 变量分组
    pub var_group: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
    /// 删除标记：0未删除，1已删除
    pub deleted: Option<i32>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
}


#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
