//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究。
//!

use sea_orm::{DbConn, DbErr, TransactionTrait};
use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::website::model::website_delivery::{
    DeliveryCreateRequest, DeliveryListQuery, DeliveryVO, WebsiteDeliveryModel,
};
use crate::modules::website::model::website_order::{WebsiteOrderModel, STATUS_PENDING_SHIP};

/// 后台发货：创建发货单 + 更新订单状态（事务）
pub async fn ship(
    db: &DbConn,
    req: DeliveryCreateRequest,
    shipper_id: i64,
    shipper_name: String,
) -> Result<i64> {
    // 前置校验订单
    let order = WebsiteOrderModel::find_by_id(db, req.order_id)
        .await?
        .ok_or_else(|| Error::from("订单不存在"))?;
    if order.status.unwrap_or(0) != STATUS_PENDING_SHIP {
        return Err(Error::from("当前订单状态不允许发货"));
    }

    let consignee_name = order.consignee_name.clone();
    let consignee_phone = order.consignee_phone.clone();
    let consignee_address = order.consignee_address.clone();
    let order_id = req.order_id;

    let req_clone = req.clone();
    let shipper_name_clone = shipper_name.clone();
    let consignee_name_clone = consignee_name.clone();
    let consignee_phone_clone = consignee_phone.clone();
    let consignee_address_clone = consignee_address.clone();

    db.transaction::<_, i64, DbErr>(|txn| {
        Box::pin(async move {
            // 1. 创建发货单
            let delivery_id = WebsiteDeliveryModel::insert(
                txn,
                &req_clone,
                shipper_id,
                shipper_name_clone,
                consignee_name_clone,
                consignee_phone_clone,
                consignee_address_clone,
            )
            .await?;

            // 2. 更新订单为待收货
            WebsiteOrderModel::update_shipped(txn, order_id).await?;

            Ok(delivery_id)
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    Ok(order_id)
}

/// 根据订单ID查询发货单列表
pub async fn find_by_order_id(db: &DbConn, order_id: i64) -> Result<Vec<DeliveryVO>> {
    let list = WebsiteDeliveryModel::find_by_order_id(db, order_id).await?;
    Ok(list.into_iter().map(|m| m.into()).collect())
}

/// 后台发货单分页列表
pub async fn admin_list(db: &DbConn, query: DeliveryListQuery) -> Result<ResultPage<Vec<DeliveryVO>>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).max(1).min(100);
    let (list, total) = WebsiteDeliveryModel::select_in_page(db, page, page_size, &query).await?;
    let list_vo: Vec<DeliveryVO> = list.into_iter().map(|m| m.into()).collect();
    Ok(ResultPage::new(list_vo, total, page, page_size))
}

/// 后台批量删除发货单
pub async fn admin_batch_delete(db: &DbConn, ids: Vec<i64>) -> Result<i64> {
    db.transaction::<_, i64, DbErr>(|txn| {
        let ids_clone = ids.clone();
        Box::pin(async move {
            WebsiteDeliveryModel::batch_soft_delete(txn, ids_clone).await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))
}
