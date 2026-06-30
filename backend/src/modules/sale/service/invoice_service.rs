use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::sale::model::invoice::{InvoiceDetailVO, InvoiceListQuery, InvoiceListVO, InvoiceModel, InvoiceSaveDTO, InvoiceSaveRequest, InvoiceUpdateRequest};
use rust_decimal::Decimal;
use sea_orm::{DbConn, TransactionTrait};

pub async fn insert(db: &DbConn, form_data: &InvoiceSaveRequest, created_by: i64) -> Result<i64> {
    let txn = db.begin().await?;

    let date_prefix = format!("INV{}", chrono::Local::now().format("%Y%m%d"));
    let max_seq = InvoiceModel::get_max_invoice_no_today(&txn, &date_prefix).await?;
    let seq = max_seq.unwrap_or(0) + 1;
    let invoice_no = format!("{}{:04}", date_prefix, seq);

    let amount = form_data.amount.unwrap_or(Decimal::from(0));
    let tax_rate = form_data.tax_rate.unwrap_or(Decimal::from(0));
    let hundred = Decimal::from(100);
    let tax_amount = form_data.tax_amount.unwrap_or(amount * tax_rate / hundred);

    let mut dto: InvoiceSaveDTO = form_data.clone().into();
    dto.invoice_no = Some(invoice_no);
    dto.status = Some(1);
    dto.tax_amount = Some(tax_amount);
    dto.create_by = Some(created_by.to_string());

    let invoice_id = InvoiceModel::insert(&txn, &dto).await?;

    txn.commit().await?;

    Ok(invoice_id)
}

pub async fn update(db: &DbConn, form_data: &InvoiceUpdateRequest, updated_by: i64) -> Result<i64> {
    let id = form_data.id.unwrap_or_default();
    if id == 0 {
        return Err(Error::from("发票ID不能为空"));
    }

    let existing = InvoiceModel::find_by_id(db, id).await?;
    if existing.is_none() {
        return Err(Error::from("发票不存在"));
    }

    let amount = form_data.amount.unwrap_or(Decimal::from(0));
    let tax_rate = form_data.tax_rate.unwrap_or(Decimal::from(0));
    let hundred = Decimal::from(100);
    let tax_amount = form_data.tax_amount.unwrap_or(amount * tax_rate / hundred);

    let mut dto: InvoiceSaveDTO = form_data.clone().into();
    dto.tax_amount = Some(tax_amount);
    dto.update_by = Some(updated_by.to_string());

    InvoiceModel::update_by_id(db, id, &dto).await?;

    Ok(id)
}

pub async fn batch_delete(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let result = InvoiceModel::batch_delete_by_ids(db, ids_vec).await?;
    Ok(result)
}

pub async fn get_detail(db: &DbConn, id: i64) -> Result<InvoiceDetailVO> {
    let invoice = InvoiceModel::find_by_id(db, id).await?;
    match invoice {
        Some(i) => Ok((&i).into()),
        None => Err(Error::from("发票不存在")),
    }
}

pub async fn get_list(db: &DbConn, query: &InvoiceListQuery) -> Result<ResultPage<Vec<InvoiceListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let (list, total) = InvoiceModel::select_in_page(
        db,
        page,
        page_size,
        query.keywords.clone(),
        query.invoice_type,
        query.status,
        query.customer_id,
    ).await?;

    let data: Vec<InvoiceListVO> = list.iter().map(|item| item.into()).collect();
    Ok(ResultPage { items: data, total, current_page: page, page_size, total_pages: 0 })
}
