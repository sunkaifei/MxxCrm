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
use crate::modules::system::model::dict::{DictDetailVO, DictListVO, DictModel, DictSaveDTO, PageWhere, TypeListQuery};
use crate::modules::system::model::dict_data::{DataListQuery, DictDataDetailVO, DictDataListVO, DictDataModel, DictDataOptionVO, DictDataSaveDTO};
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;
use sea_orm::DbConn;

/// 新增字典类型
/// * `db` : 数据库连接
/// * `form_data`: 字典类型对象
///
/// 返回：字典类型id
pub async fn insert(db: &DbConn, form_data: &DictSaveDTO) -> Result<i64> {
    let result_data = DictModel::insert(db, form_data).await?;
    Ok(result_data)
}

/// 新增字典数据
/// * `db` : 数据库连接
/// * `form_data`: 字典数据对象
///
/// 返回：字典数据id
pub async fn insert_data(db: &DbConn, form_data: &DictDataSaveDTO) -> Result<i64> {
    let result_data = DictDataModel::insert(db, &form_data).await?;
    Ok(result_data)
}

/// 批量删除字典类型
/// * `db` : 数据库连接
/// * `ids`: 字典类型id集合
///
/// 返回：删除成功数量
pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<Option<String>>) -> Result<i64> {
    let vec_ids = convert_vec_option_string_to_vec_u64(ids);
    if vec_ids.len() > 0 {
        for id in vec_ids.iter() {
            let type_data = DictModel::find_by_id(db, &Some(id.clone())).await?.ok_or_else(|| {
                Error::from(format!(
                    "{} = {}",
                    "字典类型不存在，id".to_string(),
                    &id
                ))
            })?;

            if find_data_by_code_unique(&db, &type_data.dict_code).await? {
                return Err(Error::from(format!(
                    "{} {} {}",
                    "字典编码id".to_string(),
                    &id,
                    "存在数据，请先删除数据".to_string(),
                )));
            }
        }
    }
    let result_data = DictModel::batch_delete_by_ids(db, &vec_ids).await?;
    Ok(result_data)
}

/// 批量删除字典数据
/// * `db` : 数据库连接
/// * `ids`: 字典数据id集合
///
/// 返回：删除成功数量
pub async fn batch_delete_data_by_ids(db: &DbConn, ids: Vec<Option<String>>) -> Result<i64> {
    let vec_ids = convert_vec_option_string_to_vec_u64(ids);
    let result_data = DictDataModel::batch_delete_by_ids(db, vec_ids).await?;
    Ok(result_data)
}

/// 更新字典类型
/// * `db` : 数据库连接
/// * `form_data`: 字典类型对象
/// 
/// 返回：响应成功条数
pub async fn update_by_id(db: &DbConn, form_data: &DictSaveDTO) -> Result<i64> {
    let result_data = DictModel::update(db, &form_data.id, form_data).await?;
    Ok(result_data)
}

/// 更新字典数据
/// * `db` : 数据库连接
/// * `form_data`: 字典数据对象
/// 
/// 返回: 响应成功条数
pub async fn update_data_by_id(db: &DbConn, form_data: &DictDataSaveDTO) -> Result<i64> {
    let result_data = DictDataModel::update_by_id(db, &form_data.id, form_data).await?;
    Ok(result_data)
}

/// 根据字典类型和字典值查询字典数据
/// * `db` : 数据库连接
/// * `dict_name`: 字典名称
/// * `id`: 是否排除字典id
///
/// 返回：true：存在，false：不存在
pub async fn find_by_name_unique(db: &DbConn, dict_name: &Option<String>, id: &Option<i64>) -> Result<bool>{
    let result_num = DictModel::find_by_name_unique(db, &dict_name, id).await?;
    Ok(result_num)
}

/// 根据字典类型和字典值查询字典数据
/// * `db` : 数据库连接
/// * `dict_code`: 字典编码
/// * `id`: 是否排除字典id
///
/// 返回：true：存在，false：不存在
pub async fn find_by_code_unique(db: &DbConn, dict_code: &Option<String>, id: &Option<i64>) -> Result<bool>{
    let result_num = DictModel::find_by_code_unique(db, &dict_code, id).await?;
    Ok(result_num)
}

/// 根据字典类型和字典标签查询字典数据
/// * `db` : 数据库连接
/// * `dict_code`: 字典编码
/// * `dict_label`: 字典名称
/// * `id`: 是否排除字典id
///
/// 返回：true：存在，false：不存在
pub async fn find_data_by_label_unique(db: &DbConn, dict_code: &Option<String>, dict_label: &Option<String>, id: &Option<i64>) -> Result<bool>{
    let result_num = DictDataModel::find_by_label_unique(db, dict_code, dict_label, id).await?;
    Ok(result_num)
}

/// 根据字典类型查询字典数据
/// * `db` : 数据库连接
/// * `dict_code`: 字典编码
///
/// 返回：true：存在，false：不存在
pub async fn find_data_by_code_unique(db: &DbConn, dict_code: &Option<String>) -> Result<bool>{
    let result_num = DictDataModel::find_by_code_unique(db, &dict_code).await?;
    Ok(result_num)
}

/// 根据id查询字典类型
/// * `db` : 数据库连接
/// *  `id`: 字典id
///
/// 返回：`DictDetailVO` 字典类型详情对象
pub async fn get_by_id(db: &DbConn, id: &Option<i64>) -> Result<DictDetailVO>{
    let result_data = DictModel::find_by_id(db, id).await?.ok_or_else(|| {
        Error::from(format!(
            "{}={}",
            "用户信息不存在，id".to_string(),
            &id.unwrap_or_default()
        ))
    })?;
    let result = DictDetailVO::from(result_data);
    Ok(result)
}


/// 根据id查询字典数据
pub async fn get_data_by_id(db: &DbConn, id: &Option<i64>) -> Result<DictDataDetailVO>{
    let result_data = DictDataModel::find_by_id(db, id.clone().unwrap_or_default()).await?.ok_or_else(|| {
        Error::from(format!(
            "{}={}",
            "用户信息不存在，id".to_string(),
            &id.unwrap_or_default()
        ))
    })?;
    let result = DictDataDetailVO::from(result_data);
    Ok(result)
}

pub async fn get_dict_page(db: &DbConn, query : TypeListQuery) -> Result<ResultPage<Vec<DictListVO>>> {
    let select_where = PageWhere {
        dict_name: query.dict_name,
        dict_code: query.dict_code,
        status: query.status,
    };

    let search_where = select_where.format();

    let (list, _num_pages) = DictModel::select_in_page(
        db,
        query.page_num.unwrap_or(0),
        query.page_size.unwrap_or(10),
        search_where.clone()
    ).await?;
    
    let list_data: Vec<DictListVO> = list.into_iter().map(|item| DictListVO::from(item)).collect();

    let count = DictModel::select_count(db, select_where.clone()).await.unwrap_or(0);

    let page_data = ResultPage::new_simple(list_data, count);

    Ok(page_data)
}

/// 根据字典code查询字典数据列表
pub async fn get_dict_data_list_by_code(db: &DbConn, dict_code: &Option<String>) -> Result<Vec<DictDataOptionVO>>{
    let result_data = DictDataModel::find_list_by_code(db, &dict_code).await?;
    let dict_vo: Vec<DictDataOptionVO> = result_data.into_iter().map(|dict| DictDataOptionVO {
        label: dict.dict_label,
        value: dict.dict_value,
        tag_type: dict.list_class,
    }).collect();
    Ok(dict_vo)
}

pub async fn get_dict_data_page(db: &DbConn, query : DataListQuery) -> Result<ResultPage<Vec<DictDataListVO>>> {
    let select_where = crate::modules::system::model::dict_data::PageWhere {
        dict_label: query.dict_label,
        dict_code: query.dict_code,
        status: query.status,
    };

    let search_where = select_where.format();

    let (list, _num_pages) = DictDataModel::select_in_page(
        db,
        query.page_num.unwrap_or(0),
        query.page_size.unwrap_or(10),
        search_where.clone()
    ).await?;

    let list_data: Vec<DictDataListVO> = list.into_iter().map(|item| DictDataListVO::from(item)).collect();

    let count = DictDataModel::select_count(db, select_where.clone()).await.unwrap_or(0);

    let page_data = ResultPage::new_simple(list_data, count);

    Ok(page_data)
}