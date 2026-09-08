//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 多工作台维护服务（方案 5.2：list 登录可见 / save 管理员维护）

use std::collections::HashSet;

use sea_orm::*;

use crate::core::errors::error::{Error, Result};
use crate::modules::system::entity::workspace;
use crate::modules::system::model::workspace::{WorkspaceModel, WorkspaceSaveRequest, WorkspaceVO};
use crate::modules::system::service::{admin_service, dashboard_card_service};

/// 当前用户可见工作台（方案 5.3-M3：仅需登录；status=1 且未删除，按 sort 升序）
/// - 管理员（user_type=1）：返回全部启用工作台（设计器需可为任意工作台布局）
/// - 普通用户：仅返回其可见卡片所在的工作台（无权限工作台不可见、空工作台不出现）
pub async fn list_visible(db: &DbConn, user_id: i64) -> Result<Vec<WorkspaceVO>> {
    let list = WorkspaceModel::find_all_enabled(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    if list.is_empty() {
        return Ok(vec![]);
    }
    let admin = admin_service::get_by_detail(db, &Some(user_id)).await?;
    if admin.user_type == Some(1) {
        return Ok(list.into_iter().map(vo_from_model).collect());
    }
    // page_key 即工作台码（方案 5.1），可见卡片的 page_key 集合即用户可见工作台范围
    let cards = dashboard_card_service::get_visible_cards(db, user_id).await?;
    let page_keys: HashSet<String> = cards
        .iter()
        .filter_map(|c| c.page_key.clone())
        .filter(|k| !k.trim().is_empty())
        .collect();
    Ok(list
        .into_iter()
        .filter(|w| {
            w.workspace_code
                .as_ref()
                .map_or(false, |code| page_keys.contains(code))
        })
        .map(vo_from_model)
        .collect())
}

fn vo_from_model(w: workspace::Model) -> WorkspaceVO {
    WorkspaceVO {
        id: w.id,
        workspace_code: w.workspace_code,
        workspace_name: w.workspace_name,
        icon: w.icon,
        is_default: w.is_default,
        status: w.status,
        sort: w.sort,
    }
}

/// 保存工作台（id 空新增，否则更新；is_default=1 时清除其它默认，事务原子执行）
pub async fn save(db: &DbConn, req: &WorkspaceSaveRequest, operator: &Option<String>) -> Result<i64> {
    let code = req.workspace_code.as_ref().map_or("", |s| s.trim());
    if code.is_empty() {
        return Err(Error::from("工作台编码不能为空"));
    }
    if req.workspace_name.as_ref().map_or(true, |s| s.trim().is_empty()) {
        return Err(Error::from("工作台名称不能为空"));
    }
    // 编码即 page_key 路由段：仅允许小写字母/数字/下划线，长度 ≤ 64
    if code.len() > 64 || !code.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
        return Err(Error::from("工作台编码仅允许小写字母/数字/下划线，长度不超过 64"));
    }

    match req.id {
        None => {
            if WorkspaceModel::find_by_code(db, code, None).await? > 0 {
                return Err(Error::from("工作台编码已存在"));
            }
            let req = req.clone();
            let operator = operator.clone();
            let id = db
                .transaction::<_, i64, DbErr>(|txn| {
                    Box::pin(async move {
                        let new_id = WorkspaceModel::insert(txn, &req, &operator).await?;
                        if req.is_default == Some(1) {
                            WorkspaceModel::clear_default_except(txn, new_id).await?;
                        }
                        Ok(new_id)
                    })
                })
                .await
                .map_err(|e| Error::from(e.to_string()))?;
            Ok(id)
        }
        Some(id) => {
            if WorkspaceModel::find_by_id(db, id).await?.is_none() {
                return Err(Error::from("工作台不存在"));
            }
            if WorkspaceModel::find_by_code(db, code, Some(id)).await? > 0 {
                return Err(Error::from("工作台编码已存在"));
            }
            let req = req.clone();
            db.transaction::<_, u64, DbErr>(|txn| {
                Box::pin(async move {
                    let n = WorkspaceModel::update_by_id(txn, &req).await?;
                    if req.is_default == Some(1) {
                        WorkspaceModel::clear_default_except(txn, id).await?;
                    }
                    Ok(n)
                })
            })
            .await
            .map_err(|e| Error::from(e.to_string()))?;
            Ok(id)
        }
    }
}
