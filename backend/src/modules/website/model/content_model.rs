//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::modules::website::entity::{content_model, content_model::Entity as ContentModel};
use sea_orm::prelude::DateTime;
use sea_orm::*;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

/// 内容模型新增请求
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ContentModelSaveRequest {
    // 模型编码
    pub model_code: Option<String>,
    // 模型名称
    pub model_name: Option<String>,
    // 模型图标
    pub model_icon: Option<String>,
    // 描述
    pub description: Option<String>,
    // 是否有标题
    pub has_title: Option<i32>,
    // 是否有内容
    pub has_content: Option<i32>,
    // 是否有封面
    pub has_cover: Option<i32>,
    // 是否有作者
    pub has_author: Option<i32>,
    // 是否有摘要
    pub has_summary: Option<i32>,
    // 是否有SEO
    pub has_seo: Option<i32>,
    // 是否有图集
    pub has_images: Option<i32>,
    // 是否有附件
    pub has_attachment: Option<i32>,
    // 列表模板ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub list_template_id: Option<i64>,
    // 详情模板ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub detail_template_id: Option<i64>,
    // 排序
    pub sort: Option<i32>,
    // 状态：0停用，1正常
    pub status: Option<i32>,
}

impl From<ContentModelSaveRequest> for ContentModelSaveDTO {
    fn from(form_data: ContentModelSaveRequest) -> Self {
        ContentModelSaveDTO {
            id: None,
            model_code: form_data.model_code,
            model_name: form_data.model_name,
            model_icon: form_data.model_icon,
            description: form_data.description,
            has_title: form_data.has_title,
            has_content: form_data.has_content,
            has_cover: form_data.has_cover,
            has_author: form_data.has_author,
            has_summary: form_data.has_summary,
            has_seo: form_data.has_seo,
            has_images: form_data.has_images,
            has_attachment: form_data.has_attachment,
            list_template_id: form_data.list_template_id,
            detail_template_id: form_data.detail_template_id,
            sort: form_data.sort,
            status: form_data.status,
            create_time: None,
            update_time: None,
        }
    }
}

/// 内容模型更新请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContentModelUpdateRequest {
    // 模型ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    // 模型编码
    pub model_code: Option<String>,
    // 模型名称
    pub model_name: Option<String>,
    // 模型图标
    pub model_icon: Option<String>,
    // 描述
    pub description: Option<String>,
    // 是否有标题
    pub has_title: Option<i32>,
    // 是否有内容
    pub has_content: Option<i32>,
    // 是否有封面
    pub has_cover: Option<i32>,
    // 是否有作者
    pub has_author: Option<i32>,
    // 是否有摘要
    pub has_summary: Option<i32>,
    // 是否有SEO
    pub has_seo: Option<i32>,
    // 是否有图集
    pub has_images: Option<i32>,
    // 是否有附件
    pub has_attachment: Option<i32>,
    // 列表模板ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub list_template_id: Option<i64>,
    // 详情模板ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub detail_template_id: Option<i64>,
    // 排序
    pub sort: Option<i32>,
    // 状态：0停用，1正常
    pub status: Option<i32>,
}

impl From<ContentModelUpdateRequest> for ContentModelSaveDTO {
    fn from(form_data: ContentModelUpdateRequest) -> Self {
        ContentModelSaveDTO {
            id: form_data.id,
            model_code: form_data.model_code,
            model_name: form_data.model_name,
            model_icon: form_data.model_icon,
            description: form_data.description,
            has_title: form_data.has_title,
            has_content: form_data.has_content,
            has_cover: form_data.has_cover,
            has_author: form_data.has_author,
            has_summary: form_data.has_summary,
            has_seo: form_data.has_seo,
            has_images: form_data.has_images,
            has_attachment: form_data.has_attachment,
            list_template_id: form_data.list_template_id,
            detail_template_id: form_data.detail_template_id,
            sort: form_data.sort,
            status: form_data.status,
            create_time: None,
            update_time: None,
        }
    }
}

/// 内容模型内部传输DTO
pub struct ContentModelSaveDTO {
    // 模型ID
    pub id: Option<i64>,
    // 模型编码
    pub model_code: Option<String>,
    // 模型名称
    pub model_name: Option<String>,
    // 模型图标
    pub model_icon: Option<String>,
    // 描述
    pub description: Option<String>,
    // 是否有标题
    pub has_title: Option<i32>,
    // 是否有内容
    pub has_content: Option<i32>,
    // 是否有封面
    pub has_cover: Option<i32>,
    // 是否有作者
    pub has_author: Option<i32>,
    // 是否有摘要
    pub has_summary: Option<i32>,
    // 是否有SEO
    pub has_seo: Option<i32>,
    // 是否有图集
    pub has_images: Option<i32>,
    // 是否有附件
    pub has_attachment: Option<i32>,
    // 列表模板ID
    pub list_template_id: Option<i64>,
    // 详情模板ID
    pub detail_template_id: Option<i64>,
    // 排序
    pub sort: Option<i32>,
    // 状态：0停用，1正常
    pub status: Option<i32>,
    // 创建时间
    pub create_time: Option<DateTime>,
    // 更新时间
    pub update_time: Option<DateTime>,
}

/// 内容模型列表VO
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContentModelListVO {
    // 模型ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    // 模型编码
    pub model_code: Option<String>,
    // 模型名称
    pub model_name: Option<String>,
    // 模型图标
    pub model_icon: Option<String>,
    // 描述
    pub description: Option<String>,
    // 是否有标题
    pub has_title: Option<i32>,
    // 是否有内容
    pub has_content: Option<i32>,
    // 是否有封面
    pub has_cover: Option<i32>,
    // 是否有作者
    pub has_author: Option<i32>,
    // 是否有摘要
    pub has_summary: Option<i32>,
    // 是否有SEO
    pub has_seo: Option<i32>,
    // 是否有图集
    pub has_images: Option<i32>,
    // 是否有附件
    pub has_attachment: Option<i32>,
    // 列表模板ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub list_template_id: Option<i64>,
    // 详情模板ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub detail_template_id: Option<i64>,
    // 排序
    pub sort: Option<i32>,
    // 状态：0停用，1正常
    pub status: Option<i32>,
    // 是否系统内置：0否，1是
    pub is_system: Option<i32>,
    // 创建时间
    pub create_time: Option<String>,
    // 更新时间
    pub update_time: Option<String>,
}

impl From<content_model::Model> for ContentModelListVO {
    fn from(model: content_model::Model) -> Self {
        ContentModelListVO {
            id: Option::from(model.id),
            model_code: model.model_code,
            model_name: model.model_name,
            model_icon: model.model_icon,
            description: model.description,
            has_title: model.has_title,
            has_content: model.has_content,
            has_cover: model.has_cover,
            has_author: model.has_author,
            has_summary: model.has_summary,
            has_seo: model.has_seo,
            has_images: model.has_images,
            has_attachment: model.has_attachment,
            list_template_id: model.list_template_id,
            detail_template_id: model.detail_template_id,
            sort: model.sort,
            status: model.status,
            is_system: model.is_system,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: model.update_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// 内容模型详情VO
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContentModelDetailVO {
    // 模型ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    // 模型编码
    pub model_code: Option<String>,
    // 模型名称
    pub model_name: Option<String>,
    // 模型图标
    pub model_icon: Option<String>,
    // 描述
    pub description: Option<String>,
    // 是否有标题
    pub has_title: Option<i32>,
    // 是否有内容
    pub has_content: Option<i32>,
    // 是否有封面
    pub has_cover: Option<i32>,
    // 是否有作者
    pub has_author: Option<i32>,
    // 是否有摘要
    pub has_summary: Option<i32>,
    // 是否有SEO
    pub has_seo: Option<i32>,
    // 是否有图集
    pub has_images: Option<i32>,
    // 是否有附件
    pub has_attachment: Option<i32>,
    // 列表模板ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub list_template_id: Option<i64>,
    // 详情模板ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub detail_template_id: Option<i64>,
    // 排序
    pub sort: Option<i32>,
    // 状态：0停用，1正常
    pub status: Option<i32>,
    // 是否系统内置：0否，1是
    pub is_system: Option<i32>,
    // 创建时间
    pub create_time: Option<String>,
    // 更新时间
    pub update_time: Option<String>,
}

impl From<content_model::Model> for ContentModelDetailVO {
    fn from(model: content_model::Model) -> Self {
        ContentModelDetailVO {
            id: Option::from(model.id),
            model_code: model.model_code,
            model_name: model.model_name,
            model_icon: model.model_icon,
            description: model.description,
            has_title: model.has_title,
            has_content: model.has_content,
            has_cover: model.has_cover,
            has_author: model.has_author,
            has_summary: model.has_summary,
            has_seo: model.has_seo,
            has_images: model.has_images,
            has_attachment: model.has_attachment,
            list_template_id: model.list_template_id,
            detail_template_id: model.detail_template_id,
            sort: model.sort,
            status: model.status,
            is_system: model.is_system,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: model.update_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// 查询条件
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub keywords: Option<String>,
    pub status: Option<i32>,
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
}

/// 条件
#[derive(Clone)]
pub struct PageWhere {
    pub model_name: Option<String>,
    pub status: Option<i32>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut model_name = None;
        if self.model_name != Some("".to_string()) {
            model_name = self.model_name.clone();
        }

        let mut status = None;
        if self.status == Some(0) || self.status == Some(1) {
            status = self.status;
        }

        Self {
            model_name,
            status,
        }
    }
}

pub struct ContentModelModel;

impl ContentModelModel {
    pub async fn insert(db: &DbConn, dto: &ContentModelSaveDTO) -> Result<i64, DbErr> {
        let model = content_model::ActiveModel {
            model_code: Set(dto.model_code.to_owned()),
            model_name: Set(dto.model_name.to_owned()),
            model_icon: Set(dto.model_icon.to_owned()),
            description: Set(dto.description.to_owned()),
            has_title: Set(dto.has_title.to_owned()),
            has_content: Set(dto.has_content.to_owned()),
            has_cover: Set(dto.has_cover.to_owned()),
            has_author: Set(dto.has_author.to_owned()),
            has_summary: Set(dto.has_summary.to_owned()),
            has_seo: Set(dto.has_seo.to_owned()),
            has_images: Set(dto.has_images.to_owned()),
            has_attachment: Set(dto.has_attachment.to_owned()),
            list_template_id: Set(dto.list_template_id.to_owned()),
            detail_template_id: Set(dto.detail_template_id.to_owned()),
            sort: Set(dto.sort.to_owned()),
            status: Set(dto.status.to_owned()),
            create_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let res = ContentModel::insert(model).exec(db).await?;
        Ok(res.last_insert_id)
    }

    /// 按id批量软删除
    /// * `db` 数据库链接
    /// * `ids` id数组
    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        let update_result: UpdateResult = ContentModel::update_many()
            .set(content_model::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(content_model::Column::Id.is_in(ids))
            .exec(db).await?;
        Ok(update_result.rows_affected as i64)
    }

    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, dto: &ContentModelSaveDTO) -> Result<i64, DbErr> {
        let model = content_model::ActiveModel {
            model_code: Set(dto.model_code.to_owned()),
            model_name: Set(dto.model_name.to_owned()),
            model_icon: Set(dto.model_icon.to_owned()),
            description: Set(dto.description.to_owned()),
            has_title: Set(dto.has_title.to_owned()),
            has_content: Set(dto.has_content.to_owned()),
            has_cover: Set(dto.has_cover.to_owned()),
            has_author: Set(dto.has_author.to_owned()),
            has_summary: Set(dto.has_summary.to_owned()),
            has_seo: Set(dto.has_seo.to_owned()),
            has_images: Set(dto.has_images.to_owned()),
            has_attachment: Set(dto.has_attachment.to_owned()),
            list_template_id: Set(dto.list_template_id.to_owned()),
            detail_template_id: Set(dto.detail_template_id.to_owned()),
            sort: Set(dto.sort.to_owned()),
            status: Set(dto.status.to_owned()),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let result: UpdateResult = ContentModel::update_many()
            .set(model)
            .filter(content_model::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db).await?;
        Ok(result.rows_affected as i64)
    }

    /// 按id查询
    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<content_model::Model>, DbErr> {
        let result = ContentModel::find_by_id(id.clone().unwrap_or_default())
            .filter(content_model::Column::Deleted.eq(0))
            .one(db).await?;
        Ok(result)
    }

    /// 按编码查询
    pub async fn find_by_code(db: &DbConn, code: &Option<String>) -> Result<Option<content_model::Model>, DbErr> {
        let result = ContentModel::find()
            .filter(content_model::Column::Deleted.eq(0))
            .filter(content_model::Column::ModelCode.eq(code.clone().unwrap_or_default()))
            .one(db).await?;
        Ok(result)
    }

    pub async fn select_in_page(db: &DbConn, page_num: i64, page_size: i64, where_case: PageWhere) -> Result<(Vec<content_model::Model>, u64), DbErr> {
        let paginator = ContentModel::find()
            .filter(content_model::Column::Deleted.eq(0))
            .apply_if(where_case.model_name.clone(), |query, v| {
                query.filter(content_model::Column::ModelName.contains(v))
            })
            .apply_if(where_case.status.clone(), |query, v| {
                query.filter(content_model::Column::Status.eq(v))
            })
            .order_by_desc(content_model::Column::Sort)
            .paginate(db, page_size as u64);
        let total = paginator.num_items().await?;
        let list = paginator.fetch_page(page_num as u64).await?;
        Ok((list, total))
    }

    pub async fn select_count(db: &DbConn, where_case: PageWhere) -> Result<i64, DbErr> {
        let count = ContentModel::find()
            .filter(content_model::Column::Deleted.eq(0))
            .apply_if(where_case.model_name.clone(), |query, v| {
                query.filter(content_model::Column::ModelName.contains(v))
            })
            .apply_if(where_case.status.clone(), |query, v| {
                query.filter(content_model::Column::Status.eq(v))
            })
            .count(db).await? as i64;
        Ok(count)
    }
}
