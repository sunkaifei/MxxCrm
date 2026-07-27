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
use sea_orm::{EntityTrait, QuerySelect, QueryFilter, QueryOrder, ColumnTrait, PaginatorTrait, DbErr, DbConn, Set, Condition};
use chrono::Utc;
use crate::modules::message::entity::chat_session;
use crate::modules::message::entity::chat_session::Entity as ChatSessionEntity;
use crate::modules::message::entity::chat_session_participant;
use crate::modules::message::entity::chat_session_participant::Entity as ChatSessionParticipantEntity;
use crate::modules::message::entity::chat_message;
use crate::modules::message::entity::chat_message::Entity as ChatMessageEntity;
use crate::modules::system::entity::admin::Entity as UserEntity;
use crate::modules::system::entity::admin;

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
#[serde(rename_all = "camelCase")]
pub struct GetSessionListParams {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetChatMessagesParams {
    pub session_id: i64,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkReadRequest {
    #[serde(deserialize_with = "deserialize_string_or_u64")]
    pub session_id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSessionRequest {
    #[serde(deserialize_with = "deserialize_string_or_u64")]
    pub session_id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchUserParams {
    pub keyword: String,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColleagueListParams {
    pub keyword: Option<String>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartSessionRequest {
    #[serde(deserialize_with = "deserialize_string_or_u64")]
    pub receiver_id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendChatMessageRequest {
    pub session_id: Option<i64>,
    pub receiver_id: Option<i64>,
    pub content: String,
    pub content_type: Option<i32>,
    pub file_url: Option<String>,
    pub file_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct ChatMessageDTO {
    pub message_id: i64,
    pub session_id: i64,
    pub sender_id: i64,
    pub sender_nickname: String,
    pub sender_avatar: Option<String>,
    pub content: String,
    pub message_type: i32,
    pub content_type: Option<i32>,
    pub file_url: Option<String>,
    pub file_name: Option<String>,
    pub file_size: Option<i64>,
    pub is_recalled: bool,
    pub send_time: String,
    pub is_mine: bool,
    /// 已读状态：0=未读，1=已读（仅对用户消息有意义）
    pub read_status: i32,
    /// 已读时间（RFC3339 字符串，未读时为空）
    pub read_time: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageResponse {
    pub session_id: i64,
    pub message_id: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSearchDTO {
    pub user_id: i64,
    pub nickname: String,
    pub avatar: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeptNameDTO {
    pub dept_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColleagueVO {
    pub id: i64,
    pub user_name: Option<String>,
    pub nick_name: Option<String>,
    pub avatar: Option<String>,
    pub depts: Option<Vec<DeptNameDTO>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDetailDTO {
    pub session_id: i64,
    pub session_type: i32,
    pub session_name: String,
    pub avatar_url: Option<String>,
    pub other_user_id: Option<i64>,
    pub other_nickname: Option<String>,
    pub other_avatar: Option<String>,
    pub online_status: Option<i32>,
    pub is_pinned: bool,
    pub is_muted: bool,
    pub unread_count: i32,
}

#[derive(Debug, Deserialize)]
pub struct RecallMessageRequest {
    pub message_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct TogglePinRequest {
    pub session_id: i64,
    pub is_pinned: bool,
}

#[derive(Debug, Deserialize)]
pub struct ToggleMuteRequest {
    pub session_id: i64,
    pub is_muted: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
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
            session_name: Set(other.nick_name.clone()),
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
        content_type: Option<i32>,
        file_url: Option<String>,
        file_name: Option<String>,
    ) -> Result<SendMessageResponse, DbErr> {
        let sender = UserEntity::find_by_id(sender_id).one(db).await?;
        let (sender_nickname, sender_avatar) = match sender {
            Some(s) => (s.nick_name.unwrap_or_else(|| "未知用户".to_string()), s.avatar),
            None => ("未知用户".to_string(), None),
        };

        let message_model = chat_message::ActiveModel {
            session_id: Set(session_id),
            sender_id: Set(sender_id),
            sender_nickname: Set(sender_nickname.clone()),
            sender_avatar: Set(sender_avatar.clone()),
            content: Set(content.clone()),
            message_type: Set(Some(message_type)),
            content_type: Set(content_type),
            file_url: Set(file_url.clone()),
            file_name: Set(file_name.clone()),
            is_recalled: Set(Some(0)),
            send_time: Set(Some(Utc::now().naive_utc())),
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

                // 私聊会话：session_name 是创建者视角的"对方昵称"，对当前用户可能是错的
                // 这里根据当前 user_id 重新查询会话中"另一个参与者"的信息作为展示
                let (display_name, display_avatar) = if session.session_type == SESSION_TYPE_PRIVATE {
                    let other_participant = ChatSessionParticipantEntity::find()
                        .filter(chat_session_participant::Column::SessionId.eq(session.id))
                        .filter(chat_session_participant::Column::UserId.ne(user_id))
                        .filter(chat_session_participant::Column::Deleted.eq(0))
                        .one(db)
                        .await?;
                    if let Some(op) = other_participant {
                        if let Some(other_user) = UserEntity::find_by_id(op.user_id).one(db).await? {
                            (
                                other_user.nick_name.unwrap_or_else(|| other_user.user_name.unwrap_or_else(|| "未知用户".to_string())),
                                other_user.avatar,
                            )
                        } else {
                            (session.session_name.clone().unwrap_or_else(|| "未知会话".to_string()), session.avatar_url.clone())
                        }
                    } else {
                        (session.session_name.clone().unwrap_or_else(|| "未知会话".to_string()), session.avatar_url.clone())
                    }
                } else {
                    (session.session_name.clone().unwrap_or_else(|| "未知会话".to_string()), session.avatar_url.clone())
                };

                sessions.push(ChatSessionDTO {
                    session_id: session.id,
                    session_type: session.session_type,
                    session_name: display_name,
                    avatar_url: display_avatar,
                    last_message_id: session.last_message_id,
                    last_message_content: session.last_message_content,
                    last_message_time: session.last_message_time.map(|t| t.and_utc().to_rfc3339()),
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
                content_type: m.content_type,
                file_url: m.file_url,
                file_name: m.file_name,
                file_size: m.file_size,
                is_recalled: m.is_recalled.unwrap_or(0) == 1,
                send_time: m.send_time.map(|t| t.and_utc().to_rfc3339()).unwrap_or_default(),
                is_mine: m.sender_id == user_id,
                read_status: m.read_status.unwrap_or(0),
                read_time: m.read_time.map(|t| t.and_utc().to_rfc3339()).unwrap_or_default(),
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
    ) -> Result<Vec<i64>, DbErr> {
        let session = ChatSessionEntity::find_by_id(session_id).one(db).await?;
        if session.is_none() {
            return Ok(Vec::new());
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
            return Ok(Vec::new());
        }

        // 私聊会话：查询当前用户最后已读的消息 ID，用于增量更新
        let participant = ChatSessionParticipantEntity::find()
            .filter(chat_session_participant::Column::SessionId.eq(session_id))
            .filter(chat_session_participant::Column::UserId.eq(user_id))
            .filter(chat_session_participant::Column::Deleted.eq(0))
            .one(db)
            .await?;

        let prev_last_read_id = participant
            .as_ref()
            .and_then(|p| p.last_read_message_id)
            .unwrap_or(0);

        // 找出当前用户尚未读的、对方发来的消息（id 大于上次已读位置，且不是自己发的）
        let newly_read_messages: Vec<chat_message::Model> = ChatMessageEntity::find()
            .filter(chat_message::Column::SessionId.eq(session_id))
            .filter(chat_message::Column::IsRecalled.eq(0))
            .filter(chat_message::Column::SenderId.ne(user_id))
            .filter(chat_message::Column::ReadStatus.eq(0))
            .filter(chat_message::Column::Id.gt(prev_last_read_id))
            .all(db)
            .await?;

        let newly_read_ids: Vec<i64> = newly_read_messages.iter().map(|m| m.id).collect();

        // 更新这些消息为已读状态
        if !newly_read_ids.is_empty() {
            let now = Utc::now().naive_utc();
            ChatMessageEntity::update_many()
                .col_expr(chat_message::Column::ReadStatus, sea_orm::sea_query::Expr::value(1))
                .col_expr(chat_message::Column::ReadTime, sea_orm::sea_query::Expr::value(now))
                .filter(chat_message::Column::Id.is_in(newly_read_ids.clone()))
                .exec(db)
                .await?;
        }

        // 取最大消息 ID 作为 last_read_message_id（若没有新已读消息，保留原值）
        let max_id = newly_read_ids.iter().copied().max().unwrap_or(prev_last_read_id);

        // 更新参与者的未读数和最后已读消息 ID
        if participant.is_some() {
            ChatSessionParticipantEntity::update_many()
                .col_expr(chat_session_participant::Column::UnreadCount, sea_orm::sea_query::Expr::value(0))
                .col_expr(chat_session_participant::Column::LastReadMessageId, sea_orm::sea_query::Expr::value(max_id))
                .filter(chat_session_participant::Column::SessionId.eq(session_id))
                .filter(chat_session_participant::Column::UserId.eq(user_id))
                .exec(db)
                .await?;
        }

        Ok(newly_read_ids)
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
            .filter(admin::Column::NickName.like(format!("%{}%", keyword)))
            .offset(offset as u64)
            .limit(page_size as u64)
            .all(db)
            .await?;

        let result: Vec<UserSearchDTO> = users
            .into_iter()
            .map(|u| UserSearchDTO {
                user_id: u.id,
                nickname: u.nick_name.unwrap_or_default(),
                avatar: u.avatar,
            })
            .collect();

        Ok(result)
    }

    pub async fn get_colleague_list(
        db: &DbConn,
        current_user_id: i64,
        keyword: Option<String>,
        page: i32,
        page_size: i32,
    ) -> Result<Vec<admin::Model>, DbErr> {
        let offset = (page - 1) * page_size;

        let mut query = UserEntity::find()
            .filter(admin::Column::Status.eq(1))
            .filter(admin::Column::Deleted.eq(0));

        if current_user_id > 0 {
            query = query.filter(admin::Column::Id.ne(current_user_id));
        }

        if let Some(kw) = keyword {
            if !kw.trim().is_empty() {
                query = query.filter(
                    Condition::any()
                        .add(admin::Column::NickName.like(format!("%{}%", kw)))
                        .add(admin::Column::UserName.like(format!("%{}%", kw))),
                );
            }
        }

        let users = query
            .order_by_asc(admin::Column::Id)
            .offset(offset as u64)
            .limit(page_size as u64)
            .all(db)
            .await?;

        Ok(users)
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

    pub async fn recall_message(
        db: &DbConn,
        user_id: i64,
        message_id: i64,
    ) -> Result<bool, DbErr> {
        let message = ChatMessageEntity::find_by_id(message_id).one(db).await?;
        let msg = match message {
            Some(m) => m,
            None => return Ok(false),
        };

        if msg.sender_id != user_id {
            return Ok(false);
        }

        if msg.is_recalled.unwrap_or(0) == 1 {
            return Ok(false);
        }

        let send_time = msg.send_time.unwrap_or_else(|| Utc::now().naive_utc());
        let now = Utc::now().naive_utc();
        let duration = now.signed_duration_since(send_time);
        if duration.num_minutes() > 2 {
            return Ok(false);
        }

        let result = ChatMessageEntity::update_many()
            .col_expr(chat_message::Column::IsRecalled, sea_orm::sea_query::Expr::value(1))
            .filter(chat_message::Column::Id.eq(message_id))
            .exec(db)
            .await?;

        Ok(result.rows_affected > 0)
    }

    pub async fn toggle_pin(
        db: &DbConn,
        user_id: i64,
        session_id: i64,
        is_pinned: bool,
    ) -> Result<(), DbErr> {
        let pin_val = if is_pinned { 1 } else { 0 };
        ChatSessionParticipantEntity::update_many()
            .col_expr(chat_session_participant::Column::IsPinned, sea_orm::sea_query::Expr::value(pin_val))
            .filter(chat_session_participant::Column::SessionId.eq(session_id))
            .filter(chat_session_participant::Column::UserId.eq(user_id))
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn toggle_mute(
        db: &DbConn,
        user_id: i64,
        session_id: i64,
        is_muted: bool,
    ) -> Result<(), DbErr> {
        let mute_val = if is_muted { 1 } else { 0 };
        ChatSessionParticipantEntity::update_many()
            .col_expr(chat_session_participant::Column::IsMuted, sea_orm::sea_query::Expr::value(mute_val))
            .filter(chat_session_participant::Column::SessionId.eq(session_id))
            .filter(chat_session_participant::Column::UserId.eq(user_id))
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn get_session_detail(
        db: &DbConn,
        user_id: i64,
        session_id: i64,
    ) -> Result<SessionDetailDTO, DbErr> {
        let session = ChatSessionEntity::find_by_id(session_id).one(db).await?;
        let sess = match session {
            Some(s) => s,
            None => return Err(DbErr::RecordNotFound("会话不存在".to_string())),
        };

        let participant = ChatSessionParticipantEntity::find()
            .filter(chat_session_participant::Column::SessionId.eq(session_id))
            .filter(chat_session_participant::Column::UserId.eq(user_id))
            .filter(chat_session_participant::Column::Deleted.eq(0))
            .one(db)
            .await?;

        let part = match participant {
            Some(p) => p,
            None => return Err(DbErr::RecordNotFound("会话不存在或已删除".to_string())),
        };

        let mut other_user_id: Option<i64> = None;
        let mut other_nickname: Option<String> = None;
        let mut other_avatar: Option<String> = None;

        if sess.session_type == SESSION_TYPE_PRIVATE {
            let other_participant = ChatSessionParticipantEntity::find()
                .filter(chat_session_participant::Column::SessionId.eq(session_id))
                .filter(chat_session_participant::Column::UserId.ne(user_id))
                .one(db)
                .await?;

            if let Some(op) = other_participant {
                other_user_id = Some(op.user_id);
                let user = UserEntity::find_by_id(op.user_id).one(db).await?;
                if let Some(u) = user {
                    other_nickname = u.nick_name.clone();
                    other_avatar = u.avatar.clone();
                }
            }
        }

        Ok(SessionDetailDTO {
            session_id: sess.id,
            session_type: sess.session_type,
            session_name: sess.session_name.unwrap_or_else(|| "未知会话".to_string()),
            avatar_url: sess.avatar_url.clone(),
            other_user_id,
            other_nickname,
            other_avatar,
            online_status: None,
            is_pinned: part.is_pinned.unwrap_or(0) == 1,
            is_muted: part.is_muted.unwrap_or(0) == 1,
            unread_count: part.unread_count.unwrap_or(0),
        })
    }
}
