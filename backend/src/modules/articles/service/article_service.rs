//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!


use sea_orm::{DatabaseConnection, DbConn};
use crate::core::errors::error::{Error, Result};
use crate::{SNOWFLAKE};
use crate::core::web::response::ResultPage;
use crate::modules::articles::model::article::{ArticleDetailVO, ArticleListVO, ArticleModel, ArticlesSaveDTO, QueryPageDTO, QueryPageRequest, QueryShortUrlUnique, QueryTitleUnique};
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

