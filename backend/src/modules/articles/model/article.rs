//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use std::fmt::{Debug};
use chrono::{Local, NaiveDateTime};
use sea_orm::*;
use sea_query::{Alias, Expr, Query};
use crate::core::errors::error::{Error,Result};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::articles::entity::{article, article::Entity as Article};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string, deserialize_string_to_i32};


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ArticlesSaveRequest {
    //文章分类ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub category_id: Option<i64>,
    //文章标题
    pub title: Option<String>,
    //简短标题
    pub short_title: Option<String>,
    //文章主图
    pub title_image: Option<String>,
    //文章作者
    pub author: Option<String>,
    //原文链接
    pub original_link: Option<String>,
    //文章摘要
    pub description: Option<String>,
    //文章内容
    pub content: Option<String>,
    //是否置顶
    pub istop: Option<i32>,
    //1为推荐
    pub isrecommend: Option<i32>,
    //0未审核，1审核，2未通过
    pub status: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArticlesSaveDTO {
    /// 网站id
    pub website_id: Option<i64>,
    //文章ID
    pub id: Option<i64>,
    /// 用户id
    pub user_id: Option<i64>,
    //短网址
    pub short_url: Option<String>,
    //文章分类ID
    pub category_id: Option<i64>,
    //文章标题
    pub title: Option<String>,
    //简短标题
    pub short_title: Option<String>,
    //文章主图
    pub title_image: Option<String>,
    //文章作者
    pub author: Option<String>,
    //原文链接
    pub original_link: Option<String>,
    //文章摘要
    pub description: Option<String>,
    //文章内容
    pub content: Option<String>,
    //是否置顶
    pub istop: Option<i32>,
    //1为推荐
    pub isrecommend: Option<i32>,
    //0未审核，1审核，2未通过
    pub status: Option<i32>,
}


impl From<ArticlesSaveRequest> for ArticlesSaveDTO {
    fn from(arg: ArticlesSaveRequest) -> Self {
        Self {
            id: None,
            website_id: None,
            short_url: None,
            category_id: arg.category_id,
            title: arg.title,
            short_title: arg.short_title,
            title_image: arg.title_image,
            author: arg.author,
            original_link: arg.original_link,
            description: arg.description,
            content: arg.content,
            istop: arg.istop,
            isrecommend: arg.isrecommend,
            status: arg.status,
            user_id: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArticlesUpdateRequest {
    //文章ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    //文章分类ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub category_id: Option<i64>,
    //短网址
    pub short_url: Option<String>,
    //文章标题
    pub title: Option<String>,
    //简短标题
    pub short_title: Option<String>,
    //文章主图
    pub title_image: Option<String>,
    //文章作者
    pub author: Option<String>,
    //原文链接
    pub original_link: Option<String>,
    //文章摘要
    pub description: Option<String>,
    //文章内容
    pub content: Option<String>,
    //是否置顶
    pub istop: Option<i32>,
    //1为推荐
    pub isrecommend: Option<i32>,
    //0未审核，1审核，2未通过
    pub status: Option<i32>,
}

impl From<ArticlesUpdateRequest> for ArticlesSaveDTO {
    fn from(arg: ArticlesUpdateRequest) -> Self {
        Self {
            id: arg.id,
            website_id: None,
            user_id: None,
            short_url: arg.short_url,
            category_id: arg.category_id,
            title: arg.title,
            short_title: arg.short_title,
            title_image: arg.title_image,
            author: arg.author,
            original_link: arg.original_link,
            description: arg.description,
            content: arg.content,
            istop: arg.istop,
            isrecommend: arg.isrecommend,
            status: arg.status,
        }
    }
}



impl From<ArticlesSaveDTO> for article::Model {
    fn from(arg: ArticlesSaveDTO) -> Self {
        Self {
            id: arg.id.unwrap_or_default(),
            user_id: arg.user_id,
            short_url: arg.short_url,
            category_id: arg.category_id,
            title: arg.title,
            short_title: arg.short_title,
            title_image: arg.title_image,
            author: arg.author,
            original_link: arg.original_link,
            description: arg.description,
            content: arg.content,
            count_comment: None,
            count_view: None,
            count_love: None,
            count_digg: None,
            count_burys: None,
            count_follow: None,
            istop: arg.istop,
            isclose: None,
            iscomment: None,
            iscommentshow: None,
            isposts: None,
            isaudit: None,
            deleted: None,
            isrecommend: None,
            status: Some(1),
            create_time: Some(Local::now().naive_local()),
            update_time: Some(Local::now().naive_local()),
        }
    }
    
    
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArticleDetailVO {
    //文章ID
    pub id: Option<String>,
    /// 用户id
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub user_id: Option<i64>,
    //短网址
    pub short_url: Option<String>,
    //文章分类ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub category_id:Option<i64>,
    //文章标题
    pub title: Option<String>,
    //简短标题
    pub short_title: Option<String>,
    //原文链接
    pub original_link: Option<String>,
    //文章主图
    pub title_image: Option<String>,
    //文章作者
    pub author: Option<String>,
    //文章摘要
    pub description: Option<String>,
    //文章内容
    pub content: Option<String>,
    /// 文章是否置顶
    pub istop: Option<i32>,
    /// 文章是否推荐
    pub isrecommend: Option<i32>,
    //0未审核，1审核，2未通过
    pub status: Option<i32>,
    //创建时间
    pub create_time: Option<String>,
}

impl From<article::Model> for ArticleDetailVO{
    fn from(arg: article::Model) -> Self {
        Self {
            id: Some(arg.id.to_string()),
            user_id: arg.user_id,
            short_url: arg.short_url,
            category_id: arg.category_id,
            title: arg.title,
            short_title: arg.short_title,
            original_link: arg.original_link,
            title_image: arg.title_image,
            author: arg.author,
            description: arg.description,
            content: arg.content,
            istop: arg.istop,
            isrecommend: arg.isrecommend,
            status: arg.status,
            create_time: arg.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ArticleListVO {
    /// 文章id
    pub id: Option<String>,
    /// 网站id
    pub site_id: Option<i64>,
    /// 文章标题
    pub title: Option<String>,
    /// 按文章分类ID查询
    pub category_id: Option<i64>,
    /// 文章是否置顶
    pub istop: Option<i32>,
    /// 文章是否推荐
    pub isrecommend: Option<i32>,
    /// 文章状态
    pub status: Option<i32>,
    /// 创建时间
    pub create_time: Option<String>,
}

impl From<article::Model> for ArticleListVO{
    fn from(arg: article::Model) -> Self {
        Self {
            id: Some(arg.id.to_string()),
            site_id: None,
            title: arg.title,
            category_id: arg.category_id,
            istop: arg.istop,
            isrecommend: arg.isrecommend,
            status: arg.status,
            create_time: arg.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}


/// 后台查询文章请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct QueryPageRequest {
    pub title: Option<String>,
    pub website_id: Option<i64>,
    pub category_id: Option<i64>,
    pub status: Option<i32>,
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryPageDTO {
    pub website_id: Option<i64>,
    pub category_id: Option<i64>,
    pub title: Option<String>,
    pub status: Option<i32>,
    pub page_num: i64,
    pub page_size: i64,
}

impl From<QueryPageRequest> for QueryPageDTO {
    fn from(arg: QueryPageRequest) -> Self {
        Self {
            website_id: arg.website_id,
            category_id: arg.category_id,
            title: arg.title,
            status: arg.status,
            page_num: arg.page_num.unwrap_or_default(),
            page_size: arg.page_size.unwrap_or_default(),
        }
    }
    
}

/// 查询标题唯一性
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QueryTitleUnique {
    pub id: Option<i64>,
    pub title: Option<String>,
    pub website_id: Option<i64>,
}

/// 查询短网址是否唯一性
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QueryShortUrlUnique {
    pub id: Option<i64>,
    pub short_url: Option<String>,
    pub website_id: Option<i64>,
}


pub enum ArticleTable {
    Table(String),
    Id(String),
    UserId(String),
    ShortUrl(String),
    CategoryId(String),
    Title(String),
    ShortTitle(String),
    TitleImage(String),
    Author(String),
    OriginalLink(String),
    Content(String),
    Description(String),
    CountView(String),
    CountLove(String),
    CountDigg(String),
    CountBurys(String),
    Istop(String),
    Isrecommend(String),
    Status(String),
    CreatedTime(String),
    UpdateTime(String),
}

impl Iden for ArticleTable {
    fn unquoted(&self) -> &str {
        match self {
            Self::Table(table_name) => table_name,
            Self::Id(column_name) => column_name,
            Self::UserId(column_name) => column_name,
            Self::ShortUrl(column_name) => column_name,
            Self::CategoryId(column_name) => column_name,
            Self::Title(column_name) => column_name,
            Self::ShortTitle(column_name) => column_name,
            Self::TitleImage(column_name) => column_name,
            Self::Author(column_name) => column_name,
            Self::OriginalLink(column_name) => column_name,
            Self::Content(column_name) => column_name,
            Self::Description(column_name) => column_name,
            Self::CountView(column_name) => column_name,
            Self::CountLove(column_name) => column_name,
            Self::CountDigg(column_name) => column_name,
            Self::CountBurys(column_name) => column_name,
            Self::Istop(column_name) => column_name,
            Self::Isrecommend(column_name) => column_name,
            Self::Status(column_name) => column_name,
            Self::CreatedTime(column_name) => column_name,
            Self::UpdateTime(column_name) => column_name,
        }
    }
}
pub struct ArticleModel;

impl ArticleModel {

    /// 创建文章表
    pub async fn create_tables(db: &DatabaseConnection, website_id: i64) -> Result<()> {
        let table_name = format!("mxx_article_{}", website_id);
        let sql = format!(
            r#"CREATE TABLE IF NOT EXISTS "{}" (
                "id" BIGINT NOT NULL PRIMARY KEY,
                "user_id" BIGINT,
                "short_url" VARCHAR(160) NOT NULL,
                "category_id" BIGINT DEFAULT 0,
                "title" VARCHAR(180) NOT NULL,
                "short_title" VARCHAR(160),
                "title_image" VARCHAR(255),
                "author" VARCHAR(100),
                "original_link" VARCHAR(255),
                "content" TEXT,
                "description" VARCHAR(280),
                "count_view" INTEGER DEFAULT 0,
                "count_love" INTEGER DEFAULT 0,
                "count_digg" INTEGER DEFAULT 0,
                "count_burys" INTEGER DEFAULT 0,
                "istop" SMALLINT DEFAULT 1,
                "isrecommend" SMALLINT DEFAULT 0,
                "status" SMALLINT DEFAULT 1,
                "create_time" TIMESTAMP,
                "update_time" TIMESTAMP
            )"#,
            table_name
        );
        db.execute_unprepared(&sql).await.map_err(|e| Error::from(format!("创建文章表失败: {:?}", e)))?;
        Ok(())
    }
    
    /// 插入文章
    pub async fn insert(db: &DbConn, item: ArticlesSaveDTO) -> Result<i64> {
        let payload = article::ActiveModel {
            id: Set(item.id.unwrap_or_else(|| chrono::Local::now().timestamp_millis() as i64)),
            user_id: Set(item.user_id),
            short_url: Set(item.short_url),
            category_id: Set(item.category_id),
            title: Set(item.title),
            short_title: Set(item.short_title),
            title_image: Set(item.title_image),
            author: Set(item.author),
            original_link: Set(item.original_link),
            description: Set(item.description),
            content: Set(item.content),
            count_comment: Set(Some(0)),
            count_view: Set(Some(0)),
            count_love: Set(Some(0)),
            count_digg: Set(Some(0)),
            count_burys: Set(Some(0)),
            count_follow: Set(Some(0)),
            istop: Set(item.istop),
            isclose: Set(Some(0)),
            iscomment: Set(Some(1)),
            iscommentshow: Set(Some(0)),
            isposts: Set(Some(0)),
            isaudit: Set(Some(0)),
            deleted: Set(Some(0)),
            isrecommend: Set(item.isrecommend),
            status: Set(Some(item.status.unwrap_or(0) as i32)),
            create_time: Set(Some(Local::now().naive_utc())),
            update_time: Set(Some(Local::now().naive_utc())),
            ..Default::default()
        };

        Article::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
            .map_err(|e| Error::from(format!("msg={},code=500", e)))
    }

    /// 删除文章表
    pub async fn delete_table(db: &DbConn, website_id: i64) -> Result<bool> {
        let table_name = format!("mxx_article_{}", website_id);
        let drop_table_stmt = sea_query::Table::drop()
            .table(Alias::new(&table_name))
            .if_exists()
            .to_owned();

        db.execute(&drop_table_stmt)
            .await
            .map(|_| true)
            .map_err(|e| Error::from(format!("msg={},code=500", e)))
    }

    /// 批量删除文章
    pub async fn batch_delete_by_ids(db: &DbConn, _website_id: i64, ids: Vec<i64>) -> Result<i64> {
        Article::delete_many()
            .filter(article::Column::Id.is_in(ids))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
            .map_err(|e| Error::from(format!("msg={},code=500", e)))
    }

    /// 按id更新文章内容
    /// ``` id 文章id 这里不把id放到DTO里是怕忘了传值```
    pub async fn update_by_id(db: &DbConn, id: i64, item: ArticlesSaveDTO) -> Result<i64> {
        let payload = article::ActiveModel {
            user_id: Set(item.user_id),
            short_url: Set(item.short_url),
            category_id: Set(item.category_id),
            title: Set(item.title),
            short_title: Set(item.short_title),
            title_image: Set(item.title_image),
            author: Set(item.author),
            original_link: Set(item.original_link),
            description: Set(item.description),
            content: Set(item.content),
            istop: Set(item.istop),
            isrecommend: Set(item.isrecommend),
            status: Set(Some(item.status.unwrap_or(0) as i32)),
            update_time: Set(Some(Local::now().naive_utc())),
            ..Default::default()
        };

        Article::update_many()
            .set(payload)
            .filter(article::Column::Id.eq(id))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
            .map_err(|e| Error::from(format!("msg={},code=500", e)))
    }

    /// 查询文章标题是否重复
    pub async fn find_by_title_unique(db: &DbConn, item: &QueryTitleUnique) -> Result<i64> {
        let count = Article::find()
            .filter(article::Column::Title.eq(item.title.clone().unwrap_or_default()))
            .filter(article::Column::Id.ne(item.id.unwrap_or_default()))
            .count(db)
            .await
            .map(|c| c as i64)
            .map_err(|e| Error::from(format!("msg={},code=500", e)))?;
        Ok(count)
    }

    /// 查询文章短链接是否重复
    pub async fn find_by_short_url_unique(db: &DbConn, item: &QueryShortUrlUnique) -> Result<i64> {
        let count = Article::find()
            .filter(article::Column::ShortUrl.eq(item.short_url.clone().unwrap_or_default()))
            .filter(article::Column::Id.ne(item.id.unwrap_or_default()))
            .count(db)
            .await
            .map(|c| c as i64)
            .map_err(|e| Error::from(format!("msg={},code=500", e)))?;
        Ok(count)
    }

    /// 按id查询文章详情
    pub async fn find_by_id(db: &DbConn, _website_id: i64, id: i64) -> Result<Option<ArticleDetailVO>> {
        Article::find_by_id(id)
            .one(db)
            .await
            .map(|model| model.map(ArticleDetailVO::from))
            .map_err(|e| Error::from(format!("msg={},code=500", e)))
    }

    /// 按短链接查询文章详情
    pub async fn find_by_short_url(db: &DbConn, _website_id: &Option<i64>, short_url: &Option<String>) -> Result<Option<ArticleDetailVO>> {
        Article::find()
            .filter(article::Column::ShortUrl.eq(short_url.clone().unwrap_or_default()))
            .one(db)
            .await
            .map(|model| model.map(ArticleDetailVO::from))
            .map_err(|e| Error::from(format!("msg={},code=500", e)))
    }

    /// 查询文章总数
    pub async fn get_by_count(db: &DbConn, item: &QueryPageDTO) -> Result<i64> {
        let mut query = Article::find();

        if let Some(title) = &item.title {
            if !title.is_empty() {
                query = query.filter(article::Column::Title.like(format!("%{title}%")));
            }
        }

        if let Some(category_id) = &item.category_id {
            if *category_id > 0 {
                query = query.filter(article::Column::CategoryId.eq(*category_id));
            }
        }

        if let Some(status) = &item.status {
            if *status >= 0 {
                query = query.filter(article::Column::Status.eq(*status as i32));
            }
        }

        query
            .count(db)
            .await
            .map(|c| c as i64)
            .map_err(|e| Error::from(format!("msg={},code=500", e)))
    }

    /// 分页查询文章列表
    pub async fn get_by_page(db: &DbConn, item: &QueryPageDTO) -> Result<Vec<ArticleListVO>> {
        let mut query = Article::find();

        if let Some(title) = &item.title {
            if !title.is_empty() {
                query = query.filter(article::Column::Title.like(format!("%{title}%")));
            }
        }

        if let Some(category_id) = &item.category_id {
            if *category_id > 0 {
                query = query.filter(article::Column::CategoryId.eq(*category_id));
            }
        }

        if let Some(status) = &item.status {
            if *status >= 0 {
                query = query.filter(article::Column::Status.eq(*status as i32));
            }
        }

        let page = item.page_num;
        let page_size = item.page_size;

        query
            .order_by_desc(article::Column::CreateTime)
            .limit(page_size as u64)
            .offset(((page - 1) * page_size) as u64)
            .all(db)
            .await
            .map(|models| models.into_iter().map(ArticleListVO::from).collect())
            .map_err(|e| Error::from(format!("msg={},code=500", e)))
    }
}