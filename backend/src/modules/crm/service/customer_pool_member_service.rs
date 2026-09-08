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
use crate::modules::crm::entity::{customer, customer_pool_member};
use crate::modules::crm::model::customer_pool::{CustomerPoolMemberItem, CustomerPoolMemberSimpleVO, CustomerPoolMemberVO};
use crate::modules::crm::service::customer_pool_service;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use sea_orm::sea_query::OnConflict;
use std::collections::HashMap;

/// 查询池成员列表（含用户名与特殊员工差异化覆盖值）
pub async fn list_members(db: &DbConn, pool_id: i64) -> Result<Vec<CustomerPoolMemberVO>> {
    let rows = customer_pool_member::Entity::find()
        .filter(customer_pool_member::Column::PoolId.eq(pool_id))
        .order_by_asc(customer_pool_member::Column::MemberType)
        .order_by_asc(customer_pool_member::Column::Id)
        .all(db)
        .await?;

    let user_ids: Vec<i64> = rows.iter().filter_map(|m| m.user_id).collect();
    let name_map = crate::modules::system::service::admin_service::build_admin_name_map(db, user_ids).await;

    let items = rows
        .into_iter()
        .map(|m| {
            let uid = m.user_id.unwrap_or_default();
            CustomerPoolMemberVO {
                id: Some(m.id),
                pool_id: Some(m.pool_id.unwrap_or_default()),
                user_id: Some(uid),
                user_name: name_map.get(&uid).cloned(),
                member_type: m.member_type,
                recycle_days_override: m.recycle_days_override,
                claim_daily_limit_override: m.claim_daily_limit_override,
                create_time: m.create_time,
            }
        })
        .collect();
    Ok(items)
}

/// 批量添加池成员（唯一冲突自动跳过；支持特殊员工差异化覆盖值；逐一通知新增成员）
pub async fn add_members(
    db: &DbConn,
    pool_id: i64,
    members: &[CustomerPoolMemberItem],
    operator_id: i64,
) -> Result<i64> {
    customer_pool_service::find_by_id(db, pool_id)
        .await?
        .ok_or_else(|| Error::from("客户公海池不存在".to_string()))?;

    for item in members {
        validate_override(item)?;
    }

    let now = chrono::Local::now().naive_local().to_owned();
    let mut added: i64 = 0;
    let mut added_ids: Vec<i64> = Vec::new();
    for item in members {
        let active = customer_pool_member::ActiveModel {
            pool_id: Set(Some(pool_id)),
            user_id: Set(Some(item.user_id)),
            member_type: Set(Some(item.member_type)),
            recycle_days_override: Set(item.recycle_days_override),
            claim_daily_limit_override: Set(item.claim_daily_limit_override),
            create_time: Set(Some(now.clone())),
            ..Default::default()
        };
        let result = customer_pool_member::Entity::insert(active)
            .on_conflict(
                OnConflict::columns([customer_pool_member::Column::PoolId, customer_pool_member::Column::UserId])
                    .do_nothing()
                    .to_owned(),
            )
            .exec(db)
            .await;
        match result {
            Ok(_) => {
                added += 1;
                added_ids.push(item.user_id);
            }
            Err(DbErr::RecordNotInserted) => {}
            Err(e) => return Err(Error::from(e.to_string())),
        }
    }

    // 通知新增成员
    for uid in &added_ids {
        if let Err(e) = send_member_notice(
            db,
            *uid,
            "客户公海池成员变更通知",
            "<p>您已被管理员加入客户公海池成员，现在可以查看并领取该池内的公海客户。</p>",
            operator_id,
        )
        .await
        {
            log::warn!("发送客户池成员变更通知失败: {}", e);
        }
    }
    Ok(added)
}

/// 池管理员成员行 upsert（建池/改池管理员时同步；已存在则升级为管理员，保留差异化覆盖值）
pub async fn upsert_admin(db: &DbConn, pool_id: i64, user_id: i64, operator_id: i64) -> Result<()> {
    let existing = customer_pool_member::Entity::find()
        .filter(customer_pool_member::Column::PoolId.eq(pool_id))
        .filter(customer_pool_member::Column::UserId.eq(user_id))
        .one(db)
        .await?;
    match existing {
        Some(model) => {
            if model.member_type != Some(1) {
                let mut active: customer_pool_member::ActiveModel = model.into();
                active.member_type = Set(Some(1));
                active.update(db).await?;
            }
        }
        None => {
            let now = chrono::Local::now().naive_local().to_owned();
            let active = customer_pool_member::ActiveModel {
                pool_id: Set(Some(pool_id)),
                user_id: Set(Some(user_id)),
                member_type: Set(Some(1)),
                recycle_days_override: Set(None),
                claim_daily_limit_override: Set(None),
                create_time: Set(Some(now)),
                ..Default::default()
            };
            let result = customer_pool_member::Entity::insert(active)
                .on_conflict(
                    OnConflict::columns([customer_pool_member::Column::PoolId, customer_pool_member::Column::UserId])
                        .do_nothing()
                        .to_owned(),
                )
                .exec(db)
                .await;
            match result {
                Ok(_) | Err(DbErr::RecordNotInserted) => {}
                Err(e) => return Err(Error::from(e.to_string())),
            }
        }
    }

    if let Err(e) = send_member_notice(
        db,
        user_id,
        "客户公海池管理员变更通知",
        "<p>您已被指定为客户公海池管理员，可查看全池客户并进行分配、审批等管理操作。</p>",
        operator_id,
    )
    .await
    {
        log::warn!("发送池管理员变更通知失败: {}", e);
    }
    Ok(())
}

/// 批量移除池成员（物理删除；至少保留 1 名池管理员；池管理员指定的用户需先调整池信息；逐一通知）
pub async fn remove_members(db: &DbConn, pool_id: i64, user_ids: &[i64], operator_id: i64) -> Result<i64> {
    let pool = customer_pool_service::find_by_id(db, pool_id)
        .await?
        .ok_or_else(|| Error::from("客户公海池不存在".to_string()))?;

    if user_ids.is_empty() {
        return Ok(0);
    }

    let all_members = customer_pool_member::Entity::find()
        .filter(customer_pool_member::Column::PoolId.eq(pool_id))
        .all(db)
        .await?;

    let total_admins = all_members.iter().filter(|m| m.member_type == Some(1)).count();
    let removing_admins = user_ids
        .iter()
        .filter(|uid| all_members.iter().any(|m| m.user_id == Some(**uid) && m.member_type == Some(1)))
        .count();
    if total_admins - removing_admins < 1 {
        return Err(Error::from("每池至少需保留 1 名池管理员，无法移除".to_string()));
    }
    if let Some(admin_id) = pool.pool_admin_id {
        if user_ids.contains(&admin_id) {
            return Err(Error::from("该用户为池管理员指定的管理员，请先在池信息中调整后再移除".to_string()));
        }
    }

    let result = customer_pool_member::Entity::delete_many()
        .filter(customer_pool_member::Column::PoolId.eq(pool_id))
        .filter(customer_pool_member::Column::UserId.is_in(user_ids.to_vec()))
        .exec(db)
        .await?;

    for uid in user_ids {
        if let Err(e) = send_member_notice(
            db,
            *uid,
            "客户公海池成员变更通知",
            "<p>您已被管理员移出客户公海池成员，如需继续领取该池客户请联系管理员。</p>",
            operator_id,
        )
        .await
        {
            log::warn!("发送客户池成员变更通知失败: {}", e);
        }
    }
    Ok(result.rows_affected as i64)
}

/// 候选成员列表（成员 + 差异化覆盖值 + 当前私海客户数，供指派/自动分配/负载均衡展示）
pub async fn list_candidates(db: &DbConn, pool_id: i64) -> Result<Vec<CustomerPoolMemberSimpleVO>> {
    let rows = customer_pool_member::Entity::find()
        .filter(customer_pool_member::Column::PoolId.eq(pool_id))
        .order_by_asc(customer_pool_member::Column::MemberType)
        .order_by_asc(customer_pool_member::Column::Id)
        .all(db)
        .await?;

    let user_ids: Vec<i64> = rows.iter().filter_map(|m| m.user_id).collect();
    let name_map = crate::modules::system::service::admin_service::build_admin_name_map(db, user_ids.clone()).await;
    let holding_map = private_customer_counts(db, &user_ids).await?;

    let items = rows
        .into_iter()
        .map(|m| {
            let uid = m.user_id.unwrap_or_default();
            CustomerPoolMemberSimpleVO {
                user_id: Some(uid),
                user_name: name_map.get(&uid).cloned(),
                member_type: m.member_type,
                recycle_days_override: m.recycle_days_override,
                claim_daily_limit_override: m.claim_daily_limit_override,
                holding_count: holding_map.get(&uid).copied(),
            }
        })
        .collect();
    Ok(items)
}

/// 池成员用户ID列表（自动分配候选）
pub async fn member_ids(db: &DbConn, pool_id: i64) -> Result<Vec<i64>> {
    let rows = customer_pool_member::Entity::find()
        .filter(customer_pool_member::Column::PoolId.eq(pool_id))
        .all(db)
        .await?;
    let mut ids: Vec<i64> = rows.iter().filter_map(|m| m.user_id).collect();
    ids.sort();
    Ok(ids)
}

/// 用户是否为指定池成员（含管理员）
pub async fn is_member(db: &DbConn, pool_id: i64, user_id: i64) -> Result<bool> {
    let cnt = customer_pool_member::Entity::find()
        .filter(customer_pool_member::Column::PoolId.eq(pool_id))
        .filter(customer_pool_member::Column::UserId.eq(user_id))
        .count(db)
        .await?;
    Ok(cnt > 0)
}

/// 批量统计用户当前私海客户数（口径：已分配 + 未删除）
pub async fn private_customer_counts(db: &DbConn, user_ids: &[i64]) -> Result<HashMap<i64, i64>> {
    if user_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = customer::Entity::find()
        .select_only()
        .column(customer::Column::AssignedTo)
        .column_as(customer::Column::Id.count(), "cnt")
        .filter(customer::Column::AssignedTo.is_in(user_ids.to_vec()))
        .filter(customer::Column::Deleted.eq(0))
        .group_by(customer::Column::AssignedTo)
        .into_tuple::<(i64, i64)>()
        .all(db)
        .await?;
    Ok(rows.into_iter().collect())
}

/// 差异化覆盖值边界校验（决策点 #16：成员级覆盖，NULL=用池级）
fn validate_override(item: &CustomerPoolMemberItem) -> Result<()> {
    if item.member_type != 1 && item.member_type != 2 {
        return Err(Error::from("非法的成员类型".to_string()));
    }
    if let Some(v) = item.recycle_days_override {
        if v < 0 || v > 3650 {
            return Err(Error::from("特殊员工回收天数覆盖取值范围为 0 ~ 3650".to_string()));
        }
    }
    if let Some(v) = item.claim_daily_limit_override {
        if v < 0 || v > 999 {
            return Err(Error::from("特殊员工领取上限覆盖取值范围为 0 ~ 999".to_string()));
        }
    }
    Ok(())
}

/// 发送池成员变更站内信
async fn send_member_notice(
    db: &DbConn,
    to_user_id: i64,
    title: &str,
    content: &str,
    operator_id: i64,
) -> Result<()> {
    use crate::modules::system::model::notice::{NoticeModel, NoticeSaveDTO};

    let now = chrono::Local::now().naive_local();
    let save_dto = NoticeSaveDTO {
        id: None,
        title: Some(title.to_string()),
        content: Some(content.to_string()),
        r#type: Some(4),
        level: Some("normal".to_string()),
        target_type: Some(2),
        target_user_ids: Some(to_user_id.to_string()),
        publisher_id: Some(operator_id),
        publish_status: Some(0),
        publish_time: Some(now.clone()),
        revoke_time: None,
        create_by: Some(operator_id),
        create_time: Some(now.clone()),
        update_by: Some(operator_id),
        update_time: Some(now),
    };
    let notice_id = NoticeModel::insert(db, &save_dto).await?;
    let _ = crate::modules::system::service::notice_service::update_by_id_publish(
        db,
        &Some(notice_id),
        &Some(operator_id),
    )
    .await;
    Ok(())
}
