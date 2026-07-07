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
use crate::modules::system::entity::{admin, admin::Entity as Admin};
use sea_orm::{DbConn, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set, ConnectionTrait};
use std::collections::{HashMap, HashSet};

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

    // 批量查询用户名称
    let admin_ids: Vec<i64> = records.iter()
        .filter_map(|r| r.admin_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let operated_ids: Vec<i64> = records.iter()
        .filter_map(|r| r.operated_by)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let all_ids: Vec<i64> = admin_ids.iter()
        .chain(operated_ids.iter())
        .cloned()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let admins = Admin::find()
        .filter(admin::Column::Id.is_in(all_ids))
        .all(db).await
        .map_err(|e| Error::from(format!("查询用户信息失败: {}", e)))?;

    let name_map: HashMap<i64, String> = admins.iter()
        .filter_map(|a| a.nick_name.clone().or(a.user_name.clone()).map(|n| (a.id, n)))
        .collect();

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
