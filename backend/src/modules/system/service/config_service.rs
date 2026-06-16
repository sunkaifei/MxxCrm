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
use crate::core::web::response::ResultPage;
use crate::modules::system::model::config::{ConfigDetailVO, ConfigListVO, ConfigModel, ConfigSaveDTO, ListQuery, PageWhere};
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;
use sea_orm::DbConn;

/// 新增系统配置信息
pub async fn insert(db: &DbConn, form_data: &ConfigSaveDTO) -> Result<i64>{
    let config = ConfigModel::insert(db, form_data).await?;
    Ok(config)
}

/// 批量删除
pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<Option<String>>) -> Result<i64>{
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let ids = convert_vec_option_string_to_vec_u64(ids_vec.clone());
    let config = ConfigModel::batch_delete_by_ids(db, ids).await?;
    Ok(config)
}

/// 修改系统配置信息
pub async fn update_by_id(db: &DbConn, form_data: &ConfigSaveDTO) -> Result<i64>{
    let config = ConfigModel::update_by_id(db, &form_data.id, &form_data).await?;
    Ok(config)
}

/// 根据配置名称查询是否唯一
pub async fn find_by_name_unique(db: &DbConn, name: &Option<String>, id: &Option<i64>) -> Result<bool>{
    let result_num = ConfigModel::find_by_name_unique(db, &name, id).await?;
    Ok(result_num)
}

/// 根据配置键名查询是否唯一
pub async fn find_by_key_unique(db: &DbConn, key: &Option<String>, id: &Option<i64>) -> Result<bool>{
    let result_num = ConfigModel::find_by_key_unique(db, &key, id).await?;
    Ok(result_num)
}

/// 根据键名查询
pub async fn select_by_key(db: &DbConn, key: &String)-> Result<ConfigDetailVO>{
    let config = ConfigModel::find_by_key(db, key.clone().as_str()).await?.ok_or_else(|| {
        Error::from(format!(
            "{}",
            "该系统配置信息未查到".to_string(),
        ))
    })?;
    let result = ConfigDetailVO::from(config);
    Ok(result)
}

/// 根据ID查询
pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<Option<ConfigDetailVO>>{
    let config = ConfigModel::find_by_id(db, id).await?;
    let result = match config {
        Some(config) => Some(ConfigDetailVO::from(config)),
        None => None
    };
    Ok(result)
}


/// 查询系统配置列表
pub async fn get_by_page(db: &DbConn, query : ListQuery) -> Result<ResultPage<Vec<ConfigListVO>>> {

    let select_where = PageWhere {
        config_name: query.config_name,
        config_key: query.config_key,
        config_type: query.config_type,
    };
    let search_where = select_where.format();

    let (list, _num_pages) = ConfigModel::select_in_page(
        &db,
        query.page_num.unwrap_or(0),
        query.page_size.unwrap_or(10),
        search_where.clone()
    ).await?;
    
    let list_data: Vec<ConfigListVO> = list.into_iter().map(|item| ConfigListVO::from(item)).collect();
    
    let count = ConfigModel::select_count(db, select_where.clone()).await.unwrap_or(0);

    let page_data = ResultPage::new_simple(list_data, count);

    Ok(page_data)
}