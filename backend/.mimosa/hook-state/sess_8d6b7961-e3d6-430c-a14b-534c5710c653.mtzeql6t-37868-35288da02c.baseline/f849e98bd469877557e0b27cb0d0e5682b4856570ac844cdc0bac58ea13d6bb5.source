use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::product::model::brand::{BrandDetailVO, BrandListQuery, BrandListVO, BrandModel, BrandSaveRequest};
use sea_orm::DbConn;

pub async fn insert(db: &DbConn, form_data: &BrandSaveRequest, created_by: i64) -> Result<i64> {
    let result = BrandModel::insert(db, form_data, created_by).await?;
    Ok(result)
}

pub async fn update(db: &DbConn, form_data: &BrandSaveRequest, updated_by: i64) -> Result<i64> {
    let id = form_data.id.unwrap_or_default();
    if id == 0 {
        return Err(Error::from("品牌ID不能为空".to_string()));
    }
    let result = BrandModel::update(db, id, form_data, updated_by).await?;
    Ok(result)
}

pub async fn batch_delete(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let result = BrandModel::batch_delete(db, ids_vec).await?;
    Ok(result)
}

pub async fn get_info(db: &DbConn, id: i64) -> Result<BrandDetailVO> {
    let result = BrandModel::find_by_id(db, id).await?;
    match result {
        Some(item) => Ok(item.into()),
        None => Err(Error::from("品牌不存在".to_string())),
    }
}

pub async fn get_list(db: &DbConn, query: &BrandListQuery) -> Result<ResultPage<Vec<BrandListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let (list, total) = BrandModel::find_list(
        db,
        page,
        page_size,
        query.keyword.clone(),
        query.status,
    ).await?;

    let data: Vec<BrandListVO> = list.into_iter().map(|item| item.into()).collect();
    Ok(ResultPage::new(data, total, page, page_size))
}

pub async fn get_all(db: &DbConn) -> Result<Vec<BrandListVO>> {
    let list = BrandModel::find_all(db).await?;
    let data: Vec<BrandListVO> = list.into_iter().map(|item| item.into()).collect();
    Ok(data)
}