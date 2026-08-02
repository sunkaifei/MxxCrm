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
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::articles::entity::{category, category::Entity as Category};
use crate::modules::system::entity::dept;
use crate::utils::string_utils::{u64_to_string, deserialize_string_to_u64, serialize_option_u64_to_string, deserialize_string_to_i32};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CategorySaveRequest {
    // 父分类ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub parent_id: Option<i64>,
    // 短网址
    pub short_url: Option<String>,
    // 文章分类名称
    pub category_name: Option<String>,
    // 排序
    pub sort: Option<i32>,
    //导航是否显示
    pub is_show: Option<i32>,
    //审核状态，0未审核，1已审核
    pub status: Option<i32>,
    // 页面模式：1=封面模式，2=列表模式
    pub page_type: Option<i32>,
    // 封面模板数据ID
    pub page_template_data_id: Option<i64>,
    // 栏目Banner图片
    pub banner_image: Option<String>,
    // 栏目简介
    pub description: Option<String>,
    // 内容类型：1=文章，2=产品(预留)，3=自定义链接
    pub content_type: Option<i32>,
    // 自定义链接URL
    pub link_url: Option<String>,
    //SEO标题
    pub seo_title: Option<String>,
    //SEO关键词
    pub seo_keywords: Option<String>,
    //SEO描述
    pub seo_description: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct CategorySaveDTO {
    // 帖子分类ID
    pub id: Option<i64>,
    // 父分类ID
    pub parent_id: Option<i64>,
    // 短网址
    pub short_url: Option<String>,
    // 网站ID
    pub website_id: Option<i64>,
    // 文章分类名称
    pub category_name: Option<String>,
    // 排序
    pub sort: Option<i32>,
    //导航是否显示
    pub is_show: Option<i32>,
    //审核状态，0未审核，1已审核
    pub status: Option<i32>,
    // 页面模式：1=封面模式，2=列表模式
    pub page_type: Option<i32>,
    // 封面模板数据ID
    pub page_template_data_id: Option<i64>,
    // 栏目Banner图片
    pub banner_image: Option<String>,
    // 栏目简介
    pub description: Option<String>,
    // 内容类型：1=文章，2=产品(预留)，3=自定义链接
    pub content_type: Option<i32>,
    // 自定义链接URL
    pub link_url: Option<String>,
    //SEO标题
    pub seo_title: Option<String>,
    //SEO关键词
    pub seo_keywords: Option<String>,
    //SEO描述
    pub seo_description: Option<String>,
}

impl From<CategorySaveRequest> for CategorySaveDTO {
    fn from(value: CategorySaveRequest) -> Self {
        CategorySaveDTO {
            id: None,
            parent_id: value.parent_id,
            short_url: value.short_url,
            website_id: None,
            category_name: value.category_name,
            sort: value.sort,
            is_show: value.is_show,
            status: value.status,
            page_type: value.page_type,
            page_template_data_id: value.page_template_data_id,
            banner_image: value.banner_image,
            description: value.description,
            content_type: value.content_type,
            link_url: value.link_url,
            seo_title: value.seo_title,
            seo_keywords: value.seo_keywords,
            seo_description: value.seo_description,
        }
    }

}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CategoryUpdateRequest {
    // 分类ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    // 父分类ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub parent_id: Option<i64>,
    // 短网址
    pub short_url: Option<String>,
    // 文章分类名称
    pub category_name: Option<String>,
    // 排序
    pub sort: Option<i32>,
    //导航是否显示
    pub is_show: Option<i32>,
    //审核状态，0未审核，1已审核
    pub status: Option<i32>,
    // 页面模式：1=封面模式，2=列表模式
    pub page_type: Option<i32>,
    // 封面模板数据ID
    pub page_template_data_id: Option<i64>,
    // 栏目Banner图片
    pub banner_image: Option<String>,
    // 栏目简介
    pub description: Option<String>,
    // 内容类型：1=文章，2=产品(预留)，3=自定义链接
    pub content_type: Option<i32>,
    // 自定义链接URL
    pub link_url: Option<String>,
    //SEO标题
    pub seo_title: Option<String>,
    //SEO关键词
    pub seo_keywords: Option<String>,
    //SEO描述
    pub seo_description: Option<String>,
}

impl From<CategoryUpdateRequest> for CategorySaveDTO {
    fn from(value: CategoryUpdateRequest) -> Self {
        CategorySaveDTO {
            id: value.id,
            parent_id: value.parent_id,
            short_url: value.short_url,
            website_id: None,
            category_name: value.category_name,
            sort: value.sort,
            is_show: value.is_show,
            status: value.status,
            page_type: value.page_type,
            page_template_data_id: value.page_template_data_id,
            banner_image: value.banner_image,
            description: value.description,
            content_type: value.content_type,
            link_url: value.link_url,
            seo_title: value.seo_title,
            seo_keywords: value.seo_keywords,
            seo_description: value.seo_description,
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CategoryDetailVO {
    // 帖子分类ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    // 父分类ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub parent_id: Option<i64>,
    // 短网址
    pub short_url: Option<String>,
    // 网站ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub website_id: Option<i64>,
    // 帖子分类名称
    pub category_name: Option<String>,
    // 排序
    pub sort: Option<i32>,
    // 统计帖子
    pub count_topic: Option<i32>,
    // 导航是否显示
    pub is_show: Option<i32>,
    // 审核状态，0未审核，1已审核
    pub status: Option<i32>,
    // 页面模式：1=封面模式，2=列表模式
    pub page_type: Option<i32>,
    // 封面模板数据ID
    pub page_template_data_id: Option<i64>,
    // 栏目Banner图片
    pub banner_image: Option<String>,
    // 栏目简介
    pub description: Option<String>,
    // 内容类型：1=文章，2=产品(预留)，3=自定义链接
    pub content_type: Option<i32>,
    // 自定义链接URL
    pub link_url: Option<String>,
    //SEO标题
    pub seo_title: Option<String>,
    //SEO关键词
    pub seo_keywords: Option<String>,
    //SEO描述
    pub seo_description: Option<String>,
    // 添加时间
    pub create_time: Option<String>,
    // 更新时间
    pub update_time: Option<String>,
}

impl From<category::Model> for CategoryDetailVO {
    fn from(value: category::Model) -> Self {
        Self {
            id: Option::from(value.id),
            parent_id: value.parent_id,
            short_url: value.short_url,
            website_id: value.website_id,
            category_name: value.category_name,
            sort: value.sort,
            is_show: value.is_show,
            status: value.status,
            page_type: value.page_type,
            page_template_data_id: value.page_template_data_id,
            banner_image: value.banner_image,
            description: value.description,
            content_type: value.content_type,
            link_url: value.link_url,
            seo_title: value.seo_title,
            seo_keywords: value.seo_keywords,
            seo_description: value.seo_description,
            count_topic: value.count_topic,
            create_time: value.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: value.update_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CategoryPageRequest {
    // 分类名称
    pub category_name: Option<String>,
    //导航是否显示
    pub is_show: Option<String>,
    //审核状态，0未审核，1已审核
    pub status: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct CategoryPageDTO {
    // 分类名称
    pub category_name: Option<String>,
    // 网站ID
    pub website_id: Option<i64>,
    //导航是否显示
    pub is_show: Option<String>,
    //审核状态，0未审核，1已审核
    pub status: Option<String>,
}

impl From<CategoryPageRequest> for CategoryPageDTO {
    fn from(value: CategoryPageRequest) -> Self {
        Self {
            category_name: value.category_name,
            website_id: None,
            is_show: value.is_show,
            status: value.status
        }
    }

}


///文章分类树
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CategoryOptionsVO {
    #[serde(serialize_with = "u64_to_string")]
    pub value: i64,
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<CategoryOptionsVO>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CategoryTreeVO {
    // 帖子分类ID
    #[serde(serialize_with = "u64_to_string")]
    pub id: i64,
    // 父分类ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub parent_id: Option<i64>,
    // 短网址
    pub short_url: Option<String>,
    // 网站ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub website_id: Option<i64>,
    // 帖子分类名称
    pub category_name: Option<String>,
    // 排序
    pub sort: Option<i32>,
    // 统计帖子
    pub count_topic: Option<i32>,
    // 添加时间
    pub create_time: Option<String>,
    // 导航是否显示
    pub is_show: Option<i32>,
    // 审核状态，0未审核，1已审核
    pub status: Option<i32>,
    // 页面模式：1=封面模式，2=列表模式
    pub page_type: Option<i32>,
    // 封面模板数据ID
    pub page_template_data_id: Option<i64>,
    // 栏目Banner图片
    pub banner_image: Option<String>,
    // 栏目简介
    pub description: Option<String>,
    // 内容类型：1=文章，2=产品(预留)，3=自定义链接
    pub content_type: Option<i32>,
    // 自定义链接URL
    pub link_url: Option<String>,
    //SEO标题
    pub seo_title: Option<String>,
    //SEO关键词
    pub seo_keywords: Option<String>,
    //SEO描述
    pub seo_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<CategoryTreeVO>>,
}


pub struct CategoryModel;

impl CategoryModel {

    pub async fn insert(db: &DbConn, form_data: &CategorySaveDTO)  -> Result<i64, DbErr> {
        let payload = category::ActiveModel {
            //id: Set(form_data.id.to_owned()),
            parent_id: Set(form_data.parent_id.to_owned()),
            short_url: Set(form_data.short_url.to_owned()),
            website_id: Set(form_data.website_id.to_owned()),
            category_name: Set(form_data.category_name.to_owned()),
            sort: Set(form_data.sort.to_owned()),
            is_show: Set(form_data.is_show.to_owned()),
            status: Set(form_data.status.to_owned()),
            page_type: Set(form_data.page_type.to_owned()),
            page_template_data_id: Set(form_data.page_template_data_id.to_owned()),
            banner_image: Set(form_data.banner_image.to_owned()),
            description: Set(form_data.description.to_owned()),
            content_type: Set(form_data.content_type.to_owned()),
            link_url: Set(form_data.link_url.to_owned()),
            seo_title: Set(form_data.seo_title.to_owned()),
            seo_keywords: Set(form_data.seo_keywords.to_owned()),
            seo_description: Set(form_data.seo_description.to_owned()),
            create_time:     Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            update_time:     Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        Category::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id as i64)
    }

    /// 删除网站id下全部
    pub async fn batch_delete_by_id(db: &DbConn, website_id: &Option<i64>) -> Result<i64, DbErr> {
        Category::delete_many()
            .filter(category::Column::WebsiteId.eq(website_id.clone().unwrap_or_default()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }
    
    /// 批量删除
    pub async fn batch_delete_by_ids(db: &DbConn, website_id: &Option<i64>, ids: Vec<i64>) -> Result<i64, DbErr> {
        Category::delete_many()
            .filter(category::Column::Id.is_in(ids))
            .filter(category::Column::WebsiteId.eq(website_id.clone().unwrap_or_default()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }
    
    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, form_data: &CategorySaveDTO) -> Result<i64, DbErr> {
        let update_result = Category::update_many().set(
            category::ActiveModel {
                parent_id:     Set(form_data.parent_id.to_owned()),
                short_url:     Set(form_data.short_url.to_owned()),
                category_name: Set(form_data.category_name.to_owned()),
                sort:          Set(form_data.sort.to_owned()),
                is_show:       Set(form_data.is_show.to_owned()),
                status:        Set(form_data.status.to_owned()),
                page_type:     Set(form_data.page_type.to_owned()),
                page_template_data_id: Set(form_data.page_template_data_id.to_owned()),
                banner_image:  Set(form_data.banner_image.to_owned()),
                description:   Set(form_data.description.to_owned()),
                content_type:  Set(form_data.content_type.to_owned()),
                link_url:      Set(form_data.link_url.to_owned()),
                seo_title:     Set(form_data.seo_title.to_owned()),
                seo_keywords:  Set(form_data.seo_keywords.to_owned()),
                seo_description: Set(form_data.seo_description.to_owned()),
                update_time:   Set(Option::from(chrono::Local::now().naive_local().to_owned())),
                ..Default::default()
            }
        ).filter(category::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }
    

    /// 查询分类名称是否重复
    pub async fn find_by_name_unique(
        db: &DbConn, 
        website_id: &Option<i64>, 
        category_name: &Option<String>,
        id: &Option<i64>
    ) -> Result<i64, DbErr> {
        Category::find()
            .filter(category::Column::CategoryName.eq(category_name.clone().unwrap_or_default()))
            .filter(category::Column::WebsiteId.eq(website_id.clone().unwrap_or_default()))
            .apply_if(id.clone(), |query, v| {
                query.filter(dept::Column::Id.ne(v))
            })
            .count(db)
            .await
            .map(|c| c as i64)
    }

    /// 查询短网址是否重复
    pub async fn find_by_short_url_unique(db: &DbConn, short_url: &String) -> Result<i64, DbErr> {
        Category::find()
            .filter(category::Column::ShortUrl.eq(short_url.clone()))
            .count(db)
            .await
            .map(|c| c as i64)
    }
    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<category::Model>, DbErr> {
        Category::find_by_id(id.clone().unwrap_or_default())
            .one(db)
            .await
    }

    pub async fn find_by_short_url(db: &DbConn, short_url: String) -> Result<Option<category::Model>, DbErr> {
        Category::find()
            .filter(category::Column::ShortUrl.eq(short_url))
            .one(db)
            .await
    }

    pub async fn find_all(db: &DbConn, website_id: i64) -> Result<Vec<category::Model>, DbErr> {
        Category::find()
            .filter(category::Column::WebsiteId.eq(website_id))
            .order_by_desc(category::Column::Sort)
            .all(db)
            .await
    }

}
