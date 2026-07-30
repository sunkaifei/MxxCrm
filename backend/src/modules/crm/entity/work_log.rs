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
#[sea_orm(table_name = "mxx_work_log")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub user_id: i64,
    pub user_name: Option<String>,
    /// 动作类型：1=审批 2=跟进 3=回款 4=合同 5=商机 6=其他
    pub action_type: Option<i32>,
    pub action_name: Option<String>,
    /// 业务类型：quotation/order/contract/payment/customer/lead/opportunity
    pub business_type: Option<String>,
    pub business_id: Option<i64>,
    pub business_title: Option<String>,
    pub description: Option<String>,
    /// 处理结果：1=成功 2=驳回 3=失败
    pub result: Option<i32>,
    pub work_date: Option<chrono::NaiveDate>,
    pub create_time: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
