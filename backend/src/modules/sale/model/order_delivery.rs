//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 虚拟商品交付记录 Model 层
//!

use sea_orm::*;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::sale::entity::order_delivery::{self, Entity as DeliveryEntity};
// ==================== 请求 DTO ====================

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct DeliverySaveRequest {
    pub order_id: Option<i64>,
    pub order_item_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    /// 交付方式：1=卡密, 2=下载链接, 3=账号密码, 4=激活码, 5=服务开通
    pub delivery_method: Option<i32>,
    pub card_key: Option<String>,
    pub download_url: Option<String>,
    pub account_name: Option<String>,
    pub account_password: Option<String>,
    pub extra_content: Option<String>,
    pub card_pool_id: Option<i64>,
    pub remark: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeliverySaveDTO {
    pub delivery_no: Option<String>,
    pub order_id: Option<i64>,
    pub order_item_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub delivery_method: Option<i32>,
    pub card_key: Option<String>,
    pub download_url: Option<String>,
    pub account_name: Option<String>,
    pub account_password: Option<String>,
    pub extra_content: Option<String>,
    pub status: Option<i32>,
    /// 1=自动交付, 2=手动交付
    pub deliver_type: Option<i32>,
    pub sent_time: Option<chrono::NaiveDateTime>,
    pub card_pool_id: Option<i64>,
    pub remark: Option<String>,
    pub create_by: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct DeliveryListQuery {
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_to_u64")]
    pub page_num: Option<i64>,
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_to_u64")]
    pub page_size: Option<i64>,
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_to_u64")]
    pub order_id: Option<i64>,
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_to_u64")]
    pub customer_id: Option<i64>,
    pub status: Option<i32>,
    pub delivery_method: Option<i32>,
}

impl From<DeliverySaveRequest> for DeliverySaveDTO {
    fn from(req: DeliverySaveRequest) -> Self {
        Self {
            delivery_no: None,
            order_id: req.order_id,
            order_item_id: req.order_item_id,
            customer_id: req.customer_id,
            product_id: req.product_id,
            product_name: req.product_name,
            delivery_method: req.delivery_method,
            card_key: req.card_key,
            download_url: req.download_url,
            account_name: req.account_name,
            account_password: req.account_password,
            extra_content: req.extra_content,
            status: Some(2), // 默认已发送（手动录入）
            deliver_type: Some(2), // 手动
            sent_time: Some(chrono::Local::now().naive_local()),
            card_pool_id: req.card_pool_id,
            remark: req.remark,
            create_by: None,
        }
    }
}

// ==================== VO ====================

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct DeliveryListVO {
    pub id: i64,
    pub delivery_no: Option<String>,
    pub order_id: Option<i64>,
    pub order_item_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub delivery_method: Option<i32>,
    pub delivery_method_name: Option<String>,
    /// 脱敏后的卡密（前4后4 + ****）
    pub card_key_masked: Option<String>,
    pub download_url: Option<String>,
    pub account_name: Option<String>,
    pub extra_content: Option<String>,
    pub status: Option<i32>,
    pub status_name: Option<String>,
    pub deliver_type: Option<i32>,
    pub sent_time: Option<String>,
    pub received_time: Option<String>,
    pub remark: Option<String>,
    pub create_time: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct DeliveryDetailVO {
    pub id: i64,
    pub delivery_no: Option<String>,
    pub order_id: Option<i64>,
    pub order_item_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub delivery_method: Option<i32>,
    pub delivery_method_name: Option<String>,
    pub card_key_masked: Option<String>,
    pub download_url: Option<String>,
    pub account_name: Option<String>,
    pub extra_content: Option<String>,
    pub status: Option<i32>,
    pub status_name: Option<String>,
    pub deliver_type: Option<i32>,
    pub sent_time: Option<String>,
    pub received_time: Option<String>,
    pub expire_time: Option<String>,
    pub remark: Option<String>,
    pub create_by: Option<i64>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
}

// ==================== Model ====================

pub struct DeliveryModel;

impl DeliveryModel {
    pub async fn insert<C: ConnectionTrait>(db: &C, req: &DeliverySaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = order_delivery::ActiveModel {
            delivery_no: Set(req.delivery_no.clone()),
            order_id: Set(req.order_id),
            order_item_id: Set(req.order_item_id),
            customer_id: Set(req.customer_id),
            product_id: Set(req.product_id),
            product_name: Set(req.product_name.clone()),
            delivery_method: Set(req.delivery_method),
            card_key: Set(req.card_key.clone()),
            download_url: Set(req.download_url.clone()),
            account_name: Set(req.account_name.clone()),
            account_password: Set(req.account_password.clone()),
            extra_content: Set(req.extra_content.clone()),
            status: Set(req.status.or(Some(1))),
            deliver_type: Set(req.deliver_type.or(Some(2))),
            sent_time: Set(req.sent_time.or(Some(now))),
            received_time: Set(None),
            expire_time: Set(None),
            card_pool_id: Set(req.card_pool_id),
            remark: Set(req.remark.clone()),
            create_by: Set(req.create_by),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        let result = DeliveryEntity::insert(payload).exec(db).await?;
        Ok(result.last_insert_id)
    }

    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<order_delivery::Model>, DbErr> {
        DeliveryEntity::find_by_id(id)
            .filter(order_delivery::Column::Deleted.eq(0))
            .one(db).await
    }

    pub async fn find_by_order<C: ConnectionTrait>(db: &C, order_id: i64) -> Result<Vec<order_delivery::Model>, DbErr> {
        DeliveryEntity::find()
            .filter(order_delivery::Column::OrderId.eq(order_id))
            .filter(order_delivery::Column::Deleted.eq(0))
            .order_by_desc(order_delivery::Column::Id)
            .all(db).await
    }

    pub async fn count_by_item<C: ConnectionTrait>(db: &C, order_item_id: i64) -> Result<i64, DbErr> {
        let n = DeliveryEntity::find()
            .filter(order_delivery::Column::OrderItemId.eq(order_item_id))
            .filter(order_delivery::Column::Deleted.eq(0))
            .filter(order_delivery::Column::Status.eq(2))
            .count(db).await?;
        Ok(n as i64)
    }

    pub async fn update_status<C: ConnectionTrait>(
        db: &C, id: i64, status: i32
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let mut payload = order_delivery::ActiveModel {
            status: Set(Some(status)),
            update_time: Set(Some(now)),
            ..Default::default()
        };
        if status == 3 {
            // 已签收
            payload.received_time = Set(Some(now));
        }
        let result = DeliveryEntity::update_many()
            .set(payload)
            .filter(order_delivery::Column::Id.eq(id))
            .filter(order_delivery::Column::Deleted.eq(0))
            .exec(db).await?;
        Ok(result.rows_affected as i64)
    }

    /// 批量软删除
    pub async fn batch_delete<C: ConnectionTrait>(db: &C, ids: &[i64]) -> Result<i64, DbErr> {
        if ids.is_empty() {
            return Ok(0);
        }
        let result = DeliveryEntity::update_many()
            .col_expr(order_delivery::Column::Deleted, sea_orm::sea_query::Expr::value(1))
            .filter(order_delivery::Column::Id.is_in(ids.to_vec()))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn get_max_delivery_no_today<C: ConnectionTrait>(
        db: &C, prefix: &str
    ) -> Result<Option<i64>, DbErr> {
        let pattern = format!("{}%", prefix);
        let rows: Vec<(String,)> = DeliveryEntity::find()
            .filter(order_delivery::Column::DeliveryNo.like(&pattern))
            .select_only()
            .column(order_delivery::Column::DeliveryNo)
            .into_tuple()
            .all(db).await?;
        let max_seq = rows.iter()
            .filter_map(|(no,)| no.trim_start_matches(prefix).parse::<i64>().ok())
            .max();
        Ok(max_seq)
    }
}

// ==================== 辅助函数 ====================

/// 卡密脱敏：保留前后各 4 位，中间用 **** 替代
pub fn mask_card_key(key: &str) -> String {
    let len = key.chars().count();
    if len <= 8 {
        return "****".to_string();
    }
    let chars: Vec<char> = key.chars().collect();
    let prefix: String = chars[..4].iter().collect();
    let suffix: String = chars[len - 4..].iter().collect();
    format!("{}****{}", prefix, suffix)
}

pub fn delivery_method_name(method: i32) -> &'static str {
    match method {
        1 => "卡密",
        2 => "下载链接",
        3 => "账号密码",
        4 => "激活码",
        5 => "服务开通",
        _ => "未知",
    }
}

pub fn delivery_status_name(status: i32) -> &'static str {
    match status {
        1 => "待发送",
        2 => "已发送",
        3 => "已签收",
        4 => "已撤销",
        5 => "已失效",
        _ => "未知",
    }
}
