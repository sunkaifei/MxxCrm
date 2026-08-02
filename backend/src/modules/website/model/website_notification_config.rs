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
use sea_orm::prelude::DateTime;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::core::web::response::ResultPage;
use crate::modules::website::entity::{
    website_notification_config, website_notification_config::Entity as WebsiteNotificationConfig,
};

// ==================== DTO ====================

/// 通知配置列表查询
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct NotificationConfigListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub website_id: Option<i64>,
    pub scene_code: Option<String>,
    pub enabled: Option<i32>,
}

/// 通知配置新增/编辑
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct NotificationConfigSaveDTO {
    pub id: Option<i64>,
    pub website_id: Option<i64>,
    pub scene_code: String,
    pub scene_name: Option<String>,
    pub channels: Option<String>,
    pub recipient_emails: Option<String>,
    pub email_subject: Option<String>,
    pub email_body: Option<String>,
    pub enabled: Option<i32>,
}

// ==================== VO ====================

/// 通知配置详情VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct NotificationConfigVO {
    pub id: Option<i64>,
    pub website_id: Option<i64>,
    pub scene_code: Option<String>,
    pub scene_name: Option<String>,
    pub channels: Option<String>,
    pub recipient_emails: Option<String>,
    pub email_subject: Option<String>,
    pub email_body: Option<String>,
    pub enabled: Option<i32>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

impl From<website_notification_config::Model> for NotificationConfigVO {
    fn from(item: website_notification_config::Model) -> Self {
        NotificationConfigVO {
            id: Option::from(item.id),
            website_id: item.website_id,
            scene_code: Some(item.scene_code),
            scene_name: item.scene_name,
            channels: item.channels,
            recipient_emails: item.recipient_emails,
            email_subject: item.email_subject,
            email_body: item.email_body,
            enabled: item.enabled,
            create_time: item.create_time,
            update_time: item.update_time,
        }
    }
}

// ==================== Model ====================

/// 网站通知配置数据模型操作类
pub struct NotificationConfigModel;

impl NotificationConfigModel {
    /// 分页查询
    pub async fn find_by_page(
        db: &DbConn,
        query: &NotificationConfigListQuery,
    ) -> Result<ResultPage<Vec<NotificationConfigVO>>, DbErr> {
        let page = std::cmp::max(query.page.unwrap_or(1), 1);
        let page_size = std::cmp::max(std::cmp::min(query.page_size.unwrap_or(10), 100), 1);

        let mut q = WebsiteNotificationConfig::find()
            .filter(website_notification_config::Column::Deleted.eq(0));
        if let Some(wid) = query.website_id {
            q = q.filter(website_notification_config::Column::WebsiteId.eq(wid));
        }
        if let Some(sc) = &query.scene_code {
            q = q.filter(website_notification_config::Column::SceneCode.like(format!("%{}%", sc)));
        }
        if let Some(e) = query.enabled {
            q = q.filter(website_notification_config::Column::Enabled.eq(e));
        }

        let paginator = q
            .order_by_desc(website_notification_config::Column::CreateTime)
            .paginate(db, page_size as u64);
        let total = paginator.num_items().await? as i64;
        let rows = paginator.fetch_page((page - 1) as u64).await?;
        let list_vo: Vec<NotificationConfigVO> = rows.into_iter().map(|m| m.into()).collect();
        Ok(ResultPage::new(list_vo, total, page, page_size))
    }

    /// 根据ID查询
    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<NotificationConfigVO>, DbErr> {
        let row = WebsiteNotificationConfig::find_by_id(id)
            .filter(website_notification_config::Column::Deleted.eq(0))
            .one(db)
            .await?;
        Ok(row.map(|m| m.into()))
    }

    /// 根据网站ID和场景编码查询（触发器查询用）
    pub async fn find_by_website_and_scene<C: ConnectionTrait>(
        db: &C,
        website_id: i64,
        scene_code: &str,
    ) -> Result<Option<website_notification_config::Model>, DbErr> {
        WebsiteNotificationConfig::find()
            .filter(website_notification_config::Column::WebsiteId.eq(website_id))
            .filter(website_notification_config::Column::SceneCode.eq(scene_code))
            .filter(website_notification_config::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 根据网站ID查询全部通知配置（单站设置页用）
    ///
    /// 返回该站点下所有场景的通知配置列表（已按 sort/create_time 升序），
    /// 供前端"网站设置 → 通知配置"Tab 一次性加载并批量编辑。
    pub async fn find_all_by_website(db: &DbConn, website_id: i64) -> Result<Vec<NotificationConfigVO>, DbErr> {
        let rows = WebsiteNotificationConfig::find()
            .filter(website_notification_config::Column::WebsiteId.eq(website_id))
            .filter(website_notification_config::Column::Deleted.eq(0))
            .order_by_asc(website_notification_config::Column::Id)
            .all(db)
            .await?;
        Ok(rows.into_iter().map(|m| m.into()).collect())
    }

    /// 新增
    pub async fn insert<C: ConnectionTrait>(
        db: &C,
        req: &NotificationConfigSaveDTO,
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = website_notification_config::ActiveModel {
            website_id: Set(req.website_id),
            scene_code: Set(req.scene_code.clone()),
            scene_name: Set(req.scene_name.clone()),
            channels: Set(req.channels.clone().or_else(|| Some("email".to_string()))),
            recipient_emails: Set(req.recipient_emails.clone()),
            email_subject: Set(req.email_subject.clone()),
            email_body: Set(req.email_body.clone()),
            enabled: Set(Some(req.enabled.unwrap_or(0))),
            create_time: Set(Some(now.clone())),
            update_time: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        WebsiteNotificationConfig::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 更新
    pub async fn update<C: ConnectionTrait>(
        db: &C,
        id: i64,
        req: &NotificationConfigSaveDTO,
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = website_notification_config::ActiveModel {
            website_id: Set(req.website_id),
            scene_code: Set(req.scene_code.clone()),
            scene_name: Set(req.scene_name.clone()),
            channels: Set(req.channels.clone()),
            recipient_emails: Set(req.recipient_emails.clone()),
            email_subject: Set(req.email_subject.clone()),
            email_body: Set(req.email_body.clone()),
            enabled: Set(req.enabled),
            update_time: Set(Some(now)),
            ..Default::default()
        };
        let result: UpdateResult = WebsiteNotificationConfig::update_many()
            .set(payload)
            .filter(website_notification_config::Column::Id.eq(id))
            .filter(website_notification_config::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 更新启用状态
    pub async fn update_enabled<C: ConnectionTrait>(
        db: &C,
        id: i64,
        enabled: i32,
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let result: UpdateResult = WebsiteNotificationConfig::update_many()
            .col_expr(
                website_notification_config::Column::Enabled,
                sea_orm::sea_query::Expr::value(enabled),
            )
            .col_expr(
                website_notification_config::Column::UpdateTime,
                sea_orm::sea_query::Expr::value(now),
            )
            .filter(website_notification_config::Column::Id.eq(id))
            .filter(website_notification_config::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 批量软删除
    pub async fn batch_delete<C: ConnectionTrait>(db: &C, ids: Vec<i64>) -> Result<i64, DbErr> {
        if ids.is_empty() {
            return Ok(0);
        }
        let now = chrono::Local::now().naive_local().to_owned();
        let result: UpdateResult = WebsiteNotificationConfig::update_many()
            .col_expr(
                website_notification_config::Column::Deleted,
                sea_orm::sea_query::Expr::value(1),
            )
            .col_expr(
                website_notification_config::Column::UpdateTime,
                sea_orm::sea_query::Expr::value(now),
            )
            .filter(website_notification_config::Column::Id.is_in(ids))
            .filter(website_notification_config::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }
}
