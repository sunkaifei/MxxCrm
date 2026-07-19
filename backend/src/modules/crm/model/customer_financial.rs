//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::*;
use crate::core::kit::global::Deserialize;
use crate::modules::crm::entity::customer_financial;
use crate::modules::crm::entity::customer_financial::Entity as CustomerFinancial;

/// 客户财务信息保存DTO
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerFinancialSaveDTO {
    /// 客户ID
    pub customer_id: i64,
    /// 纳税人识别号
    pub tax_id: Option<String>,
    /// 发票抬头
    pub invoice_title: Option<String>,
    /// 注册地址
    pub registered_address: Option<String>,
    /// 注册电话
    pub registered_phone: Option<String>,
    /// 财务电话
    pub finance_phone: Option<String>,
    /// 银行账户信息(JSON数组，每个元素包含 bank_name, account_number, is_default)
    pub bank_accounts: Option<serde_json::Value>,
}

/// 客户财务信息模型操作类
pub struct CustomerFinancialModel;

impl CustomerFinancialModel {
    /// 根据客户ID查询财务信息
    pub async fn find_by_customer_id(db: &impl ConnectionTrait, customer_id: i64) -> Result<Option<customer_financial::Model>, DbErr> {
        CustomerFinancial::find()
            .filter(customer_financial::Column::CustomerId.eq(customer_id))
            .filter(customer_financial::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 新增客户财务信息
    pub async fn insert(db: &impl ConnectionTrait, req: &CustomerFinancialSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = customer_financial::ActiveModel {
            customer_id: Set(req.customer_id),
            tax_id: Set(req.tax_id.clone()),
            invoice_title: Set(req.invoice_title.clone()),
            registered_address: Set(req.registered_address.clone()),
            registered_phone: Set(req.registered_phone.clone()),
            finance_phone: Set(req.finance_phone.clone()),
            bank_accounts: Set(req.bank_accounts.clone()),
            create_time: Set(Option::from(now)),
            update_time: Set(Option::from(now)),
            ..Default::default()
        };

        CustomerFinancial::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 根据客户ID更新财务信息
    pub async fn update_by_customer_id(db: &impl ConnectionTrait, customer_id: i64, updated_by: Option<i64>, req: &CustomerFinancialSaveDTO) -> Result<i64, DbErr> {
        let payload = customer_financial::ActiveModel {
            tax_id: Set(req.tax_id.clone()),
            invoice_title: Set(req.invoice_title.clone()),
            registered_address: Set(req.registered_address.clone()),
            registered_phone: Set(req.registered_phone.clone()),
            finance_phone: Set(req.finance_phone.clone()),
            bank_accounts: Set(req.bank_accounts.clone()),
            updated_by: Set(updated_by),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        let update_result: UpdateResult = CustomerFinancial::update_many()
            .set(payload)
            .filter(customer_financial::Column::CustomerId.eq(customer_id))
            .filter(customer_financial::Column::Deleted.eq(0))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }
}
