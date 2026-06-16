//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use serde::{Deserialize, Serialize, de::Error as SerdeError};
use serde_json;
use sea_orm::{EntityTrait, QuerySelect, QueryFilter, QueryOrder, ColumnTrait, PaginatorTrait, DbErr, DbConn, Set};
use chrono::Utc;
use crate::modules::message::entity::chat_session;
use crate::modules::message::entity::chat_session::Entity as ChatSessionEntity;
use crate::modules::message::entity::chat_session_participant;
use crate::modules::message::entity::chat_session_participant::Entity as ChatSessionParticipantEntity;
use crate::modules::message::entity::chat_message;
use crate::modules::message::entity::chat_message::Entity as ChatMessageEntity;
use crate::modules::user::entity::user::Entity as UserEntity;
use crate::modules::user::entity::user;

fn deserialize_string_or_u64<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: serde_json::Value = serde::Deserialize::deserialize(deserializer)?;
    match value {
        serde_json::Value::Number(n) => n.as_i64().ok_or_else(|| D::Error::custom("invalid i64 number")),
        serde_json::Value::String(s) => s.parse::<i64>().map_err(|e| D::Error::custom(format!("failed to parse string to i64: {}", e))),
        _ => Err(D::Error::custom("expected string or i64")),
    }
}

pub const SESSION_TYPE_PRIVATE: i32 = 1;
pub const SESSION_TYPE_SYSTEM: i32 = 2;
pub const MESSAGE_TYPE_SYSTEM: i32 = 1;
pub const MESSAGE_TYPE_USER: i32 = 2;

#[derive(Debug, Deserialize)]
pub struct SendChatMessageRequest {
    pub session_id: Option<i64>,
    pub receiver_id: Option<i64>,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct GetSessionListParams {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct GetChatMessagesParams {
    pub session_id: i64,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct MarkReadRequest {
    #[serde(deserialize_with = "deserialize_string_or_u64")]
    pub session_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct DeleteSessionRequest {
    #[serde(deserialize_with = "deserialize_string_or_u64")]
    pub session_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct SearchUserParams {
    pub keyword: String,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ChatSessionDTO {
    pub session_id: i64,
    pub session_type: i32,
    pub session_name: String,
    pub avatar_url: Option<String>,
    pub last_message_id: Option<i64>,
    pub last_message_content: Option<String>,
    pub last_message_time: Option<String>,
    pub unread_count: i32,
    pub last_message_sender: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatMessageDTO {
    pub message_id: i64,
    pub session_id: i64,
    pub sender_id: i64,
    pub sender_nickname: String,
    pub sender_avatar: Option<String>,
    pub content: String,
    pub message_type: i32,
    pub is_recalled: bool,
    pub send_time: String,
    pub is_mine: bool,
}

#[derive(Debug, Serialize)]
pub struct SendMessageResponse {
    pub session_id: i64,
    pub message_id: i64,
}

#[derive(Debug, Serialize)]
pub struct UserSearchDTO {
    pub user_id: i64,
    pub nickname: String,
    pub avatar: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PageResponse<T> {
    pub list: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

pub struct ChatModel;

impl ChatModel {
    pub async fn get_or_create_private_session(
        db: &DbConn,
        user_id: i64,
        other_user_id: i64,
    ) -> Result<i64, DbErr> {
        let other_user = UserEntity::find_by_id(other_user_id).one(db).await?;
        let other = match other_user {
            Some(user) => user,
            None => {
                return Err(DbErr::RecordNotFound("用户不存在".to_string()));
            }
        };

        let participants = ChatSessionParticipantEntity::find()
            .filter(chat_session_participant::Column::UserId.eq(user_id))
            .all(db)
            .await?;

        for participant in participants {
            if let Some(session) = ChatSessionEntity::find_by_id(participant.session_id).one(db).await? {
                if session.session_type == SESSION_TYPE_PRIVATE {
                    let other_participant = ChatSessionParticipantEntity::find()
                        .filter(chat_session_participant::Column::SessionId.eq(session.id))
                        .filter(chat_session_participant::Column::UserId.eq(other_user_id))
                        .one(db)
                        .await?;
                    if other_participant.is_some() {
                        return Ok(session.id);
                    }
                }
            }
        }

        let session_model = chat_session::ActiveModel {
            session_type: Set(SESSION_TYPE_PRIVATE),
            session_name: Set(other.nickname.clone()),
            avatar_url: Set(other.avatar),
            member_count: Set(Some(2)),
            ..Default::default()
        };
        let result = ChatSessionEntity::insert(session_model).exec(db).await?;
        let session_id = result.last_insert_id;

        let participant1 = chat_session_participant::ActiveModel {
            session_id: Set(session_id),
            user_id: Set(user_id),
            unread_count: Set(Some(0)),
            ..Default::default()
        };
        ChatSessionParticipantEntity::insert(participant1).exec(db).await?;

        let participant2 = chat_session_participant::ActiveModel {
            session_id: Set(session_id),
            user_id: Set(other_user_id),
            unread_count: Set(Some(0)),
            ..Default::default()
        };
        ChatSessionParticipantEntity::insert(participant2).exec(db).await?;

        Ok(session_id)
    }

    pub async fn get_or_create_system_session(db: &DbConn) -> Result<i64, DbErr> {
        let sessions = ChatSessionEntity::find()
            .filter(chat_session::Column::SessionType.eq(SESSION_TYPE_SYSTEM))
            .all(db)
            .await?;

        if let Some(session) = sessions.into_iter().next() {
            return Ok(session.id);
        }

        let session_model = chat_session::ActiveModel {
            session_type: Set(SESSION_TYPE_SYSTEM),
            session_name: Set(Some("系统消息".to_string())),
            avatar_url: Set(Some("/static/images/system_avatar.png".to_string())),
            member_count: Set(Some(0)),
            ..Default::default()
        };
        let result = ChatSessionEntity::insert(session_model).exec(db).await?;
        Ok(result.last_insert_id)
    }

    pub async fn send_message(
        db: &DbConn,
        sender_id: i64,
        session_id: i64,
        content: String,
        message_type: i32,
    ) -> Result<SendMessageResponse, DbErr> {
        let sender = UserEntity::find_by_id(sender_id).one(db).await?;
        let (sender_nickname, sender_avatar) = match sender {
            Some(s) => (s.nickname.unwrap_or_else(|| "未知用户".to_string()), s.avatar),
            None => ("未知用户".to_string(), None),
        };

        let message_model = chat_message::ActiveModel {
            session_id: Set(session_id),
            sender_id: Set(sender_id),
            sender_nickname: Set(sender_nickname.clone()),
            sender_avatar: Set(sender_avatar.clone()),
            content: Set(content.clone()),
            message_type: Set(Some(message_type)),
            is_recalled: Set(Some(0)),
            send_time: Set(Some(Utc::now())),
            ..Default::default()
        };
        let result = ChatMessageEntity::insert(message_model).exec(db).await?;
        let message_id = result.last_insert_id;

        let content_preview = if content.len() > 50 {
            format!("{}...", &content[..50])
        } else {
            content.clone()
        };

        ChatSessionEntity::update_many()
            .col_expr(chat_session::Column::LastMessageId, sea_orm::sea_query::Expr::value(message_id))
            .col_expr(chat_session::Column::LastMessageContent, sea_orm::sea_query::Expr::value(content_preview))
            .col_expr(chat_session::Column::LastMessageTime, sea_orm::sea_query::Expr::value(Utc::now()))
            .filter(chat_session::Column::Id.eq(session_id))
            .exec(db)
            .await?;

        let session = ChatSessionEntity::find_by_id(session_id).one(db).await?;
        if let Some(s) = session {
            if s.session_type != SESSION_TYPE_SYSTEM {
                let participants = ChatSessionParticipantEntity::find()
                    .filter(chat_session_participant::Column::SessionId.eq(session_id))
                    .filter(chat_session_participant::Column::UserId.ne(sender_id))
                    .all(db)
                    .await?;

                for p in participants {
                    let new_unread = p.unread_count.unwrap_or(0) + 1;
                    let _ = ChatSessionParticipantEntity::update_many()
                        .col_expr(chat_session_participant::Column::UnreadCount, sea_orm::sea_query::Expr::value(new_unread))
                        .filter(chat_session_participant::Column::Id.eq(p.id))
                        .exec(db)
                        .await;
                }
            }
        }

        Ok(SendMessageResponse {
            session_id,
            message_id,
        })
    }

    pub async fn get_session_list(
        db: &DbConn,
        user_id: i64,
        page: i32,
        page_size: i32,
    ) -> Result<PageResponse<ChatSessionDTO>, DbErr> {
        let offset = (page - 1) * page_size;

        let participants = ChatSessionParticipantEntity::find()
            .filter(chat_session_participant::Column::UserId.eq(user_id))
            .filter(chat_session_participant::Column::Deleted.eq(0))
            .order_by_desc(chat_session_participant::Column::UpdateTime)
            .offset(offset as u64)
            .limit(page_size as u64)
            .all(db)
            .await?;

        let mut sessions: Vec<ChatSessionDTO> = Vec::new();
        for participant in participants {
            if let Some(session) = ChatSessionEntity::find_by_id(participant.session_id).one(db).await? {
                let last_sender_nickname = if let Some(last_msg_id) = session.last_message_id {
                    ChatMessageEntity::find_by_id(last_msg_id)
                        .one(db)
                        .await?
                        .map(|m| m.sender_nickname)
                } else {
                    None
                };

                let unread_count = if session.session_type == SESSION_TYPE_SYSTEM {
                    let total_messages = ChatMessageEntity::find()
                        .filter(chat_message::Column::SessionId.eq(session.id))
                        .filter(chat_message::Column::IsRecalled.eq(0))
                        .filter(chat_message::Column::SenderId.ne(user_id))
                        .count(db)
                        .await? as i32;
                    std::cmp::max(total_messages - participant.unread_count.unwrap_or(0), 0)
                } else {
                    participant.unread_count.unwrap_or(0)
                };

                sessions.push(ChatSessionDTO {
                    session_id: session.id,
                    session_type: session.session_type,
                    session_name: session.session_name.unwrap_or_else(|| "未知会话".to_string()),
                    avatar_url: session.avatar_url,
                    last_message_id: session.last_message_id,
                    last_message_content: session.last_message_content,
                    last_message_time: session.last_message_time.map(|t| t.to_rfc3339()),
                    unread_count,
                    last_message_sender: last_sender_nickname,
                });
            }
        }

        let total = ChatSessionParticipantEntity::find()
            .filter(chat_session_participant::Column::UserId.eq(user_id))
            .filter(chat_session_participant::Column::Deleted.eq(0))
            .count(db).await? as i64;

        sessions.sort_by(|a, b| {
            let time_a = a.last_message_time.clone().unwrap_or_default();
            let time_b = b.last_message_time.clone().unwrap_or_default();
            time_b.cmp(&time_a)
        });

        Ok(PageResponse {
            list: sessions,
            total,
            page,
            page_size,
        })
    }

    pub async fn get_chat_messages(
        db: &DbConn,
        user_id: i64,
        session_id: i64,
        page: i32,
        page_size: i32,
    ) -> Result<PageResponse<ChatMessageDTO>, DbErr> {
        let offset = (page - 1) * page_size;

        let session = ChatSessionEntity::find_by_id(session_id).one(db).await?;
        let session_info = match session {
            Some(s) => s,
            None => {
                return Err(DbErr::RecordNotFound("会话不存在".to_string()));
            }
        };

        let participant = ChatSessionParticipantEntity::find()
            .filter(chat_session_participant::Column::SessionId.eq(session_id))
            .filter(chat_session_participant::Column::UserId.eq(user_id))
            .filter(chat_session_participant::Column::Deleted.eq(0))
            .one(db)
            .await?;

        if participant.is_none() {
            if session_info.session_type == SESSION_TYPE_SYSTEM {
                let total_messages = ChatMessageEntity::find()
                    .filter(chat_message::Column::SessionId.eq(session_id))
                    .filter(chat_message::Column::IsRecalled.eq(0))
                    .count(db)
                    .await? as i32;

                let new_participant = chat_session_participant::ActiveModel {
                    session_id: Set(session_id),
                    user_id: Set(user_id),
                    unread_count: Set(Some(total_messages)),
                    ..Default::default()
                };
                ChatSessionParticipantEntity::insert(new_participant).exec(db).await?;
            } else {
                return Err(DbErr::RecordNotFound("会话不存在或已删除".to_string()));
            }
        }

        let messages = ChatMessageEntity::find()
            .filter(chat_message::Column::SessionId.eq(session_id))
            .filter(chat_message::Column::IsRecalled.eq(0))
            .order_by_desc(chat_message::Column::SendTime)
            .offset(offset as u64)
            .limit(page_size as u64)
            .all(db)
            .await?;

        let result: Vec<ChatMessageDTO> = messages
            .into_iter()
            .map(|m| ChatMessageDTO {
                message_id: m.id,
                session_id: m.session_id,
                sender_id: m.sender_id,
                sender_nickname: m.sender_nickname,
                sender_avatar: m.sender_avatar,
                content: m.content,
                message_type: m.message_type.unwrap_or(MESSAGE_TYPE_USER),
                is_recalled: m.is_recalled.unwrap_or(0) == 1,
                send_time: m.send_time.map(|t| t.to_rfc3339()).unwrap_or_default(),
                is_mine: m.sender_id == user_id,
            })
            .collect();

        let total = ChatMessageEntity::find()
            .filter(chat_message::Column::SessionId.eq(session_id))
            .filter(chat_message::Column::IsRecalled.eq(0))
            .count(db).await? as i64;

        Ok(PageResponse {
            list: result,
            total,
            page,
            page_size,
        })
    }

    pub async fn mark_session_read(
        db: &DbConn,
        user_id: i64,
        session_id: i64,
    ) -> Result<(), DbErr> {
        let session = ChatSessionEntity::find_by_id(session_id).one(db).await?;
        if session.is_none() {
            return Ok(());
        }

        let session_info = session.unwrap();

        if session_info.session_type == SESSION_TYPE_SYSTEM {
            let total_messages = ChatMessageEntity::find()
                .filter(chat_message::Column::SessionId.eq(session_id))
                .filter(chat_message::Column::IsRecalled.eq(0))
                .count(db)
                .await? as i32;

            let existing_participant = ChatSessionParticipantEntity::find()
                .filter(chat_session_participant::Column::SessionId.eq(session_id))
                .filter(chat_session_participant::Column::UserId.eq(user_id))
                .filter(chat_session_participant::Column::Deleted.eq(0))
                .one(db)
                .await?;

            if existing_participant.is_some() {
                ChatSessionParticipantEntity::update_many()
                    .col_expr(chat_session_participant::Column::UnreadCount, sea_orm::sea_query::Expr::value(total_messages))
                    .filter(chat_session_participant::Column::SessionId.eq(session_id))
                    .filter(chat_session_participant::Column::UserId.eq(user_id))
                    .exec(db)
                    .await?;
            } else {
                let new_participant = chat_session_participant::ActiveModel {
                    session_id: Set(session_id),
                    user_id: Set(user_id),
                    unread_count: Set(Some(total_messages)),
                    ..Default::default()
                };
                ChatSessionParticipantEntity::insert(new_participant).exec(db).await?;
            }
        } else {
            ChatSessionParticipantEntity::update_many()
                .col_expr(chat_session_participant::Column::UnreadCount, sea_orm::sea_query::Expr::value(0))
                .filter(chat_session_participant::Column::SessionId.eq(session_id))
                .filter(chat_session_participant::Column::UserId.eq(user_id))
                .exec(db)
                .await?;
        }

        Ok(())
    }

    pub async fn delete_session(
        db: &DbConn,
        user_id: i64,
        session_id: i64,
    ) -> Result<(), DbErr> {
        ChatSessionParticipantEntity::update_many()
            .col_expr(chat_session_participant::Column::Deleted, sea_orm::sea_query::Expr::value(1))
            .filter(chat_session_participant::Column::SessionId.eq(session_id))
            .filter(chat_session_participant::Column::UserId.eq(user_id))
            .exec(db)
            .await?;

        Ok(())
    }

    pub async fn search_users_by_nickname(
        db: &DbConn,
        keyword: String,
        page: i32,
        page_size: i32,
    ) -> Result<Vec<UserSearchDTO>, DbErr> {
        let offset = (page - 1) * page_size;

        let users = UserEntity::find()
            .filter(user::Column::Nickname.like(format!("%{}%", keyword)))
            .offset(offset as u64)
            .limit(page_size as u64)
            .all(db)
            .await?;

        let result: Vec<UserSearchDTO> = users
            .into_iter()
            .map(|u| UserSearchDTO {
                user_id: u.id,
                nickname: u.nickname.unwrap_or_default(),
                avatar: u.avatar,
            })
            .collect();

        Ok(result)
    }

    pub async fn get_total_unread_count(db: &DbConn, user_id: i64) -> Result<i32, DbErr> {
        let participants = ChatSessionParticipantEntity::find()
            .filter(chat_session_participant::Column::UserId.eq(user_id))
            .filter(chat_session_participant::Column::Deleted.eq(0))
            .all(db)
            .await?;

        let total: i32 = participants
            .into_iter()
            .map(|p| p.unread_count.unwrap_or(0))
            .sum();

        Ok(total)
    }
}
