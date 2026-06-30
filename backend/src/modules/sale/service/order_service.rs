use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::sale::model::order::{OrderDetailVO, OrderItemModel, OrderItemSaveDTO, OrderListQuery, OrderListVO, OrderModel, OrderSaveDTO, OrderSaveRequest, OrderStatusUpdateRequest, OrderUpdateRequest};
use rust_decimal::Decimal;
use sea_orm::{DbConn, TransactionTrait};

fn calculate_product_amount(items: &Vec<OrderItemSaveDTO>) -> Decimal {
    let hundred = Decimal::from(100);
    items.iter().map(|item| {
        let qty = item.quantity.unwrap_or(1);
        let price = item.unit_price.unwrap_or(Decimal::from(0));
        let disc_rate = item.discount_rate.unwrap_or(Decimal::from(100));
        let tax_rate = item.tax_rate.unwrap_or(Decimal::from(0));

        let gross = price * Decimal::from(qty);
        let disc_amt = gross * (hundred - disc_rate) / hundred;
        let tax_amt = (gross - disc_amt) * tax_rate / hundred;
        gross - disc_amt + tax_amt
    }).fold(Decimal::from(0), |acc, x| acc + x)
}

pub async fn insert(db: &DbConn, form_data: &OrderSaveRequest, created_by: i64) -> Result<i64> {
    let items = form_data.items.clone().unwrap_or_default();
    if items.is_empty() {
        return Err(Error::from("订单明细不能为空"));
    }

    let txn = db.begin().await?;

    let date_prefix = format!("SO{}", chrono::Local::now().format("%Y%m%d"));
    let max_seq = OrderModel::get_max_order_no_today(&txn, &date_prefix).await?;
    let seq = max_seq.unwrap_or(0) + 1;
    let order_no = format!("{}{:04}", date_prefix, seq);

    let product_amount = calculate_product_amount(&items);
    let discount_amount = form_data.discount_amount.unwrap_or(Decimal::from(0));
    let shipping_fee = form_data.shipping_fee.unwrap_or(Decimal::from(0));
    let tax_amount = form_data.tax_amount.unwrap_or(Decimal::from(0));
    let other_fee = form_data.other_fee.unwrap_or(Decimal::from(0));
    let total_amount = product_amount - discount_amount + shipping_fee + tax_amount + other_fee;

    let mut dto: OrderSaveDTO = form_data.clone().into();
    dto.order_no = Some(order_no);
    dto.order_status = Some(1);
    dto.product_amount = Some(product_amount);
    dto.total_amount = Some(total_amount);
    dto.paid_amount = Some(Decimal::from(0));
    dto.unpaid_amount = Some(total_amount);
    dto.pay_status = Some(1);
    dto.create_by = Some(created_by);

    let order_id = OrderModel::insert(&txn, &dto).await?;
    OrderItemModel::insert_batch(&txn, order_id, &items).await?;

    txn.commit().await?;

    Ok(order_id)
}

pub async fn update(db: &DbConn, form_data: &OrderUpdateRequest, updated_by: i64) -> Result<i64> {
    let id = form_data.id.unwrap_or_default();
    if id == 0 {
        return Err(Error::from("订单ID不能为空"));
    }
    let items = form_data.items.clone().unwrap_or_default();
    if items.is_empty() {
        return Err(Error::from("订单明细不能为空"));
    }

    let existing = OrderModel::find_by_id(db, id).await?;
    if existing.is_none() {
        return Err(Error::from("订单不存在"));
    }

    let txn = db.begin().await?;

    let product_amount = calculate_product_amount(&items);
    let discount_amount = form_data.discount_amount.unwrap_or(Decimal::from(0));
    let shipping_fee = form_data.shipping_fee.unwrap_or(Decimal::from(0));
    let tax_amount = form_data.tax_amount.unwrap_or(Decimal::from(0));
    let other_fee = form_data.other_fee.unwrap_or(Decimal::from(0));
    let total_amount = product_amount - discount_amount + shipping_fee + tax_amount + other_fee;

    let mut dto: OrderSaveDTO = form_data.clone().into();
    dto.product_amount = Some(product_amount);
    dto.total_amount = Some(total_amount);
    dto.update_by = Some(updated_by);

    OrderModel::update_by_id(&txn, id, &dto).await?;
    OrderItemModel::delete_by_order_id(&txn, id).await?;
    OrderItemModel::insert_batch(&txn, id, &items).await?;

    txn.commit().await?;

    Ok(id)
}

pub async fn update_status(db: &DbConn, form_data: &OrderStatusUpdateRequest) -> Result<i64> {
    let id = form_data.id.unwrap_or_default();
    if id == 0 {
        return Err(Error::from("订单ID不能为空"));
    }
    let order_status = form_data.order_status.unwrap_or(1);

    let existing = OrderModel::find_by_id(db, id).await?;
    if existing.is_none() {
        return Err(Error::from("订单不存在"));
    }

    let result = OrderModel::update_status(db, id, order_status, form_data.tracking_no.clone()).await?;
    Ok(result)
}

pub async fn batch_delete(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let result = OrderModel::batch_delete_by_ids(db, ids_vec).await?;
    Ok(result)
}

pub async fn get_detail(db: &DbConn, id: i64) -> Result<OrderDetailVO> {
    let order = OrderModel::find_by_id(db, id).await?;
    match order {
        Some(o) => {
            let items = OrderItemModel::find_by_order_id(db, id).await?;
            Ok((&o, items).into())
        }
        None => Err(Error::from("订单不存在")),
    }
}

pub async fn get_list(db: &DbConn, query: &OrderListQuery) -> Result<ResultPage<Vec<OrderListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let (list, total) = OrderModel::select_in_page(
        db,
        page,
        page_size,
        query.keywords.clone(),
        query.order_status,
        query.payment_status,
        query.customer_id,
        query.owner_user_id,
        query.start_date.clone(),
        query.end_date.clone(),
    ).await?;

    let data: Vec<OrderListVO> = list.iter().map(|item| item.into()).collect();
    Ok(ResultPage { items: data, total, current_page: page, page_size, total_pages: 0 })
}
