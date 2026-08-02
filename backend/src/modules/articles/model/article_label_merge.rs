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
use crate::modules::articles::entity::article_label_merge::{self, Entity as ArticleLabelMerge};

pub struct ArticleLabelMergeModel;

impl ArticleLabelMergeModel {
    /// 设置文章的标签（先删后插）
    /// 注意：本方法不在内部开启事务，调用方需通过 `db.transaction()` 包裹以保证原子性
    pub async fn set_article_labels<C: ConnectionTrait>(
        db: &C,
        article_id: i64,
        label_ids: Vec<i64>,
    ) -> Result<i64, DbErr> {
        // 1. 删除该文章现有的所有标签关联
        ArticleLabelMerge::delete_many()
            .filter(article_label_merge::Column::ArticleId.eq(article_id))
            .exec(db)
            .await?;

        // 2. 批量插入新的标签关联
        let now = chrono::Local::now().naive_local();
        let mut affected: i64 = 0;
        for label_id in label_ids {
            let am = article_label_merge::ActiveModel {
                article_id: Set(Some(article_id)),
                label_id: Set(Some(label_id)),
                create_time: Set(Some(now)),
                ..Default::default()
            };
            ArticleLabelMerge::insert(am).exec(db).await?;
            affected += 1;
        }
        Ok(affected)
    }

    /// 获取文章的标签ID列表
    pub async fn get_labels_by_article(db: &DbConn, article_id: i64) -> Result<Vec<i64>, DbErr> {
        let rows = ArticleLabelMerge::find()
            .filter(article_label_merge::Column::ArticleId.eq(article_id))
            .all(db)
            .await?;
        Ok(rows.into_iter().filter_map(|m| m.label_id).collect())
    }

    /// 获取标签下的文章ID列表（分页）
    /// 返回 (article_ids, total)
    pub async fn get_articles_by_label(
        db: &DbConn,
        label_id: i64,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<i64>, i64), DbErr> {
        let per_page = std::cmp::max(per_page, 1) as u64;
        let paginator = ArticleLabelMerge::find()
            .filter(article_label_merge::Column::LabelId.eq(label_id))
            .order_by_desc(article_label_merge::Column::Id)
            .paginate(db, per_page);

        let total = paginator.num_items().await? as i64;
        let rows = paginator
            .fetch_page(std::cmp::max(page - 1, 0) as u64)
            .await?;
        let article_ids = rows.into_iter().filter_map(|m| m.article_id).collect();
        Ok((article_ids, total))
    }

    /// 删除文章的所有标签关联
    pub async fn delete_by_article<C: ConnectionTrait>(
        db: &C,
        article_id: i64,
    ) -> Result<i64, DbErr> {
        let res = ArticleLabelMerge::delete_many()
            .filter(article_label_merge::Column::ArticleId.eq(article_id))
            .exec(db)
            .await?;
        Ok(res.rows_affected as i64)
    }

    /// 删除标签的所有文章关联
    pub async fn delete_by_label<C: ConnectionTrait>(
        db: &C,
        label_id: i64,
    ) -> Result<i64, DbErr> {
        let res = ArticleLabelMerge::delete_many()
            .filter(article_label_merge::Column::LabelId.eq(label_id))
            .exec(db)
            .await?;
        Ok(res.rows_affected as i64)
    }
}
