//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use std::collections::HashMap;
use sea_orm::*;
use chrono::{Utc, Duration};
use crate::modules::message::entity::user_online;
use crate::modules::message::entity::user_online::Entity as UserOnlineEntity;

pub const ONLINE_STATUS_ONLINE: i32 = 1;
pub const ONLINE_STATUS_AWAY: i32 = 2;
pub const ONLINE_STATUS_BUSY: i32 = 3;
pub const ONLINE_STATUS_OFFLINE: i32 = 4;

pub struct OnlineService;

impl OnlineService {
    pub async fn update_heartbeat(
        db: &DbConn,
        user_id: i64,
        session_id: String,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(), DbErr> {
        let now = Utc::now().naive_utc();

        let existing = UserOnlineEntity::find()
            .filter(user_online::Column::UserId.eq(user_id))
            .filter(user_online::Column::SessionId.eq(&session_id))
            .one(db)
            .await?;

        if let Some(record) = existing {
            let mut active: user_online::ActiveModel = record.into();
            active.last_heartbeat = Set(now);
            active.status = Set(Some(ONLINE_STATUS_ONLINE));
            active.ip_address = Set(ip);
            active.user_agent = Set(user_agent);
            active.update_time = Set(Some(now));
            active.update(db).await?;
        } else {
            let model = user_online::ActiveModel {
                user_id: Set(user_id),
                session_id: Set(session_id),
                ip_address: Set(ip),
                user_agent: Set(user_agent),
                last_heartbeat: Set(now),
                status: Set(Some(ONLINE_STATUS_ONLINE)),
                create_time: Set(Some(now)),
                update_time: Set(Some(now)),
                ..Default::default()
            };
            UserOnlineEntity::insert(model).exec(db).await?;
        }

        Ok(())
    }

    pub async fn get_online_status(
        db: &DbConn,
        user_id: i64,
    ) -> Result<i32, DbErr> {
        let records = UserOnlineEntity::find()
            .filter(user_online::Column::UserId.eq(user_id))
            .order_by_desc(user_online::Column::LastHeartbeat)
            .all(db)
            .await?;

        if records.is_empty() {
            return Ok(ONLINE_STATUS_OFFLINE);
        }

        let latest = &records[0];
        let now = Utc::now().naive_utc();
        let duration = now.signed_duration_since(latest.last_heartbeat);

        if duration.num_seconds() > 300 {
            Ok(ONLINE_STATUS_OFFLINE)
        } else {
            Ok(latest.status.unwrap_or(ONLINE_STATUS_ONLINE))
        }
    }

    pub async fn batch_get_online_status(
        db: &DbConn,
        user_ids: Vec<i64>,
    ) -> Result<HashMap<i64, i32>, DbErr> {
        let mut result = HashMap::new();

        if user_ids.is_empty() {
            return Ok(result);
        }

        for user_id in &user_ids {
            let status = Self::get_online_status(db, *user_id).await?;
            result.insert(*user_id, status);
        }

        Ok(result)
    }

    pub async fn cleanup_offline(
        db: &DbConn,
        timeout_seconds: i64,
    ) -> Result<u64, DbErr> {
        let cutoff = Utc::now().naive_utc() - Duration::seconds(timeout_seconds);

        let result = UserOnlineEntity::delete_many()
            .filter(user_online::Column::LastHeartbeat.lt(cutoff))
            .exec(db)
            .await?;

        Ok(result.rows_affected)
    }
}
