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
#[sea_orm(table_name = "mxx_finance_payment")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 付款单号
    pub payment_no: Option<String>,

    /// 采购订单ID
    pub purchase_order_id: Option<i64>,

    /// 采购订单号
    pub purchase_order_no: Option<String>,

    /// 供应商名称
    pub supplier_name: Option<String>,

    /// 付款类型
    pub payment_type: Option<i32>,

    /// 付款金额
    pub payment_amount: Decimal,

    /// 付款方式
    pub payment_method: Option<i32>,

    /// 银行账号
    pub bank_account: Option<String>,

    /// 状态: 0=待审批 1=已审批 2=已付款 3=已取消
    pub status: Option<i32>,

    /// 申请人ID
    pub applicant_id: Option<i64>,

    /// 申请人姓名
    pub applicant_name: Option<String>,

    /// 申请时间
    pub apply_time: Option<DateTime>,

    /// 审批人ID
    pub approver_id: Option<i64>,

    /// 审批人姓名
    pub approver_name: Option<String>,

    /// 审批时间
    pub approve_time: Option<DateTime>,

    /// 审批备注
    pub approve_remark: Option<String>,

    /// 付款日期
    pub payment_date: Option<chrono::NaiveDate>,

    /// 备注
    pub remark: Option<String>,

    /// 创建人ID
    pub created_by: Option<i64>,

    /// 创建时间
    pub create_time: Option<DateTime>,

    /// 更新人ID
    pub updated_by: Option<i64>,

    /// 更新时间
    pub update_time: Option<DateTime>,

    /// 删除标识: 0=未删除 1=已删除
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
