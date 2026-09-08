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
use sea_orm::prelude::DateTime;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::core::web::response::ResultPage;
use crate::modules::articles::entity::{
    article_field, article_field::Entity as ArticleField,
    article_field_value, article_field_value::Entity as ArticleFieldValue,
};

// ==================== DTO ====================

/// 自定义字段列表查询
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ArticleFieldListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub category_id: Option<i64>,
    pub field_name: Option<String>,
    pub status: Option<i32>,
}

/// 自定义字段新增/编辑
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ArticleFieldSaveDTO {
    pub id: Option<i64>,
    pub category_id: i64,
    pub field_name: String,
    pub field_label: Option<String>,
    pub field_type: Option<i32>,
    pub field_options: Option<String>,
    pub default_value: Option<String>,
    pub is_required: Option<i32>,
    pub sort: Option<i32>,
    pub status: Option<i32>,
}

/// 字段值保存（文章编辑时提交）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ArticleFieldValueDTO {
    pub field_id: i64,
    pub field_value: Option<String>,
}

/// 批量保存文章字段值
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ArticleFieldValueBatchDTO {
    pub article_id: i64,
    pub values: Vec<ArticleFieldValueDTO>,
}

// ==================== VO ====================

/// 自定义字段详情VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ArticleFieldVO {
    pub id: Option<i64>,
    pub category_id: Option<i64>,
    pub field_name: Option<String>,
    pub field_label: Option<String>,
    pub field_type: Option<i32>,
    pub field_options: Option<String>,
    pub default_value: Option<String>,
    pub is_required: Option<i32>,
    pub sort: Option<i32>,
    pub status: Option<i32>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

impl From<article_field::Model> for ArticleFieldVO {
    fn from(item: article_field::Model) -> Self {
        ArticleFieldVO {
            id: Some(item.id),
            category_id: Some(item.category_id),
            field_name: Some(item.field_name),
            field_label: item.field_label,
            field_type: item.field_type,
            field_options: item.field_options,
            default_value: item.default_value,
            is_required: item.is_required,
            sort: item.sort,
            status: item.status,
            create_time: item.create_time,
            update_time: item.update_time,
        }
    }
}

/// 文章字段值VO（含字段定义信息）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ArticleFieldValueVO {
    pub id: Option<i64>,
    pub article_id: Option<i64>,
    pub field_id: Option<i64>,
    pub field_value: Option<String>,
    /// 关联的字段定义（前端渲染表单用）
    pub field_name: Option<String>,
    pub field_label: Option<String>,
    pub field_type: Option<i32>,
    pub field_options: Option<String>,
    pub is_required: Option<i32>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

// ==================== Model ====================

/// 文章自定义字段数据模型操作类
pub struct ArticleFieldModel;

impl ArticleFieldModel {
    /// 分页查询字段定义
    pub async fn find_by_page(
        db: &DbConn,
        query: &ArticleFieldListQuery,
    ) -> Result<ResultPage<Vec<ArticleFieldVO>>, DbErr> {
        let page = std::cmp::max(query.page.unwrap_or(1), 1);
        let page_size = std::cmp::max(std::cmp::min(query.page_size.unwrap_or(10), 100), 1);

        let mut q = ArticleField::find()
            .filter(article_field::Column::Deleted.eq(0));
        if let Some(cid) = query.category_id {
            q = q.filter(article_field::Column::CategoryId.eq(cid));
        }
        if let Some(name) = &query.field_name {
            q = q.filter(article_field::Column::FieldName.like(format!("%{}%", name)));
        }
        if let Some(s) = query.status {
            q = q.filter(article_field::Column::Status.eq(s));
        }

        let paginator = q
            .order_by_asc(article_field::Column::Sort)
            .paginate(db, page_size as u64);
        let total = paginator.num_items().await? as i64;
        let rows = paginator.fetch_page((page - 1) as u64).await?;
        let list_vo: Vec<ArticleFieldVO> = rows.into_iter().map(|m| m.into()).collect();
        Ok(ResultPage::new(list_vo, total, page, page_size))
    }

    /// 按栏目查询全部字段（供文章编辑页动态表单用）
    pub async fn find_by_category(db: &DbConn, category_id: i64) -> Result<Vec<ArticleFieldVO>, DbErr> {
        let rows = ArticleField::find()
            .filter(article_field::Column::CategoryId.eq(category_id))
            .filter(article_field::Column::Deleted.eq(0))
            .filter(article_field::Column::Status.eq(1))
            .order_by_asc(article_field::Column::Sort)
            .all(db)
            .await?;
        Ok(rows.into_iter().map(|m| m.into()).collect())
    }

    /// 根据ID查询
    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<ArticleFieldVO>, DbErr> {
        let row = ArticleField::find_by_id(id)
            .filter(article_field::Column::Deleted.eq(0))
            .one(db)
            .await?;
        Ok(row.map(|m| m.into()))
    }

    /// 新增
    pub async fn insert<C: ConnectionTrait>(
        db: &C,
        req: &ArticleFieldSaveDTO,
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = article_field::ActiveModel {
            category_id: Set(req.category_id),
            field_name: Set(req.field_name.clone()),
            field_label: Set(req.field_label.clone()),
            field_type: Set(req.field_type.or(Some(1))),
            field_options: Set(req.field_options.clone()),
            default_value: Set(req.default_value.clone()),
            is_required: Set(req.is_required.or(Some(0))),
            sort: Set(req.sort.or(Some(0))),
            status: Set(req.status.or(Some(1))),
            deleted: Set(Some(0)),
            create_time: Set(Some(now.clone())),
            update_time: Set(Some(now)),
            ..Default::default()
        };
        ArticleField::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 更新
    pub async fn update<C: ConnectionTrait>(
        db: &C,
        id: i64,
        req: &ArticleFieldSaveDTO,
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = article_field::ActiveModel {
            category_id: Set(req.category_id),
            field_name: Set(req.field_name.clone()),
            field_label: Set(req.field_label.clone()),
            field_type: Set(req.field_type),
            field_options: Set(req.field_options.clone()),
            default_value: Set(req.default_value.clone()),
            is_required: Set(req.is_required),
            sort: Set(req.sort),
            status: Set(req.status),
            update_time: Set(Some(now)),
            ..Default::default()
        };
        let result: UpdateResult = ArticleField::update_many()
            .set(payload)
            .filter(article_field::Column::Id.eq(id))
            .filter(article_field::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 批量软删除
    pub async fn batch_delete<C: ConnectionTrait>(db: &C, ids: Vec<i64>) -> Result<i64, DbErr> {
        if ids.is_empty() {
            return Ok(0);
        }
        let now = chrono::Local::now().naive_local().to_owned();
        let result: UpdateResult = ArticleField::update_many()
            .col_expr(article_field::Column::Deleted, sea_orm::sea_query::Expr::value(1))
            .col_expr(article_field::Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
            .filter(article_field::Column::Id.is_in(ids))
            .filter(article_field::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }
}

/// 文章自定义字段值数据模型操作类
pub struct ArticleFieldValueModel;

impl ArticleFieldValueModel {
    /// 查询文章的所有字段值（含字段定义信息，供文章详情页渲染）
    pub async fn find_by_article(db: &DbConn, article_id: i64) -> Result<Vec<ArticleFieldValueVO>, DbErr> {
        let values = ArticleFieldValue::find()
            .filter(article_field_value::Column::ArticleId.eq(article_id))
            .all(db)
            .await?;

        let field_ids: Vec<i64> = values.iter().map(|v| v.field_id).collect();
        let fields = if field_ids.is_empty() {
            Vec::new()
        } else {
            ArticleField::find()
                .filter(article_field::Column::Id.is_in(field_ids))
                .all(db)
                .await?
        };

        let result: Vec<ArticleFieldValueVO> = values
            .into_iter()
            .map(|v| {
                let field = fields.iter().find(|f| f.id == v.field_id);
                ArticleFieldValueVO {
                    id: Some(v.id),
                    article_id: Some(v.article_id),
                    field_id: Some(v.field_id),
                    field_value: v.field_value,
                    field_name: field.and_then(|f| Some(f.field_name.clone())),
                    field_label: field.and_then(|f| f.field_label.clone()),
                    field_type: field.and_then(|f| f.field_type),
                    field_options: field.and_then(|f| f.field_options.clone()),
                    is_required: field.and_then(|f| f.is_required),
                    create_time: v.create_time,
                    update_time: v.update_time,
                }
            })
            .collect();
        Ok(result)
    }

    /// 批量保存文章字段值（先删后插，事务包裹保证原子性）
    pub async fn save_article_values<C: ConnectionTrait>(
        db: &C,
        article_id: i64,
        values: &[ArticleFieldValueDTO],
    ) -> Result<i64, DbErr> {
        // 1. 删除旧值
        ArticleFieldValue::delete_many()
            .filter(article_field_value::Column::ArticleId.eq(article_id))
            .exec(db)
            .await?;

        // 2. 插入新值
        let now = chrono::Local::now().naive_local().to_owned();
        let mut affected: i64 = 0;
        for v in values {
            let payload = article_field_value::ActiveModel {
                article_id: Set(article_id),
                field_id: Set(v.field_id),
                field_value: Set(v.field_value.clone()),
                create_time: Set(Some(now.clone())),
                update_time: Set(Some(now.clone())),
                ..Default::default()
            };
            ArticleFieldValue::insert(payload).exec(db).await?;
            affected += 1;
        }
        Ok(affected)
    }

    /// 删除文章的所有字段值（文章删除时联动清理）
    pub async fn delete_by_article<C: ConnectionTrait>(
        db: &C,
        article_id: i64,
    ) -> Result<i64, DbErr> {
        let result = ArticleFieldValue::delete_many()
            .filter(article_field_value::Column::ArticleId.eq(article_id))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }
}
