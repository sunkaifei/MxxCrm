use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::crm::model::followup::{FollowupDetailVO, FollowupListQuery, FollowupListVO, FollowupModel, FollowupSaveDTO, FollowupSaveRequest, FollowupUpdateRequest};
use crate::modules::crm::entity::lead;
use sea_orm::DbConn;
use sea_orm::{EntityTrait, ActiveModelTrait, Set};

pub async fn insert(db: &DbConn, form_data: &FollowupSaveRequest, created_by: i64) -> Result<i64> {
    let mut dto: FollowupSaveDTO = form_data.clone().into();
    dto.created_by = Some(created_by);
    let result = FollowupModel::insert(&db, &dto).await?;

    if let Some(lead_id) = form_data.lead_id {
        let mut lead_active: lead::ActiveModel = Default::default();
        
        if let Some(status_val) = form_data.lead_status {
            lead_active.status = Set(Some(status_val));
        }
        
        if let Some(next_date) = form_data.next_follow_date {
            let next_dt = chrono::NaiveDateTime::new(next_date, chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());
            lead_active.next_follow_at = Set(Some(next_dt));
        }
        
        lead_active.id = Set(lead_id);
        let _ = lead::Entity::update(lead_active).exec(db).await;
    }

    Ok(result)
}

pub async fn update(db: &DbConn, form_data: &FollowupUpdateRequest, _updated_by: i64) -> Result<i64> {
    let dto: FollowupSaveDTO = form_data.clone().into();
    let result = FollowupModel::update_by_id(&db, &form_data.id, &dto).await?;
    Ok(result)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let result = FollowupModel::batch_delete_by_ids(&db, &ids_vec).await?;
    Ok(result)
}

pub async fn find_by_id(db: &DbConn, id: i64) -> Result<FollowupDetailVO> {
    let result = FollowupModel::find_by_id(&db, id).await?;
    match result {
        Some(item) => Ok(item.into()),
        None => Err(Error::from("跟进记录不存在".to_string())),
    }
}

pub async fn list(db: &DbConn, query: &FollowupListQuery) -> Result<ResultPage<Vec<FollowupListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    
    let (list, total) = FollowupModel::select_in_page(
        &db,
        page,
        page_size,
        query.customer_id,
        query.lead_id,
        query.opportunity_id,
    ).await?;
    
    let data: Vec<FollowupListVO> = list.into_iter().map(|item| item.into()).collect();
    Ok(ResultPage::new(data, total, page, page_size))
}