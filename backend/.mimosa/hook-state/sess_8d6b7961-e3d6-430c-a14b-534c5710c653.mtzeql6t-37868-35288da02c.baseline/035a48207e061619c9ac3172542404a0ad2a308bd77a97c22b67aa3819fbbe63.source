//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 卡密池实体（mxx_sale_card_pool）
//!

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_sale_card_pool")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub product_id: Option<i64>,
    pub batch_no: Option<String>,
    /// 卡密/激活码（加密存储）
    pub card_key: Option<String>,
    pub card_password: Option<String>,
    /// 状态：1=未售, 2=已锁定, 3=已售, 4=已作废
    pub status: Option<i32>,
    pub lock_order_id: Option<i64>,
    pub lock_expire_time: Option<DateTime>,
    pub sold_order_id: Option<i64>,
    pub sold_time: Option<DateTime>,
    pub import_batch: Option<String>,
    pub expire_time: Option<DateTime>,
    pub remark: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
