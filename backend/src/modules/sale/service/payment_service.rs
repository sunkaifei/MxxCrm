use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::sale::model::payment::{PaymentDetailVO, PaymentListQuery, PaymentListVO, PaymentModel, PaymentSaveDTO, PaymentSaveRequest, PaymentUpdateRequest};
use rust_decimal::Decimal;
use sea_orm::{DbConn, TransactionTrait};
use std::collections::HashMap;

pub async fn insert(db: &DbConn, form_data: &PaymentSaveRequest, created_by: i64) -> Result<i64> {
    let txn = db.begin().await?;

    let date_prefix = format!("HK{}", chrono::Local::now().format("%Y%m%d"));
    let max_seq = PaymentModel::get_max_payment_no_today(&txn, &date_prefix).await?;
    let seq = max_seq.unwrap_or(0) + 1;
    let payment_no = format!("{}{:04}", date_prefix, seq);

    let mut dto: PaymentSaveDTO = form_data.clone().into();
    dto.payment_no = Some(payment_no);
    dto.status = Some(1);
    dto.amount = Some(dto.amount.unwrap_or(Decimal::from(0)));
    dto.currency = Some(dto.currency.unwrap_or(1));
    dto.payment_method = Some(dto.payment_method.unwrap_or(1));
    dto.create_by = Some(created_by.to_string());

    let payment_id = PaymentModel::insert(&txn, &dto).await?;

    txn.commit().await?;

    Ok(payment_id)
}

pub async fn update(db: &DbConn, form_data: &PaymentUpdateRequest, updated_by: i64) -> Result<i64> {
    let id = form_data.id.unwrap_or_default();
    if id == 0 {
        return Err(Error::from("回款ID不能为空"));
    }

    let existing = PaymentModel::find_by_id(db, id).await?;
    if existing.is_none() {
        return Err(Error::from("回款记录不存在"));
    }

    let mut dto: PaymentSaveDTO = form_data.clone().into();
    dto.update_by = Some(updated_by.to_string());

    PaymentModel::update_by_id(db, id, &dto).await?;

    Ok(id)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let result = PaymentModel::batch_delete_by_ids(db, ids_vec).await?;
    Ok(result)
}

pub async fn find_by_id(db: &DbConn, id: i64) -> Result<PaymentDetailVO> {
    let payment = PaymentModel::find_by_id(db, id).await?;
    match payment {
        Some(p) => Ok((&p).into()),
        None => Err(Error::from("回款记录不存在")),
    }
}

pub async fn list(db: &DbConn, query: &PaymentListQuery) -> Result<ResultPage<Vec<PaymentListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let (list, total) = PaymentModel::select_in_page(
        db,
        page,
        page_size,
        query.payment_no.clone(),
        query.order_no.clone(),
        query.contract_id,
        query.customer_id,
        query.status,
        query.payment_method,
    ).await?;

    let order_ids: Vec<i64> = list.iter().filter_map(|p| p.order_id).collect();
    let orders = PaymentModel::find_orders_by_ids(db, &order_ids).await?;
    let order_map: HashMap<i64, String> = orders.into_iter()
        .filter_map(|o| o.order_no.map(|no| (o.id, no)))
        .collect();

    let mut data: Vec<PaymentListVO> = list.iter().map(|item| {
        let mut vo: PaymentListVO = item.into();
        if let Some(oid) = item.order_id {
            vo.order_no = order_map.get(&oid).cloned();
        }
        vo
    }).collect();

    Ok(ResultPage { items: data, total, current_page: page, page_size, total_pages: 0 })
}
