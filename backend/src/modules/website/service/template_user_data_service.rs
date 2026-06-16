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
use crate::modules::website::model::template_user_data::{ListQuery, PageWhere, TemplateDataDetailVO, TemplateDataListVO, TemplateDataSaveDTO, TemplateDataSaveRequest, TemplateUserDataModel};
use crate::modules::website::model::website_template_merge::WebsiteTemplateMergeModel;
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;

pub async fn insert(db: &DbConn, item: &TemplateDataSaveDTO) -> Result<i64> {
    let result = TemplateUserDataModel::insert(&db, item).await?;
    Ok(result)
}

/// 保存网站模板和新增模板的关联关系
/// * `db` 数据库链接
/// * `website_id` 网站id
/// * `template_id` 用户自定义模板id
/// 
/// 返回值：插入成功id
pub async fn save_website_template_merge(db: &DbConn, website_id: &Option<i64>, template_id: &Option<i64>) -> Result<i64> {
    let result = WebsiteTemplateMergeModel::save(&db, website_id, template_id).await?;
    Ok(result)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<Option<String>>) -> Result<i64>{
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let ids = convert_vec_option_string_to_vec_u64(ids_vec.clone());
    let config = TemplateUserDataModel::batch_delete_by_ids(db, ids).await?;
    Ok(config)
}

pub async fn update_by_id(db: &DbConn, item: &TemplateDataSaveDTO) -> Result<i64> {
    let result = TemplateUserDataModel::update_by_id(&db, &item.id, item).await?;
    Ok(result)
}

pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<TemplateDataDetailVO> {
    let result = TemplateUserDataModel::find_by_id(&db, id).await?.ok_or_else(|| {
        Error::from(format!(
            "{}={}",
            "模板数据不存在，id".to_string(),
            &id.unwrap_or_default()
        ))
    })?;
    let result = TemplateDataDetailVO::from(result);
    Ok(result)
}

pub async fn find_by_template_id(
    db: &DbConn,
    template_id: &Option<i64>,
) -> Result<Vec<TemplateDataListVO>> {
    let list = TemplateUserDataModel::find_by_template_id(&db, template_id).await?;
    let list_data: Vec<TemplateDataListVO> = list.into_iter().map(|item| TemplateDataListVO::from(item)).collect();
    Ok(list_data)
}

/// 查询最新一条数据
/// * `db` 数据库链接
/// * `template_id` 模板id
/// * `type_id` 类型id,1首页，2列表，3内容，4标签，5专题
pub async fn find_latest_by_template_and_type(
    db: &DbConn,
    template_id: &Option<i64>,
    type_id: &Option<i32>,
) -> Result<TemplateDataDetailVO> {
    let result = TemplateUserDataModel::find_latest_by_template_and_type(&db, template_id, type_id).await?.ok_or_else(|| {
        Error::from(format!(
            "{}={}",
            "模板数据不存在，template_id".to_string(),
            &template_id.unwrap_or_default()
        ))
    })?;
    let result = TemplateDataDetailVO::from(result);
    Ok(result)
}
pub async fn get_by_page(db: &DbConn, query: ListQuery) -> Result<ResultPage<Vec<TemplateDataListVO>>> {
    let select_where = PageWhere{
        name: query.keywords.clone(),
        template_id: query.template_id.clone(),
        model_id: query.model_id.clone(),
        type_id: query.type_id.clone(),
        website_id: query.website_id,
        status: query.status.clone(),
    };
    let select_where = select_where.format();

    let (list, _num_pages) = TemplateUserDataModel::select_in_page(
        &db,
        query.page_num.unwrap_or(0),
        query.page_size.unwrap_or(10),
        select_where.clone(),
    ).await?;

    let list_data: Vec<TemplateDataListVO> = list.into_iter().map(|item| TemplateDataListVO::from(item)).collect();

    let count = TemplateUserDataModel::select_count(db, select_where.clone()).await.unwrap_or(0);
    let page_data = ResultPage::new_simple(list_data, count);

    Ok(page_data)
}