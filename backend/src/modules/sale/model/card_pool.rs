//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 卡密池 Model 层
//!

use sea_orm::*;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::sale::entity::card_pool::{self, Entity as CardPoolEntity};

// ==================== 请求 DTO ====================

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CardPoolSaveRequest {
    pub product_id: Option<i64>,
    pub batch_no: Option<String>,
    pub card_key: Option<String>,
    pub card_password: Option<String>,
    pub import_batch: Option<String>,
    pub expire_time: Option<chrono::DateTime<chrono::Local>>,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CardPoolListQuery {
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_to_u64")]
    pub page_num: Option<i64>,
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_to_u64")]
    pub page_size: Option<i64>,
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_to_u64")]
    pub product_id: Option<i64>,
    pub status: Option<i32>,
    pub batch_no: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CardPoolImportRequest {
    pub product_id: Option<i64>,
    pub batch_no: Option<String>,
    pub card_keys: Vec<String>,
    pub import_batch: Option<String>,
    pub expire_time: Option<chrono::DateTime<chrono::Local>>,
    pub remark: Option<String>,
}

// ==================== VO ====================

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CardPoolListVO {
    pub id: i64,
    pub product_id: Option<i64>,
    pub batch_no: Option<String>,
    pub card_key_masked: Option<String>,
    pub status: Option<i32>,
    pub status_name: Option<String>,
    pub lock_order_id: Option<i64>,
    pub sold_order_id: Option<i64>,
    pub sold_time: Option<String>,
    pub import_batch: Option<String>,
    pub expire_time: Option<String>,
    pub remark: Option<String>,
    pub create_time: Option<String>,
}

// ==================== Model ====================

pub struct CardPoolModel;

impl CardPoolModel {
    pub async fn insert<C: ConnectionTrait>(
        db: &C, req: &CardPoolSaveRequest
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = card_pool::ActiveModel {
            product_id: Set(req.product_id),
            batch_no: Set(req.batch_no.clone()),
            card_key: Set(req.card_key.clone()),
            card_password: Set(req.card_password.clone()),
            status: Set(Some(1)),
            lock_order_id: Set(None),
            lock_expire_time: Set(None),
            sold_order_id: Set(None),
            sold_time: Set(None),
            import_batch: Set(req.import_batch.clone()),
            expire_time: Set(req.expire_time.map(|t| t.naive_local())),
            remark: Set(req.remark.clone()),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        let result = CardPoolEntity::insert(payload).exec(db).await?;
        Ok(result.last_insert_id)
    }

    /// 批量导入卡密
    pub async fn insert_batch<C: ConnectionTrait>(
        db: &C, product_id: i64, batch_no: Option<&str>, card_keys: &[String], import_batch: &str, expire_time: Option<chrono::NaiveDateTime>, remark: Option<&str>
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let models: Vec<card_pool::ActiveModel> = card_keys.iter().map(|key| card_pool::ActiveModel {
            product_id: Set(Some(product_id)),
            batch_no: Set(batch_no.map(|s| s.to_string())),
            card_key: Set(Some(key.clone())),
            card_password: Set(None),
            status: Set(Some(1)),
            lock_order_id: Set(None),
            lock_expire_time: Set(None),
            sold_order_id: Set(None),
            sold_time: Set(None),
            import_batch: Set(Some(import_batch.to_string())),
            expire_time: Set(expire_time),
            remark: Set(remark.map(|s| s.to_string())),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        }).collect();

        if models.is_empty() {
            return Ok(0);
        }
        let result = CardPoolEntity::insert_many(models).exec(db).await?;
        Ok(result.last_insert_id.unwrap_or_default())
    }

    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<card_pool::Model>, DbErr> {
        CardPoolEntity::find_by_id(id)
            .filter(card_pool::Column::Deleted.eq(0))
            .one(db).await
    }

    /// 锁定一张未售卡密（事务内调用）
    pub async fn lock_one<C: ConnectionTrait>(
        db: &C, product_id: i64
    ) -> Result<Option<card_pool::Model>, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        // 查询一张未售卡密
        let card = CardPoolEntity::find()
            .filter(card_pool::Column::ProductId.eq(product_id))
            .filter(card_pool::Column::Status.eq(1)) // 未售
            .filter(card_pool::Column::Deleted.eq(0))
            .one(db).await?;
        if let Some(c) = card {
            let card_id = c.id;
            // 直接 update_many 改状态为已售（更安全）
            let payload = card_pool::ActiveModel {
                status: Set(Some(3)),
                sold_time: Set(Some(now)),
                update_time: Set(Some(now)),
                ..Default::default()
            };
            let _ = CardPoolEntity::update_many()
                .set(payload)
                .filter(card_pool::Column::Id.eq(card_id))
                .filter(card_pool::Column::Deleted.eq(0))
                .exec(db).await?;
            return Ok(Some(c));
        }
        Ok(None)
    }

    /// 标记卡密已售
    pub async fn mark_sold<C: ConnectionTrait>(
        db: &C, card_id: i64, order_id: i64
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = card_pool::ActiveModel {
            status: Set(Some(3)),
            sold_order_id: Set(Some(order_id)),
            sold_time: Set(Some(now)),
            update_time: Set(Some(now)),
            ..Default::default()
        };
        let result = CardPoolEntity::update_many()
            .set(payload)
            .filter(card_pool::Column::Id.eq(card_id))
            .filter(card_pool::Column::Deleted.eq(0))
            .exec(db).await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn soft_delete<C: ConnectionTrait>(db: &C, id: i64) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = card_pool::ActiveModel {
            deleted: Set(Some(1)),
            status: Set(Some(4)), // 作废
            update_time: Set(Some(now)),
            ..Default::default()
        };
        let result = CardPoolEntity::update_many()
            .set(payload)
            .filter(card_pool::Column::Id.eq(id))
            .filter(card_pool::Column::Deleted.eq(0))
            .exec(db).await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn count_unsold<C: ConnectionTrait>(db: &C, product_id: i64) -> Result<i64, DbErr> {
        let n = CardPoolEntity::find()
            .filter(card_pool::Column::ProductId.eq(product_id))
            .filter(card_pool::Column::Status.eq(1))
            .filter(card_pool::Column::Deleted.eq(0))
            .count(db).await?;
        Ok(n as i64)
    }
}

pub fn card_status_name(status: i32) -> &'static str {
    match status {
        1 => "未售",
        2 => "已锁定",
        3 => "已售",
        4 => "已作废",
        _ => "未知",
    }
}
