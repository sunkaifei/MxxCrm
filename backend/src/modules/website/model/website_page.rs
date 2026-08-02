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
use crate::modules::website::entity::{website_page, website_page::Entity as WebsitePage};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};
use sea_orm::*;


#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PageSaveRequest {
    /// 页面编码
    pub page_code: Option<String>,
    /// 页面名称
    pub page_name: Option<String>,
    /// 页面标题
    pub page_title: Option<String>,
    /// 页面内容
    pub page_content: Option<String>,
    /// SEO关键词
    pub seo_keywords: Option<String>,
    /// SEO描述
    pub seo_description: Option<String>,
    /// 模板id
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub template_id: Option<i64>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
}

impl From<PageSaveRequest> for PageSaveDTO {
    fn from(req: PageSaveRequest) -> Self {
        PageSaveDTO {
            id: None,
            page_code: req.page_code,
            page_name: req.page_name,
            page_title: req.page_title,
            page_content: req.page_content,
            seo_keywords: req.seo_keywords,
            seo_description: req.seo_description,
            template_id: req.template_id,
            sort: req.sort,
            status: req.status,
        }
    }
}


#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PageUpdateRequest {
    /// 主键ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 页面编码
    pub page_code: Option<String>,
    /// 页面名称
    pub page_name: Option<String>,
    /// 页面标题
    pub page_title: Option<String>,
    /// 页面内容
    pub page_content: Option<String>,
    /// SEO关键词
    pub seo_keywords: Option<String>,
    /// SEO描述
    pub seo_description: Option<String>,
    /// 模板id
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub template_id: Option<i64>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
}

impl From<PageUpdateRequest> for PageSaveDTO {
    fn from(req: PageUpdateRequest) -> Self {
        PageSaveDTO {
            id: req.id,
            page_code: req.page_code,
            page_name: req.page_name,
            page_title: req.page_title,
            page_content: req.page_content,
            seo_keywords: req.seo_keywords,
            seo_description: req.seo_description,
            template_id: req.template_id,
            sort: req.sort,
            status: req.status,
        }
    }
}


#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PageSaveDTO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 页面编码
    pub page_code: Option<String>,
    /// 页面名称
    pub page_name: Option<String>,
    /// 页面标题
    pub page_title: Option<String>,
    /// 页面内容
    pub page_content: Option<String>,
    /// SEO关键词
    pub seo_keywords: Option<String>,
    /// SEO描述
    pub seo_description: Option<String>,
    /// 模板id
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub template_id: Option<i64>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PageListVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 页面编码
    pub page_code: Option<String>,
    /// 页面名称
    pub page_name: Option<String>,
    /// 页面标题
    pub page_title: Option<String>,
    /// 页面内容
    pub page_content: Option<String>,
    /// SEO关键词
    pub seo_keywords: Option<String>,
    /// SEO描述
    pub seo_description: Option<String>,
    /// 模板id
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub template_id: Option<i64>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
    /// 创建时间
    pub create_time: Option<String>,
    /// 更新时间
    pub update_time: Option<String>,
}

impl From<website_page::Model> for PageListVO {
    fn from(model: website_page::Model) -> Self {
        PageListVO {
            id: Option::from(model.id),
            page_code: model.page_code,
            page_name: model.page_name,
            page_title: model.page_title,
            page_content: model.page_content,
            seo_keywords: model.seo_keywords,
            seo_description: model.seo_description,
            template_id: model.template_id,
            sort: model.sort,
            status: model.status,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: model.update_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PageDetailVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 页面编码
    pub page_code: Option<String>,
    /// 页面名称
    pub page_name: Option<String>,
    /// 页面标题
    pub page_title: Option<String>,
    /// 页面内容
    pub page_content: Option<String>,
    /// SEO关键词
    pub seo_keywords: Option<String>,
    /// SEO描述
    pub seo_description: Option<String>,
    /// 模板id
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub template_id: Option<i64>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
    /// 创建时间
    pub create_time: Option<String>,
    /// 更新时间
    pub update_time: Option<String>,
}

impl From<website_page::Model> for PageDetailVO {
    fn from(model: website_page::Model) -> Self {
        PageDetailVO {
            id: Option::from(model.id),
            page_code: model.page_code,
            page_name: model.page_name,
            page_title: model.page_title,
            page_content: model.page_content,
            seo_keywords: model.seo_keywords,
            seo_description: model.seo_description,
            template_id: model.template_id,
            sort: model.sort,
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
    /// 状态：0停用，1正常
    pub status: Option<i32>,
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Clone)]
pub struct PageWhere {
    pub keywords: Option<String>,
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

        let mut status = None;
        if self.status == Some(1) || self.status == Some(0) {
            status = self.status;
        }

        Self {
            keywords,
            status,
        }
    }
}

pub struct WebsitePageModel;

impl WebsitePageModel {
    pub async fn insert(db: &DbConn, dto: &PageSaveDTO) -> Result<i64, DbErr> {
        let model = website_page::ActiveModel {
            page_code: Set(dto.page_code.to_owned()),
            page_name: Set(dto.page_name.to_owned()),
            page_title: Set(dto.page_title.to_owned()),
            page_content: Set(dto.page_content.to_owned()),
            seo_keywords: Set(dto.seo_keywords.to_owned()),
            seo_description: Set(dto.seo_description.to_owned()),
            template_id: Set(dto.template_id.to_owned()),
            sort: Set(dto.sort.to_owned()),
            status: Set(dto.status.to_owned()),
            create_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let res = WebsitePage::insert(model).exec(db).await?;
        Ok(res.last_insert_id)
    }

    /// 按id批量软删除
    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        let result: UpdateResult = WebsitePage::update_many()
            .set(website_page::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(website_page::Column::Id.is_in(ids))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, dto: &PageSaveDTO) -> Result<i64, DbErr> {
        let model = website_page::ActiveModel {
            page_code: Set(dto.page_code.to_owned()),
            page_name: Set(dto.page_name.to_owned()),
            page_title: Set(dto.page_title.to_owned()),
            page_content: Set(dto.page_content.to_owned()),
            seo_keywords: Set(dto.seo_keywords.to_owned()),
            seo_description: Set(dto.seo_description.to_owned()),
            template_id: Set(dto.template_id.to_owned()),
            sort: Set(dto.sort.to_owned()),
            status: Set(dto.status.to_owned()),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let result: UpdateResult = WebsitePage::update_many()
            .set(model)
            .filter(website_page::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<website_page::Model>, DbErr> {
        let result = website_page::Entity::find_by_id(id.clone().unwrap_or_default())
            .filter(website_page::Column::Deleted.eq(0))
            .one(db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_code(
        db: &DbConn,
        code: &Option<String>,
    ) -> Result<Option<website_page::Model>, DbErr> {
        let result = website_page::Entity::find()
            .filter(website_page::Column::PageCode.eq(code.clone().unwrap_or_default()))
            .filter(website_page::Column::Deleted.eq(0))
            .filter(website_page::Column::Status.eq(1))
            .one(db)
            .await?;
        Ok(result)
    }

    pub async fn select_in_page(
        db: &DbConn,
        page_num: i64,
        page_size: i64,
        where_case: PageWhere,
    ) -> Result<(Vec<website_page::Model>, u64), DbErr> {
        let paginator = WebsitePage::find()
            .filter(website_page::Column::Deleted.eq(0))
            .apply_if(where_case.keywords.clone(), |query, v| {
                query.filter(website_page::Column::PageName.contains(v))
            })
            .apply_if(where_case.status, |query, v| {
                query.filter(website_page::Column::Status.eq(v))
            })
            .order_by_asc(website_page::Column::Sort)
            .paginate(db, page_size as u64);
        let total = paginator.num_items().await?;
        let list = paginator.fetch_page(page_num as u64).await?;
        Ok((list, total))
    }

    pub async fn select_count(db: &DbConn, where_case: PageWhere) -> Result<i64, DbErr> {
        let count = WebsitePage::find()
            .filter(website_page::Column::Deleted.eq(0))
            .apply_if(where_case.keywords.clone(), |query, v| {
                query.filter(website_page::Column::PageName.contains(v))
            })
            .apply_if(where_case.status, |query, v| {
                query.filter(website_page::Column::Status.eq(v))
            })
            .count(db)
            .await? as i64;
        Ok(count)
    }
}
