//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!


use sea_orm::{DatabaseConnection, DbConn, Set, EntityTrait, QueryFilter, ColumnTrait, QuerySelect};
use crate::core::errors::error::{Error, Result};
use crate::{SNOWFLAKE};
use crate::core::web::response::ResultPage;
use crate::modules::articles::entity::article::{self, Entity as Article};
use crate::modules::articles::model::article::{ArticleDetailVO, ArticleListVO, ArticleModel, ArticlesSaveDTO, QueryPageDTO, QueryPageRequest, QueryShortUrlUnique, QueryTitleUnique};
use crate::modules::articles::model::article_revision::*;
use crate::utils::short_code;

///添加文章
pub async fn save_article(db: &DatabaseConnection, article_data: ArticlesSaveDTO) -> Result<i64> {
    let mut article_data= article_data;
    article_data.id = Some(SNOWFLAKE.generate() as i64);
    log::info!("article_data:{:?}", article_data);
    //获取短网址唯一性
    article_data.short_url = Option::from(find_short_url_unique(&db, article_data.website_id.unwrap_or_default()).await.unwrap_or_default());
    log::info!("=====article_data:{:?}", &article_data);
    let rows = ArticleModel::insert(&db, article_data).await?;
    log::info!("=====rows:{:?}", &rows);
    if rows > 0 {
        Ok(rows)
    }else{
        Ok(0) 
    }
}

pub async fn update_by_id(db: &DbConn, payload: ArticlesSaveDTO) -> Result<i64> {
    let article_data = payload;
    let update_result = ArticleModel::update_by_id(&db,article_data.id.unwrap_or_default(), article_data).await?;
    Ok(update_result)
}

///获取短网址唯一性
pub async fn find_short_url_unique(db: &DatabaseConnection, website_id: i64) -> Result<String> {

    let mut new_short_url = String::new();
    for _ in 0..5 {
        let short_url = short_code::generate_code();
        let query = QueryShortUrlUnique{
            id: None,
            short_url: Option::from(short_url.clone()),
            website_id: Option::from(website_id),
        };
        let unique_num = ArticleModel::find_by_short_url_unique(&db, &query).await?;
        if unique_num == 0 {
            new_short_url = short_url;
            break;
        }
    }
    Ok(new_short_url)
}



/// 查询文章标题是否重复
pub async fn find_by_title_unique(db: &DbConn, itme: &QueryTitleUnique) -> Result<bool>{
    let title_count= ArticleModel::find_by_title_unique(&db, &itme).await?;
    if title_count > 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub async fn get_by_short_url(db: &DatabaseConnection, website_id: &Option<i64>, short_url: &Option<String>) -> Result<Option<ArticleDetailVO>> {
    if short_url.is_none() {
        return Err(Error::from("文章短链接不能为空"));
    }
    let query_res = ArticleModel::find_by_short_url(&db, website_id, short_url).await?.ok_or_else(|| {
        Error::from(format!(
            "{}",
            "文章不存在，id".to_string()
        ))
    })?;
    Ok(Some(query_res))
}


fn get_pagination(page: i64, per_page: i64) -> (i64, i64) {
    let offset = (page - 1) * per_page;
    let limit = per_page;
    (offset, limit)
}

///查询文章列表
pub async fn get_by_page(db: &DatabaseConnection, itme : QueryPageRequest) -> Result<ResultPage<Vec<ArticleListVO>>> {
    let (offset, limit) = get_pagination(itme.page_num.unwrap_or(0), itme.page_size.unwrap_or(10));
    let mut dto = QueryPageDTO::from(itme);
    dto.page_num = offset;
    dto.page_size = limit;
    let article_list: Vec<ArticleListVO> = ArticleModel::get_by_page(&db, &dto).await?;

    let count = ArticleModel::get_by_count(&db, &dto).await?;
    let page_data = ResultPage::new_simple(article_list, count);
    Ok(page_data)
}

/// 文章浏览量自增
pub async fn increment_view_count(db: &DatabaseConnection, id: i64) -> Result<()> {
    ArticleModel::increment_view_count(db, id).await
}

/// 查询上一篇/下一篇文章
pub async fn find_prev_next(db: &DatabaseConnection, category_id: Option<i64>, current_id: i64) -> Result<(Option<ArticleListVO>, Option<ArticleListVO>)> {
    ArticleModel::find_prev_next(db, category_id, current_id).await
}

// ==================== 文章修订历史 ====================

/// 查询文章修订历史列表（分页，按创建时间倒序）
pub async fn list_revisions(db: &DbConn, article_id: i64, page: i64, page_size: i64) -> Result<PageResponse<ArticleRevisionVO>> {
    let result = ArticleRevisionModel::find_by_article(db, article_id, page, page_size).await?;
    Ok(result)
}

/// 查询修订记录详情
pub async fn get_revision_detail(db: &DbConn, revision_id: i64) -> Result<ArticleRevisionVO> {
    let result = ArticleRevisionModel::find_by_id(db, revision_id)
        .await?
        .ok_or_else(|| Error::from("修订记录不存在"))?;
    Ok(result)
}

/// 在更新文章前保存当前文章状态的修订快照
pub async fn save_revision_on_update(
    db: &DbConn,
    article_id: i64,
    old_article: &article::Model,
    editor_id: Option<i64>,
    editor_name: Option<String>,
) -> Result<i64> {
    let revision_no = ArticleRevisionModel::get_next_revision_no(db, article_id).await?;
    let snapshot = serde_json::to_string(old_article).ok();
    let edit_remark = Some("文章更新前自动保存".to_string());
    let result = ArticleRevisionModel::insert(
        db,
        article_id,
        revision_no,
        old_article.title.clone(),
        old_article.short_title.clone(),
        old_article.title_image.clone(),
        old_article.author.clone(),
        old_article.description.clone(),
        old_article.content.clone(),
        snapshot,
        editor_id,
        editor_name,
        edit_remark,
    )
    .await?;
    Ok(result)
}

/// 恢复文章到指定修订版本
/// 1. 获取待恢复的修订记录
/// 2. 保存当前文章状态为新修订（恢复前快照）
/// 3. 用修订记录的数据覆盖文章
pub async fn restore_revision(
    db: &DbConn,
    article_id: i64,
    revision_id: i64,
    editor_id: i64,
    editor_name: String,
) -> Result<i64> {
    // 1. 获取待恢复的修订记录
    let revision = ArticleRevisionModel::find_by_id(db, revision_id)
        .await?
        .ok_or_else(|| Error::from("修订记录不存在"))?;

    // 校验修订记录归属
    if revision.article_id.unwrap_or(0) != article_id {
        return Err(Error::from("修订记录与文章不匹配"));
    }

    // 2. 获取当前文章并保存为修订快照（恢复前）
    let current_article = Article::find_by_id(article_id)
        .one(db)
        .await
        .map_err(|e| Error::from(format!("msg={},code=500", e)))?
        .ok_or_else(|| Error::from("文章不存在"))?;

    save_revision_on_update(db, article_id, &current_article, Some(editor_id), Some(editor_name.clone())).await?;

    // 3. 用修订记录的数据覆盖文章
    let payload = article::ActiveModel {
        title: Set(revision.title),
        short_title: Set(revision.short_title),
        title_image: Set(revision.title_image),
        author: Set(revision.author),
        description: Set(revision.description),
        content: Set(revision.content),
        update_time: Set(Some(chrono::Utc::now().naive_utc())),
        ..Default::default()
    };

    let result = Article::update_many()
        .set(payload)
        .filter(article::Column::Id.eq(article_id))
        .exec(db)
        .await
        .map_err(|e| Error::from(format!("msg={},code=500", e)))?;

    Ok(result.rows_affected as i64)
}

// ==================== G-1.9: 文章批量操作 ====================

/// 批量审核文章（设置状态：1待审核 2已发布 3草稿 4驳回）
pub async fn batch_audit(db: &DbConn, ids: Vec<i64>, status: i32) -> Result<i64> {
    if ids.is_empty() {
        return Err(Error::from("文章ID不能为空"));
    }
    let now = chrono::Local::now().naive_utc();
    let result = Article::update_many()
        .col_expr(article::Column::Status, sea_orm::sea_query::Expr::value(status))
        .col_expr(article::Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
        .filter(article::Column::Id.is_in(ids))
        .exec(db)
        .await
        .map_err(|e| Error::from(format!("msg={},code=500", e)))?;
    Ok(result.rows_affected as i64)
}

/// 批量置顶/取消置顶
pub async fn batch_set_top(db: &DbConn, ids: Vec<i64>, istop: i32) -> Result<i64> {
    if ids.is_empty() {
        return Err(Error::from("文章ID不能为空"));
    }
    let now = chrono::Local::now().naive_utc();
    let result = Article::update_many()
        .col_expr(article::Column::Istop, sea_orm::sea_query::Expr::value(istop))
        .col_expr(article::Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
        .filter(article::Column::Id.is_in(ids))
        .exec(db)
        .await
        .map_err(|e| Error::from(format!("msg={},code=500", e)))?;
    Ok(result.rows_affected as i64)
}

/// 批量移动分类
pub async fn batch_move_category(db: &DbConn, ids: Vec<i64>, category_id: i64) -> Result<i64> {
    if ids.is_empty() {
        return Err(Error::from("文章ID不能为空"));
    }
    let now = chrono::Local::now().naive_utc();
    let result = Article::update_many()
        .col_expr(article::Column::CategoryId, sea_orm::sea_query::Expr::value(category_id))
        .col_expr(article::Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
        .filter(article::Column::Id.is_in(ids))
        .exec(db)
        .await
        .map_err(|e| Error::from(format!("msg={},code=500", e)))?;
    Ok(result.rows_affected as i64)
}

/// 批量设置推荐
pub async fn batch_set_recommend(db: &DbConn, ids: Vec<i64>, isrecommend: i32) -> Result<i64> {
    if ids.is_empty() {
        return Err(Error::from("文章ID不能为空"));
    }
    let now = chrono::Utc::now().naive_utc();
    let result = Article::update_many()
        .col_expr(article::Column::Isrecommend, sea_orm::sea_query::Expr::value(isrecommend))
        .col_expr(article::Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
        .filter(article::Column::Id.is_in(ids))
        .exec(db)
        .await
        .map_err(|e| Error::from(format!("msg={},code=500", e)))?;
    Ok(result.rows_affected as i64)
}

