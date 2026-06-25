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
#[sea_orm(table_name = "mxx_attachment_category")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 父级分类
    pub parent_id: Option<i64>,
    /// 文件管理类型（1image,2video）
    pub type_id: Option<i32>,
    /// 分类名称
    pub name: Option<String>,
    /// 分类目录
    pub en_name: Option<String>,
    /// 排序（升序）
    pub sort: Option<i32>,
    /// 图片数量
    pub count_pic: Option<i32>,
    /// 创建时间
    pub create_time: Option<DateTime>,
}


#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}