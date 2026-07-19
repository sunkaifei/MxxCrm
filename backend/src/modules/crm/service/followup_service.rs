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
use crate::modules::crm::model::followup::{FollowupDetailVO, FollowupListQuery, FollowupListVO, FollowupModel, FollowupSaveDTO, FollowupSaveRequest, FollowupUpdateRequest};
use crate::modules::crm::entity::{customer, lead};
use crate::modules::system::entity::admin;
use sea_orm::DbConn;
use sea_orm::{ColumnTrait, EntityTrait, ActiveModelTrait, QueryFilter, Set, TransactionTrait};

pub async fn insert(db: &DbConn, form_data: &FollowupSaveRequest, created_by: i64) -> Result<i64> {
    let txn = db.begin().await?;

    let mut dto: FollowupSaveDTO = form_data.clone().into();
    dto.created_by = Some(created_by);
    let result = FollowupModel::insert(&txn, &dto).await?;

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
        let _ = lead::Entity::update(lead_active).exec(&txn).await;
    }

    txn.commit().await?;
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
        Some(item) => {
            let mut vo: FollowupDetailVO = item.into();

            // 关联查询客户名称
            if let Some(cid) = vo.customer_id {
                if let Some(c) = customer::Entity::find()
                    .filter(customer::Column::Deleted.eq(0))
                    .filter(customer::Column::Id.eq(cid))
                    .one(db).await?
                {
                    vo.customer_name = c.company_name.clone();
                    // 客户的 assigned_to 即当前负责该客户的业务员
                    if let Some(assignee_id) = c.assigned_to {
                        if let Some(u) = admin::Entity::find_by_id(assignee_id).one(db).await? {
                            vo.assignee_name = u.nick_name.clone().or(u.user_name.clone());
                        }
                    }
                }
            }

            // 关联查询创建人名称
            if let Some(uid) = vo.created_by {
                if let Some(u) = admin::Entity::find_by_id(uid).one(db).await? {
                    vo.created_by_name = u.nick_name.clone().or(u.user_name.clone());
                }
            }

            Ok(vo)
        }
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
        query.only_customer,
    ).await?;
    
    let mut data: Vec<FollowupListVO> = list.into_iter().map(|item| item.into()).collect();

    // 批量查询优化：避免 N+1 查询。先收集所有 customer_id，一次 IN 查询所有客户
    let customer_ids: Vec<i64> = data.iter()
        .filter_map(|vo| vo.customer_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let customers: std::collections::HashMap<i64, customer::Model> = if customer_ids.is_empty() {
        std::collections::HashMap::new()
    } else {
        customer::Entity::find()
            .filter(customer::Column::Deleted.eq(0))
            .filter(customer::Column::Id.is_in(customer_ids))
            .all(db).await
            .map(|cs| cs.into_iter().map(|c| (c.id, c)).collect())
            .unwrap_or_default()
    };

    // 收集所有 assigned_to（去重），一次 IN 查询所有负责人
    let assignee_ids: Vec<i64> = customers.values()
        .filter_map(|c| c.assigned_to)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let admins: std::collections::HashMap<i64, admin::Model> = if assignee_ids.is_empty() {
        std::collections::HashMap::new()
    } else {
        admin::Entity::find()
            .filter(admin::Column::Id.is_in(assignee_ids))
            .all(db).await
            .map(|us| us.into_iter().map(|u| (u.id, u)).collect())
            .unwrap_or_default()
    };

    // 批量查询：通过 customer_id 反查对应的线索ID（lead.converted_to_customer_id）
    // 用于客户跟进记录回填 lead_id，使前端点击客户名称可打开线索详情
    let customer_ids_without_lead: Vec<i64> = data.iter()
        .filter(|vo| vo.lead_id.is_none())
        .filter_map(|vo| vo.customer_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let customer_to_lead: std::collections::HashMap<i64, i64> = if customer_ids_without_lead.is_empty() {
        std::collections::HashMap::new()
    } else {
        lead::Entity::find()
            .filter(lead::Column::Deleted.eq(0))
            .filter(lead::Column::ConvertedToCustomerId.is_in(customer_ids_without_lead))
            .all(db).await
            .map(|ls| ls.into_iter()
                .filter_map(|l| l.converted_to_customer_id.map(|cid| (cid, l.id)))
                .collect())
            .unwrap_or_default()
    };

    // 内存中填充客户名称 + 客户负责人名称 + 回填 lead_id
    for vo in data.iter_mut() {
        if let Some(cid) = vo.customer_id {
            if let Some(c) = customers.get(&cid) {
                vo.customer_name = c.company_name.clone();
                // 客户的 assigned_to 即当前负责该客户的业务员
                if let Some(assignee_id) = c.assigned_to {
                    if let Some(u) = admins.get(&assignee_id) {
                        vo.assignee_name = u.nick_name.clone().or(u.user_name.clone());
                    }
                }
            }
            // 客户跟进记录回填对应线索ID（线索转客户后关联）
            if vo.lead_id.is_none() {
                if let Some(lid) = customer_to_lead.get(&cid) {
                    vo.lead_id = Some(*lid);
                }
            }
        }
    }
    Ok(ResultPage::new(data, total, page, page_size))
}