use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, IntoActiveModel, PaginatorTrait,
    QueryFilter, QueryOrder, Set, TransactionTrait,
};

use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::sale::entity::online_payment::{self, Entity as OnlinePaymentEntity, Column};
use crate::modules::sale::model::order::OrderModel;

/// 支付状态：1=待支付, 2=已支付, 3=已关闭, 4=已退款
const PAY_STATUS_PENDING: i32 = 1;
const PAY_STATUS_PAID: i32 = 2;

/// 创建在线支付记录，返回 prepay_id / pay_url
pub async fn create_payment(
    db: &DbConn,
    order_id: i64,
    channel: i32,
    customer_id: Option<i64>,
) -> Result<serde_json::Value> {
    let order = OrderModel::find_by_id(db, order_id)
        .await?
        .ok_or_else(|| Error::from("订单不存在"))?;

    let amount = order.total_amount.unwrap_or(rust_decimal::Decimal::ZERO);
    if amount <= rust_decimal::Decimal::ZERO {
        return Err(Error::from("订单金额必须大于0"));
    }

    let date_prefix = format!("OP{}", chrono::Local::now().format("%Y%m%d"));
    let max_seq = get_max_payment_no_today(db, &date_prefix).await?;
    let seq = max_seq.unwrap_or(0) + 1;
    let payment_no = format!("{}{:04}", date_prefix, seq);

    let now = chrono::Local::now().naive_local();
    let expire_time = now + chrono::Duration::hours(2);

    let active = online_payment::ActiveModel {
        payment_no: Set(Some(payment_no.clone())),
        order_id: Set(Some(order_id)),
        customer_id: Set(customer_id.or(order.customer_id)),
        amount: Set(Some(amount)),
        currency: Set(order.currency),
        payment_channel: Set(Some(channel)),
        channel_trade_no: Set(None),
        prepay_id: Set(Some(format!("prepay_{}", &payment_no))),
        pay_url: Set(Some(format!("/pay/{}", &payment_no))),
        qr_code: Set(None),
        status: Set(Some(PAY_STATUS_PENDING)),
        paid_time: Set(None),
        expire_time: Set(Some(expire_time)),
        callback_data: Set(None),
        remark: Set(None),
        create_time: Set(Some(now)),
        update_time: Set(Some(now)),
        deleted: Set(Some(0)),
        ..Default::default()
    };

    let txn = db.begin().await?;
    let model = active.insert(&txn).await?;
    txn.commit().await?;

    Ok(serde_json::json!({
        "id": model.id,
        "paymentNo": model.payment_no,
        "prepayId": model.prepay_id,
        "payUrl": model.pay_url,
        "qrCode": model.qr_code,
        "amount": amount,
        "status": PAY_STATUS_PENDING,
    }))
}

/// 支付回调处理：更新状态为已支付，自动创建 payment 记录并核销到订单
pub async fn handle_callback(
    db: &DbConn,
    payment_no: &str,
    channel_trade_no: Option<String>,
    callback_data: Option<String>,
) -> Result<bool> {
    let payment = OnlinePaymentEntity::find()
        .filter(Column::PaymentNo.eq(payment_no))
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("支付记录不存在"))?;

    if payment.status.unwrap_or(0) == PAY_STATUS_PAID {
        return Ok(true); // 幂等：已支付直接返回成功
    }

    let order_id = payment.order_id.ok_or_else(|| Error::from("支付记录缺少订单ID"))?;
    let amount = payment.amount.unwrap_or(rust_decimal::Decimal::ZERO);
    let now = chrono::Local::now().naive_local();

    let txn = db.begin().await?;

    // 更新在线支付状态
    let mut active: online_payment::ActiveModel = payment.into_active_model();
    active.status = Set(Some(PAY_STATUS_PAID));
    active.channel_trade_no = Set(channel_trade_no);
    active.callback_data = Set(callback_data);
    active.paid_time = Set(Some(now));
    active.update_time = Set(Some(now));
    active.update(&txn).await?;

    // 自动创建 payment 记录并核销到订单（复用 payment_service）
    use crate::modules::sale::model::payment::{PaymentSaveDTO, PaymentModel};
    let payment_dto = PaymentSaveDTO {
        payment_no: None,
        contract_id: None,
        order_id: Some(order_id),
        customer_id: None,
        customer_name: None,
        amount: Some(amount),
        applied_amount: Some(amount),
        unapplied_amount: Some(rust_decimal::Decimal::ZERO),
        currency: None,
        payment_method: Some(5), // 在线支付
        payment_date: Some(now.date()),
        payer: None,
        payer_account: None,
        bank_flow_no: None,
        attachment: None,
        status: Some(3), // 已确认
        remark: Some(format!("在线支付回调自动创建: {}", payment_no)),
        owner_user_id: None,
        dept_id: None,
        create_by: None,
        update_by: None,
    };
    let _ = PaymentModel::insert(&txn, &payment_dto).await?;

    txn.commit().await?;

    Ok(true)
}

/// 查询支付详情
pub async fn get_payment_info(db: &DbConn, id: i64) -> Result<online_payment::Model> {
    OnlinePaymentEntity::find_by_id(id)
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("支付记录不存在"))
}

/// 按订单查询支付记录
pub async fn get_payment_by_order(
    db: &DbConn,
    order_id: i64,
) -> Result<ResultPage<Vec<online_payment::Model>>> {
    let paginator = OnlinePaymentEntity::find()
        .filter(Column::OrderId.eq(order_id))
        .filter(Column::Deleted.eq(0))
        .order_by_desc(Column::Id)
        .paginate(db, 50);

    let total = paginator.num_items().await.map_err(|e| Error::from(e.to_string()))? as i64;
    let items = paginator
        .fetch_page(0)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(ResultPage::new(items, total, 1, 50))
}

/// 主动查询支付状态（从数据库读取，实际对接第三方时调用 API）
pub async fn query_payment_status(db: &DbConn, payment_no: &str) -> Result<serde_json::Value> {
    let payment = OnlinePaymentEntity::find()
        .filter(Column::PaymentNo.eq(payment_no))
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("支付记录不存在"))?;

    Ok(serde_json::json!({
        "paymentNo": payment.payment_no,
        "status": payment.status,
        "paidTime": payment.paid_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        "channelTradeNo": payment.channel_trade_no,
    }))
}

/// 查询今日最大支付流水号序号
async fn get_max_payment_no_today(db: &DbConn, prefix: &str) -> Result<Option<i64>> {
    let prefix_pattern = format!("{}%", prefix);
    let payments = OnlinePaymentEntity::find()
        .filter(Column::PaymentNo.like(&prefix_pattern))
        .filter(Column::Deleted.eq(0))
        .all(db)
        .await?;

    let max_seq = payments
        .iter()
        .filter_map(|p| {
            p.payment_no
                .as_ref()
                .and_then(|no| no.get(prefix.len()..))
                .and_then(|s| s.parse::<i64>().ok())
        })
        .max();

    Ok(max_seq)
}
