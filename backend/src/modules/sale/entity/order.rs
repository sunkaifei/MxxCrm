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
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_sale_order")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub order_no: Option<String>,
    pub title: Option<String>,
    pub order_type: Option<i32>,
    #[sea_orm(column_name = "status")]
    pub order_status: Option<i32>,
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    pub contact_id: Option<i64>,
    pub contact_name: Option<String>,
    pub opportunity_id: Option<i64>,
    pub quotation_id: Option<i64>,
    pub contract_id: Option<i64>,
    pub order_date: Option<Date>,
    pub delivery_date: Option<Date>,
    pub currency: Option<i32>,
    pub exchange_rate: Option<Decimal>,
    pub product_amount: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub shipping_fee: Option<Decimal>,
    pub tax_amount: Option<Decimal>,
    pub other_fee: Option<Decimal>,
    pub total_amount: Option<Decimal>,
    pub paid_amount: Option<Decimal>,
    pub unpaid_amount: Option<Decimal>,
    #[sea_orm(column_name = "payment_status")]
    pub pay_status: Option<i32>,
    pub payment_method: Option<i32>,
    pub payment_due_date: Option<Date>,
    pub shipping_method: Option<i32>,
    pub tracking_no: Option<String>,
    pub shipping_time: Option<DateTime>,
    pub complete_time: Option<DateTime>,
    pub receiver_name: Option<String>,
    pub receiver_phone: Option<String>,
    pub shipping_address: Option<String>,
    pub billing_address: Option<String>,
    /// 付款方企业名称（买方/客户）
    pub buyer_company_name: Option<String>,
    /// 付款方账户名称
    pub buyer_account_name: Option<String>,
    /// 付款方开户行
    pub buyer_bank_name: Option<String>,
    /// 付款方银行账号
    pub buyer_account_number: Option<String>,
    /// 收款方企业名称（卖方/我方企业）
    pub seller_company_name: Option<String>,
    /// 收款方开户行
    pub seller_bank_name: Option<String>,
    /// 收款方账户名称
    pub seller_account_name: Option<String>,
    /// 收款方银行账号
    pub seller_account_number: Option<String>,
    pub remark: Option<String>,
    pub owner_user_id: Option<i64>,
    pub dept_id: Option<i64>,
    /// 审批状态（0-草稿, 1-待审批, 2-审批中, 3-已通过, 4-已驳回）
    pub approval_status: Option<i32>,
    /// 审批实例ID（关联 mxx_system_approval_instance）
    pub instance_id: Option<i64>,
    pub create_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub update_by: Option<i64>,
    pub update_time: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
