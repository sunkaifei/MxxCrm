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
use crate::modules::crm::model::contract_payment_plan::{PaymentPlanListQuery, PaymentPlanListVO, PaymentPlanModel, PaymentPlanSaveRequest, PaymentPlanVO};
use crate::modules::crm::entity::{contract, contract::Entity as Contract, customer::{Entity as Customer, Column as CustomerColumn}};
use crate::modules::system::entity::{admin, admin::Entity as Admin};
use crate::modules::system::model::admin_dept_merge::AdminDeptMergeModel;
use crate::modules::system::model::dept::DeptModel;
use crate::modules::system::service::role_service;
use sea_orm::{DbConn, DbErr, TransactionTrait, EntityTrait, ColumnTrait, QueryFilter, Condition};
use std::collections::{HashMap, HashSet};

/// 查询合同的回款计划列表
///
/// # 参数
/// * `db` - 数据库连接
/// * `contract_id` - 合同ID
///
/// # 返回
/// * `Result<Vec<PaymentPlanVO>>` - 回款计划列表
pub async fn list(db: &DbConn, contract_id: i64) -> Result<Vec<PaymentPlanVO>> {
    let list = PaymentPlanModel::find_by_contract(db, contract_id).await?;
    Ok(list)
}

/// 批量保存回款计划（事务：先删后插）
///
/// # 参数
/// * `db` - 数据库连接
/// * `req` - 批量保存请求
///
/// # 返回
/// * `Result<i64>` - 插入的记录数
pub async fn save(db: &DbConn, req: &PaymentPlanSaveRequest) -> Result<i64> {
    let contract_id = req.contract_id;
    let plans = req.plans.clone();

    // 先删后插必须事务化，避免中途失败导致数据丢失
    let result = db
        .transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move {
                let inserted = PaymentPlanModel::save_batch(txn, contract_id, plans).await?;
                Ok(inserted)
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(result)
}

/// 删除合同下所有回款计划
///
/// # 参数
/// * `db` - 数据库连接
/// * `contract_id` - 合同ID
///
/// # 返回
/// * `Result<i64>` - 删除的记录数
pub async fn delete(db: &DbConn, contract_id: i64) -> Result<i64> {
    let result = PaymentPlanModel::delete_by_contract(db, contract_id).await?;
    Ok(result)
}

async fn get_accessible_user_ids(
    db: &DbConn,
    current_user_id: i64,
    data_scope: Option<i32>,
) -> Result<Option<Vec<i64>>> {
    match data_scope {
        Some(1) => Ok(None),
        Some(5) => Ok(Some(vec![current_user_id])),
        Some(3) | Some(4) | Some(2) | Some(0) | Some(_) => {
            let user_depts = AdminDeptMergeModel::find_by_admin_id(db, current_user_id).await
                .map_err(|e| Error::from(format!("查询用户部门失败: {}", e)))?;

            let mut target_dept_ids = Vec::new();

            if data_scope == Some(2) {
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

            let mut all_target_ids = Vec::new();
            for dept_id in &target_dept_ids {
                if data_scope == Some(4) || data_scope == Some(2) {
                    all_target_ids.extend(collect_child_dept_ids(&all_depts, *dept_id));
                } else {
                    all_target_ids.push(*dept_id);
                }
            }

            all_target_ids.sort();
            all_target_ids.dedup();

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
        None => Ok(None),
    }
}

fn collect_child_dept_ids(all_depts: &[crate::modules::system::entity::dept::Model], parent_id: i64) -> Vec<i64> {
    let mut result = vec![parent_id];
    for dept in all_depts {
        if dept.parent_id == Some(parent_id) {
            result.extend(collect_child_dept_ids(all_depts, dept.id));
        }
    }
    result
}

/// 分页查询回款计划列表
///
/// # 参数
/// * `db` - 数据库连接
/// * `query` - 查询参数
/// * `current_user_id` - 当前用户ID
///
/// # 返回
/// * `Result<ResultPage<Vec<PaymentPlanListVO>>>` - 分页结果
pub async fn page_list(db: &DbConn, query: &PaymentPlanListQuery, current_user_id: i64) -> Result<ResultPage<Vec<PaymentPlanListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let list_type = query.list_type.as_deref().unwrap_or("all");

    let owner_user_ids_opt: Option<Vec<i64>> = match list_type {
        "my" => {
            Some(vec![current_user_id])
        }
        "subordinate" => {
            let roles = role_service::select_by_admin_id(db, &Some(current_user_id)).await?;
            let data_scope = roles.iter()
                .filter_map(|r| r.data_scope)
                .min();

            match data_scope {
                Some(5) => {
                    Some(Vec::new())
                }
                Some(1) | None => {
                    let all_admins = Admin::find()
                        .filter(admin::Column::Id.ne(current_user_id))
                        .all(db)
                        .await
                        .map_err(|e| Error::from(format!("查询用户列表失败: {}", e)))?;
                    Some(all_admins.iter().map(|u| u.id).collect())
                }
                _ => {
                    let user_ids = get_accessible_user_ids(db, current_user_id, data_scope).await?
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|id| *id != current_user_id)
                        .collect::<Vec<_>>();
                    Some(user_ids)
                }
            }
        }
        _ => {
            let roles = role_service::select_by_admin_id(db, &Some(current_user_id)).await?;
            let data_scope = roles.iter()
                .filter_map(|r| r.data_scope)
                .min();
            get_accessible_user_ids(db, current_user_id, data_scope).await?
        }
    };

    let contract_ids_filter: Option<Vec<i64>> = if query.contract_id.is_some() || query.customer_id.is_some() {
        let mut contract_query = Contract::find().filter(contract::Column::Deleted.eq(0));

        if let Some(cid) = query.contract_id {
            contract_query = contract_query.filter(contract::Column::Id.eq(cid));
        }
        if let Some(cust_id) = query.customer_id {
            contract_query = contract_query.filter(contract::Column::CustomerId.eq(cust_id));
        }

        let contracts = contract_query.all(db).await?;
        let ids: Vec<i64> = contracts.iter().map(|c| c.id).collect();
        if ids.is_empty() {
            return Ok(ResultPage::new(vec![], 0, page, page_size));
        }
        Some(ids)
    } else {
        None
    };

    let result = PaymentPlanModel::select_in_page_by_owner_user_ids(
        db,
        page,
        page_size,
        query.keywords.clone(),
        query.status,
        contract_ids_filter,
        owner_user_ids_opt,
    ).await?;

    let list = result.0;
    let total = result.1;

    let contract_ids: Vec<i64> = list.iter()
        .filter_map(|p| p.contract_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let (contract_map, customer_name_map) = if !contract_ids.is_empty() {
        let contracts = Contract::find()
            .filter(contract::Column::Id.is_in(contract_ids.clone()))
            .all(db)
            .await?;

        let customer_ids: Vec<i64> = contracts.iter()
            .filter_map(|c| c.customer_id)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let customer_map: HashMap<i64, String> = if !customer_ids.is_empty() {
            Customer::find()
                .filter(CustomerColumn::Id.is_in(customer_ids))
                .all(db)
                .await?
                .into_iter()
                .map(|c| (c.id, c.company_name.or(c.short_name).unwrap_or_default()))
                .collect()
        } else {
            HashMap::new()
        };

        let contract_map: HashMap<i64, (String, Option<i64>)> = contracts.into_iter()
            .map(|c| (c.id, (c.contract_no.unwrap_or_default(), c.customer_id)))
            .collect();

        (contract_map, customer_map)
    } else {
        (HashMap::new(), HashMap::new())
    };

    let data: Vec<PaymentPlanListVO> = list.into_iter().map(|item| {
        let contract_info = item.contract_id.and_then(|cid| contract_map.get(&cid));
        let contract_no = contract_info.map(|(no, _)| no.clone());
        let customer_id = contract_info.and_then(|(_, cid)| *cid);
        let customer_name = customer_id.and_then(|cid| customer_name_map.get(&cid).cloned());

        PaymentPlanListVO {
            id: Some(item.id),
            contract_id: item.contract_id,
            contract_no,
            customer_id,
            customer_name,
            stage_name: item.stage_name,
            payment_type: item.payment_type,
            plan_amount: item.plan_amount,
            received_amount: item.received_amount,
            plan_date: item.plan_date,
            actual_date: item.actual_date,
            status: item.status,
            sort: item.sort,
            remark: item.remark,
            owner_user_id: item.owner_user_id,
            create_time: item.create_time,
        }
    }).collect();

    Ok(ResultPage::new(data, total, page, page_size))
}
