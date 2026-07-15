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
use crate::modules::crm::model::customer::{CustomerModel, CustomerSaveDTO};
use crate::modules::crm::model::lead::{LeadDetailVO, LeadListQuery, LeadListVO, LeadModel, LeadSaveDTO, LeadSaveRequest, LeadUpdateRequest};
use crate::modules::system::entity::{admin, admin::Entity as Admin};
use crate::modules::system::model::admin_dept_merge::AdminDeptMergeModel;
use crate::modules::system::model::dept::DeptModel;
use crate::modules::system::service::role_service;
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

pub async fn list(db: &DbConn, query: &LeadListQuery, current_user_id: i64) -> Result<ResultPage<Vec<LeadListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let list_type = query.list_type.as_deref().unwrap_or("my");

    let (list, total) = match list_type {
        "subordinate" => {
            // 下属线索：显示用户 data_scope 范围内的其他人的线索（排除自己）
            let roles = role_service::select_by_admin_id(db, &Some(current_user_id)).await?;
            let data_scope = roles.iter().filter_map(|r| r.data_scope).min();

            if data_scope == Some(5) {
                // 仅本人数据权限的人，无法看到下属线索
                return Ok(ResultPage::new(Vec::<LeadListVO>::new(), 0, page, page_size));
            }

            let user_ids = get_accessible_user_ids(db, current_user_id, data_scope).await?
                .unwrap_or_default()
                .into_iter()
                .filter(|id| *id != current_user_id)
                .collect::<Vec<_>>();

            let assigned_ids = if user_ids.is_empty() { None } else { Some(user_ids) };
            LeadModel::select_in_page_by_assigned_ids(
                &db, page, page_size,
                query.keywords.clone(), query.status, query.level.clone(), query.source.clone(),
                assigned_ids,
            ).await?
        }
        "pool" => {
            // 公海线索：显示未领取（assigned_to IS NULL）的线索
            LeadModel::select_pool_page(
                &db, page, page_size,
                query.keywords.clone(), query.level.clone(), query.source.clone(),
            ).await?
        }
        "todayFollow" => {
            // 今日跟进线索：关联 followup 表过滤
            let roles = role_service::select_by_admin_id(db, &Some(current_user_id)).await?;
            let data_scope = roles.iter().filter_map(|r| r.data_scope).min();
            let user_ids = get_accessible_user_ids(db, current_user_id, data_scope).await?;

            LeadModel::select_today_follow_page(
                &db, page, page_size,
                query.keywords.clone(), query.status, query.level.clone(), query.source.clone(),
                user_ids,
            ).await?
        }
        _ => {
            // my（默认）：只看自己负责的线索
            LeadModel::select_in_page(
                &db,
                page,
                page_size,
                query.keywords.clone(),
                query.status,
                query.level.clone(),
                query.source.clone(),
                Some(current_user_id),
            ).await?
        }
    };

    // 收集所有 created_by 和 assigned_to 的 id，批量查询 admin 用户名
    let creator_ids: Vec<i64> = list.iter()
        .filter_map(|item| item.created_by)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let assignee_ids: Vec<i64> = list.iter()
        .filter_map(|item| item.assigned_to)
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

    let mut assignee_map: HashMap<i64, String> = HashMap::new();
    if !assignee_ids.is_empty() {
        let admins = Admin::find()
            .filter(admin::Column::Id.is_in(assignee_ids))
            .all(db)
            .await?;
        for a in admins {
            if let Some(name) = a.user_name {
                assignee_map.insert(a.id, name);
            }
        }
    }

    let data: Vec<LeadListVO> = list.into_iter().map(|item| {
        let created_by = item.created_by;
        let created_by_name = created_by.and_then(|id| creator_map.get(&id).cloned());
        let assigned_to = item.assigned_to;
        let assignee = assigned_to.and_then(|id| assignee_map.get(&id).cloned());
        let mut vo: LeadListVO = item.into();
        vo.created_by_name = created_by_name;
        vo.assignee = assignee;
        vo
    }).collect();

    Ok(ResultPage::new(data, total, page, page_size))
}

/// 根据当前用户的数据权限，计算可见的用户ID列表
/// 返回 None 表示全部数据（不限制负责人）；Some(vec) 表示仅这些用户的数据
async fn get_accessible_user_ids(
    db: &DbConn,
    current_user_id: i64,
    data_scope: Option<i32>,
) -> Result<Option<Vec<i64>>> {
    match data_scope {
        Some(1) => {
            // 全部数据 - 不限制
            Ok(None)
        }
        Some(5) => {
            // 仅本人数据
            Ok(Some(vec![current_user_id]))
        }
        Some(3) | Some(4) | Some(2) => {
            // 获取用户的部门
            let user_depts = AdminDeptMergeModel::find_by_admin_id(db, current_user_id).await
                .map_err(|e| Error::from(format!("查询用户部门失败: {}", e)))?;

            let mut target_dept_ids = Vec::new();

            if data_scope == Some(2) {
                // 自定义数据权限 - 查询角色关联的部门
                let roles = role_service::select_by_admin_id(db, &Some(current_user_id)).await?;
                for role in roles {
                    if role.data_scope == Some(2) {
                        if let Some(role_id) = role.id {
                            let dept_result = crate::modules::system::model::role_dept_merge::RoleDeptMergeModel::find_by_role_id(db, &Some(role_id)).await
                                .map_err(|e| Error::from(format!("查询角色部门关联失败: {}", e)))?;
                            for merge in dept_result {
                                if let Some(dept_id) = merge.dept_id {
                                    target_dept_ids.push(dept_id);
                                }
                            }
                        }
                    }
                }
            } else {
                // data_scope = 3 或 4：基于用户所在部门
                for merge in &user_depts {
                    if let Some(dept_id) = merge.dept_id {
                        target_dept_ids.push(dept_id);
                    }
                }
            }

            if target_dept_ids.is_empty() {
                return Ok(Some(vec![current_user_id]));
            }

            let all_depts = DeptModel::find_all(db).await
                .map_err(|e| Error::from(format!("查询部门列表失败: {}", e)))?;

            // 收集所有目标部门ID（含子部门）
            let mut all_target_ids = Vec::new();
            for dept_id in &target_dept_ids {
                if data_scope == Some(4) || data_scope == Some(2) {
                    // 本部门及以下 / 自定义：包含子部门
                    all_target_ids.extend(collect_child_dept_ids(&all_depts, *dept_id));
                } else {
                    // data_scope = 3：仅本部门
                    all_target_ids.push(*dept_id);
                }
            }

            // 去重
            all_target_ids.sort();
            all_target_ids.dedup();

            // 查询这些部门下的所有用户
            let dept_merges = AdminDeptMergeModel::find_by_dept_id(db, all_target_ids).await
                .map_err(|e| Error::from(format!("查询部门用户失败: {}", e)))?;

            let mut user_ids: Vec<i64> = dept_merges.iter()
                .filter_map(|m| m.admin_id)
                .collect();
            user_ids.sort();
            user_ids.dedup();

            if user_ids.is_empty() {
                Ok(Some(vec![current_user_id]))
            } else {
                Ok(Some(user_ids))
            }
        }
        _ => {
            // 默认仅本人
            Ok(Some(vec![current_user_id]))
        }
    }
}

/// 递归收集部门下的所有子部门ID（含自身）
fn collect_child_dept_ids(all_depts: &[crate::modules::system::entity::dept::Model], parent_id: i64) -> Vec<i64> {
    let mut ids = Vec::new();
    for dept in all_depts {
        if dept.parent_id == Some(parent_id) {
            ids.push(dept.id);
            ids.extend(collect_child_dept_ids(all_depts, dept.id));
        }
    }
    ids
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
        customer_no: None,
        company_name: lead.company_name.clone(),
        short_name: None,
        country: lead.country.clone(),
        region: lead.region.clone(),
        address: lead.address.clone(),
        website: lead.website.clone(),
        industry: lead.industry,
        level: lead.level,
        source: lead.source.clone().map(|s| s.to_i32()),
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