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
use crate::modules::website::entity::{website_banner, website_banner::Entity as WebsiteBanner};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};
use sea_orm::prelude::DateTime;
use sea_orm::*;


#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct BannerSaveRequest {
    /// 标题
    pub title: Option<String>,
    /// 图片地址
    pub image_url: Option<String>,
    /// 链接地址
    pub link_url: Option<String>,
    /// 替换文本
    pub alt_text: Option<String>,
    /// 显示位置
    pub position: Option<String>,
    /// 打开方式：_self, _blank
    pub target: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 开始时间
    pub start_time: Option<String>,
    /// 结束时间
    pub end_time: Option<String>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
}

impl From<BannerSaveRequest> for BannerSaveDTO {
    fn from(req: BannerSaveRequest) -> Self {
        BannerSaveDTO {
            id: None,
            title: req.title,
            image_url: req.image_url,
            link_url: req.link_url,
            alt_text: req.alt_text,
            position: req.position,
            target: req.target,
            sort: req.sort,
            start_time: req.start_time.and_then(|s| chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").ok()),
            end_time: req.end_time.and_then(|s| chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").ok()),
            status: req.status,
        }
    }
}


#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct BannerUpdateRequest {
    /// 主键ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 标题
    pub title: Option<String>,
    /// 图片地址
    pub image_url: Option<String>,
    /// 链接地址
    pub link_url: Option<String>,
    /// 替换文本
    pub alt_text: Option<String>,
    /// 显示位置
    pub position: Option<String>,
    /// 打开方式：_self, _blank
    pub target: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 开始时间
    pub start_time: Option<String>,
    /// 结束时间
    pub end_time: Option<String>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
}

impl From<BannerUpdateRequest> for BannerSaveDTO {
    fn from(req: BannerUpdateRequest) -> Self {
        BannerSaveDTO {
            id: req.id,
            title: req.title,
            image_url: req.image_url,
            link_url: req.link_url,
            alt_text: req.alt_text,
            position: req.position,
            target: req.target,
            sort: req.sort,
            start_time: req.start_time.and_then(|s| chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").ok()),
            end_time: req.end_time.and_then(|s| chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").ok()),
            status: req.status,
        }
    }
}


#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BannerSaveDTO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 标题
    pub title: Option<String>,
    /// 图片地址
    pub image_url: Option<String>,
    /// 链接地址
    pub link_url: Option<String>,
    /// 替换文本
    pub alt_text: Option<String>,
    /// 显示位置
    pub position: Option<String>,
    /// 打开方式：_self, _blank
    pub target: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 开始时间
    pub start_time: Option<DateTime>,
    /// 结束时间
    pub end_time: Option<DateTime>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BannerListVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 标题
    pub title: Option<String>,
    /// 图片地址
    pub image_url: Option<String>,
    /// 链接地址
    pub link_url: Option<String>,
    /// 替换文本
    pub alt_text: Option<String>,
    /// 显示位置
    pub position: Option<String>,
    /// 打开方式：_self, _blank
    pub target: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 开始时间
    pub start_time: Option<String>,
    /// 结束时间
    pub end_time: Option<String>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
    /// 创建时间
    pub create_time: Option<String>,
    /// 更新时间
    pub update_time: Option<String>,
}

impl From<website_banner::Model> for BannerListVO {
    fn from(model: website_banner::Model) -> Self {
        BannerListVO {
            id: Option::from(model.id),
            title: model.title,
            image_url: model.image_url,
            link_url: model.link_url,
            alt_text: model.alt_text,
            position: model.position,
            target: model.target,
            sort: model.sort,
            start_time: model.start_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            end_time: model.end_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            status: model.status,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: model.update_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BannerDetailVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 标题
    pub title: Option<String>,
    /// 图片地址
    pub image_url: Option<String>,
    /// 链接地址
    pub link_url: Option<String>,
    /// 替换文本
    pub alt_text: Option<String>,
    /// 显示位置
    pub position: Option<String>,
    /// 打开方式：_self, _blank
    pub target: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 开始时间
    pub start_time: Option<String>,
    /// 结束时间
    pub end_time: Option<String>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
    /// 创建时间
    pub create_time: Option<String>,
    /// 更新时间
    pub update_time: Option<String>,
}

impl From<website_banner::Model> for BannerDetailVO {
    fn from(model: website_banner::Model) -> Self {
        BannerDetailVO {
            id: Option::from(model.id),
            title: model.title,
            image_url: model.image_url,
            link_url: model.link_url,
            alt_text: model.alt_text,
            position: model.position,
            target: model.target,
            sort: model.sort,
            start_time: model.start_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            end_time: model.end_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            status: model.status,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: model.update_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub keywords: Option<String>,
    /// 显示位置
    pub position: Option<String>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Clone)]
pub struct PageWhere {
    pub keywords: Option<String>,
    /// 显示位置
    pub position: Option<String>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut keywords = None;
        if self.keywords != Some("".to_string()) {
            keywords = self.keywords.clone();
        }

        let mut position = None;
        if self.position != Some("".to_string()) {
            position = self.position.clone();
        }

        let mut status = None;
        if self.status == Some(1) || self.status == Some(0) {
            status = self.status;
        }

        Self {
            keywords,
            position,
            status,
        }
    }
}

pub struct WebsiteBannerModel;

impl WebsiteBannerModel {
    pub async fn insert(db: &DbConn, dto: &BannerSaveDTO) -> Result<i64, DbErr> {
        let model = website_banner::ActiveModel {
            title: Set(dto.title.to_owned()),
            image_url: Set(dto.image_url.to_owned()),
            link_url: Set(dto.link_url.to_owned()),
            alt_text: Set(dto.alt_text.to_owned()),
            position: Set(dto.position.to_owned()),
            target: Set(dto.target.to_owned()),
            sort: Set(dto.sort.to_owned()),
            start_time: Set(dto.start_time.to_owned()),
            end_time: Set(dto.end_time.to_owned()),
            status: Set(dto.status.to_owned()),
            create_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let res = WebsiteBanner::insert(model).exec(db).await?;
        Ok(res.last_insert_id)
    }

    /// 按id批量软删除
    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        let result: UpdateResult = WebsiteBanner::update_many()
            .set(website_banner::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(website_banner::Column::Id.is_in(ids))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, dto: &BannerSaveDTO) -> Result<i64, DbErr> {
        let model = website_banner::ActiveModel {
            title: Set(dto.title.to_owned()),
            image_url: Set(dto.image_url.to_owned()),
            link_url: Set(dto.link_url.to_owned()),
            alt_text: Set(dto.alt_text.to_owned()),
            position: Set(dto.position.to_owned()),
            target: Set(dto.target.to_owned()),
            sort: Set(dto.sort.to_owned()),
            start_time: Set(dto.start_time.to_owned()),
            end_time: Set(dto.end_time.to_owned()),
            status: Set(dto.status.to_owned()),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let result: UpdateResult = WebsiteBanner::update_many()
            .set(model)
            .filter(website_banner::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<website_banner::Model>, DbErr> {
        let result = website_banner::Entity::find_by_id(id.clone().unwrap_or_default())
            .filter(website_banner::Column::Deleted.eq(0))
            .one(db)
            .await?;
        Ok(result)
    }

    pub async fn select_by_position(
        db: &DbConn,
        position: &Option<String>,
    ) -> Result<Vec<website_banner::Model>, DbErr> {
        let list = WebsiteBanner::find()
            .filter(website_banner::Column::Deleted.eq(0))
            .filter(website_banner::Column::Status.eq(1))
            .filter(website_banner::Column::Position.eq(position.clone().unwrap_or_default()))
            .order_by_asc(website_banner::Column::Sort)
            .all(db)
            .await?;
        Ok(list)
    }

    pub async fn select_in_page(
        db: &DbConn,
        page_num: i64,
        page_size: i64,
        where_case: PageWhere,
    ) -> Result<(Vec<website_banner::Model>, u64), DbErr> {
        let paginator = WebsiteBanner::find()
            .filter(website_banner::Column::Deleted.eq(0))
            .apply_if(where_case.keywords.clone(), |query, v| {
                query.filter(website_banner::Column::Title.contains(v))
            })
            .apply_if(where_case.position.clone(), |query, v| {
                query.filter(website_banner::Column::Position.eq(v))
            })
            .apply_if(where_case.status, |query, v| {
                query.filter(website_banner::Column::Status.eq(v))
            })
            .order_by_asc(website_banner::Column::Sort)
            .paginate(db, page_size as u64);
        let total = paginator.num_items().await?;
        let list = paginator.fetch_page(page_num as u64).await?;
        Ok((list, total))
    }

    pub async fn select_count(db: &DbConn, where_case: PageWhere) -> Result<i64, DbErr> {
        let count = WebsiteBanner::find()
            .filter(website_banner::Column::Deleted.eq(0))
            .apply_if(where_case.keywords.clone(), |query, v| {
                query.filter(website_banner::Column::Title.contains(v))
            })
            .apply_if(where_case.position.clone(), |query, v| {
                query.filter(website_banner::Column::Position.eq(v))
            })
            .apply_if(where_case.status, |query, v| {
                query.filter(website_banner::Column::Status.eq(v))
            })
            .count(db)
            .await? as i64;
        Ok(count)
    }
}
