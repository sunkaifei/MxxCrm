use crate::core::errors::error::{Error, Result};
use crate::modules::system::entity::admin::{self, Entity as AdminEntity};
use crate::modules::system::entity::admin::Column as AdminColumn;
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
