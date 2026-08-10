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
use crate::core::web::response::ResultPage;
use crate::modules::sale::entity::order_item;
use crate::modules::sale::model::order::{OrderItemModel, OrderModel};
use crate::modules::sale::model::shipment::{
    ShipmentDetailVO, ShipmentItemModel, ShipmentListQuery, ShipmentListVO,
    ShipmentModel, ShipmentSaveDTO, ShipmentSaveRequest, ShipmentUpdateRequest,
};
use crate::modules::system::entity::{admin, admin::Entity as Admin};
use crate::modules::system::model::admin_dept_merge::AdminDeptMergeModel;
use crate::modules::system::model::dept::DeptModel;
use crate::modules::system::service::role_service;
use rust_decimal::Decimal;
use sea_orm::{ConnectionTrait, DbConn, EntityTrait, ColumnTrait, QueryFilter, TransactionTrait};
use std::collections::HashSet;

/// 根据用户ID获取其数据权限范围内的所有用户ID
///
/// 已迁移至 [`data_scope_service::get_accessible_user_ids`]，支持多角色合并。
/// 参数 `data_scope` 已弃用，内部会自动查询用户所有角色并合并权限。
async fn get_accessible_user_ids(
    db: &DbConn,
    current_user_id: i64,
    _data_scope: Option<i32>,
) -> Result<Option<Vec<i64>>> {
    crate::modules::system::service::data_scope_service::get_accessible_user_ids(db, current_user_id).await
}



pub async fn get_list(db: &DbConn, query: &ShipmentListQuery, current_user_id: i64) -> Result<ResultPage<Vec<ShipmentListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let list_type = query.list_type.as_deref().unwrap_or("all");

    let owner_user_ids_opt: Option<Vec<i64>> = match list_type {
        "my" => {
            Some(vec![current_user_id])
        }
        "subordinate" => {
            // 下属发货单：获取数据权限范围内的其他用户（排除自己）
            let accessible = crate::modules::system::service::data_scope_service
                ::get_accessible_user_ids(db, current_user_id).await?;
            match accessible {
                None => {
                    // 全部数据权限：获取所有用户，排除自己
                    let all_admins = Admin::find()
                        .filter(admin::Column::Id.ne(current_user_id))
                        .all(db)
                        .await
                        .map_err(|e| Error::from(format!("查询用户列表失败: {}", e)))?;
                    Some(all_admins.iter().map(|u| u.id).collect())
                }
                Some(ids) => {
                    // 部门/仅本人权限：排除自己
                    Some(ids.into_iter().filter(|id| *id != current_user_id).collect())
                }
            }
        }
        _ => {
            // all：按多角色合并后的数据权限过滤
            crate::modules::system::service::data_scope_service
                ::get_accessible_user_ids(db, current_user_id).await?
        }
    };

    let (list, total) = if list_type == "my" {
        ShipmentModel::select_in_page_by_owner_user_ids(
            db,
            page,
            page_size,
            query.keywords.clone(),
            query.status,
            query.order_id,
            query.customer_id,
            query.contract_id,
            query.start_date.clone(),
            query.end_date.clone(),
            Some(vec![current_user_id]),
        ).await?
    } else {
        ShipmentModel::select_in_page_by_owner_user_ids(
            db,
            page,
            page_size,
            query.keywords.clone(),
            query.status,
            query.order_id,
            query.customer_id,
            query.contract_id,
            query.start_date.clone(),
            query.end_date.clone(),
            owner_user_ids_opt,
        ).await?
    };

    let data: Vec<ShipmentListVO> = list.iter().map(|item| item.into()).collect();
    Ok(ResultPage {
        items: data,
        total,
        current_page: page,
        page_size,
        total_pages: 0,
    })
}

pub async fn get_detail(db: &DbConn, id: i64) -> Result<ShipmentDetailVO> {
    let shipment = ShipmentModel::find_by_id(db, id).await?;
    match shipment {
        Some(s) => {
            let items = ShipmentItemModel::find_by_shipment_id(db, id).await?;
            Ok((&s, items).into())
        }
        None => Err(Error::from("发货单不存在")),
    }
}

pub async fn create(db: &DbConn, form_data: &ShipmentSaveRequest, created_by: i64) -> Result<i64> {
    let order_id = form_data.order_id.ok_or_else(|| Error::from("订单ID不能为空"))?;
    if order_id == 0 {
        return Err(Error::from("订单ID不能为空"));
    }

    let items = form_data.items.clone().unwrap_or_default();
    if items.is_empty() {
        return Err(Error::from("发货明细不能为空"));
    }

    // 校验订单存在
    let _order = OrderModel::find_by_id(db, order_id)
        .await?
        .ok_or_else(|| Error::from("订单不存在"))?;

    // === 虚拟商品与服务订单支持：发货明细必须全部为实物 ===
    use crate::modules::sale::model::order::{needs_shipping, PRODUCT_TYPE_PHYSICAL};
    use crate::modules::sale::model::order_item::OrderItemModel as OrderItemModelById;
    for item in &items {
        if let Some(order_item_id) = item.order_item_id {
            let order_item = OrderItemModelById::find_by_id(db, order_item_id).await?
                .ok_or_else(|| Error::from("订单明细不存在"))?;
            let pt = order_item.product_type.unwrap_or(PRODUCT_TYPE_PHYSICAL);
            if !needs_shipping(pt) {
                return Err(Error::from("非实物商品不能创建发货单（请改用虚拟商品交付）"));
            }
        }
    }

    // 计算总数量
    let total_quantity: i32 = items.iter().map(|i| i.quantity.unwrap_or(0)).sum();

    let txn = db.begin().await?;

    // 生成发货单号 SH + yyyyMMdd + 4位序号
    let date_prefix = format!("SH{}", chrono::Local::now().format("%Y%m%d"));
    let max_seq = ShipmentModel::get_max_shipment_no_today(&txn, &date_prefix).await?;
    let seq = max_seq.unwrap_or(0) + 1;
    let shipment_no = format!("{}{:04}", date_prefix, seq);

    let mut dto: ShipmentSaveDTO = form_data.clone().into();
    dto.shipment_no = Some(shipment_no.clone());
    dto.status = Some(1);
    dto.total_quantity = Some(total_quantity);
    dto.created_by = Some(created_by);

    let shipment_id = ShipmentModel::insert(&txn, &dto).await?;

    // 保存发货明细，并累加订单明细的已发数量
    ShipmentItemModel::insert_batch(&txn, shipment_id, &items).await?;

    for item in &items {
        if let Some(order_item_id) = item.order_item_id {
            let qty = item.quantity.unwrap_or(0);
            if qty > 0 {
                ShipmentItemModel::add_delivered_quantity(&txn, order_item_id, qty).await?;
            }
        }
    }

    // 自动更新订单状态：首次发货→5部分发货，全部发完→6已发货
    update_order_ship_status(&txn, order_id).await?;

    txn.commit().await?;

    Ok(shipment_id)
}

pub async fn update(db: &DbConn, form_data: &ShipmentUpdateRequest) -> Result<i64> {
    let id = form_data.id.unwrap_or_default();
    if id == 0 {
        return Err(Error::from("发货单ID不能为空"));
    }

    let existing = ShipmentModel::find_by_id(db, id)
        .await?
        .ok_or_else(|| Error::from("发货单不存在"))?;

    let txn = db.begin().await?;

    // 如果有明细变更，先回滚旧的 delivered_quantity，再应用新的
    if let Some(new_items) = &form_data.items {
        // 回滚旧明细的 delivered_quantity
        let old_items = ShipmentItemModel::find_by_shipment_id(&txn, id).await?;
        for old_item in &old_items {
            if let Some(order_item_id) = old_item.order_item_id {
                let qty = old_item.quantity.unwrap_or(0);
                if qty > 0 {
                    ShipmentItemModel::sub_delivered_quantity(&txn, order_item_id, qty).await?;
                }
            }
        }

        // 删除旧明细
        ShipmentItemModel::delete_by_shipment_id(&txn, id).await?;

        // 重新计算总数量
        let total_quantity: i32 = new_items.iter().map(|i| i.quantity.unwrap_or(0)).sum();

        // 插入新明细
        ShipmentItemModel::insert_batch(&txn, id, new_items).await?;

        // 累加新的 delivered_quantity
        for item in new_items {
            if let Some(order_item_id) = item.order_item_id {
                let qty = item.quantity.unwrap_or(0);
                if qty > 0 {
                    ShipmentItemModel::add_delivered_quantity(&txn, order_item_id, qty).await?;
                }
            }
        }

        // 更新发货单总数量
        let mut dto: ShipmentSaveDTO = form_data.clone().into();
        dto.total_quantity = Some(total_quantity);
        ShipmentModel::update_by_id(&txn, id, &dto).await?;

        // 重新计算订单发货状态
        if let Some(order_id) = existing.order_id {
            update_order_ship_status(&txn, order_id).await?;
        }
    } else {
        // 仅更新发货单主表
        let dto: ShipmentSaveDTO = form_data.clone().into();
        ShipmentModel::update_by_id(&txn, id, &dto).await?;
    }

    txn.commit().await?;

    Ok(id)
}

pub async fn delete(db: &DbConn, id: i64) -> Result<i64> {
    let existing = ShipmentModel::find_by_id(db, id)
        .await?
        .ok_or_else(|| Error::from("发货单不存在"))?;

    let txn = db.begin().await?;

    // 回滚订单明细的 delivered_quantity
    let items = ShipmentItemModel::find_by_shipment_id(&txn, id).await?;
    for item in &items {
        if let Some(order_item_id) = item.order_item_id {
            let qty = item.quantity.unwrap_or(0);
            if qty > 0 {
                ShipmentItemModel::sub_delivered_quantity(&txn, order_item_id, qty).await?;
            }
        }
    }

    // 删除发货明细
    ShipmentItemModel::delete_by_shipment_id(&txn, id).await?;

    // 软删除发货单
    ShipmentModel::soft_delete(&txn, id).await?;

    // 重新计算订单发货状态
    if let Some(order_id) = existing.order_id {
        update_order_ship_status(&txn, order_id).await?;
    }

    txn.commit().await?;

    Ok(id)
}

pub async fn sign(db: &DbConn, id: i64) -> Result<i64> {
    let existing = ShipmentModel::find_by_id(db, id)
        .await?
        .ok_or_else(|| Error::from("发货单不存在"))?;

    // 状态校验：只有已发货状态(status=2)可以签收，这里允许宽松处理
    if existing.status.unwrap_or(1) == 3 {
        return Err(Error::from("该发货单已签收"));
    }

    let txn = db.begin().await?;

    // 更新发货单状态为已签收(3)
    ShipmentModel::update_status(&txn, id, 3).await?;

    // 检查订单下所有发货单是否都已签收，如是则订单状态→9已签收
    if let Some(order_id) = existing.order_id {
        let shipments = ShipmentModel::find_by_order_id(&txn, order_id).await?;
        let all_signed = !shipments.is_empty()
            && shipments.iter().all(|s| s.status.unwrap_or(0) == 3);
        if all_signed {
            OrderModel::update_status(&txn, order_id, 9, None, None).await?;
        }
    }

    txn.commit().await?;

    Ok(id)
}

/// 根据订单明细的发货情况更新订单状态
/// 首次发货→5部分发货，全部发完→6已发货
async fn update_order_ship_status<C: ConnectionTrait>(db: &C, order_id: i64) -> Result<()> {
    let order_items = OrderItemModel::find_by_order_id(db, order_id).await?;
    if order_items.is_empty() {
        return Ok(());
    }

    // 仅对实物商品（product_type=1）参与发货状态判定
    let physical_items: Vec<&order_item::Model> = order_items
        .iter()
        .filter(|i| i.product_type.unwrap_or(1) == 1)
        .collect();

    if physical_items.is_empty() {
        return Ok(());
    }

    let mut all_delivered = true;
    let mut has_delivered = false;
    for item in &physical_items {
        let qty = item.quantity.unwrap_or(Decimal::from(0));
        let delivered = item
            .delivered_quantity
            .unwrap_or(Decimal::from(0));

        if delivered > Decimal::from(0) {
            has_delivered = true;
        }
        if delivered < qty {
            all_delivered = false;
        }
    }

    let new_status = if all_delivered { 6 } else if has_delivered { 5 } else { 4 };

    // 仅当订单当前状态小于5时才更新（避免覆盖已签收等后续状态）
    let order = OrderModel::find_by_id(db, order_id).await?;
    if let Some(o) = order {
        let current = o.order_status.unwrap_or(0);
        if current < 5 && current != new_status {
            OrderModel::update_status(db, order_id, new_status, None, None).await?;
        }
    }

    Ok(())
}
