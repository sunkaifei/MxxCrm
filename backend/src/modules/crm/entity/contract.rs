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
use crate::core::r#enum::contract_status_enum::ContractStatus;
use crate::core::r#enum::contract_type_enum::ContractType;
use crate::core::r#enum::currency_code_enum::CurrencyCode;

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_crm_contract")]
pub struct Model {
    /// 合同ID（主键）
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,

    /// 合同编号
    pub contract_no: Option<String>,

    /// 关联客户ID
    pub customer_id: Option<i64>,

    /// 关联商机ID
    pub opportunity_id: Option<i64>,

    /// 合同名称
    #[sea_orm(column_name = "name")]
    pub title: Option<String>,

    /// 合同类型
    pub contract_type: Option<ContractType>,

    /// 合同金额
    pub amount: Option<Decimal>,

    /// 币种
    pub currency: Option<CurrencyCode>,

    /// 税额
    pub tax_amount: Option<Decimal>,

    /// 总金额（含税）
    pub total_amount: Option<Decimal>,

    /// 合同状态
    pub status: Option<ContractStatus>,

    /// 开始日期
    pub start_date: Option<Date>,

    /// 结束日期
    pub end_date: Option<Date>,

    /// 签订日期
    pub sign_date: Option<Date>,

    /// 付款条款
    pub payment_terms: Option<String>,

    /// 交付条款
    pub delivery_terms: Option<String>,

    /// 负责人ID
    pub assigned_to: Option<i64>,

    /// 描述/备注
    pub description: Option<String>,

    /// 合同文件URL
    pub file_url: Option<String>,

    /// 合同文件（备用）
    pub contract_file: Option<String>,

    /// 合同扫描件图片（JSON数组）
    pub contract_images: Option<String>,

    /// 审批状态（0-草稿, 1-待审批, 2-审批中, 3-已通过, 4-已驳回）
    pub approval_status: Option<i32>,

    /// 当前审批阶段（1-第一级审批, 2-第二级审批）
    pub current_approval_stage: Option<i32>,

    /// 下一审批人ID
    pub next_approver_id: Option<i64>,

    /// 审批金额阈值（用于分级审批）
    pub approval_amount_limit: Option<Decimal>,

    /// 审批实例ID（关联 mxx_system_approval_instance）
    pub instance_id: Option<i64>,

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

    /// 软删除标识（0-未删除，1-已删除）
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
