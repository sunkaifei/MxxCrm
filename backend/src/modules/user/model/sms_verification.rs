//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::{prelude::*, DbConn, DbErr, Set, QueryOrder};
use crate::modules::user::entity::sms_verification::{ActiveModel, Column, Entity, Model};
use crate::core::kit::global::Serialize;
use serde::Deserialize;
use chrono::{Local, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmsVerificationVO {
    pub id: i64,
    pub phone: String,
    pub code: String,
    pub scene: String,
    pub status: i32,
    pub send_count: i32,
    pub error_count: i32,
    pub lock_until: Option<DateTime>,
    pub expire_time: DateTime,
    pub ip: Option<String>,
    pub create_time: DateTime,
    pub update_time: Option<DateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmsVerificationRequest {
    pub phone: String,
    pub scene: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmsLoginRequest {
    pub phone: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmsSendResult {
    pub request_id: String,
    pub phone: String,
    pub expire_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationResult {
    pub success: bool,
    pub message: String,
    pub remain_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmsLoginResult {
    pub token: String,
    pub expire_time: i64,
    pub user_info: crate::modules::user::model::user_platform::UserInfo,
}

pub struct SmsVerificationModel;

impl SmsVerificationModel {
    pub async fn find_by_phone_and_scene(db: &DbConn, phone: &str, scene: &str) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(Column::Phone.eq(phone))
            .filter(Column::Scene.eq(scene))
            .one(db)
            .await
    }

    pub async fn find_latest(db: &DbConn, phone: &str, scene: &str) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(Column::Phone.eq(phone))
            .filter(Column::Scene.eq(scene))
            .order_by_desc(Column::CreateTime)
            .one(db)
            .await
    }

    pub async fn insert(db: &DbConn, phone: &str, code: &str, scene: &str, ip: Option<&str>) -> Result<i64, DbErr> {
        let now = Local::now().naive_local();
        let expire_time = now + Duration::minutes(5);

        let active_model = ActiveModel {
            phone: Set(phone.to_string()),
            code: Set(code.to_string()),
            scene: Set(scene.to_string()),
            status: Set(0),
            send_count: Set(1),
            error_count: Set(0),
            lock_until: Set(None),
            expire_time: Set(expire_time),
            ip: Set(ip.map(|s| s.to_string())),
            create_time: Set(now),
            update_time: Set(Some(now)),
            ..Default::default()
        };

        Entity::insert(active_model).exec(db).await.map(|r| r.last_insert_id)
    }

    pub async fn update_send_count(db: &DbConn, id: i64) -> Result<Model, DbErr> {
        let now = Local::now().naive_local();
        let expire_time = now + Duration::minutes(5);

        let model = Entity::find_by_id(id).one(db).await?;
        if let Some(m) = model {
            let active_model = ActiveModel {
                id: Set(id),
                send_count: Set(m.send_count + 1),
                expire_time: Set(expire_time),
                update_time: Set(Some(now)),
                ..m.into()
            };
            Entity::update(active_model).exec(db).await
        } else {
            Err(DbErr::RecordNotFound(String::from("Record not found")))
        }
    }

    pub async fn update_status(db: &DbConn, id: i64, status: i32) -> Result<Model, DbErr> {
        let active_model = ActiveModel {
            id: Set(id),
            status: Set(status),
            update_time: Set(Some(Local::now().naive_local())),
            ..Default::default()
        };

        Entity::update(active_model).exec(db).await
    }

    pub async fn update_error_count(db: &DbConn, id: i64, error_count: i32) -> Result<Model, DbErr> {
        let active_model = ActiveModel {
            id: Set(id),
            error_count: Set(error_count),
            update_time: Set(Some(Local::now().naive_local())),
            ..Default::default()
        };

        Entity::update(active_model).exec(db).await
    }

    pub async fn update_error_and_lock(db: &DbConn, id: i64, error_count: i32, lock_until: chrono::NaiveDateTime) -> Result<Model, DbErr> {
        let active_model = ActiveModel {
            id: Set(id),
            error_count: Set(error_count),
            lock_until: Set(Some(lock_until)),
            update_time: Set(Some(Local::now().naive_local())),
            ..Default::default()
        };

        Entity::update(active_model).exec(db).await
    }

    pub async fn get_today_send_count(db: &DbConn, phone: &str) -> Result<i32, DbErr> {
        let today_start = Local::now().naive_local().date().and_hms_opt(0, 0, 0).unwrap_or_default();
        let today_end = Local::now().naive_local().date().and_hms_opt(23, 59, 59).unwrap_or_default();

        let count = Entity::find()
            .filter(Column::Phone.eq(phone))
            .filter(Column::CreateTime.between(today_start, today_end))
            .count(db).await? as i64;

        Ok(count as i32)
    }
}
