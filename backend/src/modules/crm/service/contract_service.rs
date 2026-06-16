use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::crm::model::contract::{ContractDetailVO, ContractListQuery, ContractListVO, ContractModel, ContractSaveDTO, ContractSaveRequest, ContractUpdateRequest};
use sea_orm::DbConn;

pub async fn insert(db: &DbConn, form_data: &ContractSaveRequest, created_by: i64) -> Result<i64> {
    let mut dto: ContractSaveDTO = form_data.clone().into();
    dto.created_by = Some(created_by);
    let result = ContractModel::insert(&db, &dto).await?;
    Ok(result)
}

pub async fn update(db: &DbConn, form_data: &ContractUpdateRequest, updated_by: i64) -> Result<i64> {
    let mut dto: ContractSaveDTO = form_data.clone().into();
    dto.updated_by = Some(updated_by);
    let result = ContractModel::update_by_id(&db, &form_data.id, &dto).await?;
    Ok(result)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let result = ContractModel::batch_delete_by_ids(&db, &ids_vec).await?;
    Ok(result)
}

pub async fn find_by_id(db: &DbConn, id: i64) -> Result<ContractDetailVO> {
    let result = ContractModel::find_by_id(&db, id).await?;
    match result {
        Some(item) => Ok(item.into()),
        None => Err(Error::from("合同不存在".to_string())),
    }
}

pub async fn list(db: &DbConn, query: &ContractListQuery) -> Result<ResultPage<Vec<ContractListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    
    let (list, total) = ContractModel::select_in_page(
        &db,
        page,
        page_size,
        query.keywords.clone(),
        query.status.clone(),
        query.customer_id,
    ).await?;
    
    let data: Vec<ContractListVO> = list.into_iter().map(|item| item.into()).collect();
    Ok(ResultPage::new(data, total, page, page_size))
}