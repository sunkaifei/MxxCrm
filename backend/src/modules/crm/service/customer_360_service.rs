use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use sea_orm::prelude::Decimal;

use crate::core::errors::error::{Error, Result};
use crate::modules::crm::entity::customer::Entity as CustomerEntity;
use crate::modules::crm::entity::service_ticket::Entity as TicketEntity;
use crate::modules::sale::entity::order::Entity as OrderEntity;
use crate::modules::sale::entity::entitlement::Entity as EntEntity;
use crate::modules::sale::entity::order_delivery::Entity as DeliveryEntity;
use crate::modules::sale::entity::payment::Entity as PaymentEntity;
use crate::modules::sale::entity::invoice::Entity as InvoiceEntity;

/// 客户 360 视图：聚合所有关联数据
pub async fn get_360_view(db: &DbConn, customer_id: i64) -> Result<serde_json::Value> {
    let customer = CustomerEntity::find_by_id(customer_id)
        .one(db)
        .await?
        .ok_or_else(|| Error::from("客户不存在"))?;

    // 基本信息
    let basic_info = serde_json::json!({
        "id": customer.id,
        "customerNo": customer.customer_no,
        "customerType": customer.customer_type,
        "companyName": customer.company_name,
        "shortName": customer.short_name,
        "personName": customer.person_name,
        "level": customer.level,
        "industry": customer.industry,
        "region": customer.region,
        "address": customer.address,
    });

    // 订单列表（最近10条）
    let orders = OrderEntity::find()
        .filter(crate::modules::sale::entity::order::Column::CustomerId.eq(customer_id))
        .filter(crate::modules::sale::entity::order::Column::Deleted.eq(0))
        .order_by_desc(crate::modules::sale::entity::order::Column::Id)
        .limit(10)
        .all(db)
        .await?;

    // 合同列表
    use crate::modules::crm::entity::contract::Entity as ContractEntity;
    let contracts = ContractEntity::find()
        .filter(crate::modules::crm::entity::contract::Column::CustomerId.eq(customer_id))
        .filter(crate::modules::crm::entity::contract::Column::Deleted.eq(0))
        .order_by_desc(crate::modules::crm::entity::contract::Column::Id)
        .all(db)
        .await?;

    // 权益列表
    let entitlements = EntEntity::find()
        .filter(crate::modules::sale::entity::entitlement::Column::CustomerId.eq(customer_id))
        .filter(crate::modules::sale::entity::entitlement::Column::Deleted.eq(0))
        .order_by_desc(crate::modules::sale::entity::entitlement::Column::Id)
        .all(db)
        .await?;

    // 交付记录
    let deliveries = DeliveryEntity::find()
        .filter(crate::modules::sale::entity::order_delivery::Column::CustomerId.eq(customer_id))
        .filter(crate::modules::sale::entity::order_delivery::Column::Deleted.eq(0))
        .order_by_desc(crate::modules::sale::entity::order_delivery::Column::Id)
        .all(db)
        .await?;

    // 回款记录
    let payments = PaymentEntity::find()
        .filter(crate::modules::sale::entity::payment::Column::CustomerId.eq(customer_id))
        .filter(crate::modules::sale::entity::payment::Column::Deleted.eq(0))
        .order_by_desc(crate::modules::sale::entity::payment::Column::Id)
        .all(db)
        .await?;

    // 发票记录
    let invoices = InvoiceEntity::find()
        .filter(crate::modules::sale::entity::invoice::Column::CustomerId.eq(customer_id))
        .filter(crate::modules::sale::entity::invoice::Column::Deleted.eq(0))
        .order_by_desc(crate::modules::sale::entity::invoice::Column::Id)
        .all(db)
        .await?;

    // 工单列表
    let tickets = TicketEntity::find()
        .filter(crate::modules::crm::entity::service_ticket::Column::CustomerId.eq(customer_id))
        .filter(crate::modules::crm::entity::service_ticket::Column::Deleted.eq(0))
        .order_by_desc(crate::modules::crm::entity::service_ticket::Column::Id)
        .all(db)
        .await?;

    // 统计汇总
    let total_orders = orders.len();
    let total_deal_amount: Decimal = orders.iter()
        .filter_map(|o| o.total_amount)
        .sum();
    let active_entitlements = entitlements.iter()
        .filter(|e| e.status == Some(2))
        .count();
    let pending_payment_amount: Decimal = payments.iter()
        .filter(|p| {
            // 未确认或部分确认的回款
            p.status != Some(3)
        })
        .filter_map(|p| p.amount)
        .sum();

    let summary = serde_json::json!({
        "totalOrders": total_orders,
        "totalDealAmount": total_deal_amount,
        "activeEntitlements": active_entitlements,
        "pendingPaymentAmount": pending_payment_amount,
    });

    Ok(serde_json::json!({
        "basicInfo": basic_info,
        "orders": orders,
        "contracts": contracts,
        "entitlements": entitlements,
        "deliveries": deliveries,
        "payments": payments,
        "invoices": invoices,
        "tickets": tickets,
        "summary": summary,
    }))
}

/// 仅统计汇总
pub async fn get_summary(db: &DbConn, customer_id: i64) -> Result<serde_json::Value> {
    let orders = OrderEntity::find()
        .filter(crate::modules::sale::entity::order::Column::CustomerId.eq(customer_id))
        .filter(crate::modules::sale::entity::order::Column::Deleted.eq(0))
        .all(db)
        .await?;

    let entitlements = EntEntity::find()
        .filter(crate::modules::sale::entity::entitlement::Column::CustomerId.eq(customer_id))
        .filter(crate::modules::sale::entity::entitlement::Column::Deleted.eq(0))
        .all(db)
        .await?;

    let payments = PaymentEntity::find()
        .filter(crate::modules::sale::entity::payment::Column::CustomerId.eq(customer_id))
        .filter(crate::modules::sale::entity::payment::Column::Deleted.eq(0))
        .all(db)
        .await?;

    let total_orders = orders.len();
    let total_deal_amount: Decimal = orders.iter()
        .filter_map(|o| o.total_amount)
        .sum();
    let active_entitlements = entitlements.iter()
        .filter(|e| e.status == Some(2))
        .count();
    let pending_payment_amount: Decimal = payments.iter()
        .filter(|p| p.status != Some(3))
        .filter_map(|p| p.amount)
        .sum();

    Ok(serde_json::json!({
        "totalOrders": total_orders,
        "totalDealAmount": total_deal_amount,
        "activeEntitlements": active_entitlements,
        "pendingPaymentAmount": pending_payment_amount,
    }))
}
