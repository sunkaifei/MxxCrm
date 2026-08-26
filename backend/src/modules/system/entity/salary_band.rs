//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// 岗位薪资带宽（mxx_hr_salary_band）：老板维护岗位最低/最高工资，供入职定薪参照
#[derive(Clone, Default, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_hr_salary_band")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 岗位ID（mxx_system_post.id）
    pub post_id: i64,
    /// 带宽下限
    pub min_salary: Decimal,
    /// 带宽上限
    pub max_salary: Decimal,
    /// 1启用 0停用
    pub status: Option<i32>,
    /// 备注
    pub remark: Option<String>,
    pub create_by: Option<String>,
    pub create_time: Option<NaiveDateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<NaiveDateTime>,
    /// 是否删除（smallint）
    pub deleted: Option<i16>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
