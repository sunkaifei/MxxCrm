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
#[sea_orm(table_name = "mxx_system_dict")]
pub struct Model {
    // 字典主键
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    // 字典名称
    pub dict_name: Option<String>,
    // 字典编码
    pub dict_code: Option<String>,
    // 排序
    pub sort: Option<i32>,
    // 状态（0正常 1停用）
    pub status: Option<i32>,
    // 创建者
    pub create_by: Option<String>,
    // 创建时间
    pub create_time: Option<DateTime>,
    // 更新者
    pub update_by: Option<String>,
    // 更新时间
    pub update_time: Option<DateTime>,
    // 备注
    pub remark: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}