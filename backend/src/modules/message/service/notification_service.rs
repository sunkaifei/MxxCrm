//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::DbErr;
use crate::modules::message::model::notification::*;

#[derive(Debug)]
pub enum NotificationServiceError {
    InvalidParameter(String),
    DatabaseError(DbErr),
}

impl From<DbErr> for NotificationServiceError {
    fn from(err: DbErr) -> Self {
        NotificationServiceError::DatabaseError(err)
    }
}

pub struct NotificationService;

impl NotificationService {
    pub async fn get_notification_list(
        db: &sea_orm::DbConn,
        user_id: i64,
        query: NotificationListQuery,
    ) -> Result<PageResponse<NotificationDTO>, NotificationServiceError> {
        let page = std::cmp::max(query.page.unwrap_or(1), 1);
        let page_size = std::cmp::min(std::cmp::max(query.page_size.unwrap_or(20), 1), 100);

        let query = NotificationListQuery {
            page: Some(page),
            page_size: Some(page_size),
            ..query
        };

        let result = NotificationModel::get_notification_list(db, user_id, query).await?;
        Ok(result)
    }

    pub async fn get_unread_count(
        db: &sea_orm::DbConn,
        user_id: i64,
    ) -> Result<i32, NotificationServiceError> {
        let count = NotificationModel::get_unread_count(db, user_id).await?;
        Ok(count)
    }

    pub async fn mark_as_read(
        db: &sea_orm::DbConn,
        user_id: i64,
        notification_id: i64,
    ) -> Result<i64, NotificationServiceError> {
        if notification_id <= 0 {
            return Err(NotificationServiceError::InvalidParameter("通知ID不能为空".to_string()));
        }
        let result = NotificationModel::mark_as_read(db, user_id, notification_id).await?;
        Ok(result)
    }

    pub async fn mark_all_read(
        db: &sea_orm::DbConn,
        user_id: i64,
    ) -> Result<i64, NotificationServiceError> {
        let result = NotificationModel::mark_all_read(db, user_id).await?;
        Ok(result)
    }

    pub async fn delete_notification(
        db: &sea_orm::DbConn,
        user_id: i64,
        notification_id: i64,
    ) -> Result<i64, NotificationServiceError> {
        if notification_id <= 0 {
            return Err(NotificationServiceError::InvalidParameter("通知ID不能为空".to_string()));
        }
        let result = NotificationModel::delete_notification(db, user_id, notification_id).await?;
        Ok(result)
    }

    pub async fn send_notification(
        db: &sea_orm::DbConn,
        sender_id: Option<i64>,
        receiver_ids: Vec<i64>,
        dto: SendNotificationRequest,
    ) -> Result<i64, NotificationServiceError> {
        if dto.title.trim().is_empty() {
            return Err(NotificationServiceError::InvalidParameter("通知标题不能为空".to_string()));
        }
        if receiver_ids.is_empty() {
            return Err(NotificationServiceError::InvalidParameter("接收人不能为空".to_string()));
        }
        let result = NotificationModel::send_notification(db, sender_id, receiver_ids, dto).await?;
        Ok(result)
    }

    pub async fn send_system_notification(
        db: &sea_orm::DbConn,
        receiver_id: i64,
        title: String,
        content: String,
        notification_type: i32,
        link_url: Option<String>,
    ) -> Result<i64, NotificationServiceError> {
        if title.trim().is_empty() {
            return Err(NotificationServiceError::InvalidParameter("通知标题不能为空".to_string()));
        }
        if receiver_id <= 0 {
            return Err(NotificationServiceError::InvalidParameter("接收人ID不能为空".to_string()));
        }
        let result = NotificationModel::send_system_notification(
            db,
            receiver_id,
            title,
            content,
            notification_type,
            link_url,
        ).await?;
        Ok(result)
    }
}
