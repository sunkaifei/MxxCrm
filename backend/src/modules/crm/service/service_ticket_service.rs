//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 服务工单业务逻辑层
//!

use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::crm::entity::customer::{self, Entity as CustomerEntity};
use crate::modules::crm::entity::service_ticket::{self, Entity, Column};
use crate::modules::crm::entity::service_ticket_log;
use crate::modules::sale::entity::entitlement::{self as entitlement_entity, Entity as EntitlementEntity};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DbConn, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct TicketListQuery {
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub customer_id: Option<i64>,
    pub status: Option<i32>,
    pub assigned_to: Option<i64>,
    pub keywords: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TicketDetailVO {
    #[serde(flatten)]
    pub ticket: service_ticket::Model,
    pub logs: Vec<service_ticket_log::Model>,
}

/// 创建工单 TK+yyyyMMdd+4位，设置 SLA 截止时间
pub async fn create_ticket(
    db: &DbConn,
    customer_id: i64,
    title: String,
    desc: Option<String>,
    priority: Option<i32>,
    ticket_type: Option<i32>,
    channel: Option<i32>,
    entitlement_id: Option<i64>,
    user_id: i64,
) -> Result<i64> {
    // 查询客户名称
    let customer = CustomerEntity::find_by_id(customer_id)
        .filter(customer::Column::Deleted.eq(0))
        .one(db)
        .await?;
    let customer_name = customer.as_ref().and_then(|c| {
        c.company_name.clone().or(c.short_name.clone()).or(c.person_name.clone())
    });

    // 生成编号
    let date_prefix = format!("TK{}", chrono::Local::now().format("%Y%m%d"));
    let today_records = Entity::find()
        .filter(Column::TicketNo.starts_with(&date_prefix))
        .filter(Column::Deleted.eq(0))
        .all(db)
        .await?;
    let max_seq = today_records.iter()
        .filter_map(|t| t.ticket_no.as_ref())
        .filter_map(|no| no.get(date_prefix.len()..).and_then(|s| s.parse::<u32>().ok()))
        .max()
        .unwrap_or(0);
    let ticket_no = format!("{}{:04}", date_prefix, max_seq + 1);

    // 基于 entitlement 设置 SLA 截止时间
    let now = chrono::Local::now().naive_local();
    let (sla_response, sla_resolution) = if let Some(eid) = entitlement_id {
        let ent = EntitlementEntity::find_by_id(eid)
            .filter(entitlement_entity::Column::Deleted.eq(0))
            .one(db)
            .await?;
        if let Some(e) = ent {
            let resp = e.response_time_hours
                .map(|h| now + chrono::Duration::hours(h as i64));
            let resol = e.resolution_time_hours
                .map(|h| now + chrono::Duration::hours(h as i64));
            (resp, resol)
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    let txn = db.begin().await?;

    let model = service_ticket::ActiveModel {
        ticket_no: Set(Some(ticket_no)),
        customer_id: Set(Some(customer_id)),
        customer_name: Set(customer_name),
        title: Set(Some(title)),
        description: Set(desc),
        priority: Set(priority),
        ticket_type: Set(ticket_type),
        channel: Set(channel),
        entitlement_id: Set(entitlement_id),
        status: Set(Some(1)),
        sla_response_deadline: Set(sla_response),
        sla_resolution_deadline: Set(sla_resolution),
        create_by: Set(Some(user_id)),
        create_time: Set(Some(now)),
        ..Default::default()
    };
    let result = model.insert(&txn).await?;
    let ticket_id = result.id;

    // 写入创建日志
    let log = service_ticket_log::ActiveModel {
        ticket_id: Set(Some(ticket_id)),
        action_type: Set(Some(1)),
        to_status: Set(Some(1)),
        content: Set(Some("工单创建".to_string())),
        operator_id: Set(Some(user_id)),
        create_time: Set(Some(now)),
        ..Default::default()
    };
    log.insert(&txn).await?;

    txn.commit().await?;

    Ok(ticket_id)
}

/// 分配
pub async fn assign_ticket(db: &DbConn, id: i64, assigned_to: i64, dept_id: Option<i64>) -> Result<i64> {
    let existing = Entity::find_by_id(id)
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("工单不存在"))?;

    let now = chrono::Local::now().naive_local();
    let from_status = existing.status;
    let txn = db.begin().await?;
    let mut active: service_ticket::ActiveModel = existing.into();
    active.assigned_to = Set(Some(assigned_to));
    if let Some(did) = dept_id {
        active.assigned_dept = Set(Some(did));
    }
    // 分配后状态变为处理中(2)
    if from_status.unwrap_or(0) == 1 {
        active.status = Set(Some(2));
    }
    active.update_time = Set(Some(now));
    let updated = active.update(&txn).await?;

    // 写入分配日志
    let log = service_ticket_log::ActiveModel {
        ticket_id: Set(Some(id)),
        action_type: Set(Some(2)),
        from_status: Set(from_status),
        to_status: Set(updated.status),
        content: Set(Some(format!("工单分配给用户 {}", assigned_to))),
        operator_id: Set(Some(assigned_to)),
        create_time: Set(Some(now)),
        ..Default::default()
    };
    log.insert(&txn).await?;

    txn.commit().await?;
    Ok(id)
}

/// 回复（记录日志，设 responded_at，推进状态）
pub async fn respond_ticket(db: &DbConn, id: i64, content: String, operator_id: i64) -> Result<i64> {
    let existing = Entity::find_by_id(id)
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("工单不存在"))?;

    let now = chrono::Local::now().naive_local();
    let from_status = existing.status;
    let was_responded = existing.responded_at.is_some();
    let txn = db.begin().await?;
    let mut active: service_ticket::ActiveModel = existing.into();
    if !was_responded {
        active.responded_at = Set(Some(now));
    }
    // 状态推进到处理中(2)
    if from_status.unwrap_or(0) <= 2 {
        active.status = Set(Some(2));
    }
    active.update_time = Set(Some(now));
    let updated = active.update(&txn).await?;

    // 写入回复日志
    let log = service_ticket_log::ActiveModel {
        ticket_id: Set(Some(id)),
        action_type: Set(Some(3)),
        from_status: Set(from_status),
        to_status: Set(updated.status),
        content: Set(Some(content)),
        operator_id: Set(Some(operator_id)),
        create_time: Set(Some(now)),
        ..Default::default()
    };
    log.insert(&txn).await?;

    txn.commit().await?;
    Ok(id)
}

/// 解决（设 resolved_at，状态→待确认）
pub async fn resolve_ticket(db: &DbConn, id: i64, resolution: String, operator_id: i64) -> Result<i64> {
    let existing = Entity::find_by_id(id)
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("工单不存在"))?;

    let from_status = existing.status.unwrap_or(0);
    // 仅处理中(2)/待回复(3)状态可解决
    if from_status != 2 && from_status != 3 {
        return Err(Error::from(format!("当前状态({})不允许解决操作", from_status)));
    }

    let now = chrono::Local::now().naive_local();
    let txn = db.begin().await?;
    let mut active: service_ticket::ActiveModel = existing.into();
    active.resolved_at = Set(Some(now));
    active.resolution = Set(Some(resolution));
    active.status = Set(Some(4)); // 待确认
    active.update_time = Set(Some(now));
    active.update(&txn).await?;

    // 写入解决日志
    let log = service_ticket_log::ActiveModel {
        ticket_id: Set(Some(id)),
        action_type: Set(Some(4)),
        from_status: Set(Some(from_status)),
        to_status: Set(Some(4)),
        content: Set(Some("工单已解决，等待确认".to_string())),
        operator_id: Set(Some(operator_id)),
        create_time: Set(Some(now)),
        ..Default::default()
    };
    log.insert(&txn).await?;

    txn.commit().await?;
    Ok(id)
}

/// 关闭
pub async fn close_ticket(db: &DbConn, id: i64, satisfaction: Option<i32>, remark: Option<String>) -> Result<i64> {
    let existing = Entity::find_by_id(id)
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("工单不存在"))?;

    let from_status = existing.status.unwrap_or(0);
    // 仅待确认(4)状态可关闭
    if from_status != 4 && from_status != 2 {
        return Err(Error::from(format!("当前状态({})不允许关闭操作", from_status)));
    }

    let now = chrono::Local::now().naive_local();
    let txn = db.begin().await?;
    let mut active: service_ticket::ActiveModel = existing.into();
    active.status = Set(Some(5)); // 已关闭
    if let Some(s) = satisfaction {
        active.satisfaction = Set(Some(s));
    }
    if let Some(r) = remark {
        active.satisfaction_remark = Set(Some(r));
    }
    active.update_time = Set(Some(now));
    active.update(&txn).await?;

    // 写入关闭日志
    let log = service_ticket_log::ActiveModel {
        ticket_id: Set(Some(id)),
        action_type: Set(Some(5)),
        from_status: Set(Some(from_status)),
        to_status: Set(Some(5)),
        content: Set(Some("工单已关闭".to_string())),
        create_time: Set(Some(now)),
        ..Default::default()
    };
    log.insert(&txn).await?;

    txn.commit().await?;
    Ok(id)
}

/// 详情（含日志列表）
pub async fn get_info(db: &DbConn, id: i64) -> Result<TicketDetailVO> {
    let ticket = Entity::find_by_id(id)
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("工单不存在"))?;

    let logs = service_ticket_log::Entity::find()
        .filter(service_ticket_log::Column::TicketId.eq(id))
        .order_by_asc(service_ticket_log::Column::Id)
        .all(db)
        .await?;

    Ok(TicketDetailVO { ticket, logs })
}

/// 分页列表（支持 customer_id/status/assigned_to 过滤）
pub async fn get_list(db: &DbConn, query: &TicketListQuery) -> Result<ResultPage<Vec<service_ticket::Model>>> {
    let page = query.page_num.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).max(1);

    let mut cond = Condition::all().add(Column::Deleted.eq(0));
    if let Some(customer_id) = query.customer_id {
        if customer_id > 0 {
            cond = cond.add(Column::CustomerId.eq(customer_id));
        }
    }
    if let Some(status) = query.status {
        if status > 0 {
            cond = cond.add(Column::Status.eq(status));
        }
    }
    if let Some(assigned_to) = query.assigned_to {
        if assigned_to > 0 {
            cond = cond.add(Column::AssignedTo.eq(assigned_to));
        }
    }
    if let Some(keywords) = &query.keywords {
        if !keywords.is_empty() {
            cond = cond.add(
                Condition::any()
                    .add(Column::TicketNo.contains(keywords))
                    .add(Column::Title.contains(keywords)),
            );
        }
    }

    let total = Entity::find()
        .filter(cond.clone())
        .count(db)
        .await? as i64;

    let items = Entity::find()
        .filter(cond)
        .order_by_desc(Column::Id)
        .offset(((page - 1) * page_size) as u64)
        .limit(page_size as u64)
        .all(db)
        .await?;

    Ok(ResultPage::new(items, total, page, page_size))
}

/// 按客户查询
pub async fn get_tickets_by_customer(db: &DbConn, customer_id: i64) -> Result<Vec<service_ticket::Model>> {
    let list = Entity::find()
        .filter(Column::CustomerId.eq(customer_id))
        .filter(Column::Deleted.eq(0))
        .order_by_desc(Column::Id)
        .all(db)
        .await?;
    Ok(list)
}
