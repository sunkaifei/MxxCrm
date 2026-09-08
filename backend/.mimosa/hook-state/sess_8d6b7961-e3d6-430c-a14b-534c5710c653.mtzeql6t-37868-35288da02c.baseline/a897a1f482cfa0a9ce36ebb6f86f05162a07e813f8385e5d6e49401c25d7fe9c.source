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

use std::time::Duration;

use crate::core::errors::error::{Error, Result};
use crate::core::kit::CONTEXT;
use crate::core::web::response::ResultPage;
use crate::modules::system::entity::{dashboard_card, dashboard_user_layout, role};
use crate::modules::system::model::admin_role_merge::AdminRoleMergeModel;
use crate::modules::system::model::dashboard_card::{
    CardLayoutItem, CardLayoutSaveRequest, CardLayoutVO, DashboardCardAssignRolesRequest,
    DashboardCardListQuery, DashboardCardModel, DashboardCardRoleMergeModel,
    DashboardCardRoleMergeSaveDTO, DashboardCardSaveRequest, DashboardCardVO,
};
use crate::modules::system::model::dashboard_user_layout::{
    DashboardUserLayoutModel, UserLayoutItem, UserLayoutSaveRequest, UserLayoutVO,
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
            default_x: c.default_x,
            default_y: c.default_y,
            default_w: c.default_w,
            default_h: c.default_h,
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
    invalidate_visible_cards().await;
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
    invalidate_visible_cards().await;
    Ok(result)
}

/// 批量删除卡片（软删除 + 清理角色关联 + 清理个人布局残留，事务原子执行）
pub async fn batch_delete_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64> {
    // 先取待删卡片编码，供个人布局级联清理（防止残留记录阻塞用户后续保存布局）
    let cards = dashboard_card::Entity::find()
        .filter(dashboard_card::Column::Id.is_in(ids.clone()))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    let card_codes: Vec<String> = cards.into_iter().filter_map(|c| c.card_code).collect();

    let ids = ids.clone();
    let result = db
        .transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move {
                DashboardUserLayoutModel::delete_by_card_codes(txn, &card_codes).await?;
                DashboardCardRoleMergeModel::delete_by_card_ids(txn, &ids).await?;
                DashboardCardModel::soft_delete_by_ids(txn, &ids).await
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    invalidate_visible_cards().await;
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
    invalidate_visible_cards().await;
    Ok(result)
}

/// visible 卡片缓存键前缀（三期 A2：按角色集合缓存，管理页写操作全量失效）
const VISIBLE_CACHE_PREFIX: &str = "dashboard:visible:";

/// visible 缓存 TTL（秒）：写操作即时失效，TTL 仅兜底 user_type 等罕见变更
const VISIBLE_CACHE_TTL_SECS: u64 = 300;

/// 构建角色集合对应的缓存键（排序去重，同角色集合共享缓存）
fn visible_cache_key(is_admin: bool, role_ids: &[i64]) -> String {
    if is_admin {
        format!("{}admin", VISIBLE_CACHE_PREFIX)
    } else if role_ids.is_empty() {
        format!("{}empty", VISIBLE_CACHE_PREFIX)
    } else {
        format!(
            "{}r:{}",
            VISIBLE_CACHE_PREFIX,
            role_ids.iter().map(|r| r.to_string()).collect::<Vec<_>>().join(",")
        )
    }
}

/// 全量失效 visible 缓存（管理页卡片写操作后调用；低频操作，直接 keys+del）
pub async fn invalidate_visible_cards() {
    if let Ok(keys) = CONTEXT.cache_service.keys(&format!("{}*", VISIBLE_CACHE_PREFIX)).await {
        for k in keys {
            let _ = CONTEXT.cache_service.del(&k).await;
        }
    }
}

/// 当前用户可见卡片列表（按页动态渲染）
///
/// - 超级管理员/系统管理员（user_type=1）：返回全部启用卡片
/// - 普通用户：仅返回其角色关联的启用卡片
/// - 三期 A2：结果按角色集合缓存（mem/redis 双模式），管理页写操作即时失效
pub async fn get_visible_cards(db: &DbConn, user_id: i64) -> Result<Vec<DashboardCardVO>> {
    let all = DashboardCardModel::find_all_enabled(db).await?;
    if all.is_empty() {
        return Ok(vec![]);
    }

    // 管理员直接返回全部
    let admin = admin_service::get_by_detail(db, &Some(user_id)).await?;
    let is_admin = admin.user_type == Some(1);

    // 缓存键按角色集合构建：角色增减即换 key，天然规避用户角色变更后的陈旧读
    let mut role_ids: Vec<i64> = if is_admin {
        vec![]
    } else {
        AdminRoleMergeModel::find_by_admin_id(db, &Some(user_id))
            .await
            .map_err(|e| Error::from(e.to_string()))?
            .into_iter()
            .filter_map(|m| m.role_id)
            .collect()
    };
    role_ids.sort();
    role_ids.dedup();
    let cache_key = visible_cache_key(is_admin, &role_ids);
    if let Ok(cached) = CONTEXT.cache_service.get_json::<Vec<DashboardCardVO>>(&cache_key).await {
        return Ok(cached);
    }

    let vo_list: Vec<DashboardCardVO> = if is_admin {
        all.into_iter().map(vo_from_model).collect()
    } else {
        if role_ids.is_empty() {
            return Ok(vec![]);
        }
        let card_ids = DashboardCardRoleMergeModel::find_card_ids_by_role_ids(db, &role_ids)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        all.into_iter()
            .filter(|c| card_ids.contains(&c.id))
            .map(vo_from_model)
            .collect()
    };

    let _ = CONTEXT.cache_service.set_string_ex(
        &cache_key,
        &serde_json::to_string(&vo_list).unwrap_or_default(),
        Some(Duration::from_secs(VISIBLE_CACHE_TTL_SECS)),
    ).await;
    Ok(vo_list)
}

/// 角色视角预览（管理页增强，方案 6-#9）：返回该角色可见的启用卡片组合
///
/// 与 get_visible_cards 普通用户语义一致：启用卡片 ∩ 角色关联卡片；
/// 低频管理操作不读缓存，避免 visible 缓存键引入角色维度
pub async fn get_role_visible_cards(db: &DbConn, role_id: i64) -> Result<Vec<DashboardCardVO>> {
    if role_id <= 0 {
        return Err(Error::from("角色ID不能为空"));
    }
    let role_exists = role::Entity::find()
        .filter(role::Column::Id.eq(role_id))
        .filter(role::Column::Deleted.ne(2))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    if role_exists.is_none() {
        return Err(Error::from("角色不存在"));
    }
    let all = DashboardCardModel::find_all_enabled(db).await?;
    if all.is_empty() {
        return Ok(vec![]);
    }
    let role_ids = vec![role_id];
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
        default_x: c.default_x,
        default_y: c.default_y,
        default_w: c.default_w,
        default_h: c.default_h,
        status: c.status,
        remark: c.remark,
        role_ids: vec![],
        create_time: c.create_time,
        update_time: c.update_time,
    }
}

/// 工作台卡片化总开关（方案 4.5-2：workspace_card_enabled，默认开；关闭即全员回旧渲染）
pub async fn is_card_mode_enabled(db: &DbConn) -> bool {
    match crate::modules::system::model::config::ConfigModel::find_by_key(db, "workspace_card_enabled").await {
        Ok(Some(config)) => config.config_value.map(|v| v.trim() == "1").unwrap_or(true),
        _ => true,
    }
}

// ==================== 二期：布局与多工作台（方案 5.2） ====================

/// 布局条目边界校验（12 列栅格，卡片内部 overflow 滚动——方案 5.4-2）
fn validate_layout_items(items: &[CardLayoutItem]) -> Result<()> {
    for item in items {
        let code = item.card_code.as_ref().map_or("", |s| s.trim());
        if code.is_empty() {
            return Err(Error::from("布局条目缺少 card_code"));
        }
        let w = item.w.unwrap_or(12);
        let h = item.h.unwrap_or(6);
        if !(1..=12).contains(&w) {
            return Err(Error::from(format!("卡片 {} 宽度须在 1-12 列之间", code)));
        }
        if h < 1 {
            return Err(Error::from(format!("卡片 {} 高度不能小于 1", code)));
        }
        if item.x.unwrap_or(0) < 0 || item.y.unwrap_or(0) < 0 {
            return Err(Error::from(format!("卡片 {} 坐标不能为负", code)));
        }
    }
    Ok(())
}

/// 设计器模板布局（管理员维护；含停用卡片置灰展示，方案 5.2）
pub async fn get_card_layout(db: &DbConn, page_key: &str) -> Result<Vec<CardLayoutVO>> {
    let cards = DashboardCardModel::find_by_page_key(db, page_key)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(cards
        .into_iter()
        .map(|c| CardLayoutVO {
            id: c.id,
            card_code: c.card_code,
            card_name: c.card_name,
            page_key: c.page_key,
            status: c.status,
            x: c.default_x.unwrap_or(0),
            y: c.default_y.unwrap_or(0),
            w: c.default_w.unwrap_or(12),
            h: c.default_h.unwrap_or(6),
        })
        .collect())
}

/// 设计器保存模板布局（整页全量提交，逐卡按编码更新，事务原子执行）
pub async fn save_card_layout(db: &DbConn, req: &CardLayoutSaveRequest) -> Result<i64> {
    let page_key = req.page_key.as_ref().map_or("", |s| s.trim());
    if page_key.is_empty() {
        return Err(Error::from("page_key 不能为空"));
    }
    let items = req.items.clone().unwrap_or_default();
    validate_layout_items(&items)?;

    let result = db
        .transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move {
                let mut affected: i64 = 0;
                for item in &items {
                    let code = item.card_code.as_ref().map_or("", |s| s.trim());
                    affected += DashboardCardModel::update_layout_by_code(
                        txn,
                        code,
                        item.x.unwrap_or(0),
                        item.y.unwrap_or(0),
                        item.w.unwrap_or(12),
                        item.h.unwrap_or(6),
                    )
                    .await? as i64;
                }
                Ok(affected)
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    invalidate_visible_cards().await;
    Ok(result)
}

/// 个人布局（模板布局 + 个人覆盖合并，仅启用卡片；方案 5.2：仅需登录）
pub async fn get_user_layout(db: &DbConn, user_id: i64, page_key: &str) -> Result<Vec<UserLayoutVO>> {
    let cards = DashboardCardModel::find_all_enabled(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    let overrides = DashboardUserLayoutModel::find_by_user_page(db, user_id, page_key)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    let mut override_map: std::collections::HashMap<String, dashboard_user_layout::Model> =
        std::collections::HashMap::new();
    for o in overrides {
        if let Some(code) = o.card_code.clone() {
            override_map.insert(code, o);
        }
    }
    // 交叉场景（方案 5.4-3）：停用/不可见卡片记录保留不返回，角色恢复自动回来
    Ok(cards
        .into_iter()
        .filter(|c| c.page_key.as_ref().map_or(false, |pk| pk == page_key))
        .map(|c| {
            let code = c.card_code.clone().unwrap_or_default();
            let o = override_map.get(&code);
            UserLayoutVO {
                card_code: c.card_code,
                x: o.and_then(|m| m.x).or(c.default_x).unwrap_or(0),
                y: o.and_then(|m| m.y).or(c.default_y).unwrap_or(0),
                w: o.and_then(|m| m.w).or(c.default_w).unwrap_or(12),
                h: o.and_then(|m| m.h).or(c.default_h).unwrap_or(6),
                hidden: o.and_then(|m| m.hidden).map(|v| v as i32).unwrap_or(0),
            }
        })
        .collect())
}

/// 个人布局边界校验（UserLayoutItem 版本：与设计器校验规则一致，方案 5.4-2）
fn validate_user_layout_items(items: &[UserLayoutItem]) -> Result<()> {
    for item in items {
        let code = item.card_code.as_ref().map_or("", |s| s.trim());
        if code.is_empty() {
            return Err(Error::from("布局条目缺少 card_code"));
        }
        let w = item.w.unwrap_or(12);
        let h = item.h.unwrap_or(6);
        if !(1..=12).contains(&w) {
            return Err(Error::from(format!("卡片 {} 宽度须在 1-12 列之间", code)));
        }
        if h < 1 {
            return Err(Error::from(format!("卡片 {} 高度不能小于 1", code)));
        }
        if item.x.unwrap_or(0) < 0 || item.y.unwrap_or(0) < 0 {
            return Err(Error::from(format!("卡片 {} 坐标不能为负", code)));
        }
    }
    Ok(())
}

/// 保存个人布局（reset=true 清空该页覆盖回模板；否则整页删除重插，事务原子执行）
pub async fn save_user_layout(db: &DbConn, user_id: i64, req: &UserLayoutSaveRequest) -> Result<i64> {
    let page_key = req.page_key.as_ref().map_or("", |s| s.trim());
    if page_key.is_empty() {
        return Err(Error::from("page_key 不能为空"));
    }
    if req.reset.unwrap_or(false) {
        let n = DashboardUserLayoutModel::delete_by_user_page(db, user_id, page_key)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        return Ok(n as i64);
    }
    let items = req.cards.clone().unwrap_or_default();
    validate_user_layout_items(&items)?;
    for item in &items {
        // 卡片须存在（不要求启用/属于该页：停用或角色暂不可见时记录保留——方案 5.4-3）
        let code = item.card_code.as_ref().map_or("", |s| s.trim());
        if DashboardCardModel::find_by_code(db, code, None).await? == 0 {
            return Err(Error::from(format!("卡片 {} 不存在", code)));
        }
    }

    let page_key = page_key.to_string();
    let result = db
        .transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move {
                DashboardUserLayoutModel::delete_by_user_page(txn, user_id, &page_key).await?;
                DashboardUserLayoutModel::insert_batch(txn, user_id, &page_key, &items).await
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(result)
}
