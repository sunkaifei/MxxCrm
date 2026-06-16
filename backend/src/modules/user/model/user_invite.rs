//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::kit::global::Serialize;
use crate::modules::user::entity::{user_invite, user_invite::Entity as UserInvite};
use crate::utils::string_utils::{serialize_option_u64_to_string, u64_to_string};
use sea_orm::prelude::DateTime;
use sea_orm::*;
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Validate, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserInviteSaveRequest {
    pub to_user_id: Option<i64>,
    pub form_user_id: Option<i64>,
    pub status: Option<i32>,
}

impl From<UserInviteSaveRequest> for UserInviteSaveDTO {
    fn from(req: UserInviteSaveRequest) -> Self {
        UserInviteSaveDTO {
            id: None,
            to_user_id: req.to_user_id,
            form_user_id: req.form_user_id,
            status: req.status,
            create_time: None,
        }
    }
}

#[derive(Debug, Validate, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserInviteUpdateRequest {
    pub id: Option<i64>,
    pub to_user_id: Option<i64>,
    pub form_user_id: Option<i64>,
    pub status: Option<i32>,
}

impl From<UserInviteUpdateRequest> for UserInviteSaveDTO {
    fn from(req: UserInviteUpdateRequest) -> Self {
        UserInviteSaveDTO {
            id: req.id,
            to_user_id: req.to_user_id,
            form_user_id: req.form_user_id,
            status: req.status,
            create_time: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInviteSaveDTO {
    pub id: Option<i64>,
    pub to_user_id: Option<i64>,
    pub form_user_id: Option<i64>,
    pub status: Option<i32>,
    pub create_time: Option<DateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserInviteDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub to_user_id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub form_user_id: Option<i64>,
    pub status: Option<i32>,
    pub create_time: Option<String>,
}

impl From<user_invite::Model> for UserInviteDetailVO {
    fn from(model: user_invite::Model) -> Self {
        Self {
            id: Option::from(model.id),
            to_user_id: Option::from(model.to_user_id),
            form_user_id: Option::from(model.form_user_id),
            status: Option::from(model.status),
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserInviteListVO {
    #[serde(serialize_with = "u64_to_string")]
    pub id: i64,
    #[serde(serialize_with = "u64_to_string")]
    pub to_user_id: i64,
    #[serde(serialize_with = "u64_to_string")]
    pub form_user_id: i64,
    pub status: Option<i32>,
    pub create_time: Option<String>,
}

impl From<user_invite::Model> for UserInviteListVO {
    fn from(model: user_invite::Model) -> Self {
        Self {
            id: model.id,
            to_user_id: model.to_user_id,
            form_user_id: model.form_user_id,
            status: model.status,
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub to_user_id: Option<i64>,
    pub form_user_id: Option<i64>,
    pub status: Option<i32>,
}

#[derive(Clone)]
pub struct PageWhere {
    pub to_user_id: Option<i64>,
    pub form_user_id: Option<i64>,
    pub status: Option<i32>,
}

impl PageWhere {
    pub fn format(&self) -> Self {
        let mut to_user_id = None;
        if self.to_user_id != Some(0) {
            to_user_id = self.to_user_id;
        }

        let mut form_user_id = None;
        if self.form_user_id != Some(0) {
            form_user_id = self.form_user_id;
        }

        let mut status = None;
        if self.status != Some(0) {
            status = self.status;
        }

        Self {
            to_user_id,
            form_user_id,
            status,
        }
    }
}

pub struct UserInviteModel;

impl UserInviteModel {
    pub async fn insert(db: &DbConn, form_data: &UserInviteSaveDTO) -> Result<i64, DbErr> {
        let payload = user_invite::ActiveModel {
            to_user_id: Set(form_data.to_user_id.unwrap_or_default()),
            form_user_id: Set(form_data.form_user_id.unwrap_or_default()),
            status: Set(form_data.status),
            create_time: Set(Option::from(chrono::Local::now().naive_local())),
            ..Default::default()
        };

        UserInvite::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    pub async fn batch_delete_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64, DbErr> {
        Ok(UserInvite::delete_many()
            .filter(user_invite::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await?.rows_affected as i64)
    }

    pub async fn update_by_id(
        db: &DbConn,
        id: i64,
        form_data: &UserInviteSaveDTO,
    ) -> Result<i64, DbErr> {
        let payload = user_invite::ActiveModel {
            id: Set(id),
            to_user_id: Set(form_data.to_user_id.unwrap_or_default()),
            form_user_id: Set(form_data.form_user_id.unwrap_or_default()),
            status: Set(form_data.status),
            ..Default::default()
        };

        UserInvite::update_many()
            .set(payload)
            .filter(user_invite::Column::Id.eq(id))
            .exec(db)
            .await
            .map(|result| result.rows_affected as i64)
    }

    pub async fn find_by_id(db: &DbConn, id: &i64) -> Result<Option<user_invite::Model>, DbErr> {
        UserInvite::find_by_id(id.clone())
            .one(db)
            .await
    }

    pub async fn find_by_to_user_id(db: &DbConn, to_user_id: &i64) -> Result<Vec<user_invite::Model>, DbErr> {
        UserInvite::find()
            .filter(user_invite::Column::ToUserId.eq(to_user_id.clone()))
            .all(db)
            .await
    }

    pub async fn find_by_form_user_id(db: &DbConn, form_user_id: &i64) -> Result<Vec<user_invite::Model>, DbErr> {
        UserInvite::find()
            .filter(user_invite::Column::FormUserId.eq(form_user_id.clone()))
            .all(db)
            .await
    }

    pub async fn select_count(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<i64, DbErr> {
        UserInvite::find()
            .apply_if(wheres.to_user_id, |query, v| {
                query.filter(user_invite::Column::ToUserId.eq(v))
            })
            .apply_if(wheres.form_user_id, |query, v| {
                query.filter(user_invite::Column::FormUserId.eq(v))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(user_invite::Column::Status.eq(v))
            })
            .count(db)
            .await
            .map(|c| c as i64)
    }

    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        wheres: PageWhere,
    ) -> Result<(Vec<user_invite::Model>, i64), DbErr> {
        let paginator = UserInvite::find()
            .apply_if(wheres.to_user_id, |query, v| {
                query.filter(user_invite::Column::ToUserId.eq(v))
            })
            .apply_if(wheres.form_user_id, |query, v| {
                query.filter(user_invite::Column::FormUserId.eq(v))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(user_invite::Column::Status.eq(v))
            })
            .order_by_desc(user_invite::Column::Id)
            .paginate(db, per_page as u64);

        let total = paginator.num_items().await? as i64;
        let items = paginator.fetch_page(page.saturating_sub(1) as u64).await?;

        Ok((items, total))
    }
}
