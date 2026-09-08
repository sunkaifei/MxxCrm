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
use crate::modules::message::service::online_service::OnlineService;
use crate::modules::message::websocket::ConnectionRegistry;
use crate::modules::system::service::dept_service;
use chrono::Utc;

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
        content_type: Option<i32>,
        file_url: Option<String>,
        file_name: Option<String>,
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
            content_type,
            file_url,
            file_name,
        ).await?;

        // 推送 WebSocket 通知给会话中其他参与者（含多端同步）
        push_message_via_websocket(db, sender_id, target_session_id, &response).await;

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
            None,
            None,
            None,
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
    ) -> Result<Vec<i64>, ChatServiceError> {
        let newly_read_ids = ChatModel::mark_session_read(db, user_id, session_id).await?;

        // 推送"对方已读"事件给该会话其他参与者（发送方）
        if !newly_read_ids.is_empty() {
            push_read_receipt_via_websocket(db, user_id, session_id, &newly_read_ids).await;
        }

        Ok(newly_read_ids)
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

    pub async fn get_colleague_list(
        db: &DbConn,
        current_user_id: i64,
        keyword: Option<String>,
        page: i32,
        page_size: i32,
    ) -> Result<Vec<ColleagueVO>, ChatServiceError> {
        let page = std::cmp::max(page, 1);
        let page_size = std::cmp::min(std::cmp::max(page_size, 1), 200);

        let users = ChatModel::get_colleague_list(db, current_user_id, keyword.clone(), page, page_size).await?;

        let id_list: Vec<i64> = users.iter().map(|u| u.id).collect();

        let dept_list = dept_service::select_by_ids(db, id_list).await.unwrap_or_default();

        let mut result: Vec<ColleagueVO> = Vec::new();
        for user in users {
            let mut depts_data: Vec<DeptNameDTO> = Vec::new();
            for dept_entity in &dept_list {
                if dept_entity.admin_id == Some(user.id) {
                    depts_data.push(DeptNameDTO {
                        dept_name: dept_entity.dept_name.clone(),
                    });
                }
            }
            result.push(ColleagueVO {
                id: user.id,
                user_name: user.user_name,
                nick_name: user.nick_name,
                avatar: user.avatar,
                depts: if depts_data.is_empty() { None } else { Some(depts_data) },
            });
        }

        Ok(result)
    }

    pub async fn get_total_unread_count(
        db: &DbConn,
        user_id: i64,
    ) -> Result<i32, ChatServiceError> {
        let count = ChatModel::get_total_unread_count(db, user_id).await?;
        Ok(count)
    }

    pub async fn recall_message(
        db: &DbConn,
        user_id: i64,
        message_id: i64,
    ) -> Result<bool, ChatServiceError> {
        if message_id <= 0 {
            return Err(ChatServiceError::InvalidParameter("消息ID不能为空".to_string()));
        }
        let result = ChatModel::recall_message(db, user_id, message_id).await?;
        Ok(result)
    }

    pub async fn toggle_pin(
        db: &DbConn,
        user_id: i64,
        session_id: i64,
        is_pinned: bool,
    ) -> Result<(), ChatServiceError> {
        if session_id <= 0 {
            return Err(ChatServiceError::InvalidParameter("会话ID不能为空".to_string()));
        }
        ChatModel::toggle_pin(db, user_id, session_id, is_pinned).await?;
        Ok(())
    }

    pub async fn toggle_mute(
        db: &DbConn,
        user_id: i64,
        session_id: i64,
        is_muted: bool,
    ) -> Result<(), ChatServiceError> {
        if session_id <= 0 {
            return Err(ChatServiceError::InvalidParameter("会话ID不能为空".to_string()));
        }
        ChatModel::toggle_mute(db, user_id, session_id, is_muted).await?;
        Ok(())
    }

    pub async fn get_session_detail(
        db: &DbConn,
        user_id: i64,
        session_id: i64,
    ) -> Result<SessionDetailDTO, ChatServiceError> {
        if session_id <= 0 {
            return Err(ChatServiceError::InvalidParameter("会话ID不能为空".to_string()));
        }
        let mut detail = ChatModel::get_session_detail(db, user_id, session_id).await?;

        if let Some(other_uid) = detail.other_user_id {
            let status = OnlineService::get_online_status(db, other_uid).await?;
            detail.online_status = Some(status);
        }

        Ok(detail)
    }
}

/// 通过 WebSocket 推送新消息通知给会话其他参与者
async fn push_message_via_websocket(
    db: &DbConn,
    sender_id: i64,
    session_id: i64,
    response: &SendMessageResponse,
) {
    // 查询该会话的所有参与者，推送 WebSocket 消息
    let participants = match crate::modules::message::entity::chat_session_participant::Entity::find()
        .filter(crate::modules::message::entity::chat_session_participant::Column::SessionId.eq(session_id))
        .filter(crate::modules::message::entity::chat_session_participant::Column::Deleted.eq(0))
        .all(db)
        .await
    {
        Ok(list) => list,
        Err(e) => {
            log::warn!("[WebSocket推送] 查询会话参与者失败: {:?}", e);
            return;
        }
    };

    // 查询消息详情用于推送
    let message = match crate::modules::message::entity::chat_message::Entity::find_by_id(response.message_id)
        .one(db)
        .await
    {
        Ok(Some(m)) => m,
        Ok(None) => {
            log::warn!("[WebSocket推送] 消息不存在: {}", response.message_id);
            return;
        }
        Err(e) => {
            log::warn!("[WebSocket推送] 查询消息失败: {:?}", e);
            return;
        }
    };

    // 构造推送 payload
    let payload = serde_json::json!({
        "type": "chat_message",
        "data": {
            "messageId": message.id,
            "sessionId": message.session_id,
            "senderId": message.sender_id,
            "senderNickname": message.sender_nickname,
            "senderAvatar": message.sender_avatar,
            "content": message.content,
            "messageType": message.message_type,
            "contentType": message.content_type,
            "fileUrl": message.file_url,
            "fileName": message.file_name,
            "readStatus": message.read_status.unwrap_or(0),
            "sendTime": message.send_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    });
    let payload_str = payload.to_string();

    let registry = ConnectionRegistry::global();
    for p in &participants {
        // 推送给所有参与者（含发送者多端同步，前端会自己过滤自己的消息）
        // 给每个接收方附带其当前的未读数
        let unread = p.unread_count.unwrap_or(0);
        let personalized = if unread > 0 {
            serde_json::json!({
                "type": "chat_message",
                "data": {
                    "messageId": message.id,
                    "sessionId": message.session_id,
                    "senderId": message.sender_id,
                    "senderNickname": message.sender_nickname,
                    "senderAvatar": message.sender_avatar,
                    "content": message.content,
                    "messageType": message.message_type,
                    "contentType": message.content_type,
                    "fileUrl": message.file_url,
                    "fileName": message.file_name,
                    "readStatus": message.read_status.unwrap_or(0),
                    "sendTime": message.send_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
                    "unreadCount": unread,
                }
            }).to_string()
        } else {
            payload_str.clone()
        };
        registry.send_to_user(p.user_id, personalized);
    }
}

/// 推送"对方已读"回执给会话中其他参与者（通常是发送方）
async fn push_read_receipt_via_websocket(
    db: &DbConn,
    reader_id: i64,
    session_id: i64,
    read_message_ids: &[i64],
) {
    let participants = match crate::modules::message::entity::chat_session_participant::Entity::find()
        .filter(crate::modules::message::entity::chat_session_participant::Column::SessionId.eq(session_id))
        .filter(crate::modules::message::entity::chat_session_participant::Column::Deleted.eq(0))
        .all(db)
        .await
    {
        Ok(list) => list,
        Err(e) => {
            log::warn!("[WebSocket已读回执] 查询会话参与者失败: {:?}", e);
            return;
        }
    };

    let payload = serde_json::json!({
        "type": "message_read",
        "data": {
            "sessionId": session_id,
            "readerId": reader_id,
            "messageIds": read_message_ids,
            "readTime": chrono::Utc::now().to_rfc3339(),
        }
    });
    let payload_str = payload.to_string();

    let registry = ConnectionRegistry::global();
    for p in participants {
        // 只推送给非读取者（即发送方），支持多端同步
        if p.user_id == reader_id {
            continue;
        }
        registry.send_to_user(p.user_id, payload_str.clone());
    }
}
