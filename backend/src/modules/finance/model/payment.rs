//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use serde::{Deserialize, Serialize};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use crate::modules::finance::entity::payment;

/// 付款记录DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentDTO {
    pub id: i64,
    pub payment_no: Option<String>,
    pub purchase_order_id: Option<i64>,
    pub purchase_order_no: Option<String>,
    pub supplier_name: Option<String>,
    pub payment_type: Option<i32>,
    pub payment_amount: f64,
    pub payment_method: Option<i32>,
    pub bank_account: Option<String>,
    pub status: Option<i32>,
    pub applicant_id: Option<i64>,
    pub applicant_name: Option<String>,
    pub apply_time: Option<String>,
    pub approver_id: Option<i64>,
    pub approver_name: Option<String>,
    pub approve_time: Option<String>,
    pub approve_remark: Option<String>,
    pub payment_date: Option<String>,
    pub remark: Option<String>,
    pub create_time: Option<String>,
}

impl From<payment::Model> for PaymentDTO {
    fn from(model: payment::Model) -> Self {
        Self {
            id: model.id,
            payment_no: model.payment_no,
            purchase_order_id: model.purchase_order_id,
            purchase_order_no: model.purchase_order_no,
            supplier_name: model.supplier_name,
            payment_type: model.payment_type,
            payment_amount: model.payment_amount.to_f64().unwrap_or_default(),
            payment_method: model.payment_method,
            bank_account: model.bank_account,
            status: model.status,
            applicant_id: model.applicant_id,
            applicant_name: model.applicant_name,
            apply_time: model.apply_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            approver_id: model.approver_id,
            approver_name: model.approver_name,
            approve_time: model.approve_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            approve_remark: model.approve_remark,
            payment_date: model.payment_date.map(|d| d.format("%Y-%m-%d").to_string()),
            remark: model.remark,
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// 付款查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub payment_no: Option<String>,
    pub purchase_order_no: Option<String>,
    pub supplier_name: Option<String>,
    pub status: Option<i32>,
}

/// 申请付款请求体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentApplyDTO {
    pub purchase_order_id: Option<i64>,
    pub purchase_order_no: Option<String>,
    pub supplier_name: Option<String>,
    pub payment_type: Option<i32>,
    pub payment_amount: f64,
    pub payment_method: Option<i32>,
    pub bank_account: Option<String>,
    pub remark: Option<String>,
}

/// 审批请求体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentApproveDTO {
    pub id: i64,
    pub approved: bool,
    pub remark: Option<String>,
}

/// 确认付款
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentConfirmDTO {
    pub id: i64,
    pub payment_date: String,
}

/// 取消请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentCancelDTO {
    pub id: i64,
    pub remark: String,
}

/// f64 转 Decimal
pub fn to_decimal(value: f64) -> Decimal {
    Decimal::from_f64(value).unwrap_or_default()
}
