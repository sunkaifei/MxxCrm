//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::base_controller::get_user;
use crate::core::web::response::MetaResp;
use crate::modules::message::model::notification::{NotificationListQuery, NotificationDTO, PageResponse, SendNotificationRequest};
use crate::modules::message::service::notification_service::NotificationService;
use actix_web::{web, HttpRequest, HttpResponse};
use sea_orm::DbErr;
use sea_orm::{EntityTrait, QuerySelect, QueryFilter, QueryOrder, ColumnTrait, PaginatorTrait};
use crate::modules::message::entity::system_notification;
use crate::modules::message::entity::system_notification::Entity as SystemNotificationEntity;

pub async fn send_notification(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<SendNotificationRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let admin_token: JWTToken = get_user(&req).unwrap_or_default();
    let sender_id = admin_token.id.unwrap_or(0);

    let receiver_ids = if let Some(ids) = &item.receiver_ids {
        ids.clone()
    } else if let Some(rid) = item.receiver_id {
        vec![rid]
    } else {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "接收人不能为空", "local")));
    };

    let result = crate::modules::message::service::notification_service::NotificationService::send_notification(
        db,
        Some(sender_id),
        receiver_ids,
        item.into_inner(),
    ).await;

    match result {
        Ok(count) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(serde_json::json!({"success": true, "count": count}), "local")))
        }
        Err(e) => {
            let msg = match e {
                crate::modules::message::service::notification_service::NotificationServiceError::InvalidParameter(m) => m,
                _ => "发送通知失败".to_string(),
            };
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &msg, "local")))
        }
    }
}

pub async fn get_notification_list(
    state: web::Data<AppState>,
    query: web::Query<NotificationListQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;

    let notifications = get_admin_notification_list(db, query.into_inner()).await;

    match notifications {
        Ok(page_data) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local")))
        }
        Err(_) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "获取通知列表失败", "local")))
        }
    }
}

async fn get_admin_notification_list(
    db: &sea_orm::DbConn,
    query: NotificationListQuery,
) -> std::result::Result<PageResponse<NotificationDTO>, DbErr> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let offset = (page - 1) * page_size;

    let mut select = SystemNotificationEntity::find();

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

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/notification")
            .route("/send", web::post().to(send_notification))
            .route("/list", web::get().to(get_notification_list)),
    );
}
