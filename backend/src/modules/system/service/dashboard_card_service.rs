//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 工作台卡片配置服务
//!
//! 集中管理各页面统计/概览卡片对哪些角色可见：
//! - 管理侧：卡片 CRUD + 角色分配（后台可视化配置）
//! - 运行侧：`get_visible_cards` 返回当前用户可见卡片，前端按页动态渲染

use sea_orm::*;

use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::system::entity::{dashboard_card, role};
use crate::modules::system::model::admin_role_merge::AdminRoleMergeModel;
use crate::modules::system::model::dashboard_card::{
    DashboardCardAssignRolesRequest, DashboardCardListQuery, DashboardCardModel,
    DashboardCardRoleMergeModel, DashboardCardRoleMergeSaveDTO, DashboardCardSaveRequest,
    DashboardCardVO,
};
use crate::modules::system::service::admin_service;

/// 分页查询卡片列表（含已分配角色ID）
pub async fn get_by_page(db: &DbConn, query: DashboardCardListQuery) -> Result<ResultPage<Vec<DashboardCardVO>>> {
    let page_num = std::cmp::max(query.page_num.unwrap_or(1), 1);
    let page_size = std::cmp::max(query.page_size.unwrap_or(10), 1);

    let mut q = dashboard_card::Entity::find().filter(dashboard_card::Column::Deleted.eq(0));
    if let Some(kw) = query.keywords.clone() {
        if !kw.trim().is_empty() {
            let kw = format!("%{}%", kw.trim());
            q = q.filter(
                sea_query::Expr::col(dashboard_card::Column::CardName)
                    .like(&kw)
                    .or(sea_query::Expr::col(dashboard_card::Column::CardCode).like(&kw)),
            );
        }
    }
    if let Some(pk) = query.page_key.clone() {
        if !pk.trim().is_empty() {
            q = q.filter(dashboard_card::Column::PageKey.contains(pk.trim()));
        }
    }
    if let Some(st) = query.status {
        q = q.filter(dashboard_card::Column::Status.eq(st));
    }

    let paginator = q
        .order_by_asc(dashboard_card::Column::SortOrder)
        .order_by_desc(dashboard_card::Column::Id)
        .paginate(db, page_size as u64);
    let total = paginator.num_items().await.map_err(|e| Error::from(e.to_string()))? as i64;
    let items = paginator
        .fetch_page((page_num - 1) as u64)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    // 批量加载卡片-角色关联，组装 role_ids
    let card_ids: Vec<i64> = items.iter().map(|c| c.id).collect();
    let mut role_map: std::collections::HashMap<i64, Vec<i64>> = std::collections::HashMap::new();
    if !card_ids.is_empty() {
        let merges = DashboardCardRoleMergeModel::find_by_card_ids(db, &card_ids)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        for m in merges {
            if let Some(card_id) = m.card_id {
                role_map.entry(card_id).or_default().push(m.role_id.unwrap_or_default());
            }
        }
    }

    let vo_list: Vec<DashboardCardVO> = items
        .into_iter()
        .map(|c| DashboardCardVO {
            id: c.id,
            card_code: c.card_code,
            card_name: c.card_name,
            page_key: c.page_key,
            sort_order: c.sort_order,
            status: c.status,
            remark: c.remark,
            role_ids: role_map.get(&c.id).cloned().unwrap_or_default(),
            create_time: c.create_time,
            update_time: c.update_time,
        })
        .collect();

    Ok(ResultPage::new(vo_list, total, page_num, page_size))
}

/// 新增卡片
pub async fn insert(db: &DbConn, req: &DashboardCardSaveRequest, create_by: &Option<String>) -> Result<i64> {
    if req.card_code.as_ref().map_or(true, |s| s.trim().is_empty()) {
        return Err(Error::from("卡片编码不能为空"));
    }
    if req.card_name.as_ref().map_or(true, |s| s.trim().is_empty()) {
        return Err(Error::from("卡片名称不能为空"));
    }
    let code = req.card_code.as_ref().unwrap().trim();
    if DashboardCardModel::find_by_code(db, code, None).await? > 0 {
        return Err(Error::from("卡片编码已存在"));
    }

    let req = req.clone();
    let create_by = create_by.clone();
    let result = db
        .transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move { DashboardCardModel::insert(txn, &req, &create_by).await })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(result)
}

/// 更新卡片
pub async fn update_by_id(db: &DbConn, req: &DashboardCardSaveRequest, update_by: &Option<String>) -> Result<i64> {
    let id = match req.id {
        Some(id) => id,
        None => return Err(Error::from("卡片ID不能为空")),
    };
    if DashboardCardModel::find_by_id(db, id).await?.is_none() {
        return Err(Error::from("卡片不存在"));
    }
    if let Some(code) = req.card_code.clone() {
        if code.trim().is_empty() {
            return Err(Error::from("卡片编码不能为空"));
        }
        if DashboardCardModel::find_by_code(db, code.trim(), Some(id)).await? > 0 {
            return Err(Error::from("卡片编码已存在"));
        }
    }
    if req.card_name.as_ref().map_or(false, |s| s.trim().is_empty()) {
        return Err(Error::from("卡片名称不能为空"));
    }

    let req = req.clone();
    let update_by = update_by.clone();
    let result = db
        .transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move { DashboardCardModel::update_by_id(txn, &req, &update_by).await })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(result)
}

/// 批量删除卡片（软删除 + 清理角色关联，事务原子执行）
pub async fn batch_delete_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64> {
    let ids = ids.clone();
    let result = db
        .transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move {
                DashboardCardRoleMergeModel::delete_by_card_ids(txn, &ids).await?;
                DashboardCardModel::soft_delete_by_ids(txn, &ids).await
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(result)
}

/// 分配卡片可见角色（删除旧关联 + 插入新关联，事务原子执行）
pub async fn update_card_roles(db: &DbConn, req: &DashboardCardAssignRolesRequest) -> Result<i64> {
    let card_id = match req.card_id {
        Some(id) => id,
        None => return Err(Error::from("卡片ID不能为空")),
    };
    if DashboardCardModel::find_by_id(db, card_id).await?.is_none() {
        return Err(Error::from("卡片不存在"));
    }

    let role_ids = req.role_ids.clone().unwrap_or_default();
    if !role_ids.is_empty() {
        // 校验角色有效性
        let valid: Vec<i64> = role::Entity::find()
            .filter(role::Column::Deleted.ne(2))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?
            .into_iter()
            .map(|r| r.id)
            .collect();
        if let Some(invalid) = role_ids.iter().find(|rid| !valid.contains(rid)) {
            return Err(Error::from(format!("存在无效的角色ID: {}", invalid)));
        }
    }

    let merge_list: Vec<DashboardCardRoleMergeSaveDTO> = role_ids
        .iter()
        .map(|rid| DashboardCardRoleMergeSaveDTO {
            id: None,
            card_id: Some(card_id),
            role_id: Some(*rid),
            create_time: None,
        })
        .collect();
    let card_id_opt = Some(card_id);
    let result = db
        .transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move {
                DashboardCardRoleMergeModel::delete_by_card_id(txn, &card_id_opt).await?;
                if merge_list.is_empty() {
                    // 清空卡片可见角色属合法操作：旧关联已删除，返回成功
                    return Ok(1);
                }
                DashboardCardRoleMergeModel::insert_batch(txn, &merge_list).await
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(result)
}

/// 当前用户可见卡片列表（按页动态渲染）
///
/// - 超级管理员/系统管理员（user_type=1）：返回全部启用卡片
/// - 普通用户：仅返回其角色关联的启用卡片
pub async fn get_visible_cards(db: &DbConn, user_id: i64) -> Result<Vec<DashboardCardVO>> {
    let all = DashboardCardModel::find_all_enabled(db).await?;
    if all.is_empty() {
        return Ok(vec![]);
    }

    // 管理员直接返回全部
    let admin = admin_service::get_by_detail(db, &Some(user_id)).await?;
    let is_admin = admin.user_type == Some(1);
    if is_admin {
        return Ok(all.into_iter().map(vo_from_model).collect());
    }

    // 普通用户按角色过滤
    let merges = AdminRoleMergeModel::find_by_admin_id(db, &Some(user_id))
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    let role_ids: Vec<i64> = merges.into_iter().filter_map(|m| m.role_id).collect();
    if role_ids.is_empty() {
        return Ok(vec![]);
    }
    let card_ids = DashboardCardRoleMergeModel::find_card_ids_by_role_ids(db, &role_ids)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(all
        .into_iter()
        .filter(|c| card_ids.contains(&c.id))
        .map(vo_from_model)
        .collect())
}

fn vo_from_model(c: dashboard_card::Model) -> DashboardCardVO {
    DashboardCardVO {
        id: c.id,
        card_code: c.card_code,
        card_name: c.card_name,
        page_key: c.page_key,
        sort_order: c.sort_order,
        status: c.status,
        remark: c.remark,
        role_ids: vec![],
        create_time: c.create_time,
        update_time: c.update_time,
    }
}
