//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
use crate::modules::company::entity::company_account;
use crate::modules::company::entity::company_info::{ActiveModel, Column, Entity, Model};
use sea_orm::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyInfoVO {
    pub id: i64,
    pub company_name: Option<String>,
    pub credit_code: Option<String>,
    pub legal_person: Option<String>,
    pub legal_phone: Option<String>,
    pub register_address: Option<String>,
    pub business_scope: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub logo_url: Option<String>,
    pub tax_number: Option<String>,
    pub invoice_title: Option<String>,
    pub remark: Option<String>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyAccountVO {
    pub id: i64,
    pub bank_name: Option<String>,
    pub account_name: Option<String>,
    pub account_number: Option<String>,
    pub account_type: Option<i32>,
    pub account_type_name: Option<String>,
    pub is_default: Option<i32>,
    pub sort_order: Option<i32>,
    pub remark: Option<String>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyInfoSaveRequest {
    pub company_name: Option<String>,
    pub credit_code: Option<String>,
    pub legal_person: Option<String>,
    pub legal_phone: Option<String>,
    pub register_address: Option<String>,
    pub business_scope: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub logo_url: Option<String>,
    pub tax_number: Option<String>,
    pub invoice_title: Option<String>,
    pub remark: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyAccountSaveRequest {
    pub id: Option<i64>,
    pub bank_name: Option<String>,
    pub account_name: Option<String>,
    pub account_number: Option<String>,
    pub account_type: Option<i32>,
    pub is_default: Option<i32>,
    pub sort_order: Option<i32>,
    pub remark: Option<String>,
}

/// 账户类型名称：1=基本户, 2=一般户, 3=其他
pub fn account_type_name(t: Option<i32>) -> Option<String> {
    match t.unwrap_or(0) {
        1 => Some("基本户".to_string()),
        2 => Some("一般户".to_string()),
        3 => Some("其他".to_string()),
        _ => None,
    }
}

/// 手机号脱敏：保留前3后4，中间用****替换
pub fn mask_phone(phone: &str) -> String {
    let len = phone.len();
    if len <= 7 {
        return phone.to_string();
    }
    let prefix = &phone[..3];
    let suffix = &phone[len - 4..];
    format!("{}****{}", prefix, suffix)
}

impl From<Model> for CompanyInfoVO {
    fn from(model: Model) -> Self {
        CompanyInfoVO {
            id: model.id,
            company_name: model.company_name,
            credit_code: model.credit_code,
            legal_person: model.legal_person,
            legal_phone: model.legal_phone,
            register_address: model.register_address,
            business_scope: model.business_scope,
            contact_phone: model.contact_phone,
            contact_email: model.contact_email,
            logo_url: model.logo_url,
            tax_number: model.tax_number,
            invoice_title: model.invoice_title,
            remark: model.remark,
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: model.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

impl From<company_account::Model> for CompanyAccountVO {
    fn from(model: company_account::Model) -> Self {
        CompanyAccountVO {
            id: model.id,
            bank_name: model.bank_name,
            account_name: model.account_name,
            account_number: model.account_number,
            account_type: model.account_type,
            account_type_name: account_type_name(model.account_type),
            is_default: model.is_default,
            sort_order: model.sort_order,
            remark: model.remark,
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: model.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// 查询企业信息记录（id=1）
pub async fn find_info(db: &DbConn) -> Result<Option<Model>, DbErr> {
    Entity::find_by_id(1)
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await
}

/// 更新企业信息
pub async fn update_info(db: &DbConn, req: &CompanyInfoSaveRequest, updated_by: i64) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local().to_owned();
    let am = ActiveModel {
        company_name: Set(req.company_name.clone()),
        credit_code: Set(req.credit_code.clone()),
        legal_person: Set(req.legal_person.clone()),
        legal_phone: Set(req.legal_phone.clone()),
        register_address: Set(req.register_address.clone()),
        business_scope: Set(req.business_scope.clone()),
        contact_phone: Set(req.contact_phone.clone()),
        contact_email: Set(req.contact_email.clone()),
        logo_url: Set(req.logo_url.clone()),
        tax_number: Set(req.tax_number.clone()),
        invoice_title: Set(req.invoice_title.clone()),
        remark: Set(req.remark.clone()),
        update_by: Set(Some(updated_by)),
        update_time: Set(Some(now)),
        ..Default::default()
    };
    let res = Entity::update_many()
        .set(am)
        .filter(Column::Id.eq(1))
        .filter(Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(res.rows_affected as i64)
}

/// 查询所有未删除的银行账户
pub async fn find_accounts(db: &DbConn) -> Result<Vec<company_account::Model>, DbErr> {
    company_account::Entity::find()
        .filter(company_account::Column::Deleted.eq(0))
        .order_by_asc(company_account::Column::SortOrder)
        .all(db)
        .await
}

/// 根据ID查询银行账户
pub async fn find_account_by_id(db: &DbConn, id: i64) -> Result<Option<company_account::Model>, DbErr> {
    company_account::Entity::find_by_id(id)
        .filter(company_account::Column::Deleted.eq(0))
        .one(db)
        .await
}

/// 新增银行账户
pub async fn insert_account(db: &DbConn, req: &CompanyAccountSaveRequest, created_by: i64) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local().to_owned();
    let am = company_account::ActiveModel {
        bank_name: Set(req.bank_name.clone()),
        account_name: Set(req.account_name.clone()),
        account_number: Set(req.account_number.clone()),
        account_type: Set(req.account_type.or(Some(1))),
        is_default: Set(req.is_default.or(Some(0))),
        sort_order: Set(req.sort_order.or(Some(0))),
        remark: Set(req.remark.clone()),
        deleted: Set(Some(0)),
        create_by: Set(Some(created_by)),
        update_by: Set(Some(created_by)),
        create_time: Set(Some(now)),
        update_time: Set(Some(now)),
        ..Default::default()
    };
    let res = company_account::Entity::insert(am).exec(db).await?;
    Ok(res.last_insert_id)
}

/// 更新银行账户
pub async fn update_account(db: &DbConn, id: i64, req: &CompanyAccountSaveRequest, updated_by: i64) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local().to_owned();
    let am = company_account::ActiveModel {
        bank_name: Set(req.bank_name.clone()),
        account_name: Set(req.account_name.clone()),
        account_number: Set(req.account_number.clone()),
        account_type: Set(req.account_type),
        is_default: Set(req.is_default),
        sort_order: Set(req.sort_order),
        remark: Set(req.remark.clone()),
        update_by: Set(Some(updated_by)),
        update_time: Set(Some(now)),
        ..Default::default()
    };
    let res = company_account::Entity::update_many()
        .set(am)
        .filter(company_account::Column::Id.eq(id))
        .filter(company_account::Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(res.rows_affected as i64)
}

/// 软删除银行账户
pub async fn delete_account(db: &DbConn, id: i64) -> Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local().to_owned();
    let am = company_account::ActiveModel {
        deleted: Set(Some(1)),
        update_time: Set(Some(now)),
        ..Default::default()
    };
    let res = company_account::Entity::update_many()
        .set(am)
        .filter(company_account::Column::Id.eq(id))
        .filter(company_account::Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(res.rows_affected as i64)
}
