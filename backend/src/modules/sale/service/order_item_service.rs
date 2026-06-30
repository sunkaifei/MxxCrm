use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::sale::model::order_item::{OrderItemDetailVO, OrderItemListQuery, OrderItemListVO, OrderItemModel, OrderItemSaveDTO, OrderItemSaveRequest, OrderItemUpdateRequest};
use sea_orm::DbConn;

pub async fn insert(db: &DbConn, form_data: &OrderItemSaveRequest, created_by: i64) -> Result<i64> {
    let mut dto: OrderItemSaveDTO = form_data.clone().into();
    dto.create_by = Some(created_by);
    let result = OrderItemModel::insert(&db, &dto).await?;
    Ok(result)
}

pub async fn update(db: &DbConn, form_data: &OrderItemUpdateRequest, updated_by: i64) -> Result<i64> {
    let mut dto: OrderItemSaveDTO = form_data.clone().into();
    dto.update_by = Some(updated_by);
    let result = OrderItemModel::update_by_id(&db, &form_data.id, &dto).await?;
    Ok(result)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let result = OrderItemModel::batch_delete_by_ids(&db, &ids_vec).await?;
    Ok(result)
}

pub async fn find_by_id(db: &DbConn, id: i64) -> Result<OrderItemDetailVO> {
    let result = OrderItemModel::find_by_id(&db, id).await?;
    match result {
        Some(item) => Ok(item.into()),
        None => Err(Error::from("订单明细不存在".to_string())),
    }
}

pub async fn list(db: &DbConn, query: &OrderItemListQuery) -> Result<ResultPage<Vec<OrderItemListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    
    let (list, total) = OrderItemModel::select_in_page(
        &db,
        page,
        page_size,
        query.keywords.clone(),
        query.order_id,
    ).await?;
    
    let data: Vec<OrderItemListVO> = list.into_iter().map(|item| item.into()).collect();
    Ok(ResultPage::new(data, total, page, page_size))
}
