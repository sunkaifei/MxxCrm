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
use crate::modules::crm::entity::customer_assign_history::{self, Entity as AssignHistory};
use crate::modules::crm::model::customer_assign_history::AssignHistoryVO;
use sea_orm::{DbConn, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set, ConnectionTrait};

/// 记录客户领取历史
pub async fn record_claim(db: &impl ConnectionTrait, customer_id: i64, admin_id: i64) -> Result<()> {
    let now = chrono::Local::now().naive_local();
    let payload = customer_assign_history::ActiveModel {
        customer_id: Set(Some(customer_id)),
        admin_id: Set(Some(admin_id)),
        action_type: Set(Some(1)),
        start_time: Set(Some(now)),
        end_time: Set(None),
        remark: Set(None),
        operated_by: Set(Some(admin_id)),
        create_time: Set(Some(now)),
        ..Default::default()
    };
    AssignHistory::insert(payload).exec(db).await
        .map_err(|e| Error::from(format!("记录领取历史失败: {}", e)))?;
    Ok(())
}

/// 记录退回公海历史（关闭当前正在负责的记录）
pub async fn record_release(db: &impl ConnectionTrait, customer_id: i64, admin_id: i64) -> Result<()> {
    let now = chrono::Local::now().naive_local();
    // 找到当前正在负责的记录（end_time IS NULL）
    let current = AssignHistory::find()
        .filter(customer_assign_history::Column::CustomerId.eq(customer_id))
        .filter(customer_assign_history::Column::AdminId.eq(admin_id))
        .filter(customer_assign_history::Column::EndTime.is_null())
        .one(db).await
        .map_err(|e| Error::from(format!("查询当前负责记录失败: {}", e)))?;

    if let Some(record) = current {
        let mut active: customer_assign_history::ActiveModel = record.into();
        active.end_time = Set(Some(now));
        active.remark = Set(Some("退回公海".to_string()));
        AssignHistory::update(active).exec(db).await
            .map_err(|e| Error::from(format!("更新退回历史失败: {}", e)))?;
    }
    Ok(())
}

/// 记录客户转移历史
/// 1. 关闭原负责人的当前负责记录（end_time = now, remark = "转移给 {to_user_name}"）
/// 2. 新增新负责人的负责记录（action_type=4, start_time = now, end_time = NULL）
pub async fn record_transfer(
    db: &impl ConnectionTrait,
    customer_id: i64,
    from_admin_id: i64,
    to_admin_id: i64,
    transfer_reason: &str,
    operated_by: i64,
) -> Result<()> {
    let now = chrono::Local::now().naive_local();

    // 1. 关闭原负责人的当前负责记录
    let current = AssignHistory::find()
        .filter(customer_assign_history::Column::CustomerId.eq(customer_id))
        .filter(customer_assign_history::Column::AdminId.eq(from_admin_id))
        .filter(customer_assign_history::Column::EndTime.is_null())
        .one(db).await
        .map_err(|e| Error::from(format!("查询当前负责记录失败: {}", e)))?;

    if let Some(record) = current {
        let mut active: customer_assign_history::ActiveModel = record.into();
        active.end_time = Set(Some(now));
        active.remark = Set(Some(format!("转移给其他负责人：{}", transfer_reason)));
        AssignHistory::update(active).exec(db).await
            .map_err(|e| Error::from(format!("更新转移历史失败: {}", e)))?;
    }

    // 2. 新增新负责人的负责记录
    let payload = customer_assign_history::ActiveModel {
        customer_id: Set(Some(customer_id)),
        admin_id: Set(Some(to_admin_id)),
        action_type: Set(Some(4)), // 4=客户转移
        start_time: Set(Some(now)),
        end_time: Set(None),
        remark: Set(Some(transfer_reason.to_string())),
        operated_by: Set(Some(operated_by)),
        create_time: Set(Some(now)),
        ..Default::default()
    };
    AssignHistory::insert(payload).exec(db).await
        .map_err(|e| Error::from(format!("记录转移历史失败: {}", e)))?;
    Ok(())
}

/// 查询客户的分配历史（按开始时间倒序）
pub async fn list_by_customer(db: &DbConn, customer_id: i64) -> Result<Vec<AssignHistoryVO>> {
    let records = AssignHistory::find()
        .filter(customer_assign_history::Column::CustomerId.eq(customer_id))
        .order_by_desc(customer_assign_history::Column::StartTime)
        .all(db).await
        .map_err(|e| Error::from(format!("查询分配历史失败: {}", e)))?;

    if records.is_empty() {
        return Ok(Vec::new());
    }

    // 批量查询用户名称（admin_id + operated_by 合并为一次 IN，统一调用共用方法）
    let all_ids: Vec<i64> = records.iter()
        .flat_map(|r| [r.admin_id, r.operated_by])
        .flatten()
        .collect();
    let name_map = crate::modules::system::service::admin_service::build_admin_name_map(db, all_ids).await;

    let data: Vec<AssignHistoryVO> = records.into_iter().map(|item| {
        let mut vo: AssignHistoryVO = item.into();
        if let Some(aid) = vo.admin_id {
            vo.admin_name = name_map.get(&aid).cloned();
        }
        if let Some(oid) = vo.operated_by {
            vo.operated_by_name = name_map.get(&oid).cloned();
        }
        vo
    }).collect();

    Ok(data)
}
