//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::{Error, Result};
use sea_orm::DbConn;
use crate::core::web::response::ResultPage;
use crate::modules::user::model::user_group::{GroupDetailVO, GroupListVO, GroupSaveDTO, ListQuery, PageWhere, UserGroupModel};
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;



pub async fn insert(db: &DbConn, form_data: &GroupSaveDTO) -> crate::core::errors::error::Result<i64> {
    let result = UserGroupModel::insert(&db, form_data).await?;
    Ok(result)
}



pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<Option<String>>) -> crate::core::errors::error::Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let ids = convert_vec_option_string_to_vec_u64(ids_vec.clone());
    let config = UserGroupModel::batch_delete_by_ids(db, ids).await?;
    Ok(config)
}

pub async fn update_by_id(db: &DbConn, form_data: &GroupSaveDTO) -> Result<i64> {
    let result = UserGroupModel::update_by_id(&db, &form_data.id, &form_data).await?;
    Ok(result)
}

pub async fn find_by_name_unique(db: &DbConn, name: &Option<String>, id: &Option<i64>) -> Result<bool> {
    let result = UserGroupModel::find_by_name_unique(&db, &name, &id).await?;
    if result > 0 {
        return Ok(true);
    }
    Ok(false)
}

pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<GroupDetailVO> {
    let result_data = UserGroupModel::find_by_id(db, id).await?.ok_or_else(|| {
        Error::from(format!(
            "{}={}",
            "用户组信息不存在，id".to_string(),
            &id.unwrap_or_default()
        ))
    })?;
    let result = GroupDetailVO::from(result_data);
    Ok(result)
}


pub async fn get_by_page(db: &DbConn, query : ListQuery) -> Result<ResultPage<Vec<GroupListVO>>> {

    let select_where = PageWhere {
        name: query.keywords,
        status: query.status,
    };
    let search_where = select_where.format();

    let (list, _num_pages) = UserGroupModel::select_in_page(
        &db,
        query.page_num.unwrap_or(0),
        query.page_size.unwrap_or(10),
        search_where.clone()
    ).await?;

    let list_data: Vec<GroupListVO> = list.into_iter().map(|item| GroupListVO::from(item)).collect();

    let count = UserGroupModel::select_count(db, select_where.clone()).await.unwrap_or(0);

    let page_data = ResultPage::new_simple(list_data, count);

    Ok(page_data)
}