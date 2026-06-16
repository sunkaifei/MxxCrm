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
#[sea_orm(table_name = "mxx_template_user_data")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 模板id
    pub template_id: Option<i64>,
    /// 模型id
    pub model_id: Option<i32>,
    /// 模板类型，1首页，2列表，3内容，4标签，5专题
    pub type_id: Option<i32>,
    /// 模板名称
    pub name: Option<String>,
    /// 模板页面内容
    pub temptext: Option<String>,
    /// 添加时间
    pub create_time: Option<DateTime>,
    /// 排序
    pub sort: Option<i32>,
    /// 是否删除：0否，1是
    pub deleted: Option<i32>,
    /// 审核状态：0不显示，1显示
    pub status: Option<i32>,
}


#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}