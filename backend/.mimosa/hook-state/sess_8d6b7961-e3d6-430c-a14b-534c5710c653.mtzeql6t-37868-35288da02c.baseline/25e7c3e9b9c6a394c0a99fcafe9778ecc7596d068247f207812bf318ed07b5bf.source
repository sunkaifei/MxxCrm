//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 金税开票业务逻辑层
//!

use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::sale::entity::invoice::{self as invoice_entity, Entity as SaleInvoice};
use crate::modules::sale::entity::tax_invoice::{self, Entity, Column};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DbConn, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct TaxInvoiceListQuery {
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub invoice_id: Option<i64>,
    pub order_id: Option<i64>,
    pub status: Option<i32>,
    pub platform: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct TaxInvoiceListVO {
    #[serde(flatten)]
    pub model: tax_invoice::Model,
    pub customer_name: Option<String>,
}

/// 创建金税开票申请，生成编号 TI+yyyyMMdd+4位
pub async fn create_tax_invoice(
    db: &DbConn,
    invoice_id: i64,
    platform: Option<i32>,
    category: Option<i32>,
    user_id: i64,
) -> Result<i64> {
    // 查询关联发票，回填关联字段
    let invoice = SaleInvoice::find_by_id(invoice_id)
        .filter(invoice_entity::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("关联发票不存在"))?;

    let date_prefix = format!("TI{}", chrono::Local::now().format("%Y%m%d"));
    let today_records = Entity::find()
        .filter(Column::TaxInvoiceNo.starts_with(&date_prefix))
        .filter(Column::Deleted.eq(0))
        .all(db)
        .await?;
    let max_seq = today_records
        .iter()
        .filter_map(|t| t.tax_invoice_no.as_ref())
        .filter_map(|no| {
            no.get(date_prefix.len()..).and_then(|s| s.parse::<u32>().ok())
        })
        .max()
        .unwrap_or(0);
    let tax_invoice_no = format!("{}{:04}", date_prefix, max_seq + 1);

    let amount = invoice.amount;
    let tax_amount = invoice.tax_amount;
    let total_amount = match (&amount, &tax_amount) {
        (Some(a), Some(t)) => Some(*a + *t),
        (Some(a), None) => Some(*a),
        _ => None,
    };

    let now = chrono::Local::now().naive_local();

    let txn = db.begin().await?;
    let model = tax_invoice::ActiveModel {
        invoice_id: Set(Some(invoice_id)),
        order_id: Set(invoice.order_id),
        customer_id: Set(invoice.customer_id),
        tax_invoice_no: Set(Some(tax_invoice_no)),
        platform: Set(platform),
        invoice_category: Set(category),
        status: Set(Some(1)),
        amount: Set(amount),
        tax_amount: Set(tax_amount),
        total_amount: Set(total_amount),
        buyer_name: Set(invoice.buyer_name),
        buyer_tax_no: Set(invoice.tax_no.clone()),
        buyer_address: Set(invoice.buyer_address.clone()),
        buyer_bank_account: Set(invoice.buyer_bank.clone()),
        create_time: Set(Some(now)),
        ..Default::default()
    };
    let result = model.insert(&txn).await?;
    txn.commit().await?;

    let _ = user_id;
    Ok(result.id)
}

/// 执行开票（占位：设置 status=2, issue_time=now, pdf_url=占位）
pub async fn issue_tax_invoice(db: &DbConn, id: i64) -> Result<i64> {
    let existing = Entity::find_by_id(id)
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("金税发票不存在"))?;

    let status = existing.status.unwrap_or(0);
    if status != 1 {
        return Err(Error::from(format!("当前状态({})不允许执行开票，仅待开票(1)状态可执行", status)));
    }

    let now = chrono::Local::now().naive_local();
    let txn = db.begin().await?;
    let mut active: tax_invoice::ActiveModel = existing.into();
    active.status = Set(Some(2));
    active.issue_time = Set(Some(now));
    active.pdf_url = Set(Some(format!("/api/system/sale/tax-invoice/pdf/{}", id)));
    active.update_time = Set(Some(now));
    active.update(&txn).await?;
    txn.commit().await?;

    Ok(id)
}

/// 作废（status=3）
pub async fn void_tax_invoice(db: &DbConn, id: i64, reason: String) -> Result<i64> {
    let existing = Entity::find_by_id(id)
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("金税发票不存在"))?;

    let status = existing.status.unwrap_or(0);
    if status == 3 {
        return Err(Error::from("该金税发票已作废，不可重复操作"));
    }

    let now = chrono::Local::now().naive_local();
    let txn = db.begin().await?;
    let mut active: tax_invoice::ActiveModel = existing.into();
    active.status = Set(Some(3));
    active.void_time = Set(Some(now));
    active.void_reason = Set(Some(reason));
    active.update_time = Set(Some(now));
    active.update(&txn).await?;
    txn.commit().await?;

    Ok(id)
}

/// 详情
pub async fn get_info(db: &DbConn, id: i64) -> Result<tax_invoice::Model> {
    Entity::find_by_id(id)
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("金税发票不存在"))
}

/// 分页列表（支持 invoice_id/order_id/status/platform 过滤）
pub async fn get_list(db: &DbConn, query: &TaxInvoiceListQuery) -> Result<ResultPage<Vec<TaxInvoiceListVO>>> {
    let page = query.page_num.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).max(1);

    let mut cond = Condition::all().add(Column::Deleted.eq(0));
    if let Some(invoice_id) = query.invoice_id {
        if invoice_id > 0 {
            cond = cond.add(Column::InvoiceId.eq(invoice_id));
        }
    }
    if let Some(order_id) = query.order_id {
        if order_id > 0 {
            cond = cond.add(Column::OrderId.eq(order_id));
        }
    }
    if let Some(status) = query.status {
        if status > 0 {
            cond = cond.add(Column::Status.eq(status));
        }
    }
    if let Some(platform) = query.platform {
        if platform > 0 {
            cond = cond.add(Column::Platform.eq(platform));
        }
    }

    let total = Entity::find()
        .filter(cond.clone())
        .count(db)
        .await? as i64;

    let list = Entity::find()
        .filter(cond)
        .order_by_desc(Column::Id)
        .offset(((page - 1) * page_size) as u64)
        .limit(page_size as u64)
        .all(db)
        .await?;

    let data: Vec<TaxInvoiceListVO> = list
        .into_iter()
        .map(|m| TaxInvoiceListVO {
            customer_name: None,
            model: m,
        })
        .collect();

    Ok(ResultPage::new(data, total, page, page_size))
}
