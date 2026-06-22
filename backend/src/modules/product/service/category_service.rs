use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::product::model::category::{CategoryDetailVO, CategoryListQuery, CategoryListVO, CategoryModel, CategorySaveDTO, CategorySaveRequest, CategoryUpdateRequest};
use sea_orm::DbConn;

pub async fn insert(db: &DbConn, form_data: &CategorySaveRequest) -> Result<i64> {
    let dto: CategorySaveDTO = form_data.clone().into();
    let result = CategoryModel::insert(&db, &dto).await?;
    Ok(result)
}

pub async fn update(db: &DbConn, form_data: &CategoryUpdateRequest) -> Result<i64> {
    let dto: CategorySaveDTO = form_data.clone().into();
    let result = CategoryModel::update_by_id(&db, &form_data.id, &dto).await?;
    Ok(result)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let result = CategoryModel::batch_delete_by_ids(&db, &ids_vec).await?;
    Ok(result)
}

pub async fn find_by_id(db: &DbConn, id: i64) -> Result<CategoryDetailVO> {
    let result = CategoryModel::find_by_id(&db, id).await?;
    match result {
        Some(item) => Ok(item.into()),
        None => Err(Error::from("分类不存在".to_string())),
    }
}

pub async fn list(db: &DbConn, query: &CategoryListQuery) -> Result<ResultPage<Vec<CategoryListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    
    let (list, total) = CategoryModel::select_in_page(
        &db,
        page,
        page_size,
        query.keywords.clone(),
        query.parent_id,
    ).await?;
    
    let data: Vec<CategoryListVO> = list.into_iter().map(|item| item.into()).collect();
    Ok(ResultPage::new(data, total, page, page_size))
}
