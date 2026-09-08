//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 销售退货单实体层
//!

use sea_orm::entity::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_sale_refund")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub refund_no: Option<String>,
    pub title: Option<String>,
    pub order_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    /// 退货类型：1=整单退货, 2=部分退货
    pub refund_type: Option<i16>,
    pub refund_reason: Option<String>,
    /// 退货状态：1=草稿,2=待审批,3=审批通过,4=待收货,5=已收货,6=质检中,7=已完成,8=已驳回,9=已取消
    pub refund_status: Option<i16>,
    /// 审批状态：0=草稿,1=待审批,2=审批中,3=已通过,4=已驳回
    pub approval_status: Option<i16>,
    pub instance_id: Option<i64>,
    pub total_amount: Option<Decimal>,
    pub restocking_fee: Option<Decimal>,
    pub refund_amount: Option<Decimal>,
    pub refunded_amount: Option<Decimal>,
    pub warehouse_id: Option<i64>,
    pub receiver: Option<String>,
    pub receiver_phone: Option<String>,
    pub receiver_address: Option<String>,
    pub logistics_no: Option<String>,
    pub logistics_company: Option<String>,
    /// 质检结果：0=未质检,1=合格,2=不合格
    pub quality_check_result: Option<i16>,
    pub quality_check_remark: Option<String>,
    pub owner_user_id: Option<i64>,
    pub dept_id: Option<i64>,
    pub remark: Option<String>,
    pub create_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub update_by: Option<i64>,
    pub update_time: Option<DateTime>,
    pub deleted: Option<i16>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
