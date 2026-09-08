//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 服务权益操作日志（mxx_sale_entitlement_log，P0.7）
//!

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_sale_entitlement_log")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub entitlement_id: i64,
    /// 1创建(自动)2创建(手动)3激活4暂停5取消6续约7删除8还原9配额扣减10到期流转
    pub action: Option<i32>,
    pub from_status: Option<i32>,
    pub to_status: Option<i32>,
    pub operator_id: Option<i64>,
    pub operator_name: Option<String>,
    pub remark: Option<String>,
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
