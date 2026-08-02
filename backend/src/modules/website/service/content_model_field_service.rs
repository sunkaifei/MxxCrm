//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::DbConn;
use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::website::model::content_model_field::{ListQuery, PageWhere, FieldDetailVO, FieldListVO, ContentModelFieldModel, FieldSaveDTO};
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;

pub async fn insert(db: &DbConn, form_data: &FieldSaveDTO) -> Result<i64> {
    let result = ContentModelFieldModel::insert(&db, form_data).await?;
    Ok(result)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<Option<String>>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let ids = convert_vec_option_string_to_vec_u64(ids_vec.clone());
    let result = ContentModelFieldModel::batch_delete_by_ids(db, ids).await?;
    Ok(result)
}

pub async fn update_by_id(db: &DbConn, form_data: &FieldSaveDTO) -> Result<i64> {
    let result = ContentModelFieldModel::update_by_id(&db, &form_data.id, form_data).await?;
    Ok(result)
}

pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<FieldDetailVO> {
    let result = ContentModelFieldModel::find_by_id(&db, id).await?.ok_or_else(|| {
        Error::from(format!("{}={}", "模型字段不存在，id".to_string(), &id.unwrap_or_default()))
    })?;
    let result = FieldDetailVO::from(result);
    Ok(result)
}

/// 根据模型ID查询所有字段
pub async fn get_by_model_id(db: &DbConn, model_id: &Option<i64>) -> Result<Vec<FieldListVO>> {
    let list = ContentModelFieldModel::find_by_model_id(&db, model_id).await?;
    let list_data: Vec<FieldListVO> = list.into_iter().map(|item| FieldListVO::from(item)).collect();
    Ok(list_data)
}

pub async fn get_by_page(db: &DbConn, query: ListQuery) -> Result<ResultPage<Vec<FieldListVO>>> {
    let select_where = PageWhere {
        model_id: query.model_id,
        field_name: query.field_name,
        status: query.status,
    };
    let select_where = select_where.format();
    let (list, _num_pages) = ContentModelFieldModel::select_in_page(&db, query.page_num.unwrap_or(0), query.page_size.unwrap_or(10), select_where.clone()).await?;
    let list_data: Vec<FieldListVO> = list.into_iter().map(|item| FieldListVO::from(item)).collect();
    let count = ContentModelFieldModel::select_count(db, select_where.clone()).await.unwrap_or(0);
    let page_data = ResultPage::new_simple(list_data, count);
    Ok(page_data)
}
