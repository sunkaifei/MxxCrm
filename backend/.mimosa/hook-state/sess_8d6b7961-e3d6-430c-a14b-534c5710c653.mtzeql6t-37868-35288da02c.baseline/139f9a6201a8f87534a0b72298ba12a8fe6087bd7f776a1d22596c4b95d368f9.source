//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use chrono::{Datelike, Local};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, Set,
};

use crate::modules::crm::entity::work_log::{ActiveModel, Column, Entity, Model};
use crate::modules::crm::model::work_log::{
    TodaySummaryVO, TodoTypeSummary, WeekWorkloadVO, WorkLogCreateDTO, WorkLogVO,
};
use crate::modules::crm::service::todo_service::TodoService;

/// 插入工作日志（泛型约束，支持事务连接与普通连接）
pub async fn insert(db: &impl ConnectionTrait, dto: &WorkLogCreateDTO) -> Result<i64, sea_orm::DbErr> {
    let now = Local::now().naive_local();
    let active = ActiveModel {
        user_id: Set(dto.user_id),
        user_name: Set(dto.user_name.clone()),
        action_type: Set(dto.action_type),
        action_name: Set(dto.action_name.clone()),
        business_type: Set(dto.business_type.clone()),
        business_id: Set(dto.business_id),
        business_title: Set(dto.business_title.clone()),
        description: Set(dto.description.clone()),
        result: Set(dto.result),
        work_date: Set(dto.work_date),
        create_time: Set(Some(now)),
        deleted: Set(Some(0)),
        ..Default::default()
    };
    let result = Entity::insert(active).exec(db).await?;
    Ok(result.last_insert_id)
}

/// 查询今日工作日志（按 create_time 降序）
pub async fn find_today_list(
    db: &DatabaseConnection,
    user_id: i64,
) -> Result<Vec<WorkLogVO>, sea_orm::DbErr> {
    let today = Local::now().naive_local().date();
    let today_start = today.and_hms_opt(0, 0, 0).unwrap();
    let today_end = today.and_hms_opt(23, 59, 59).unwrap();

    let list: Vec<Model> = Entity::find()
        .filter(Column::UserId.eq(user_id))
        .filter(Column::Deleted.eq(0))
        .filter(Column::CreateTime.between(today_start, today_end))
        .order_by_desc(Column::CreateTime)
        .all(db)
        .await?;

    Ok(list.into_iter().map(|m| WorkLogVO {
        id: m.id,
        user_id: m.user_id,
        user_name: m.user_name,
        action_type: m.action_type,
        action_name: m.action_name,
        business_type: m.business_type,
        business_id: m.business_id,
        business_title: m.business_title,
        description: m.description,
        result: m.result,
        work_date: m.work_date,
        create_time: m.create_time,
    }).collect())
}

/// 查询本周每天工作数量（按 work_date 升序，本周一到本周日）
pub async fn find_week_workload(
    db: &DatabaseConnection,
    user_id: i64,
) -> Result<Vec<WeekWorkloadVO>, sea_orm::DbErr> {
    let today = Local::now().naive_local().date();
    // 计算本周一：ISO weekday 周一=1，周日=7
    let weekday = today.weekday().num_days_from_monday() as i64;
    let monday = today - chrono::Duration::days(weekday);
    let sunday = monday + chrono::Duration::days(6);

    let list: Vec<Model> = Entity::find()
        .filter(Column::UserId.eq(user_id))
        .filter(Column::Deleted.eq(0))
        .filter(Column::WorkDate.is_not_null())
        .filter(Column::WorkDate.between(monday, sunday))
        .order_by_asc(Column::WorkDate)
        .all(db)
        .await?;

    // 按日期分组统计
    let mut result: Vec<WeekWorkloadVO> = Vec::new();
    for item in list {
        if let Some(d) = item.work_date {
            if let Some(existing) = result.iter_mut().find(|v| v.date == d.to_string()) {
                existing.count += 1;
            } else {
                result.push(WeekWorkloadVO {
                    date: d.to_string(),
                    count: 1,
                });
            }
        }
    }
    Ok(result)
}

/// 今日待办汇总：已处理数（来自 mxx_work_log）+ 剩余数（实时查询 TodoService）
pub async fn find_today_summary(
    db: &DatabaseConnection,
    user_id: i64,
) -> crate::core::errors::error::Result<TodaySummaryVO> {
    let today = Local::now().naive_local().date();
    let today_start = today.and_hms_opt(0, 0, 0).unwrap();
    let today_end = today.and_hms_opt(23, 59, 59).unwrap();

    // 查询今日所有工作日志（action_type=1审批/2跟进/3回款）
    let logs: Vec<Model> = Entity::find()
        .filter(Column::UserId.eq(user_id))
        .filter(Column::Deleted.eq(0))
        .filter(Column::CreateTime.between(today_start, today_end))
        .all(db)
        .await?;

    // 按 action_type 分组统计已处理数
    let mut approval_processed: i64 = 0;
    let mut follow_up_processed: i64 = 0;
    let mut payment_processed: i64 = 0;
    for log in &logs {
        match log.action_type {
            Some(1) => approval_processed += 1,
            Some(2) => follow_up_processed += 1,
            Some(3) => payment_processed += 1,
            _ => {}
        }
    }
    let todo_processed = approval_processed + follow_up_processed + payment_processed;

    // 实时查询剩余待办数（复用 TodoService.summary 的逻辑）
    let summary = TodoService::summary(db, user_id).await?;
    let approval_remaining = summary.pending_approval;
    let follow_up_remaining = summary.overdue_follow_up + summary.today_follow_up;
    let payment_remaining = summary.pending_payment;
    let todo_remaining = approval_remaining + follow_up_remaining + payment_remaining;

    let todo_total = todo_processed + todo_remaining;
    let completion_rate = if todo_total > 0 {
        ((todo_processed as f64 / todo_total as f64) * 100.0) as i32
    } else {
        0
    };

    Ok(TodaySummaryVO {
        todo_processed,
        todo_remaining,
        todo_total,
        completion_rate,
        by_type: TodoTypeSummary {
            approval_processed,
            approval_remaining,
            follow_up_processed,
            follow_up_remaining,
            payment_processed,
            payment_remaining,
        },
    })
}
