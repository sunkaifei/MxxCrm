//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::*;
use crate::modules::message::model::chat::*;

#[derive(Debug)]
pub enum ChatServiceError {
    InvalidParameter(String),
    UserNotFound,
    SessionNotFound,
    PermissionDenied,
    DatabaseError(DbErr),
}

impl From<DbErr> for ChatServiceError {
    fn from(err: DbErr) -> Self {
        ChatServiceError::DatabaseError(err)
    }
}

pub struct ChatService;

impl ChatService {
    pub async fn get_or_create_session(
        db: &DbConn,
        user_id: i64,
        receiver_id: i64,
    ) -> Result<i64, ChatServiceError> {
        if receiver_id == user_id {
            return Err(ChatServiceError::InvalidParameter("不能和自己聊天".to_string()));
        }

        let session_id = ChatModel::get_or_create_private_session(db, user_id, receiver_id).await?;
        Ok(session_id)
    }

    pub async fn get_system_session(db: &DbConn) -> Result<i64, ChatServiceError> {
        let session_id = ChatModel::get_or_create_system_session(db).await?;
        Ok(session_id)
    }

    pub async fn send_message(
        db: &DbConn,
        sender_id: i64,
        session_id: Option<i64>,
        receiver_id: Option<i64>,
        content: String,
    ) -> Result<SendMessageResponse, ChatServiceError> {
        if content.trim().is_empty() {
            return Err(ChatServiceError::InvalidParameter("消息内容不能为空".to_string()));
        }

        let target_session_id = if let Some(sid) = session_id {
            sid
        } else if let Some(rid) = receiver_id {
            ChatModel::get_or_create_private_session(db, sender_id, rid).await?
        } else {
            return Err(ChatServiceError::InvalidParameter("必须指定会话ID或接收人ID".to_string()));
        };

        let response = ChatModel::send_message(
            db,
            sender_id,
            target_session_id,
            content,
            MESSAGE_TYPE_USER,
        ).await?;

        Ok(response)
    }

    pub async fn send_system_message(
        db: &DbConn,
        content: String,
    ) -> Result<SendMessageResponse, ChatServiceError> {
        if content.trim().is_empty() {
            return Err(ChatServiceError::InvalidParameter("消息内容不能为空".to_string()));
        }

        let system_user_id = 0i64;
        let session_id = ChatModel::get_or_create_system_session(db).await?;

        let response = ChatModel::send_message(
            db,
            system_user_id,
            session_id,
            content,
            MESSAGE_TYPE_SYSTEM,
        ).await?;

        Ok(response)
    }

    pub async fn get_session_list(
        db: &DbConn,
        user_id: i64,
        page: i32,
        page_size: i32,
    ) -> Result<PageResponse<ChatSessionDTO>, ChatServiceError> {
        let page = std::cmp::max(page, 1);
        let page_size = std::cmp::min(std::cmp::max(page_size, 1), 50);

        let result = ChatModel::get_session_list(db, user_id, page, page_size).await?;
        Ok(result)
    }

    pub async fn get_chat_messages(
        db: &DbConn,
        user_id: i64,
        session_id: i64,
        page: i32,
        page_size: i32,
    ) -> Result<PageResponse<ChatMessageDTO>, ChatServiceError> {
        let page = std::cmp::max(page, 1);
        let page_size = std::cmp::min(std::cmp::max(page_size, 1), 50);

        let result = ChatModel::get_chat_messages(db, user_id, session_id, page, page_size).await?;
        Ok(result)
    }

    pub async fn mark_session_read(
        db: &DbConn,
        user_id: i64,
        session_id: i64,
    ) -> Result<(), ChatServiceError> {
        ChatModel::mark_session_read(db, user_id, session_id).await?;
        Ok(())
    }

    pub async fn delete_session(
        db: &DbConn,
        user_id: i64,
        session_id: i64,
    ) -> Result<(), ChatServiceError> {
        ChatModel::delete_session(db, user_id, session_id).await?;
        Ok(())
    }

    pub async fn search_users(
        db: &DbConn,
        keyword: String,
        page: i32,
        page_size: i32,
    ) -> Result<Vec<UserSearchDTO>, ChatServiceError> {
        if keyword.trim().is_empty() {
            return Err(ChatServiceError::InvalidParameter("搜索关键词不能为空".to_string()));
        }

        let page = std::cmp::max(page, 1);
        let page_size = std::cmp::min(std::cmp::max(page_size, 1), 50);

        let result = ChatModel::search_users_by_nickname(db, keyword, page, page_size).await?;
        Ok(result)
    }

    pub async fn get_total_unread_count(
        db: &DbConn,
        user_id: i64,
    ) -> Result<i32, ChatServiceError> {
        let count = ChatModel::get_total_unread_count(db, user_id).await?;
        Ok(count)
    }
}
