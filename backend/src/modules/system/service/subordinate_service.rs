use crate::core::errors::error::{Error, Result};
use crate::modules::system::entity::admin::{self, Entity as AdminEntity};
use crate::modules::system::entity::admin::Column as AdminColumn;
use crate::modules::system::service::data_scope_service;
use crate::modules::system::service::permission_cache_service;
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter};

/// 按 direct_manager_id 递归获取所有下属用户ID（含跨级别的下属的下属）
///
/// 例如：A→B→C→D（direct_manager_id 链）
/// A 的下属 = {B, C, D}
///
/// ## 参数
/// - `db`: 数据库连接
/// - `manager_id`: 上级用户ID
/// - `max_depth`: 最大递归深度（防止循环引用，默认10层）
///
/// ## 返回
/// 所有下属用户ID列表（不含 manager_id 本身）
pub async fn get_subordinate_ids(
    db: &DbConn,
    manager_id: i64,
    max_depth: usize,
) -> Result<Vec<i64>> {
    let mut result: Vec<i64> = Vec::new();
    let mut current_level: Vec<i64> = vec![manager_id];
    let mut visited: std::collections::HashSet<i64> = std::collections::HashSet::new();
    visited.insert(manager_id);

    for _ in 0..max_depth {
        if current_level.is_empty() {
            break;
        }

        // 查询 direct_manager_id 在 current_level 中的所有用户
        let users = AdminEntity::find()
            .filter(AdminColumn::DirectManagerId.is_in(current_level.clone()))
            .filter(AdminColumn::Deleted.eq(0))
            .filter(AdminColumn::Status.eq(1))
            .all(db)
            .await
            .map_err(|e| Error::from(format!("查询下属失败: {}", e)))?;

        let mut next_level: Vec<i64> = Vec::new();
        for u in users {
            if visited.insert(u.id) {
                result.push(u.id);
                next_level.push(u.id);
            }
        }

        current_level = next_level;
    }

    Ok(result)
}

/// 兼容方法：获取下属用户ID（默认最大深度10层）
pub async fn get_subordinate_ids_default(
    db: &DbConn,
    manager_id: i64,
) -> Result<Vec<i64>> {
    get_subordinate_ids(db, manager_id, 10).await
}

/// 获取用户汇报线全部下属ID（P1-2：含缓存，TTL 与权限缓存对齐）
///
/// 按 direct_manager_id 递归展开（含隔级），缓存键 `rl:{user_id}`。
/// 未命中时回源 BFS 计算；空列表同样写入缓存，避免反复回源。
pub async fn get_report_line_user_ids(db: &DbConn, user_id: i64) -> Result<Vec<i64>> {
    if let Some(cached) = permission_cache_service::get_report_line_cache(user_id).await {
        log::debug!("[汇报线缓存] 命中 user_id={}", user_id);
        return Ok(cached.unwrap_or_default());
    }
    let ids = get_subordinate_ids_default(db, user_id).await?;
    permission_cache_service::set_report_line_cache(user_id, &Some(ids.clone())).await;
    Ok(ids)
}

/// 下属业务口径的用户集合（P1-2）：数据权限可见集合 ∪ 汇报线全部下属
///
/// 仅用于"下属"语义的查询口径（客户/线索/商机/联系人/合同/回款/跟进/发货等
/// `list_type=subordinate` 分支），不改变数据权限五档本身的语义。
///
/// 返回值语义：
/// - `None`：数据权限为全量（超管/全量档），调用方不过滤
/// - `Some(ids)`：可见用户集合 ∪ 汇报线下属，调用方按 ID 集合过滤
pub async fn get_subordinate_scope_ids(db: &DbConn, user_id: i64) -> Result<Option<Vec<i64>>> {
    let accessible = data_scope_service::get_accessible_user_ids(db, user_id).await?;
    let Some(accessible_ids) = accessible else {
        return Ok(None);
    };
    let report_line = get_report_line_user_ids(db, user_id).await?;
    let mut merged: std::collections::HashSet<i64> = accessible_ids.into_iter().collect();
    merged.extend(report_line);
    Ok(Some(merged.into_iter().collect()))
}
