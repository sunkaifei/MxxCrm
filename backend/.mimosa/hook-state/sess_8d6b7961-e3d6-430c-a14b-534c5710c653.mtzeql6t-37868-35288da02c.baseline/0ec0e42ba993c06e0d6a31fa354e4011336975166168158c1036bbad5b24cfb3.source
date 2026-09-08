//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究。
//!

use sea_orm::*;
use sea_orm::prelude::DateTime;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::website::entity::{website_delivery, website_delivery::Entity as WebsiteDelivery};
// Note: Delivery model does not use Decimal fields

// ==================== DTO ====================

/// 后台发货请求（在 order model 中已定义 ShipRequest，此处保留兼容）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct DeliveryCreateRequest {
    pub order_id: i64,
    pub order_no: Option<String>,
    pub delivery_no: String,
    pub delivery_company: String,
    pub delivery_type: Option<i32>,
    pub item_count: Option<i32>,
    pub remark: Option<String>,
}

/// 发货列表查询
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct DeliveryListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub order_no: Option<String>,
    pub delivery_no: Option<String>,
    pub order_id: Option<i64>,
}

// ==================== VO ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct DeliveryVO {
    pub id: Option<i64>,
    pub order_id: Option<i64>,
    pub order_no: Option<String>,
    pub delivery_no: Option<String>,
    pub delivery_company: Option<String>,
    pub delivery_type: Option<i32>,
    pub status: Option<i32>,
    pub shipper_id: Option<i64>,
    pub shipper_name: Option<String>,
    pub consignee_name: Option<String>,
    pub consignee_phone: Option<String>,
    pub consignee_address: Option<String>,
    pub item_count: Option<i32>,
    pub remark: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

impl From<website_delivery::Model> for DeliveryVO {
    fn from(item: website_delivery::Model) -> Self {
        DeliveryVO {
            id: Option::from(item.id),
            order_id: Some(item.order_id),
            order_no: item.order_no,
            delivery_no: item.delivery_no,
            delivery_company: item.delivery_company,
            delivery_type: item.delivery_type,
            status: item.status,
            shipper_id: item.shipper_id,
            shipper_name: item.shipper_name,
            consignee_name: item.consignee_name,
            consignee_phone: item.consignee_phone,
            consignee_address: item.consignee_address,
            item_count: item.item_count,
            remark: item.remark,
            create_time: item.create_time,
            update_time: item.update_time,
        }
    }
}

// ==================== Model ====================

pub struct WebsiteDeliveryModel;

impl WebsiteDeliveryModel {
    /// 新增发货单
    pub async fn insert<C: ConnectionTrait>(
        db: &C,
        req: &DeliveryCreateRequest,
        shipper_id: i64,
        shipper_name: String,
        consignee_name: Option<String>,
        consignee_phone: Option<String>,
        consignee_address: Option<String>,
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = website_delivery::ActiveModel {
            order_id: Set(req.order_id),
            order_no: Set(req.order_no.clone()),
            delivery_no: Set(Some(req.delivery_no.clone())),
            delivery_company: Set(Some(req.delivery_company.clone())),
            delivery_type: Set(req.delivery_type.or(Some(1))),
            status: Set(Some(0)),
            shipper_id: Set(Some(shipper_id)),
            shipper_name: Set(Some(shipper_name)),
            consignee_name: Set(consignee_name),
            consignee_phone: Set(consignee_phone),
            consignee_address: Set(consignee_address),
            item_count: Set(req.item_count),
            remark: Set(req.remark.clone()),
            create_time: Set(Some(now.clone())),
            update_time: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        WebsiteDelivery::insert(payload).exec(db).await.map(|r| r.last_insert_id)
    }

    /// 根据订单ID查询发货单列表
    pub async fn find_by_order_id<C: ConnectionTrait>(db: &C, order_id: i64) -> Result<Vec<website_delivery::Model>, DbErr> {
        WebsiteDelivery::find()
            .filter(website_delivery::Column::OrderId.eq(order_id))
            .filter(website_delivery::Column::Deleted.eq(0))
            .order_by_desc(website_delivery::Column::CreateTime)
            .all(db)
            .await
    }

    /// 分页查询
    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        query: &DeliveryListQuery,
    ) -> Result<(Vec<website_delivery::Model>, i64), DbErr> {
        let mut q = WebsiteDelivery::find()
            .filter(website_delivery::Column::Deleted.eq(0));
        if let Some(no) = &query.order_no { q = q.filter(website_delivery::Column::OrderNo.like(format!("%{}%", no))); }
        if let Some(no) = &query.delivery_no { q = q.filter(website_delivery::Column::DeliveryNo.like(format!("%{}%", no))); }
        if let Some(oid) = query.order_id { q = q.filter(website_delivery::Column::OrderId.eq(oid)); }
        let paginator = q
            .order_by_desc(website_delivery::Column::CreateTime)
            .paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;
        let rows = paginator.fetch_page((page - 1) as u64).await?;
        Ok((rows, total))
    }

    /// 软删除
    pub async fn batch_soft_delete<C: ConnectionTrait>(db: &C, ids: Vec<i64>) -> Result<i64, DbErr> {
        if ids.is_empty() { return Ok(0); }
        let now = chrono::Local::now().naive_local().to_owned();
        let result: UpdateResult = WebsiteDelivery::update_many()
            .col_expr(website_delivery::Column::Deleted, sea_orm::sea_query::Expr::value(1))
            .col_expr(website_delivery::Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
            .filter(website_delivery::Column::Id.is_in(ids))
            .filter(website_delivery::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }
}
