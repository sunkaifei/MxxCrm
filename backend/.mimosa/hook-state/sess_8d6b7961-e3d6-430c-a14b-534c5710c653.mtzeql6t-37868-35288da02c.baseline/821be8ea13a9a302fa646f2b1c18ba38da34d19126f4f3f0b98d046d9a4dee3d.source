//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use serde::{Deserialize, Serialize};
use sea_orm::{EntityTrait, QuerySelect, QueryFilter, QueryOrder, ColumnTrait, PaginatorTrait, DbErr, DbConn, Set};
use chrono::Utc;
use crate::modules::message::entity::system_notification;
use crate::modules::message::entity::system_notification::Entity as SystemNotificationEntity;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationListQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub r#type: Option<i32>,
    pub is_read: Option<i32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationDTO {
    pub id: i64,
    pub title: String,
    pub content: Option<String>,
    pub r#type: i32,
    pub biz_type: Option<String>,
    pub biz_id: Option<i64>,
    pub sender_id: Option<i64>,
    pub receiver_id: i64,
    pub is_read: bool,
    pub read_time: Option<String>,
    pub link_url: Option<String>,
    pub create_time: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendNotificationRequest {
    pub title: String,
    pub content: Option<String>,
    pub r#type: i32,
    pub biz_type: Option<String>,
    pub biz_id: Option<i64>,
    pub receiver_id: Option<i64>,
    pub receiver_ids: Option<Vec<i64>>,
    pub link_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadNotificationRequest {
    pub id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkAllReadRequest {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteNotificationRequest {
    pub id: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageResponse<T> {
    pub list: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

pub struct NotificationModel;

impl NotificationModel {
    pub async fn get_notification_list(
        db: &DbConn,
        user_id: i64,
        query: NotificationListQuery,
    ) -> Result<PageResponse<NotificationDTO>, DbErr> {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        let offset = (page - 1) * page_size;

        let mut select = SystemNotificationEntity::find()
            .filter(system_notification::Column::ReceiverId.eq(user_id));

        if let Some(t) = query.r#type {
            select = select.filter(system_notification::Column::Type.eq(t));
        }
        if let Some(read) = query.is_read {
            select = select.filter(system_notification::Column::IsRead.eq(read));
        }

        let total = select.clone().count(db).await? as i64;

        let list = select
            .order_by_desc(system_notification::Column::CreateTime)
            .offset(offset as u64)
            .limit(page_size as u64)
            .all(db)
            .await?;

        let dto_list: Vec<NotificationDTO> = list
            .into_iter()
            .map(|n| NotificationDTO {
                id: n.id,
                title: n.title,
                content: n.content,
                r#type: n.r#type,
                biz_type: n.biz_type,
                biz_id: n.biz_id,
                sender_id: n.sender_id,
                receiver_id: n.receiver_id,
                is_read: n.is_read.unwrap_or(0) == 1,
                read_time: n.read_time.map(|t| t.and_utc().to_rfc3339()),
                link_url: n.link_url,
                create_time: n.create_time.map(|t| t.and_utc().to_rfc3339()),
            })
            .collect();

        Ok(PageResponse {
            list: dto_list,
            total,
            page,
            page_size,
        })
    }

    pub async fn get_unread_count(db: &DbConn, user_id: i64) -> Result<i32, DbErr> {
        let count = SystemNotificationEntity::find()
            .filter(system_notification::Column::ReceiverId.eq(user_id))
            .filter(system_notification::Column::IsRead.eq(0))
            .count(db)
            .await?;
        Ok(count as i32)
    }

    pub async fn mark_as_read(
        db: &DbConn,
        user_id: i64,
        notification_id: i64,
    ) -> Result<i64, DbErr> {
        let result = SystemNotificationEntity::update_many()
            .col_expr(system_notification::Column::IsRead, sea_orm::sea_query::Expr::value(1))
            .col_expr(system_notification::Column::ReadTime, sea_orm::sea_query::Expr::value(Utc::now().naive_utc()))
            .filter(system_notification::Column::Id.eq(notification_id))
            .filter(system_notification::Column::ReceiverId.eq(user_id))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn mark_all_read(db: &DbConn, user_id: i64) -> Result<i64, DbErr> {
        let result = SystemNotificationEntity::update_many()
            .col_expr(system_notification::Column::IsRead, sea_orm::sea_query::Expr::value(1))
            .col_expr(system_notification::Column::ReadTime, sea_orm::sea_query::Expr::value(Utc::now().naive_utc()))
            .filter(system_notification::Column::ReceiverId.eq(user_id))
            .filter(system_notification::Column::IsRead.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn delete_notification(
        db: &DbConn,
        user_id: i64,
        notification_id: i64,
    ) -> Result<i64, DbErr> {
        let result = SystemNotificationEntity::delete_many()
            .filter(system_notification::Column::Id.eq(notification_id))
            .filter(system_notification::Column::ReceiverId.eq(user_id))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn send_notification(
        db: &DbConn,
        sender_id: Option<i64>,
        receiver_ids: Vec<i64>,
        dto: SendNotificationRequest,
    ) -> Result<i64, DbErr> {
        let mut count = 0i64;
        for receiver_id in receiver_ids {
            let model = system_notification::ActiveModel {
                title: Set(dto.title.clone()),
                content: Set(dto.content.clone()),
                r#type: Set(dto.r#type),
                biz_type: Set(dto.biz_type.clone()),
                biz_id: Set(dto.biz_id),
                sender_id: Set(sender_id),
                receiver_id: Set(receiver_id),
                is_read: Set(Some(0)),
                link_url: Set(dto.link_url.clone()),
                create_time: Set(Some(Utc::now().naive_utc())),
                ..Default::default()
            };
            let result = SystemNotificationEntity::insert(model).exec(db).await?;
            if result.last_insert_id > 0 {
                count += 1;
            }
        }
        Ok(count)
    }

    pub async fn send_system_notification(
        db: &DbConn,
        receiver_id: i64,
        title: String,
        content: String,
        notification_type: i32,
        link_url: Option<String>,
    ) -> Result<i64, DbErr> {
        let model = system_notification::ActiveModel {
            title: Set(title),
            content: Set(Some(content)),
            r#type: Set(notification_type),
            receiver_id: Set(receiver_id),
            is_read: Set(Some(0)),
            link_url: Set(link_url),
            create_time: Set(Some(Utc::now().naive_utc())),
            ..Default::default()
        };
        let result = SystemNotificationEntity::insert(model).exec(db).await?;
        Ok(result.last_insert_id)
    }
}
