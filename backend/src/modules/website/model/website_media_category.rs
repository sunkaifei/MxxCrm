//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!


use crate::modules::website::entity::{website_media_category, website_media_category::Entity as WebsiteMediaCategory};
use sea_orm::*;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

/// 媒体分类新增请求DTO
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct MediaCategorySaveRequest {
    /// 分类名称
    pub category_name: Option<String>,
    /// 父分类ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub parent_id: Option<i64>,
    /// 排序
    pub sort: Option<i32>,
}

impl From<MediaCategorySaveRequest> for MediaCategorySaveDTO {
    fn from(form_data: MediaCategorySaveRequest) -> Self {
        MediaCategorySaveDTO {
            id: None,
            category_name: form_data.category_name,
            parent_id: form_data.parent_id,
            sort: form_data.sort,
        }
    }
}

/// 媒体分类更新请求DTO
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct MediaCategoryUpdateRequest {
    /// 分类ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 分类名称
    pub category_name: Option<String>,
    /// 父分类ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub parent_id: Option<i64>,
    /// 排序
    pub sort: Option<i32>,
}

impl From<MediaCategoryUpdateRequest> for MediaCategorySaveDTO {
    fn from(form_data: MediaCategoryUpdateRequest) -> Self {
        MediaCategorySaveDTO {
            id: form_data.id,
            category_name: form_data.category_name,
            parent_id: form_data.parent_id,
            sort: form_data.sort,
        }
    }
}

/// 媒体分类内部传输DTO
pub struct MediaCategorySaveDTO {
    pub id: Option<i64>,
    pub category_name: Option<String>,
    pub parent_id: Option<i64>,
    pub sort: Option<i32>,
}

/// 媒体分类列表VO
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaCategoryListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 分类名称
    pub category_name: Option<String>,
    /// 父分类ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub parent_id: Option<i64>,
    /// 排序
    pub sort: Option<i32>,
    /// 创建时间
    pub create_time: Option<String>,
    /// 子分类
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<MediaCategoryListVO>>,
}

impl From<website_media_category::Model> for MediaCategoryListVO {
    fn from(model: website_media_category::Model) -> Self {
        MediaCategoryListVO {
            id: Option::from(model.id),
            category_name: model.category_name,
            parent_id: model.parent_id,
            sort: model.sort,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            children: None,
        }
    }
}

/// 媒体分类详情VO
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaCategoryDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 分类名称
    pub category_name: Option<String>,
    /// 父分类ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub parent_id: Option<i64>,
    /// 排序
    pub sort: Option<i32>,
    /// 创建时间
    pub create_time: Option<String>,
}

impl From<website_media_category::Model> for MediaCategoryDetailVO {
    fn from(model: website_media_category::Model) -> Self {
        MediaCategoryDetailVO {
            id: Option::from(model.id),
            category_name: model.category_name,
            parent_id: model.parent_id,
            sort: model.sort,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// 媒体分类下拉选项VO
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaCategorySelectVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 分类名称
    pub category_name: Option<String>,
    /// 父分类ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub parent_id: Option<i64>,
    /// 子分类
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<MediaCategorySelectVO>>,
}

impl From<website_media_category::Model> for MediaCategorySelectVO {
    fn from(model: website_media_category::Model) -> Self {
        MediaCategorySelectVO {
            id: Option::from(model.id),
            category_name: model.category_name,
            parent_id: model.parent_id,
            children: None,
        }
    }
}

pub struct WebsiteMediaCategoryModel;

impl WebsiteMediaCategoryModel {
    /// 新增媒体分类
    pub async fn insert(db: &DbConn, dto: &MediaCategorySaveDTO) -> Result<i64, DbErr> {
        let model = website_media_category::ActiveModel {
            category_name: Set(dto.category_name.to_owned()),
            parent_id: Set(dto.parent_id.to_owned().or(Some(0))),
            sort: Set(dto.sort.to_owned()),
            deleted: Set(Some(0)),
            create_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let res = WebsiteMediaCategory::insert(model).exec(db).await?;
        Ok(res.last_insert_id)
    }

    /// 按id软删除
    pub async fn delete_by_id(db: &DbConn, id: i64) -> Result<i64, DbErr> {
        let result: UpdateResult = WebsiteMediaCategory::update_many()
            .set(website_media_category::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(website_media_category::Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 按id更新
    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, dto: &MediaCategorySaveDTO) -> Result<i64, DbErr> {
        let model = website_media_category::ActiveModel {
            category_name: Set(dto.category_name.to_owned()),
            parent_id: Set(dto.parent_id.to_owned()),
            sort: Set(dto.sort.to_owned()),
            ..Default::default()
        };
        let result: UpdateResult = WebsiteMediaCategory::update_many()
            .set(model)
            .filter(website_media_category::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 按id查询
    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<website_media_category::Model>, DbErr> {
        let result = WebsiteMediaCategory::find_by_id(id.clone().unwrap_or_default())
            .filter(website_media_category::Column::Deleted.eq(0))
            .one(db)
            .await?;
        Ok(result)
    }

    /// 查询所有未删除分类
    pub async fn select_all(db: &DbConn) -> Result<Vec<website_media_category::Model>, DbErr> {
        let result = WebsiteMediaCategory::find()
            .filter(website_media_category::Column::Deleted.eq(0))
            .order_by_asc(website_media_category::Column::Sort)
            .all(db)
            .await?;
        Ok(result)
    }
}
