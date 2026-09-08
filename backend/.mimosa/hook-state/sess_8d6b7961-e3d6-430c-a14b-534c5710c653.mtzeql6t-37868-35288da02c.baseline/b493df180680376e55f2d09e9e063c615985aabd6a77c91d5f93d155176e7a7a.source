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
use crate::modules::crm::entity::{customer, customer_pool, customer_pool_config, customer_pool_member};
use crate::modules::crm::model::customer_pool::{
    CustomerPoolConfigVO, CustomerPoolIdQuery, CustomerPoolListQuery, CustomerPoolListVO, CustomerPoolSaveRequest,
    CustomerPoolStatusUpdateQuery, MyCustomerPoolVO,
};
use crate::modules::crm::service::customer_pool_config_service;
use crate::modules::crm::service::customer_pool_member_service;
use crate::modules::crm::service::delete_guard_service;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use std::collections::{HashMap, HashSet};

/// 默认客户公海池 ID（v57 种子固定为 1；代码层 pool_id NULL 亦映射到默认池）
pub const DEFAULT_CUSTOMER_POOL_ID: i64 = 1;

/// 分页查询客户公海池（含成员数、在池客户数、池配置、池管理员）
pub async fn page(db: &DbConn, query: &CustomerPoolListQuery) -> Result<ResultPage<Vec<CustomerPoolListVO>>> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let mut select = customer_pool::Entity::find()
        .filter(customer_pool::Column::Deleted.eq(0));

    if let Some(k) = query.keywords.as_ref().filter(|v| !v.trim().is_empty()) {
        select = select.filter(customer_pool::Column::Name.contains(k));
    }
    if let Some(s) = query.status {
        select = select.filter(customer_pool::Column::Status.eq(s));
    }

    let paginator = select
        .order_by_asc(customer_pool::Column::Sort)
        .order_by_asc(customer_pool::Column::Id)
        .paginate(db, page_size as u64);
    let total = paginator.num_items().await? as i64;
    let rows = paginator.fetch_page((page - 1) as u64).await?;

    let pool_ids: Vec<i64> = rows.iter().map(|p| p.id).collect();
    let admin_ids: Vec<i64> = rows.iter().filter_map(|p| p.pool_admin_id).collect();
    let admin_name_map = crate::modules::system::service::admin_service::build_admin_name_map(db, admin_ids).await;
    let member_counts = count_members_batch(db, &pool_ids).await?;
    let customer_counts = pool_customer_counts(db, &pool_ids).await?;
    let config_map = load_config_map(db, &pool_ids).await?;

    let items: Vec<CustomerPoolListVO> = rows
        .into_iter()
        .map(|p| {
            let pid = p.id;
            CustomerPoolListVO {
                id: Some(pid),
                name: p.name,
                description: p.description,
                status: p.status,
                sort: p.sort,
                is_default: p.is_default,
                pool_admin_id: p.pool_admin_id,
                pool_admin_name: p.pool_admin_id.and_then(|id| admin_name_map.get(&id).cloned()),
                create_time: p.create_time,
                update_time: p.update_time,
                member_count: member_counts.get(&pid).copied(),
                customer_count: customer_counts.get(&pid).copied(),
                config: config_map.get(&pid).cloned().flatten(),
            }
        })
        .collect();

    Ok(ResultPage::new(items, total, page, page_size))
}

/// 池详情（带统计与配置）
pub async fn find_vo_by_id(db: &DbConn, id: i64) -> Result<CustomerPoolListVO> {
    let model = find_by_id(db, id)
        .await?
        .ok_or_else(|| Error::from("客户公海池不存在".to_string()))?;

    let pid = model.id;
    let admin_name_map =
        crate::modules::system::service::admin_service::build_admin_name_map(db, model.pool_admin_id.into_iter().collect())
            .await;
    let member_counts = count_members_batch(db, &[pid]).await?;
    let customer_counts = pool_customer_counts(db, &[pid]).await?;
    let config_map = load_config_map(db, &[pid]).await?;

    Ok(CustomerPoolListVO {
        id: Some(pid),
        name: model.name,
        description: model.description,
        status: model.status,
        sort: model.sort,
        is_default: model.is_default,
        pool_admin_id: model.pool_admin_id,
        pool_admin_name: model.pool_admin_id.and_then(|id| admin_name_map.get(&id).cloned()),
        create_time: model.create_time,
        update_time: model.update_time,
        member_count: member_counts.get(&pid).copied(),
        customer_count: customer_counts.get(&pid).copied(),
        config: config_map.get(&pid).cloned().flatten(),
    })
}

/// 新增客户公海池（名称唯一校验；同步创建默认配置行与池管理员成员行）
pub async fn insert(db: &DbConn, req: &CustomerPoolSaveRequest, created_by: i64) -> Result<i64> {
    let name = req
        .name
        .as_ref()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| Error::from("池名称不能为空".to_string()))?;

    check_name_unique(db, &name, None).await?;

    let now = chrono::Local::now().naive_local().to_owned();
    let active = customer_pool::ActiveModel {
        name: Set(Some(name)),
        description: Set(req.description.clone()),
        status: Set(Some(1)),
        sort: Set(Some(req.sort.unwrap_or(0))),
        is_default: Set(Some(0)),
        pool_admin_id: Set(req.pool_admin_id),
        created_by: Set(Some(created_by)),
        create_time: Set(Some(now.clone())),
        updated_by: Set(Some(created_by)),
        update_time: Set(Some(now)),
        deleted: Set(Some(0)),
        ..Default::default()
    };
    let model = active.insert(db).await?;

    // 新池同步创建配置行（取 DB 默认参数）与池管理员成员行
    customer_pool_config_service::ensure_config(db, model.id).await?;
    if let Some(admin_id) = req.pool_admin_id {
        customer_pool_member_service::upsert_admin(db, model.id, admin_id, created_by).await?;
    }
    Ok(model.id)
}

/// 更新客户公海池（默认池允许改名称/描述/排序，不允许改状态标识）
pub async fn update(db: &DbConn, req: &CustomerPoolSaveRequest, updated_by: i64) -> Result<i64> {
    let id = req.id.ok_or_else(|| Error::from("池ID不能为空".to_string()))?;
    let model = find_by_id(db, id)
        .await?
        .ok_or_else(|| Error::from("客户公海池不存在".to_string()))?;

    if let Some(name) = req.name.as_ref().map(|v| v.trim().to_string()).filter(|v| !v.is_empty()) {
        check_name_unique(db, &name, Some(id)).await?;
    }

    let mut active: customer_pool::ActiveModel = model.into();
    if let Some(name) = req.name.as_ref().map(|v| v.trim().to_string()).filter(|v| !v.is_empty()) {
        active.name = Set(Some(name));
    }
    if req.description.is_some() {
        active.description = Set(req.description.clone());
    }
    if let Some(sort) = req.sort {
        active.sort = Set(Some(sort));
    }
    if req.pool_admin_id.is_some() {
        active.pool_admin_id = Set(req.pool_admin_id);
    }
    active.updated_by = Set(Some(updated_by));
    active.update_time = Set(Some(chrono::Local::now().naive_local().to_owned()));
    let result = active.update(db).await?;

    // 池管理员变更时同步成员行（member_type=1）
    if let Some(admin_id) = req.pool_admin_id {
        customer_pool_member_service::upsert_admin(db, id, admin_id, updated_by).await?;
    }
    Ok(result.id)
}

/// 批量删除客户公海池（软删；默认池禁删；在池客户未清空禁删）
pub async fn batch_delete_by_ids(db: &DbConn, ids: &[i64], deleted_by: i64) -> Result<i64> {
    if ids.is_empty() {
        return Ok(0);
    }
    let models = customer_pool::Entity::find()
        .filter(customer_pool::Column::Id.is_in(ids.to_vec()))
        .filter(customer_pool::Column::Deleted.eq(0))
        .all(db)
        .await?;
    if models.len() != ids.len() {
        return Err(Error::from("部分客户公海池不存在或已删除，请刷新后重试".to_string()));
    }

    let customer_counts = pool_customer_counts(db, ids).await?;
    for m in &models {
        if m.is_default.unwrap_or(0) == 1 {
            return Err(Error::from(format!(
                "【{}】默认客户公海池不允许删除",
                m.name.clone().unwrap_or_default()
            )));
        }
        let cnt = customer_counts.get(&m.id).copied().unwrap_or(0);
        if cnt > 0 {
            return Err(Error::from(format!(
                "【{}】仍有 {} 条在池客户，请先迁出或分配后再删除",
                m.name.clone().unwrap_or_default(),
                cnt
            )));
        }
    }

    let now = chrono::Local::now().naive_local().to_owned();
    let mut affected: i64 = 0;
    for m in &models {
        let mut active: customer_pool::ActiveModel = m.clone().into();
        active.deleted = Set(Some(1));
        active.delete_by = Set(Some(deleted_by));
        active.delete_time = Set(Some(now.clone()));
        active.update_time = Set(Some(now.clone()));
        active.update(db).await?;
        // 同步清理成员关系（成员表为物理删除设计）
        customer_pool_member::Entity::delete_many()
            .filter(customer_pool_member::Column::PoolId.eq(m.id))
            .exec(db)
            .await?;
        affected += 1;
    }
    Ok(affected)
}

/// 更新池状态（默认池不可停用；停用时在池客户必须为空）
pub async fn update_status(db: &DbConn, query: &CustomerPoolStatusUpdateQuery, updated_by: i64) -> Result<i64> {
    let id = query.id.ok_or_else(|| Error::from("池ID不能为空".to_string()))?;
    let status = query.status.ok_or_else(|| Error::from("目标状态不能为空".to_string()))?;
    if status != 1 && status != 2 {
        return Err(Error::from("非法的状态值".to_string()));
    }

    let model = find_by_id(db, id)
        .await?
        .ok_or_else(|| Error::from("客户公海池不存在".to_string()))?;

    if model.is_default.unwrap_or(0) == 1 && status == 2 {
        return Err(Error::from("默认客户公海池不允许停用".to_string()));
    }
    if status == 2 {
        let cnt = pool_customer_counts(db, &[id]).await?.get(&id).copied().unwrap_or(0);
        if cnt > 0 {
            return Err(Error::from(format!("该池仍有 {} 条在池客户，请先迁出或分配后再停用", cnt)));
        }
    }

    let mut active: customer_pool::ActiveModel = model.into();
    active.status = Set(Some(status));
    active.updated_by = Set(Some(updated_by));
    active.update_time = Set(Some(chrono::Local::now().naive_local().to_owned()));
    let result = active.update(db).await?;
    Ok(result.id)
}

/// 工作台可见池：我所在的启用池（含我管理的池）
pub async fn my_pools(db: &DbConn, user_id: i64) -> Result<Vec<MyCustomerPoolVO>> {
    let member_rows = customer_pool_member::Entity::find()
        .filter(customer_pool_member::Column::UserId.eq(user_id))
        .all(db)
        .await?;
    let mut pool_ids: Vec<i64> = member_rows.iter().filter_map(|m| m.pool_id).collect();
    pool_ids.sort();
    pool_ids.dedup();

    if pool_ids.is_empty() {
        return Ok(Vec::new());
    }

    let pools = customer_pool::Entity::find()
        .filter(customer_pool::Column::Id.is_in(pool_ids.clone()))
        .filter(customer_pool::Column::Deleted.eq(0))
        .filter(customer_pool::Column::Status.eq(1))
        .order_by_desc(customer_pool::Column::IsDefault)
        .order_by_asc(customer_pool::Column::Sort)
        .all(db)
        .await?;

    let visible_ids: Vec<i64> = pools.iter().map(|p| p.id).collect();
    let customer_counts = pool_customer_counts(db, &visible_ids).await?;
    let config_map = load_config_map(db, &visible_ids).await?;

    let items = pools
        .into_iter()
        .map(|p| {
            let pid = p.id;
            let is_manager = member_rows
                .iter()
                .any(|m| m.pool_id == Some(pid) && m.member_type == Some(1))
                || p.pool_admin_id == Some(user_id);
            let config = config_map.get(&pid).cloned().flatten();
            MyCustomerPoolVO {
                id: Some(pid),
                name: p.name,
                is_default: p.is_default,
                claim_mode: config.as_ref().and_then(|c| c.claim_mode),
                auto_assign_enabled: config.as_ref().and_then(|c| c.auto_assign_enabled),
                mask_fields: config.as_ref().and_then(|c| c.mask_fields.clone()),
                is_manager: Some(is_manager),
                customer_count: customer_counts.get(&pid).copied(),
            }
        })
        .collect();
    Ok(items)
}

/// 用户是否为指定池的池管理员（成员行 member_type=1 或 pool.pool_admin_id 指定）
pub async fn is_pool_manager(db: &DbConn, pool_id: i64, user_id: i64) -> Result<bool> {
    if let Some(pool) = find_by_id(db, pool_id).await? {
        if pool.pool_admin_id == Some(user_id) {
            return Ok(true);
        }
    }
    let cnt = customer_pool_member::Entity::find()
        .filter(customer_pool_member::Column::PoolId.eq(pool_id))
        .filter(customer_pool_member::Column::UserId.eq(user_id))
        .filter(customer_pool_member::Column::MemberType.eq(1))
        .count(db)
        .await?;
    Ok(cnt > 0)
}

/// 用户是否为指定池的成员（含管理员）
pub async fn is_pool_member(db: &DbConn, pool_id: i64, user_id: i64) -> Result<bool> {
    let cnt = customer_pool_member::Entity::find()
        .filter(customer_pool_member::Column::PoolId.eq(pool_id))
        .filter(customer_pool_member::Column::UserId.eq(user_id))
        .count(db)
        .await?;
    Ok(cnt > 0)
}

/// 当前用户对指定池的管理员/超管身份综合判定（超管视为管理员）
pub async fn is_manager_or_super(db: &DbConn, pool_id: i64, user_id: i64) -> Result<bool> {
    if delete_guard_service::is_super_admin(db, user_id).await? {
        return Ok(true);
    }
    is_pool_manager(db, pool_id, user_id).await
}

/// 批量统计各池在池未分配客户数（口径：pool_id 命中 + 未分配 + 未删除）
pub async fn pool_customer_counts(db: &DbConn, pool_ids: &[i64]) -> Result<HashMap<i64, i64>> {
    if pool_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = customer::Entity::find()
        .select_only()
        .column(customer::Column::PoolId)
        .column_as(customer::Column::Id.count(), "cnt")
        .filter(customer::Column::PoolId.is_in(pool_ids.to_vec()))
        .filter(customer::Column::Deleted.eq(0))
        .filter(customer::Column::AssignedTo.is_null())
        .group_by(customer::Column::PoolId)
        .into_tuple::<(Option<i64>, i64)>()
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|(pid, cnt)| (pid.unwrap_or(DEFAULT_CUSTOMER_POOL_ID), cnt))
        .collect())
}

/// 批量统计各池成员数
async fn count_members_batch(db: &DbConn, pool_ids: &[i64]) -> Result<HashMap<i64, i64>> {
    if pool_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = customer_pool_member::Entity::find()
        .select_only()
        .column(customer_pool_member::Column::PoolId)
        .column_as(customer_pool_member::Column::Id.count(), "cnt")
        .filter(customer_pool_member::Column::PoolId.is_in(pool_ids.to_vec()))
        .group_by(customer_pool_member::Column::PoolId)
        .into_tuple::<(i64, i64)>()
        .all(db)
        .await?;
    Ok(rows.into_iter().collect())
}

/// 批量加载池配置并转 VO（键为池ID）
pub async fn load_config_map(
    db: &DbConn,
    pool_ids: &[i64],
) -> Result<HashMap<i64, Option<CustomerPoolConfigVO>>> {
    if pool_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = customer_pool_config::Entity::find()
        .filter(customer_pool_config::Column::PoolId.is_in(pool_ids.to_vec()))
        .all(db)
        .await?;
    let mut map: HashMap<i64, Option<CustomerPoolConfigVO>> = HashMap::new();
    for pid in pool_ids {
        map.insert(*pid, None);
    }
    for row in rows {
        map.insert(row.pool_id, Some(customer_pool_config_service::model_to_vo(&row)));
    }
    Ok(map)
}

/// 池名称唯一校验（未删除范围内）
async fn check_name_unique(db: &DbConn, name: &str, exclude_id: Option<i64>) -> Result<()> {
    let mut select = customer_pool::Entity::find()
        .filter(customer_pool::Column::Name.eq(name))
        .filter(customer_pool::Column::Deleted.eq(0));
    if let Some(exclude) = exclude_id {
        select = select.filter(customer_pool::Column::Id.ne(exclude));
    }
    let cnt = select.count(db).await?;
    if cnt > 0 {
        return Err(Error::from("池名称已存在".to_string()));
    }
    Ok(())
}

/// 按 ID 查询池（供其他服务校验使用）
pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<customer_pool::Model>> {
    customer_pool::Entity::find()
        .filter(customer_pool::Column::Id.eq(id))
        .filter(customer_pool::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))
}

/// 兼容查询参数入口（InfoId 风格）
pub async fn find_vo_by_query(db: &DbConn, query: &CustomerPoolIdQuery) -> Result<CustomerPoolListVO> {
    let id = query.pool_id.ok_or_else(|| Error::from("池ID不能为空".to_string()))?;
    find_vo_by_id(db, id).await
}
