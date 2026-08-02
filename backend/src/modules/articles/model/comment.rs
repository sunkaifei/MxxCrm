//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::utils::string_utils::{serialize_option_u64_to_string, deserialize_string_to_u64};
use sea_orm::*;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::articles::entity::{comment, comment::Entity as Comment};

/// 前台提交评论请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommentSaveRequest {
    /// 文章ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub article_id: Option<i64>,
    /// 评论内容
    pub content: Option<String>,
    /// 上级评论ID（回复）
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub refer_id: Option<i64>,
    /// 用户ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub user_id: Option<i64>,
}

/// 后台审核请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommentAdminUpdateRequest {
    /// 评论ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 0未审核，1已审核(通过)，2未通过
    pub status: Option<i32>,
    /// 0公开 1不公开
    pub ispublic: Option<i32>,
}

/// 评论列表展示
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommentListVO {
    /// 评论ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 文章ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub article_id: Option<i64>,
    /// 评论内容
    pub content: Option<String>,
    /// 0未审核，1已审核(通过)，2未通过
    pub status: Option<i32>,
    /// 楼层
    pub storey: Option<i32>,
    /// 创建时间
    pub create_time: Option<prelude::DateTime>,
    /// 用户ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub user_id: Option<i64>,
    /// 上级评论ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub refer_id: Option<i64>,
    /// 支持数
    pub count_digg: Option<i32>,
    /// 被评论数
    pub count_comment: Option<i32>,
}

impl From<comment::Model> for CommentListVO {
    fn from(arg: comment::Model) -> Self {
        Self {
            id: Option::from(arg.id),
            article_id: arg.article_id,
            content: arg.content,
            status: arg.status,
            storey: arg.storey,
            create_time: arg.create_time,
            user_id: arg.user_id,
            refer_id: arg.refer_id,
            count_digg: arg.count_digg,
            count_comment: arg.count_comment,
        }
    }
}

/// 评论详情
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommentDetailVO {
    /// 评论ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 文章ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub article_id: Option<i64>,
    /// 评论内容
    pub content: Option<String>,
    /// 0未审核，1已审核(通过)，2未通过
    pub status: Option<i32>,
    /// 楼层
    pub storey: Option<i32>,
    /// 创建时间
    pub create_time: Option<String>,
    /// 用户ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub user_id: Option<i64>,
    /// 上级评论ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub refer_id: Option<i64>,
    /// 支持数
    pub count_digg: Option<i32>,
    /// 被评论数
    pub count_comment: Option<i32>,
    /// 0公开 1不公开
    pub ispublic: Option<i32>,
}

impl From<comment::Model> for CommentDetailVO {
    fn from(arg: comment::Model) -> Self {
        Self {
            id: Option::from(arg.id),
            article_id: arg.article_id,
            content: arg.content,
            status: arg.status,
            storey: arg.storey,
            create_time: arg.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            user_id: arg.user_id,
            refer_id: arg.refer_id,
            count_digg: arg.count_digg,
            count_comment: arg.count_comment,
            ispublic: arg.ispublic,
        }
    }
}

/// 分页查询
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub article_id: Option<i64>,
    /// 0未审核，1已审核(通过)，2未通过
    pub status: Option<i32>,
    /// 关键字（按内容搜索）
    pub keywords: Option<String>,
}

/// 条件
#[derive(Clone)]
pub struct PageWhere {
    pub article_id: Option<i64>,
    /// 0未审核，1已审核(通过)，2未通过
    pub status: Option<i32>,
    pub keywords: Option<String>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut keywords = None;
        if self.keywords.as_ref().map_or(false, |s| !s.trim().is_empty()) {
            keywords = self.keywords.clone();
        }

        let mut status = None;
        if let Some(s) = self.status {
            if s == 0 || s == 1 || s == 2 {
                status = self.status;
            }
        }

        Self {
            article_id: self.article_id,
            status,
            keywords,
        }
    }
}

pub struct CommentModel;

impl CommentModel {
    /// 新增评论（create_time 设为当前时间，status 默认 0 待审核）
    pub async fn insert(db: &DbConn, form_data: &CommentSaveRequest) -> Result<i64, DbErr> {
        let payload = comment::ActiveModel {
            article_id:     Set(form_data.article_id.to_owned()),
            content:        Set(form_data.content.to_owned()),
            refer_id:       Set(form_data.refer_id.to_owned()),
            user_id:        Set(form_data.user_id.to_owned()),
            count_comment:  Set(Some(0).to_owned()),
            count_digg:     Set(Some(0).to_owned()),
            count_burys:    Set(Some(0).to_owned()),
            ispublic:       Set(Some(0).to_owned()),
            storey:         Set(Some(0).to_owned()),
            status:         Set(Some(0).to_owned()),
            deleted:        Set(Some(0).to_owned()),
            create_time:    Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        Comment::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id as i64)
    }

    /// 根据指定多个id批量假删除数据（设 deleted=1）
    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        let update_result: UpdateResult = Comment::update_many()
            .set(comment::ActiveModel {
                deleted: Set(Some(1).to_owned()),
                ..Default::default()
            })
            .filter(comment::Column::Id.is_in(ids))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// 更新审核状态
    pub async fn update_status(db: &DbConn, id: i64, status: i32) -> Result<i64, DbErr> {
        let update_result: UpdateResult = Comment::update_many()
            .set(comment::ActiveModel {
                status: Set(Some(status).to_owned()),
                update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
                ..Default::default()
            })
            .filter(comment::Column::Id.eq(id))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// 根据id查询
    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<comment::Model>, DbErr> {
        let res = Comment::find_by_id(id.clone().unwrap_or_default())
            .one(db)
            .await?;
        Ok(res)
    }

    /// 按文章分页查询（已审核 status=1 且 deleted=0）
    pub async fn find_by_article(
        db: &DbConn,
        article_id: i64,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<comment::Model>, i64), DbErr> {
        let count = Comment::find()
            .filter(comment::Column::ArticleId.eq(article_id))
            .filter(comment::Column::Status.eq(1))
            .filter(comment::Column::Deleted.eq(0))
            .count(db)
            .await? as i64;

        let paginator = Comment::find()
            .filter(comment::Column::ArticleId.eq(article_id))
            .filter(comment::Column::Status.eq(1))
            .filter(comment::Column::Deleted.eq(0))
            .order_by_desc(comment::Column::Id)
            .paginate(db, per_page as u64);

        let list = paginator.fetch_page((page - 1) as u64).await?;
        Ok((list, count))
    }

    /// 计数（后台列表，仅未删除）
    pub async fn select_count(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<i64, DbErr> {
        Comment::find()
            .filter(comment::Column::Deleted.eq(0))
            .apply_if(wheres.article_id, |query, v| {
                query.filter(comment::Column::ArticleId.eq(v))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(comment::Column::Status.eq(v))
            })
            .apply_if(wheres.keywords, |query, v| {
                query.filter(comment::Column::Content.contains(format!("%{}%", v).as_str()))
            })
            .count(db)
            .await
            .map(|c| c as i64)
    }

    /// 分页查询（后台列表，仅未删除）
    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        wheres: PageWhere,
    ) -> Result<(Vec<comment::Model>, i64), DbErr> {
        let paginator = Comment::find()
            .filter(comment::Column::Deleted.eq(0))
            .apply_if(wheres.article_id, |query, v| {
                query.filter(comment::Column::ArticleId.eq(v))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(comment::Column::Status.eq(v))
            })
            .apply_if(wheres.keywords, |query, v| {
                query.filter(comment::Column::Content.contains(format!("%{}%", v).as_str()))
            })
            .order_by_desc(comment::Column::Id)
            .paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
}
