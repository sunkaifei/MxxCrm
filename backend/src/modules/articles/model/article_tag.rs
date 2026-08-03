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
use crate::modules::articles::entity::{article_tag, article_tag::Entity as ArticleTag};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};
use sea_orm::*;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ArticleTagSaveRequest {
    /// 标签名称
    pub name: Option<String>,
    /// 标签别名
    pub slug: Option<String>,
    /// 标签颜色
    pub color: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：0禁用，1启用
    pub status: Option<i32>,
}

impl From<ArticleTagSaveRequest> for ArticleTagSaveDTO {
    fn from(req: ArticleTagSaveRequest) -> Self {
        ArticleTagSaveDTO {
            id: None,
            name: req.name,
            slug: req.slug,
            color: req.color,
            sort: req.sort,
            status: req.status,
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ArticleTagUpdateRequest {
    /// 主键ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 标签名称
    pub name: Option<String>,
    /// 标签别名
    pub slug: Option<String>,
    /// 标签颜色
    pub color: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：0禁用，1启用
    pub status: Option<i32>,
}

impl From<ArticleTagUpdateRequest> for ArticleTagSaveDTO {
    fn from(req: ArticleTagUpdateRequest) -> Self {
        ArticleTagSaveDTO {
            id: req.id,
            name: req.name,
            slug: req.slug,
            color: req.color,
            sort: req.sort,
            status: req.status,
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArticleTagSaveDTO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 标签名称
    pub name: Option<String>,
    /// 标签别名
    pub slug: Option<String>,
    /// 标签颜色
    pub color: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：0禁用，1启用
    pub status: Option<i32>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArticleTagListVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 标签名称
    pub name: Option<String>,
    /// 标签别名
    pub slug: Option<String>,
    /// 标签颜色
    pub color: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：0禁用，1启用
    pub status: Option<i32>,
    /// 文章数量
    pub article_count: Option<i64>,
    /// 创建时间
    pub create_time: Option<String>,
}

impl From<article_tag::Model> for ArticleTagListVO {
    fn from(model: article_tag::Model) -> Self {
        ArticleTagListVO {
            id: Option::from(model.id),
            name: model.name,
            slug: model.slug,
            color: model.color,
            sort: model.sort,
            status: model.status,
            article_count: None,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArticleTagDetailVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 标签名称
    pub name: Option<String>,
    /// 标签别名
    pub slug: Option<String>,
    /// 标签颜色
    pub color: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：0禁用，1启用
    pub status: Option<i32>,
    /// 创建时间
    pub create_time: Option<String>,
    /// 更新时间
    pub update_time: Option<String>,
}

impl From<article_tag::Model> for ArticleTagDetailVO {
    fn from(model: article_tag::Model) -> Self {
        ArticleTagDetailVO {
            id: Option::from(model.id),
            name: model.name,
            slug: model.slug,
            color: model.color,
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
    /// 状态：0禁用，1启用
    pub status: Option<i32>,
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Clone)]
pub struct PageWhere {
    pub keywords: Option<String>,
    /// 状态：0禁用，1启用
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

pub struct ArticleTagModel;

impl ArticleTagModel {
    pub async fn insert(db: &DbConn, dto: &ArticleTagSaveDTO) -> Result<i64, DbErr> {
        let model = article_tag::ActiveModel {
            name: Set(dto.name.to_owned()),
            slug: Set(dto.slug.to_owned()),
            color: Set(dto.color.to_owned()),
            sort: Set(dto.sort.to_owned()),
            status: Set(dto.status.to_owned()),
            create_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let res = ArticleTag::insert(model).exec(db).await?;
        Ok(res.last_insert_id)
    }

    /// 按id批量软删除
    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        let result: UpdateResult = ArticleTag::update_many()
            .set(article_tag::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(article_tag::Column::Id.is_in(ids))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, dto: &ArticleTagSaveDTO) -> Result<i64, DbErr> {
        let model = article_tag::ActiveModel {
            name: Set(dto.name.to_owned()),
            slug: Set(dto.slug.to_owned()),
            color: Set(dto.color.to_owned()),
            sort: Set(dto.sort.to_owned()),
            status: Set(dto.status.to_owned()),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let result: UpdateResult = ArticleTag::update_many()
            .set(model)
            .filter(article_tag::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<article_tag::Model>, DbErr> {
        let result = article_tag::Entity::find_by_id(id.clone().unwrap_or_default())
            .filter(article_tag::Column::Deleted.eq(0))
            .one(db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_name_unique(db: &DbConn, name: &Option<String>, id: &Option<i64>) -> Result<i64, DbErr> {
        let res = ArticleTag::find()
            .filter(article_tag::Column::Name.eq(name.clone().unwrap_or_default()))
            .filter(article_tag::Column::Deleted.eq(0))
            .apply_if(id.clone(), |query, v| {
                query.filter(article_tag::Column::Id.ne(v))
            })
            .count(db)
            .await? as i64;
        Ok(res)
    }

    pub async fn select_in_page(
        db: &DbConn,
        page_num: i64,
        page_size: i64,
        where_case: PageWhere,
    ) -> Result<(Vec<article_tag::Model>, u64), DbErr> {
        let paginator = ArticleTag::find()
            .filter(article_tag::Column::Deleted.eq(0))
            .apply_if(where_case.keywords.clone(), |query, v| {
                query.filter(article_tag::Column::Name.contains(v))
            })
            .apply_if(where_case.status, |query, v| {
                query.filter(article_tag::Column::Status.eq(v))
            })
            .order_by_asc(article_tag::Column::Sort)
            .paginate(db, page_size as u64);
        let total = paginator.num_items().await?;
        let list = paginator.fetch_page(page_num as u64).await?;
        Ok((list, total))
    }

    pub async fn select_count(db: &DbConn, where_case: PageWhere) -> Result<i64, DbErr> {
        let count = ArticleTag::find()
            .filter(article_tag::Column::Deleted.eq(0))
            .apply_if(where_case.keywords.clone(), |query, v| {
                query.filter(article_tag::Column::Name.contains(v))
            })
            .apply_if(where_case.status, |query, v| {
                query.filter(article_tag::Column::Status.eq(v))
            })
            .count(db)
            .await? as i64;
        Ok(count)
    }

    pub async fn select_all(db: &DbConn) -> Result<Vec<article_tag::Model>, DbErr> {
        let list = ArticleTag::find()
            .filter(article_tag::Column::Deleted.eq(0))
            .filter(article_tag::Column::Status.eq(1))
            .order_by_asc(article_tag::Column::Sort)
            .all(db)
            .await?;
        Ok(list)
    }

    /// 统计标签关联的文章数量
    pub async fn count_articles(db: &DbConn, tag_id: &Option<i64>) -> Result<i64, DbErr> {
        use crate::modules::articles::entity::article_tag_merge;
        let count = article_tag_merge::Entity::find()
            .filter(article_tag_merge::Column::TagId.eq(tag_id.clone().unwrap_or_default()))
            .count(db)
            .await? as i64;
        Ok(count)
    }
}