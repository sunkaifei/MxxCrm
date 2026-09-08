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

#[derive(Clone, Default, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_hr_profile_log")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub admin_id: i64,
    pub field: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    /// 操作类型：1本人首填 2本人修改 3HR代改 4HR解锁
    pub operate_type: Option<i32>,
    pub operator_id: Option<i64>,
    pub operator_name: Option<String>,
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
