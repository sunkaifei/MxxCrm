//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究。
//!

use sea_orm::*;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::articles::entity::{article_revision, article_revision::Entity as ArticleRevision};

/// 文章修订历史列表查询
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RevisionListQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub article_id: Option<i64>,
}

/// 文章修订历史 VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArticleRevisionVO {
    pub id: Option<i64>,
    pub article_id: Option<i64>,
    pub revision_no: Option<i32>,
    pub title: Option<String>,
    pub short_title: Option<String>,
    pub title_image: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub snapshot: Option<String>,
    pub editor_id: Option<i64>,
    pub editor_name: Option<String>,
    pub edit_remark: Option<String>,
    pub create_time: Option<String>,
}

impl From<article_revision::Model> for ArticleRevisionVO {
    fn from(m: article_revision::Model) -> Self {
        Self {
            id: Some(m.id),
            article_id: Some(m.article_id),
            revision_no: Some(m.revision_no),
            title: m.title,
            short_title: m.short_title,
            title_image: m.title_image,
            author: m.author,
            description: m.description,
            content: m.content,
            snapshot: m.snapshot,
            editor_id: m.editor_id,
            editor_name: m.editor_name,
            edit_remark: m.edit_remark,
            create_time: m.create_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// 分页响应数据结构
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageResponse<T> {
    pub list: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

pub struct ArticleRevisionModel;

impl ArticleRevisionModel {
    /// 分页查询文章的修订历史，按创建时间倒序
    pub async fn find_by_article(
        db: &DbConn,
        article_id: i64,
        page: i64,
        page_size: i64,
    ) -> Result<PageResponse<ArticleRevisionVO>, DbErr> {
        let page = if page < 1 { 1 } else { page };
        let page_size = if page_size < 1 { 20 } else { page_size };
        let offset = (page - 1) * page_size;

        let select = ArticleRevision::find()
            .filter(article_revision::Column::ArticleId.eq(article_id));

        let total = select.clone().count(db).await? as i64;

        let list = select
            .order_by_desc(article_revision::Column::CreateTime)
            .offset(offset as u64)
            .limit(page_size as u64)
            .all(db)
            .await?;

        let vo_list: Vec<ArticleRevisionVO> = list.into_iter().map(ArticleRevisionVO::from).collect();

        Ok(PageResponse {
            list: vo_list,
            total,
            page: page as i32,
            page_size: page_size as i32,
        })
    }

    /// 按 id 查询修订记录
    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<ArticleRevisionVO>, DbErr> {
        ArticleRevision::find_by_id(id)
            .one(db)
            .await
            .map(|m| m.map(ArticleRevisionVO::from))
    }

    /// 插入修订记录（支持事务，传入 ConnectionTrait）
    pub async fn insert<C: ConnectionTrait>(
        db: &C,
        article_id: i64,
        revision_no: i32,
        title: Option<String>,
        short_title: Option<String>,
        title_image: Option<String>,
        author: Option<String>,
        description: Option<String>,
        content: Option<String>,
        snapshot: Option<String>,
        editor_id: Option<i64>,
        editor_name: Option<String>,
        edit_remark: Option<String>,
    ) -> Result<i64, DbErr> {
        let model = article_revision::ActiveModel {
            article_id: Set(article_id),
            revision_no: Set(revision_no),
            title: Set(title),
            short_title: Set(short_title),
            title_image: Set(title_image),
            author: Set(author),
            description: Set(description),
            content: Set(content),
            snapshot: Set(snapshot),
            editor_id: Set(editor_id),
            editor_name: Set(editor_name),
            edit_remark: Set(edit_remark),
            create_time: Set(Some(chrono::Local::now().naive_utc())),
            ..Default::default()
        };

        let result = ArticleRevision::insert(model).exec(db).await?;
        Ok(result.last_insert_id)
    }

    /// 获取指定文章的下一个修订号（当前最大 revision_no + 1，无记录时返回 1）
    pub async fn get_next_revision_no(db: &DbConn, article_id: i64) -> Result<i32, DbErr> {
        let latest = ArticleRevision::find()
            .filter(article_revision::Column::ArticleId.eq(article_id))
            .order_by_desc(article_revision::Column::RevisionNo)
            .one(db)
            .await?;

        Ok(latest.map(|m| m.revision_no + 1).unwrap_or(1))
    }
}
