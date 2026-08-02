//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!


use crate::modules::website::entity::{website_media, website_media::Entity as WebsiteMedia};
use sea_orm::*;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

/// 媒体上传请求DTO
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct MediaSaveRequest {
    /// 原始文件名
    pub original_name: Option<String>,
    /// 存储文件名
    pub storage_name: Option<String>,
    /// 文件存储路径
    pub file_path: Option<String>,
    /// 文件访问URL
    pub file_url: Option<String>,
    /// 文件扩展名
    pub file_ext: Option<String>,
    /// 文件大小（字节）
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub file_size: Option<i64>,
    /// 文件类型：1=图片, 2=视频, 3=文档, 4=音频, 5=其他
    pub file_type: Option<i32>,
    /// MIME类型
    pub mime_type: Option<String>,
    /// 图片宽度
    pub width: Option<i32>,
    /// 图片高度
    pub height: Option<i32>,
    /// 小缩略图URL
    pub thumb_small: Option<String>,
    /// 中缩略图URL
    pub thumb_medium: Option<String>,
    /// 大缩略图URL
    pub thumb_large: Option<String>,
    /// 替代文本
    pub alt_text: Option<String>,
    /// 标题
    pub title: Option<String>,
    /// 说明文字
    pub caption: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 分类ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub category_id: Option<i64>,
    /// 标签数组
    pub tags: Option<Vec<String>>,
    /// 是否有水印：0无，1有
    pub has_watermark: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
    /// 创建人ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub create_by: Option<i64>,
}

impl From<MediaSaveRequest> for MediaSaveDTO {
    fn from(form_data: MediaSaveRequest) -> Self {
        MediaSaveDTO {
            id: None,
            original_name: form_data.original_name,
            storage_name: form_data.storage_name,
            file_path: form_data.file_path,
            file_url: form_data.file_url,
            file_ext: form_data.file_ext,
            file_size: form_data.file_size,
            file_type: form_data.file_type,
            mime_type: form_data.mime_type,
            width: form_data.width,
            height: form_data.height,
            thumb_small: form_data.thumb_small,
            thumb_medium: form_data.thumb_medium,
            thumb_large: form_data.thumb_large,
            alt_text: form_data.alt_text,
            title: form_data.title,
            caption: form_data.caption,
            description: form_data.description,
            category_id: form_data.category_id,
            tags: form_data.tags,
            has_watermark: form_data.has_watermark,
            sort: form_data.sort,
            status: form_data.status,
            create_by: form_data.create_by,
        }
    }
}

/// 媒体更新请求DTO
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct MediaUpdateRequest {
    /// 替代文本
    pub alt_text: Option<String>,
    /// 标题
    pub title: Option<String>,
    /// 说明文字
    pub caption: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 分类ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub category_id: Option<i64>,
    /// 标签数组
    pub tags: Option<Vec<String>>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
}

impl From<MediaUpdateRequest> for MediaSaveDTO {
    fn from(form_data: MediaUpdateRequest) -> Self {
        MediaSaveDTO {
            id: None,
            original_name: None,
            storage_name: None,
            file_path: None,
            file_url: None,
            file_ext: None,
            file_size: None,
            file_type: None,
            mime_type: None,
            width: None,
            height: None,
            thumb_small: None,
            thumb_medium: None,
            thumb_large: None,
            alt_text: form_data.alt_text,
            title: form_data.title,
            caption: form_data.caption,
            description: form_data.description,
            category_id: form_data.category_id,
            tags: form_data.tags,
            has_watermark: None,
            sort: form_data.sort,
            status: form_data.status,
            create_by: None,
        }
    }
}

/// 媒体内部传输DTO
pub struct MediaSaveDTO {
    pub id: Option<i64>,
    pub original_name: Option<String>,
    pub storage_name: Option<String>,
    pub file_path: Option<String>,
    pub file_url: Option<String>,
    pub file_ext: Option<String>,
    pub file_size: Option<i64>,
    pub file_type: Option<i32>,
    pub mime_type: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub thumb_small: Option<String>,
    pub thumb_medium: Option<String>,
    pub thumb_large: Option<String>,
    pub alt_text: Option<String>,
    pub title: Option<String>,
    pub caption: Option<String>,
    pub description: Option<String>,
    pub category_id: Option<i64>,
    pub tags: Option<Vec<String>>,
    pub has_watermark: Option<i32>,
    pub sort: Option<i32>,
    pub status: Option<i32>,
    pub create_by: Option<i64>,
}

/// 媒体列表VO
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 原始文件名
    pub original_name: Option<String>,
    /// 存储文件名
    pub storage_name: Option<String>,
    /// 文件访问URL
    pub file_url: Option<String>,
    /// 文件扩展名
    pub file_ext: Option<String>,
    /// 文件大小（字节）
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub file_size: Option<i64>,
    /// 文件类型：1=图片, 2=视频, 3=文档, 4=音频, 5=其他
    pub file_type: Option<i32>,
    /// MIME类型
    pub mime_type: Option<String>,
    /// 图片宽度
    pub width: Option<i32>,
    /// 图片高度
    pub height: Option<i32>,
    /// 小缩略图URL
    pub thumb_small: Option<String>,
    /// 中缩略图URL
    pub thumb_medium: Option<String>,
    /// 大缩略图URL
    pub thumb_large: Option<String>,
    /// 标题
    pub title: Option<String>,
    /// 分类ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub category_id: Option<i64>,
    /// 标签数组
    pub tags: Option<Vec<String>>,
    /// 引用计数
    pub ref_count: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
    /// 创建时间
    pub create_time: Option<String>,
}

impl From<website_media::Model> for MediaListVO {
    fn from(model: website_media::Model) -> Self {
        MediaListVO {
            id: Option::from(model.id),
            original_name: model.original_name,
            storage_name: model.storage_name,
            file_url: model.file_url,
            file_ext: model.file_ext,
            file_size: model.file_size,
            file_type: model.file_type,
            mime_type: model.mime_type,
            width: model.width,
            height: model.height,
            thumb_small: model.thumb_small,
            thumb_medium: model.thumb_medium,
            thumb_large: model.thumb_large,
            title: model.title,
            category_id: model.category_id,
            tags: model.tags,
            ref_count: model.ref_count,
            sort: model.sort,
            status: model.status,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// 媒体详情VO
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 原始文件名
    pub original_name: Option<String>,
    /// 存储文件名
    pub storage_name: Option<String>,
    /// 文件存储路径
    pub file_path: Option<String>,
    /// 文件访问URL
    pub file_url: Option<String>,
    /// 文件扩展名
    pub file_ext: Option<String>,
    /// 文件大小（字节）
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub file_size: Option<i64>,
    /// 文件类型：1=图片, 2=视频, 3=文档, 4=音频, 5=其他
    pub file_type: Option<i32>,
    /// MIME类型
    pub mime_type: Option<String>,
    /// 图片宽度
    pub width: Option<i32>,
    /// 图片高度
    pub height: Option<i32>,
    /// 小缩略图URL
    pub thumb_small: Option<String>,
    /// 中缩略图URL
    pub thumb_medium: Option<String>,
    /// 大缩略图URL
    pub thumb_large: Option<String>,
    /// 替代文本
    pub alt_text: Option<String>,
    /// 标题
    pub title: Option<String>,
    /// 说明文字
    pub caption: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 分类ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub category_id: Option<i64>,
    /// 标签数组
    pub tags: Option<Vec<String>>,
    /// 引用计数
    pub ref_count: Option<i32>,
    /// 是否有水印：0无，1有
    pub has_watermark: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
    /// 创建人ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub create_by: Option<i64>,
    /// 创建时间
    pub create_time: Option<String>,
    /// 更新时间
    pub update_time: Option<String>,
}

impl From<website_media::Model> for MediaDetailVO {
    fn from(model: website_media::Model) -> Self {
        MediaDetailVO {
            id: Option::from(model.id),
            original_name: model.original_name,
            storage_name: model.storage_name,
            file_path: model.file_path,
            file_url: model.file_url,
            file_ext: model.file_ext,
            file_size: model.file_size,
            file_type: model.file_type,
            mime_type: model.mime_type,
            width: model.width,
            height: model.height,
            thumb_small: model.thumb_small,
            thumb_medium: model.thumb_medium,
            thumb_large: model.thumb_large,
            alt_text: model.alt_text,
            title: model.title,
            caption: model.caption,
            description: model.description,
            category_id: model.category_id,
            tags: model.tags,
            ref_count: model.ref_count,
            has_watermark: model.has_watermark,
            sort: model.sort,
            status: model.status,
            create_by: model.create_by,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: model.update_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// 分页查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    /// 关键词（按原始文件名模糊查询）
    pub keywords: Option<String>,
    /// 文件类型：1=图片, 2=视频, 3=文档, 4=音频, 5=其他
    pub file_type: Option<i32>,
    /// 分类ID
    pub category_id: Option<i64>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
}

/// 查询条件
#[derive(Clone)]
pub struct PageWhere {
    pub keywords: Option<String>,
    pub file_type: Option<i32>,
    pub category_id: Option<i64>,
    pub status: Option<i32>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut keywords = None;
        if self.keywords != Some("".to_string()) {
            keywords = self.keywords.clone();
        }

        let mut file_type = None;
        if self.file_type > Some(0) {
            file_type = self.file_type.clone();
        }

        let mut category_id = None;
        if self.category_id > Some(0) {
            category_id = self.category_id.clone();
        }

        let mut status = None;
        if self.status > Some(0) {
            status = self.status.clone();
        }

        Self {
            keywords,
            file_type,
            category_id,
            status,
        }
    }
}

pub struct WebsiteMediaModel;

impl WebsiteMediaModel {
    /// 新增媒体记录
    pub async fn insert(db: &DbConn, dto: &MediaSaveDTO) -> Result<i64, DbErr> {
        let model = website_media::ActiveModel {
            original_name: Set(dto.original_name.to_owned()),
            storage_name: Set(dto.storage_name.to_owned()),
            file_path: Set(dto.file_path.to_owned()),
            file_url: Set(dto.file_url.to_owned()),
            file_ext: Set(dto.file_ext.to_owned()),
            file_size: Set(dto.file_size.to_owned()),
            file_type: Set(dto.file_type.to_owned()),
            mime_type: Set(dto.mime_type.to_owned()),
            width: Set(dto.width.to_owned()),
            height: Set(dto.height.to_owned()),
            thumb_small: Set(dto.thumb_small.to_owned()),
            thumb_medium: Set(dto.thumb_medium.to_owned()),
            thumb_large: Set(dto.thumb_large.to_owned()),
            alt_text: Set(dto.alt_text.to_owned()),
            title: Set(dto.title.to_owned()),
            caption: Set(dto.caption.to_owned()),
            description: Set(dto.description.to_owned()),
            category_id: Set(dto.category_id.to_owned()),
            tags: Set(dto.tags.to_owned()),
            has_watermark: Set(dto.has_watermark.to_owned()),
            sort: Set(dto.sort.to_owned()),
            status: Set(dto.status.to_owned().or(Some(1))),
            deleted: Set(Some(0)),
            create_by: Set(dto.create_by.to_owned()),
            create_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let res = WebsiteMedia::insert(model).exec(db).await?;
        Ok(res.last_insert_id)
    }

    /// 按id批量软删除
    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        let result: UpdateResult = WebsiteMedia::update_many()
            .set(website_media::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(website_media::Column::Id.is_in(ids))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 按id更新媒体元数据
    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, dto: &MediaSaveDTO) -> Result<i64, DbErr> {
        let model = website_media::ActiveModel {
            alt_text: Set(dto.alt_text.to_owned()),
            title: Set(dto.title.to_owned()),
            caption: Set(dto.caption.to_owned()),
            description: Set(dto.description.to_owned()),
            category_id: Set(dto.category_id.to_owned()),
            tags: Set(dto.tags.to_owned()),
            sort: Set(dto.sort.to_owned()),
            status: Set(dto.status.to_owned()),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let result: UpdateResult = WebsiteMedia::update_many()
            .set(model)
            .filter(website_media::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 按id查询
    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<website_media::Model>, DbErr> {
        let result = WebsiteMedia::find_by_id(id.clone().unwrap_or_default())
            .filter(website_media::Column::Deleted.eq(0))
            .one(db)
            .await?;
        Ok(result)
    }

    /// 分页查询
    pub async fn select_in_page(db: &DbConn, page_num: i64, page_size: i64, where_case: PageWhere) -> Result<(Vec<website_media::Model>, u64), DbErr> {
        let paginator = WebsiteMedia::find()
            .filter(website_media::Column::Deleted.eq(0))
            .apply_if(where_case.keywords.clone(), |query, v| {
                query.filter(website_media::Column::OriginalName.contains(v))
            })
            .apply_if(where_case.file_type, |query, v| {
                query.filter(website_media::Column::FileType.eq(v))
            })
            .apply_if(where_case.category_id, |query, v| {
                query.filter(website_media::Column::CategoryId.eq(v))
            })
            .apply_if(where_case.status, |query, v| {
                query.filter(website_media::Column::Status.eq(v))
            })
            .order_by_desc(website_media::Column::CreateTime)
            .paginate(db, page_size as u64);
        let total = paginator.num_items().await?;
        let list = paginator.fetch_page(page_num as u64).await?;
        Ok((list, total))
    }

    /// 查询总数
    pub async fn select_count(db: &DbConn, where_case: PageWhere) -> Result<i64, DbErr> {
        let count = WebsiteMedia::find()
            .filter(website_media::Column::Deleted.eq(0))
            .apply_if(where_case.keywords.clone(), |query, v| {
                query.filter(website_media::Column::OriginalName.contains(v))
            })
            .apply_if(where_case.file_type, |query, v| {
                query.filter(website_media::Column::FileType.eq(v))
            })
            .apply_if(where_case.category_id, |query, v| {
                query.filter(website_media::Column::CategoryId.eq(v))
            })
            .apply_if(where_case.status, |query, v| {
                query.filter(website_media::Column::Status.eq(v))
            })
            .count(db)
            .await? as i64;
        Ok(count)
    }

    /// 引用计数增减
    pub async fn update_ref_count(db: &DbConn, id: i64, delta: i32) -> Result<i64, DbErr> {
        let media = WebsiteMedia::find_by_id(id).one(db).await?;
        if let Some(m) = media {
            let new_count = std::cmp::max(m.ref_count.unwrap_or(0) + delta, 0);
            let result: UpdateResult = WebsiteMedia::update_many()
                .set(website_media::ActiveModel {
                    ref_count: Set(Some(new_count)),
                    ..Default::default()
                })
                .filter(website_media::Column::Id.eq(id))
                .exec(db)
                .await?;
            Ok(result.rows_affected as i64)
        } else {
            Ok(0)
        }
    }
}
