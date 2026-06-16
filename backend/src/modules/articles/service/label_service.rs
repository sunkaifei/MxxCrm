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
use crate::modules::articles::model::article::{ArticleModel, QueryShortUrlUnique};
use crate::modules::articles::model::label::{LabelDetailVO, LabelListVO, LabelModel, LabelSaveDTO, LabelSaveRequest, LabelUpdateRequest, ListQuery, PageWhere};
use crate::utils::short_code;
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;

/// 新增标签
pub async fn insert(db: &DbConn, payload: &LabelSaveRequest) -> Result<i64> {
    let mut label_entity = LabelSaveDTO::from(payload.clone());
    label_entity.id = Some(SNOWFLAKE.generate() as i64);
    if payload.short_url.as_ref().map_or(true, |short_url| short_url.trim().is_empty()) {
        label_entity.short_url = Some(find_short_url_unique(db).await?);
    }
    let result = LabelModel::insert(&db, &label_entity).await?;
    Ok(result)
}

/// 根据id删除标签
pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<Option<String>>) -> Result<i64> {
    let ids = convert_vec_option_string_to_vec_u64(ids_vec.clone());
    let result = LabelModel::batch_delete_by_ids(&db, ids).await?;
    Ok(result)
}

/// 根据id修改标签
pub async fn update_by_id(db: &DbConn, payload: LabelUpdateRequest) -> Result<i64> {
    let label_entity = LabelSaveDTO::from(payload);
    let result = LabelModel::update_by_id(&db, &label_entity.id, &label_entity).await?;
    Ok(result)
}

/// 标签名称唯一性校验
pub async fn find_by_title_unique(db: &DbConn, title: &Option<String>, id: &Option<i64>) -> Result<bool> {
    let result_count = LabelModel::find_by_title_unique(&db, title, id).await?;
    if result_count > 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// 标签短链接唯一性校验
pub async fn find_by_short_url_unique(db: &DbConn, short_url: &Option<String>, id: &Option<i64>) -> Result<bool> {
    let result_count = LabelModel::find_by_short_url_unique(&db, short_url, id).await?;
    if result_count > 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}

///获取短网址唯一性
pub async fn find_short_url_unique(db: &DbConn) -> Result<String> {

    let mut new_short_url = String::new();
    for _ in 0..5 {
        let short_url = short_code::generate_code();
        let unique_num = LabelModel::find_by_short_url_unique(&db, &Some(short_url.clone()), &None).await?;
        if unique_num == 0 {
            new_short_url = short_url;
            break;
        }
    }
    if new_short_url.is_empty() {
        return Err(Error::from("短网址生成失败，请重试！"));
    }
    Ok(new_short_url)
}
      

/// 根据id查询标签详情
pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<Option<LabelDetailVO>> {
    let label_data = LabelModel::find_by_id(&db, id).await?;
    match label_data {
        None => {
            Err(Error::from(format!(
                "{}={}",
                "标签信息不存在，id".to_string(),
                &id.unwrap_or_default()
            )))
        }
        Some(label) => {
            Ok(Some(LabelDetailVO::from(label)))
        }
    }
}


/// 根据分页查询标签列表
pub async fn get_by_page(db: &DbConn, query : ListQuery) -> Result<ResultPage<Vec<LabelListVO>>> {

    let select_where = PageWhere {
        title: query.title,
        status: query.status,
    };
    let search_where = select_where.format();

    let (list, _num_pages) = LabelModel::select_in_page(
        &db,
        query.page_num.unwrap_or(0),
        query.page_size.unwrap_or(10),
        search_where.clone()
    ).await?;

    let list_data: Vec<LabelListVO> = list.into_iter().map(|item| LabelListVO::from(item)).collect();

    let count = LabelModel::select_count(db, select_where.clone()).await.unwrap_or(0);

    let page_data = ResultPage::new_simple(list_data, count);

    Ok(page_data)
}