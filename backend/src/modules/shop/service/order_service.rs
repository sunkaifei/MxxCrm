//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::{Error, Result};
use crate::modules::shop::model::order::*;
use sea_orm::DbConn;
use rust_decimal::prelude::FromPrimitive;

pub async fn create_order(db: &DbConn, user_id: i64, req: OrderRequest) -> Result<i64> {
    let order_no = generate_order_no();

    let dto = OrderDTO {
        user_id,
        shop_id: req.shop_id.unwrap_or(0),
        total_amount: rust_decimal::Decimal::from_f64(req.total_amount.unwrap_or(0.0)).unwrap_or_default(),
        freight_amount: rust_decimal::Decimal::from_f64(req.freight_amount.unwrap_or(0.0)).unwrap_or_default(),
        goods_count: req.goods_count.unwrap_or(0),
        receiver_name: req.receiver_name.unwrap_or_default(),
        receiver_phone: req.receiver_phone.unwrap_or_default(),
        receiver_address: req.receiver_address.unwrap_or_default(),
        buyer_remark: req.buyer_remark,
    };

    let order_id = OrderModel::insert(db, &dto, &order_no).await
        .map_err(|e| Error::from(format!("创建订单失败: {:?}", e)))?;

    if let Some(items) = req.items {
        let mut item_dtos = Vec::new();
        for item in items {
            item_dtos.push(OrderItemDTO {
                order_id,
                spu_id: item.spu_id.unwrap_or(0),
                sku_id: item.sku_id.unwrap_or(0),
                goods_title: item.goods_title.unwrap_or_default(),
                goods_image: item.goods_image.unwrap_or_default(),
                spec_desc: item.spec_desc,
                price: rust_decimal::Decimal::from_f64(item.price.unwrap_or(0.0)).unwrap_or_default(),
                quantity: item.quantity.unwrap_or(1),
                base_price: rust_decimal::Decimal::from_f64(item.base_price.unwrap_or(0.0)).unwrap_or_default(),
                commission_amount: rust_decimal::Decimal::from_f64(0.0).unwrap_or_default(),
                settlement_amount: rust_decimal::Decimal::from_f64(0.0).unwrap_or_default(),
            });
        }
        OrderItemModel::batch_insert(db, &item_dtos).await
            .map_err(|e| Error::from(format!("创建订单项失败: {:?}", e)))?;
    }

    Ok(order_id)
}

pub async fn get_order_by_id(db: &DbConn, id: i64) -> Result<Option<OrderVO>> {
    let model = OrderModel::find_by_id(db, id).await
        .map_err(|e| Error::from(format!("查询订单失败: {:?}", e)))?;
    if let Some(order) = model {
        let mut vo: OrderVO = order.into();
        let items = OrderItemModel::find_by_order_id(db, id).await
            .map_err(|e| Error::from(format!("查询订单项失败: {:?}", e)))?;
        vo.items = Some(items.into_iter().map(|m| m.into()).collect());
        Ok(Some(vo))
    } else {
        Ok(None)
    }
}

pub async fn get_user_orders(db: &DbConn, user_id: i64) -> Result<Vec<OrderVO>> {
    let list = OrderModel::find_by_user_id(db, user_id).await
        .map_err(|e| Error::from(format!("查询订单列表失败: {:?}", e)))?;
    Ok(list.into_iter().map(|m| m.into()).collect())
}

pub async fn get_order_page(db: &DbConn, query: OrderPageQuery) -> Result<(Vec<OrderVO>, i64)> {
    let page_num = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(10);

    let (list, total) = OrderModel::find_page(db, page_num, page_size, query.user_id, query.shop_id, query.status, query.order_no).await
        .map_err(|e| Error::from(format!("查询订单分页失败: {:?}", e)))?;

    let vo_list: Vec<OrderVO> = list.into_iter().map(|m| m.into()).collect();
    Ok((vo_list, total))
}

pub async fn update_order_status(db: &DbConn, id: i64, status: i16) -> Result<i64> {
    OrderModel::update_status(db, id, status).await
        .map_err(|e| Error::from(format!("更新订单状态失败: {:?}", e)))
}

pub async fn pay_order(db: &DbConn, id: i64, pay_method: &str) -> Result<i64> {
    OrderModel::update_pay_info(db, id, pay_method, 1).await
        .map_err(|e| Error::from(format!("支付失败: {:?}", e)))
}

fn generate_order_no() -> String {
    let now = chrono::Local::now();
    let timestamp = now.format("%Y%m%d%H%M%S").to_string();
    let rand = rand::random::<u32>() % 10000;
    format!("ORD{}{:04}", timestamp, rand)
}
