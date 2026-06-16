//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::shop::entity::shop_category::{self, Entity as CategoryEntity};
use crate::utils::string_utils::serialize_option_u64_to_string;
use sea_orm::prelude::DateTime;
use sea_orm::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CategoryRequest {
    pub parent_id: Option<i64>,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub sort_order: Option<i32>,
    pub is_show: Option<i16>,
    pub level: Option<i16>,
}

pub struct CategoryDTO {
    pub parent_id: i64,
    pub name: String,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub is_show: i16,
    pub level: i16,
}

impl From<CategoryRequest> for CategoryDTO {
    fn from(req: CategoryRequest) -> Self {
        CategoryDTO {
            parent_id: req.parent_id.unwrap_or(0),
            name: req.name.unwrap_or_default(),
            icon: req.icon,
            sort_order: req.sort_order.unwrap_or(0),
            is_show: req.is_show.unwrap_or(1),
            level: req.level.unwrap_or(1),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CategoryVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub parent_id: Option<i64>,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub sort_order: Option<i32>,
    pub is_show: Option<i16>,
    pub level: Option<i16>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
}

impl From<shop_category::Model> for CategoryVO {
    fn from(model: shop_category::Model) -> Self {
        Self {
            id: Some(model.id),
            parent_id: Some(model.parent_id),
            name: Some(model.name),
            icon: model.icon,
            sort_order: Some(model.sort_order),
            is_show: Some(model.is_show),
            level: Some(model.level),
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: model.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CategoryTreeVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub sort_order: Option<i32>,
    pub is_show: Option<i16>,
    pub level: Option<i16>,
    pub children: Option<Vec<CategoryTreeVO>>,
}

pub struct CategoryModel;

impl CategoryModel {
    pub async fn insert<C: ConnectionTrait>(db: &C, form: &CategoryDTO) -> Result<i64, DbErr> {
        let active = shop_category::ActiveModel {
            parent_id: Set(form.parent_id),
            name: Set(form.name.clone()),
            icon: Set(form.icon.clone()),
            sort_order: Set(form.sort_order),
            is_show: Set(form.is_show),
            level: Set(form.level),
            create_time: Set(Some(chrono::Local::now().naive_local())),
            update_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        CategoryEntity::insert(active).exec(db).await.map(|r| r.last_insert_id)
    }

    pub async fn update_by_id<C: ConnectionTrait>(db: &C, form: &CategoryDTO, id: i64) -> Result<i64, DbErr> {
        let active = shop_category::ActiveModel {
            id: Set(id),
            parent_id: Set(form.parent_id),
            name: Set(form.name.clone()),
            icon: Set(form.icon.clone()),
            sort_order: Set(form.sort_order),
            is_show: Set(form.is_show),
            level: Set(form.level),
            update_time: Set(Some(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        CategoryEntity::update_many()
            .set(active)
            .filter(shop_category::Column::Id.eq(id))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<shop_category::Model>, DbErr> {
        CategoryEntity::find_by_id(id).one(db).await
    }

    pub async fn find_by_parent_id<C: ConnectionTrait>(db: &C, parent_id: i64) -> Result<Vec<shop_category::Model>, DbErr> {
        CategoryEntity::find()
            .filter(shop_category::Column::ParentId.eq(parent_id))
            .order_by_asc(shop_category::Column::SortOrder)
            .all(db)
            .await
    }

    pub async fn find_all<C: ConnectionTrait>(db: &C) -> Result<Vec<shop_category::Model>, DbErr> {
        CategoryEntity::find()
            .order_by_asc(shop_category::Column::SortOrder)
            .all(db)
            .await
    }

    pub async fn delete_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<i64, DbErr> {
        CategoryEntity::delete_by_id(id).exec(db).await.map(|r| r.rows_affected as i64)
    }

    pub async fn find_page<C: ConnectionTrait>(db: &C, page: i64, per_page: i64) -> Result<(Vec<shop_category::Model>, i64), DbErr> {
        let paginator = CategoryEntity::find()
            .order_by_asc(shop_category::Column::SortOrder)
            .paginate(db, per_page as u64);

        let total = paginator.num_items().await? as i64;
        let items = paginator.fetch_page(page.saturating_sub(1) as u64).await?;
        Ok((items, total))
    }
}
