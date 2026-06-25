use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::crm::model::customer::{CustomerModel, CustomerSaveDTO};
use crate::modules::crm::model::lead::{LeadDetailVO, LeadListQuery, LeadListVO, LeadModel, LeadSaveDTO, LeadSaveRequest, LeadUpdateRequest};
use crate::modules::system::entity::{admin, admin::Entity as Admin};
use sea_orm::{DbConn, DbErr, TransactionTrait, ColumnTrait, EntityTrait, QueryFilter};
use std::collections::{HashMap, HashSet};

pub async fn insert(db: &DbConn, form_data: &LeadSaveRequest, created_by: i64) -> Result<i64> {
    let mut dto: LeadSaveDTO = form_data.clone().into();
    dto.created_by = Some(created_by);
    let result = LeadModel::insert(&db, &dto).await?;
    Ok(result)
}

pub async fn update(db: &DbConn, form_data: &LeadUpdateRequest, updated_by: i64) -> Result<i64> {
    let mut dto: LeadSaveDTO = form_data.clone().into();
    dto.updated_by = Some(updated_by);
    let result = LeadModel::update_by_id(&db, &form_data.id, &dto).await?;
    Ok(result)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let result = LeadModel::batch_delete_by_ids(&db, &ids_vec).await?;
    Ok(result)
}

pub async fn find_by_id(db: &DbConn, id: i64) -> Result<LeadDetailVO> {
    let result = LeadModel::find_by_id(&db, id).await?;
    match result {
        Some(item) => {
            let mut vo: LeadDetailVO = item.into();
            let followups = crate::modules::crm::model::followup::FollowupModel::select_by_lead_id(&db, id).await?;

            let creator_ids: Vec<i64> = followups.iter()
                .filter_map(|f| f.created_by)
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();

            let mut creator_map: HashMap<i64, String> = HashMap::new();
            if !creator_ids.is_empty() {
                let admins = Admin::find()
                    .filter(admin::Column::Id.is_in(creator_ids))
                    .all(db)
                    .await?;
                for a in admins {
                    if let Some(name) = a.user_name {
                        creator_map.insert(a.id, name);
                    }
                }
            }

            let followup_vo_list: Vec<crate::modules::crm::model::followup::FollowupListVO> = followups.into_iter().map(|f| {
                let mut followup_vo: crate::modules::crm::model::followup::FollowupListVO = f.into();
                if let Some(created_by) = followup_vo.created_by {
                    followup_vo.created_by_name = creator_map.get(&created_by).cloned();
                }
                followup_vo
            }).collect();

            vo.followups = Some(followup_vo_list);
            Ok(vo)
        }
        None => Err(Error::from("线索不存在".to_string())),
    }
}

pub async fn list(db: &DbConn, query: &LeadListQuery) -> Result<ResultPage<Vec<LeadListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let (list, total) = LeadModel::select_in_page(
        &db,
        page,
        page_size,
        query.keywords.clone(),
        query.status.clone(),
        query.level.clone(),
        query.source.clone(),
        query.assigned_to,
    ).await?;

    // 收集所有 created_by 的 id，批量查询 admin 用户名
    let creator_ids: Vec<i64> = list.iter()
        .filter_map(|item| item.created_by)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let mut creator_map: HashMap<i64, String> = HashMap::new();
    if !creator_ids.is_empty() {
        let admins = Admin::find()
            .filter(admin::Column::Id.is_in(creator_ids))
            .all(db)
            .await?;
        for a in admins {
            if let Some(name) = a.user_name {
                creator_map.insert(a.id, name);
            }
        }
    }

    let data: Vec<LeadListVO> = list.into_iter().map(|item| {
        let created_by = item.created_by;
        let created_by_name = created_by.and_then(|id| creator_map.get(&id).cloned());
        let mut vo: LeadListVO = item.into();
        vo.created_by_name = created_by_name;
        vo
    }).collect();

    Ok(ResultPage::new(data, total, page, page_size))
}

pub async fn update_status(db: &DbConn, id: i64, status: i32, updated_by: Option<i64>) -> Result<i64> {
    // 已转客户的线索不允许修改状态，防止状态不一致
    let lead = LeadModel::find_by_id(db, id)
        .await?
        .ok_or_else(|| Error::from("线索不存在".to_string()))?;
    if lead.converted_to_customer_id.is_some() && status != 3 {
        return Err(Error::from("该线索已转为客户，不能修改状态".to_string()));
    }

    let result = LeadModel::update_status(db, id, status, updated_by).await?;
    Ok(result)
}

pub async fn add_to_pool(db: &DbConn, id: i64, updated_by: Option<i64>) -> Result<i64> {
    // 已转客户的线索不允许加入线索池
    let lead = LeadModel::find_by_id(db, id)
        .await?
        .ok_or_else(|| Error::from("线索不存在".to_string()))?;
    if lead.converted_to_customer_id.is_some() {
        return Err(Error::from("该线索已转为客户，不能加入线索池".to_string()));
    }
    let result = LeadModel::add_to_pool(db, id, updated_by).await?;
    Ok(result)
}

/// 领取线索：从线索创建客户，并将线索标记为已转客户
pub async fn claim(db: &DbConn, id: i64, user_id: i64) -> Result<i64> {
    let lead = LeadModel::find_by_id(db, id)
        .await?
        .ok_or_else(|| Error::from("线索不存在".to_string()))?;

    if lead.converted_to_customer_id.is_some() {
        return Err(Error::from("该线索已被领取".to_string()));
    }

    let customer_dto = CustomerSaveDTO {
        id: None,
        company_name: lead.company_name.clone(),
        short_name: None,
        country: lead.country.clone(),
        region: lead.region.clone(),
        address: lead.address.clone(),
        website: lead.website.clone(),
        industry: lead.industry.clone(),
        level: lead.level,
        source: lead.source.clone(),
        currency: lead.currency.clone(),
        credit_limit: None,
        credit_days: None,
        assigned_to: Some(user_id),
        cooperated_at: None,
        birthday_month: None,
        description: lead.description.clone(),
        custom_fields: lead.custom_fields.clone(),
        deleted: None,
        created_by: Some(user_id),
        create_time: None,
        updated_by: None,
        update_time: None,
    };

    // 使用事务确保创建客户和更新线索原子执行
    let customer_id = db.transaction::<_, _, DbErr>(|txn| {
        Box::pin(async move {
            let customer_id = CustomerModel::insert(txn, &customer_dto).await?;
            LeadModel::claim(txn, id, user_id, customer_id).await?;
            Ok(customer_id)
        })
    }).await.map_err(|e| Error::from(e.to_string()))?;

    Ok(customer_id)
}