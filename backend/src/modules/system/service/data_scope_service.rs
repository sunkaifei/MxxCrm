//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 数据权限公共服务
//!
//! 统一所有业务模块（customer/lead/contact/opportunity/contract/order/quotation/payment/invoice/shipment 等）
//! 的数据权限过滤逻辑，避免代码重复和实现不一致。
//!
//! ## data_scope 取值
//! - 1：全部数据权限（返回 None，不过滤）
//! - 2：自定数据权限（按角色关联的部门过滤，含子部门）
//! - 3：本部门数据权限（仅用户所在部门）
//! - 4：本部门及以下数据权限（用户所在部门 + 递归子部门）
//! - 5：仅本人数据权限（返回 Some([current_user_id])）
//! - None/其他：默认仅本人（保守策略，避免越权）

use crate::core::errors::error::{Error, Result};
use crate::modules::system::entity::dept;
use crate::modules::system::model::admin_dept_merge::AdminDeptMergeModel;
use crate::modules::system::model::dept::DeptModel;
use crate::modules::system::model::role_dept_merge::RoleDeptMergeModel;
use crate::modules::system::service::role_service;
use sea_orm::{DbConn, EntityTrait};
use std::collections::HashSet;

/// 递归获取指定部门及其所有子部门的ID列表
fn collect_child_dept_ids(all_depts: &[dept::Model], parent_id: i64) -> Vec<i64> {
    let mut ids = vec![parent_id];
    for d in all_depts {
        if d.parent_id == Some(parent_id) {
            ids.extend(collect_child_dept_ids(all_depts, d.id));
        }
    }
    ids
}

/// 根据当前用户的所有角色计算数据权限可见用户ID列表
///
/// ## 多角色合并策略
/// 用户拥有多个角色时，对每个角色按其 data_scope 分别计算可见用户集合，
/// 最后取所有集合的**并集**（而非 min 取单一最宽角色），确保多角色权限叠加生效。
///
/// 例如：用户同时拥有 data_scope=2（自定义部门 A）和 data_scope=3（本部门 B）的角色，
/// 合并后可见部门 A ∪ 部门 B 的所有用户。
///
/// ## 返回值
/// - `Ok(None)`：不限制（全部数据权限）
/// - `Ok(Some(user_ids))`：仅可见这些用户负责的数据
/// - `Err(...)`：查询失败
pub async fn get_accessible_user_ids(
    db: &DbConn,
    current_user_id: i64,
) -> Result<Option<Vec<i64>>> {
    // 超级管理员（user_type=1）直接返回 None（全部数据）
    // 注意：此处不依赖 role_id=1 的硬编码判定，由调用方在传入前自行处理超管场景
    let roles = role_service::select_by_admin_id(db, &Some(current_user_id)).await?;

    // 无任何角色：默认仅本人（保守策略）
    if roles.is_empty() {
        return Ok(Some(vec![current_user_id]));
    }

    // 收集所有角色的 data_scope
    let data_scopes: Vec<i32> = roles.iter()
        .filter_map(|r| r.data_scope)
        .collect();

    // 无任何 data_scope 配置：默认仅本人
    if data_scopes.is_empty() {
        return Ok(Some(vec![current_user_id]));
    }

    // 如果任一角色为 data_scope=1（全部数据），直接返回 None
    if data_scopes.iter().any(|&s| s == 1) {
        return Ok(None);
    }

    // 多角色合并：对每个角色的 data_scope 分别计算可见用户集合，最后取并集
    let mut merged_user_ids: HashSet<i64> = HashSet::new();
    let mut has_scope_5_only = true; // 标记是否所有角色都是"仅本人"

    for scope in &data_scopes {
        match scope {
            1 => {
                // 全部数据（理论上前面已拦截，此处防御性处理）
                return Ok(None);
            }
            5 => {
                // 仅本人：加入当前用户
                merged_user_ids.insert(current_user_id);
            }
            2 | 3 | 4 => {
                has_scope_5_only = false;
                let user_ids = resolve_dept_scope_users(db, current_user_id, *scope).await?;
                merged_user_ids.extend(user_ids);
            }
            _ => {
                // 未识别的 data_scope：保守处理为仅本人
                merged_user_ids.insert(current_user_id);
            }
        }
    }

    // 所有角色都是 data_scope=5（仅本人），或合并结果为空
    if merged_user_ids.is_empty() || has_scope_5_only {
        return Ok(Some(vec![current_user_id]));
    }

    // 确保当前用户至少能看到自己的数据
    merged_user_ids.insert(current_user_id);

    let mut result: Vec<i64> = merged_user_ids.into_iter().collect();
    result.sort();
    Ok(Some(result))
}

/// 按部门型 data_scope（2/3/4）解析可见用户列表
async fn resolve_dept_scope_users(
    db: &DbConn,
    current_user_id: i64,
    data_scope: i32,
) -> Result<Vec<i64>> {
    let mut target_dept_ids = Vec::new();

    if data_scope == 2 {
        // 自定义数据权限：查询该用户所有 data_scope=2 的角色关联的部门
        let roles = role_service::select_by_admin_id(db, &Some(current_user_id)).await?;
        for role in roles {
            if role.data_scope == Some(2) {
                if let Some(role_id) = role.id {
                    let dept_result = RoleDeptMergeModel::find_by_role_id(db, &Some(role_id)).await
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
        let user_depts = AdminDeptMergeModel::find_by_admin_id(db, current_user_id).await
            .map_err(|e| Error::from(format!("查询用户部门失败: {}", e)))?;
        for merge in &user_depts {
            if let Some(dept_id) = merge.dept_id {
                target_dept_ids.push(dept_id);
            }
        }
    }

    if target_dept_ids.is_empty() {
        return Ok(Vec::new());
    }

    // 查询所有部门（用于递归收集子部门）
    let all_depts = DeptModel::find_all(db).await
        .map_err(|e| Error::from(format!("查询部门列表失败: {}", e)))?;

    // 收集所有目标部门ID（含子部门）
    let mut all_target_ids = Vec::new();
    for dept_id in &target_dept_ids {
        if data_scope == 4 || data_scope == 2 {
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

    Ok(user_ids)
}

/// 兼容旧接口：根据单一 data_scope 计算可见用户ID列表
///
/// 已弃用，建议使用 [`get_accessible_user_ids`]（自动处理多角色合并）。
/// 保留此函数是为了逐步迁移调用方，避免一次性大范围重构。
#[deprecated(note = "使用 get_accessible_user_ids(db, current_user_id) 替代，自动处理多角色合并")]
pub async fn get_accessible_user_ids_by_scope(
    db: &DbConn,
    current_user_id: i64,
    data_scope: Option<i32>,
) -> Result<Option<Vec<i64>>> {
    match data_scope {
        Some(1) => Ok(None),
        Some(5) => Ok(Some(vec![current_user_id])),
        Some(2) | Some(3) | Some(4) => {
            let scope = data_scope.unwrap();
            let mut user_ids = resolve_dept_scope_users(db, current_user_id, scope).await?;
            if user_ids.is_empty() {
                user_ids = vec![current_user_id];
            }
            Ok(Some(user_ids))
        }
        _ => {
            // None 或其他未识别值：默认仅本人（保守策略）
            Ok(Some(vec![current_user_id]))
        }
    }
}
