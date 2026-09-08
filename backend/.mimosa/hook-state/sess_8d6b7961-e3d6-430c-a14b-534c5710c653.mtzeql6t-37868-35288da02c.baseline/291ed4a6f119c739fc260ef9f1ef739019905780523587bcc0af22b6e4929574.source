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
use crate::modules::crm::entity::{lead, lead_pool, lead_pool_config, lead_pool_member};
use crate::modules::crm::model::lead::{LeadDetailVO, LeadListVO};
use crate::modules::crm::model::lead_pool::{PoolIdQuery, PoolLeadPageQuery, PoolLeadPageVO, PoolListQuery, PoolListVO, PoolSaveRequest, PoolStatusUpdateQuery};
use crate::modules::crm::service::delete_guard_service;
use crate::modules::crm::service::pool_config_service;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use std::collections::{HashMap, HashSet};

/// 默认公海池 ID（v56 种子固定为 1；代码层 pool_id NULL 亦映射到默认池，见开发方案 §3.1/§4.3）
pub const DEFAULT_POOL_ID: i64 = 1;

/// 分页查询公海池（含成员数、在池线索数、池配置）
pub async fn page(db: &DbConn, query: &PoolListQuery) -> Result<ResultPage<Vec<PoolListVO>>> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let mut select = lead_pool::Entity::find()
        .filter(lead_pool::Column::Deleted.eq(0));

    if let Some(k) = query.keywords.as_ref().filter(|v| !v.trim().is_empty()) {
        select = select.filter(lead_pool::Column::Name.contains(k));
    }
    if let Some(s) = query.status {
        select = select.filter(lead_pool::Column::Status.eq(s));
    }

    let paginator = select
        .order_by_asc(lead_pool::Column::Sort)
        .order_by_asc(lead_pool::Column::Id)
        .paginate(db, page_size as u64);
    let total = paginator.num_items().await? as i64;
    let rows = paginator.fetch_page((page - 1) as u64).await?;

    let pool_ids: Vec<i64> = rows.iter().map(|p| p.id).collect();
    let member_counts = count_members_batch(db, &pool_ids).await?;
    let lead_counts = pool_lead_counts(db, &pool_ids).await?;
    let config_map = load_config_map(db, &pool_ids).await?;

    let items: Vec<PoolListVO> = rows
        .into_iter()
        .map(|p| {
            let pid = p.id;
            PoolListVO {
                id: Some(pid),
                name: p.name,
                description: p.description,
                status: p.status,
                sort: p.sort,
                is_default: p.is_default,
                create_time: p.create_time,
                update_time: p.update_time,
                member_count: member_counts.get(&pid).copied(),
                lead_count: lead_counts.get(&pid).copied(),
                config: config_map.get(&pid).cloned().flatten(),
            }
        })
        .collect();

    Ok(ResultPage::new(items, total, page, page_size))
}

/// 池详情（带统计与配置）
pub async fn find_vo_by_id(db: &DbConn, id: i64) -> Result<PoolListVO> {
    let model = lead_pool::Entity::find()
        .filter(lead_pool::Column::Id.eq(id))
        .filter(lead_pool::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("公海池不存在".to_string()))?;

    let pid = model.id;
    let member_counts = count_members_batch(db, &[pid]).await?;
    let lead_counts = pool_lead_counts(db, &[pid]).await?;
    let config_map = load_config_map(db, &[pid]).await?;

    Ok(PoolListVO {
        id: Some(pid),
        name: model.name,
        description: model.description,
        status: model.status,
        sort: model.sort,
        is_default: model.is_default,
        create_time: model.create_time,
        update_time: model.update_time,
        member_count: member_counts.get(&pid).copied(),
        lead_count: lead_counts.get(&pid).copied(),
        config: config_map.get(&pid).cloned().flatten(),
    })
}

/// 新增公海池（名称唯一校验；同步创建默认配置行）
pub async fn insert(db: &DbConn, req: &PoolSaveRequest, created_by: i64) -> Result<i64> {
    let name = req
        .name
        .as_ref()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| Error::from("池名称不能为空".to_string()))?;

    check_name_unique(db, &name, None).await?;

    let now = chrono::Local::now().naive_local().to_owned();
    let active = lead_pool::ActiveModel {
        name: Set(Some(name)),
        description: Set(req.description.clone()),
        status: Set(Some(1)),
        sort: Set(Some(req.sort.unwrap_or(0))),
        is_default: Set(Some(0)),
        created_by: Set(Some(created_by)),
        create_time: Set(Some(now.clone())),
        updated_by: Set(Some(created_by)),
        update_time: Set(Some(now)),
        deleted: Set(Some(0)),
        ..Default::default()
    };
    let model = active.insert(db).await?;

    // 新池同步创建配置行（取 DB 默认参数）
    pool_config_service::ensure_config(db, model.id).await?;
    Ok(model.id)
}

/// 更新公海池（默认池允许改名称/描述/排序，不允许改状态标识）
pub async fn update(db: &DbConn, req: &PoolSaveRequest, updated_by: i64) -> Result<i64> {
    let id = req.id.ok_or_else(|| Error::from("池ID不能为空".to_string()))?;
    let model = lead_pool::Entity::find()
        .filter(lead_pool::Column::Id.eq(id))
        .filter(lead_pool::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("公海池不存在".to_string()))?;

    if let Some(name) = req.name.as_ref().map(|v| v.trim().to_string()).filter(|v| !v.is_empty()) {
        check_name_unique(db, &name, Some(id)).await?;
        let mut active: lead_pool::ActiveModel = model.into();
        active.name = Set(Some(name));
        active.description = Set(req.description.clone());
        active.sort = Set(Some(req.sort.unwrap_or(0)));
        active.updated_by = Set(Some(updated_by));
        active.update_time = Set(Some(chrono::Local::now().naive_local().to_owned()));
        let result = active.update(db).await?;
        Ok(result.id)
    } else {
        let mut active: lead_pool::ActiveModel = model.into();
        if req.description.is_some() {
            active.description = Set(req.description.clone());
        }
        if let Some(sort) = req.sort {
            active.sort = Set(Some(sort));
        }
        active.updated_by = Set(Some(updated_by));
        active.update_time = Set(Some(chrono::Local::now().naive_local().to_owned()));
        let result = active.update(db).await?;
        Ok(result.id)
    }
}

/// 批量删除公海池（软删；默认池禁删；在池线索未清空禁删）
pub async fn batch_delete_by_ids(db: &DbConn, ids: &[i64], deleted_by: i64) -> Result<i64> {
    if ids.is_empty() {
        return Ok(0);
    }
    let models = lead_pool::Entity::find()
        .filter(lead_pool::Column::Id.is_in(ids.to_vec()))
        .filter(lead_pool::Column::Deleted.eq(0))
        .all(db)
        .await?;
    if models.len() != ids.len() {
        return Err(Error::from("部分公海池不存在或已删除，请刷新后重试".to_string()));
    }

    let lead_counts = pool_lead_counts(db, ids).await?;
    for m in &models {
        if m.is_default.unwrap_or(0) == 1 {
            return Err(Error::from(format!("【{}】默认公海池不允许删除", m.name.clone().unwrap_or_default())));
        }
        let cnt = lead_counts.get(&m.id).copied().unwrap_or(0);
        if cnt > 0 {
            return Err(Error::from(format!(
                "【{}】仍有 {} 条在池线索，请先迁出或分配后再删除",
                m.name.clone().unwrap_or_default(),
                cnt
            )));
        }
    }

    let now = chrono::Local::now().naive_local().to_owned();
    let mut affected: i64 = 0;
    for m in &models {
        let mut active: lead_pool::ActiveModel = m.clone().into();
        active.deleted = Set(Some(1));
        active.delete_by = Set(Some(deleted_by));
        active.delete_time = Set(Some(now.clone()));
        active.update_time = Set(Some(now.clone()));
        active.update(db).await?;
        // 同步清理成员关系（成员表为物理删除设计）
        lead_pool_member::Entity::delete_many()
            .filter(lead_pool_member::Column::PoolId.eq(m.id))
            .exec(db)
            .await?;
        affected += 1;
    }
    Ok(affected)
}

/// 更新池状态（默认池不可停用；停用时在池线索必须为空）
pub async fn update_status(db: &DbConn, query: &PoolStatusUpdateQuery, updated_by: i64) -> Result<i64> {
    let id = query.id.ok_or_else(|| Error::from("池ID不能为空".to_string()))?;
    let status = query.status.ok_or_else(|| Error::from("目标状态不能为空".to_string()))?;
    if status != 1 && status != 2 {
        return Err(Error::from("非法的状态值".to_string()));
    }

    let model = lead_pool::Entity::find()
        .filter(lead_pool::Column::Id.eq(id))
        .filter(lead_pool::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("公海池不存在".to_string()))?;

    if model.is_default.unwrap_or(0) == 1 && status == 2 {
        return Err(Error::from("默认公海池不允许停用".to_string()));
    }
    if status == 2 {
        let cnt = pool_lead_counts(db, &[id]).await?.get(&id).copied().unwrap_or(0);
        if cnt > 0 {
            return Err(Error::from(format!("该池仍有 {} 条在池线索，请先迁出或分配后再停用", cnt)));
        }
    }

    let mut active: lead_pool::ActiveModel = model.into();
    active.status = Set(Some(status));
    active.updated_by = Set(Some(updated_by));
    active.update_time = Set(Some(chrono::Local::now().naive_local().to_owned()));
    let result = active.update(db).await?;
    Ok(result.id)
}

/// 公海工作台可见池：我管理的池 ∪ 我所在的启用池
pub async fn my_pools(db: &DbConn, user_id: i64) -> Result<Vec<crate::modules::crm::model::lead_pool::MyPoolVO>> {
    // 我所在的池（含管理/普通成员）
    let member_rows = lead_pool_member::Entity::find()
        .filter(lead_pool_member::Column::UserId.eq(user_id))
        .all(db)
        .await?;
    let mut pool_ids: Vec<i64> = member_rows.iter().map(|m| m.pool_id.unwrap_or_default()).collect();
    pool_ids.sort();
    pool_ids.dedup();

    if pool_ids.is_empty() {
        return Ok(Vec::new());
    }

    let pools = lead_pool::Entity::find()
        .filter(lead_pool::Column::Id.is_in(pool_ids.clone()))
        .filter(lead_pool::Column::Deleted.eq(0))
        .filter(lead_pool::Column::Status.eq(1))
        .order_by_desc(lead_pool::Column::IsDefault)
        .order_by_asc(lead_pool::Column::Sort)
        .all(db)
        .await?;

    let visible_ids: Vec<i64> = pools.iter().map(|p| p.id).collect();
    let lead_counts = pool_lead_counts(db, &visible_ids).await?;
    let config_map = load_config_map(db, &visible_ids).await?;

    let items = pools
        .into_iter()
        .map(|p| {
            let pid = p.id;
            let is_manager = member_rows
                .iter()
                .any(|m| m.pool_id == Some(pid) && m.member_type == Some(1));
            let config = config_map.get(&pid).cloned().flatten();
            crate::modules::crm::model::lead_pool::MyPoolVO {
                id: Some(pid),
                name: p.name,
                is_default: p.is_default,
                status: p.status,
                claim_mode: config.as_ref().and_then(|c| c.claim_mode),
                auto_assign_enabled: config.as_ref().and_then(|c| c.auto_assign_enabled),
                mask_fields: config.as_ref().and_then(|c| c.mask_fields.clone()),
                is_manager: Some(is_manager),
                lead_count: lead_counts.get(&pid).copied(),
            }
        })
        .collect();
    Ok(items)
}

/// 用户是否为指定池的池管理员
pub async fn is_pool_manager(db: &DbConn, pool_id: i64, user_id: i64) -> Result<bool> {
    let cnt = lead_pool_member::Entity::find()
        .filter(lead_pool_member::Column::PoolId.eq(pool_id))
        .filter(lead_pool_member::Column::UserId.eq(user_id))
        .filter(lead_pool_member::Column::MemberType.eq(1))
        .count(db)
        .await?;
    Ok(cnt > 0)
}

/// 用户是否为指定池的成员（含管理员）
pub async fn is_pool_member(db: &DbConn, pool_id: i64, user_id: i64) -> Result<bool> {
    let cnt = lead_pool_member::Entity::find()
        .filter(lead_pool_member::Column::PoolId.eq(pool_id))
        .filter(lead_pool_member::Column::UserId.eq(user_id))
        .count(db)
        .await?;
    Ok(cnt > 0)
}

/// 批量统计各池在池未分配线索数（口径：pool_id 命中 + 未分配 + 未删除 + 未转客户）
pub async fn pool_lead_counts(db: &DbConn, pool_ids: &[i64]) -> Result<HashMap<i64, i64>> {
    if pool_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = lead::Entity::find()
        .select_only()
        .column(lead::Column::PoolId)
        .column_as(lead::Column::Id.count(), "cnt")
        .filter(lead::Column::PoolId.is_in(pool_ids.to_vec()))
        .filter(lead::Column::Deleted.eq(0))
        .filter(lead::Column::AssignedTo.is_null())
        .filter(lead::Column::ConvertedToCustomerId.is_null())
        .group_by(lead::Column::PoolId)
        .into_tuple::<(i64, i64)>()
        .all(db)
        .await?;
    Ok(rows.into_iter().collect())
}

/// 批量统计各池成员数
async fn count_members_batch(db: &DbConn, pool_ids: &[i64]) -> Result<HashMap<i64, i64>> {
    if pool_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = lead_pool_member::Entity::find()
        .select_only()
        .column(lead_pool_member::Column::PoolId)
        .column_as(lead_pool_member::Column::Id.count(), "cnt")
        .filter(lead_pool_member::Column::PoolId.is_in(pool_ids.to_vec()))
        .group_by(lead_pool_member::Column::PoolId)
        .into_tuple::<(i64, i64)>()
        .all(db)
        .await?;
    Ok(rows.into_iter().collect())
}

/// 批量加载池配置并转 VO（键为池ID）
pub async fn load_config_map(
    db: &DbConn,
    pool_ids: &[i64],
) -> Result<HashMap<i64, Option<crate::modules::crm::model::lead_pool::PoolConfigVO>>> {
    if pool_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = lead_pool_config::Entity::find()
        .filter(lead_pool_config::Column::PoolId.is_in(pool_ids.to_vec()))
        .all(db)
        .await?;
    let mut map: HashMap<i64, Option<crate::modules::crm::model::lead_pool::PoolConfigVO>> = HashMap::new();
    for pid in pool_ids {
        map.insert(*pid, None);
    }
    for row in rows {
        map.insert(row.pool_id, Some(pool_config_service::model_to_vo(&row)));
    }
    Ok(map)
}

/// 池名称唯一校验（未删除范围内）
async fn check_name_unique(db: &DbConn, name: &str, exclude_id: Option<i64>) -> Result<()> {
    let mut select = lead_pool::Entity::find()
        .filter(lead_pool::Column::Name.eq(name))
        .filter(lead_pool::Column::Deleted.eq(0));
    if let Some(exclude) = exclude_id {
        select = select.filter(lead_pool::Column::Id.ne(exclude));
    }
    let cnt = select.count(db).await?;
    if cnt > 0 {
        return Err(Error::from("池名称已存在".to_string()));
    }
    Ok(())
}

/// 按 ID 查询池（供其他服务校验使用）
pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<lead_pool::Model>> {
    lead_pool::Entity::find()
        .filter(lead_pool::Column::Id.eq(id))
        .filter(lead_pool::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))
}

/// 兼容查询参数入口（InfoId 风格）
pub async fn find_vo_by_query(db: &DbConn, query: &PoolIdQuery) -> Result<PoolListVO> {
    let id = query.pool_id.ok_or_else(|| Error::from("池ID不能为空".to_string()))?;
    find_vo_by_id(db, id).await
}

/// 公海池线索分页（公海优化 §5.3/§5.9）
/// - 可见范围 = 我的池 ∪ 我管理的池（超管全量）；指定 pool_id 时须在可见范围内
/// - 返回池级渲染参数 claim_mode / mask_fields（指定池时）
/// - 脱敏：普通池成员对 mask_fields 命中字段打码；池管理员与超管明文
pub async fn pool_lead_page(
    db: &DbConn,
    query: &PoolLeadPageQuery,
    current_user_id: i64,
) -> Result<PoolLeadPageVO> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let is_super = delete_guard_service::is_super_admin(db, current_user_id).await?;

    // 一次查询成员关系：全部所在池 + 我管理的池
    let member_rows: Vec<lead_pool_member::Model> = if is_super {
        Vec::new()
    } else {
        lead_pool_member::Entity::find()
            .filter(lead_pool_member::Column::UserId.eq(current_user_id))
            .all(db)
            .await?
    };
    let member_pool_ids: HashSet<i64> = member_rows.iter().filter_map(|m| m.pool_id).collect();
    let managed_pools: HashSet<i64> = member_rows.iter()
        .filter(|m| m.member_type == Some(1))
        .filter_map(|m| m.pool_id)
        .collect();

    // 指定池时校验可见性（超管放行）
    if let Some(pid) = query.pool_id {
        if !is_super && !member_pool_ids.contains(&pid) {
            return Err(Error::from("无权访问该公海池".to_string()));
        }
    }

    // 在池口径：未分配 + 未删除 + 未转客户（唯一口径 assigned_to IS NULL，见 §3.1）
    let mut select = lead::Entity::find()
        .filter(lead::Column::Deleted.eq(0))
        .filter(lead::Column::AssignedTo.is_null())
        .filter(lead::Column::ConvertedToCustomerId.is_null());
    match query.pool_id {
        Some(pid) => {
            select = select.filter(lead::Column::PoolId.eq(pid));
        }
        // 超管"全部池"：不再限定 pool_id，天然覆盖 pool_id IS NULL 的默认池存量
        //（v3.1 §3.1 口径：assigned_to IS NULL 即公海，池归属由 pool_id（NULL=默认池）决定）
        None if is_super => {}
        None => {
            if member_pool_ids.is_empty() {
                return Ok(PoolLeadPageVO { claim_mode: None, mask_fields: None, total: 0, items: Vec::new() });
            }
            select = select.filter(lead::Column::PoolId.is_in(member_pool_ids.iter().copied().collect::<Vec<_>>()));
        }
    }
    if let Some(k) = query.keywords.as_ref().filter(|v| !v.trim().is_empty()) {
        select = select.filter(lead::Column::CompanyName.contains(k));
    }
    if let Some(s) = query.status {
        select = select.filter(lead::Column::Status.eq(s));
    }
    if let Some(l) = query.level.as_ref().filter(|v| !v.trim().is_empty()) {
        select = select.filter(lead::Column::Level.eq(l.clone()));
    }
    if let Some(s) = query.source.as_ref().filter(|v| !v.trim().is_empty()) {
        select = select.filter(lead::Column::Source.eq(s.clone()));
    }
    if let Some(c) = query.contact_name.as_ref().filter(|v| !v.trim().is_empty()) {
        select = select.filter(lead::Column::ContactName.contains(c));
    }
    if let Some(m) = query.mobile.as_ref().filter(|v| !v.trim().is_empty()) {
        select = select.filter(lead::Column::Mobile.contains(m));
    }
    if let Some(ind) = query.industry {
        select = select.filter(lead::Column::Industry.eq(ind));
    }

    let paginator = select
        .order_by_desc(lead::Column::CreateTime)
        .paginate(db, page_size as u64);
    let total = paginator.num_items().await? as i64;
    let rows = paginator.fetch_page((page - 1) as u64).await?;

    // 涉及池的配置（脱敏字段来源）：pool_id NULL 的行视为默认池（§3.1 代码层映射）
    let involved_pools: Vec<i64> = match query.pool_id {
        Some(pid) => vec![pid],
        None => rows
            .iter()
            .filter_map(|l| l.pool_id.or(Some(DEFAULT_POOL_ID)))
            .collect::<HashSet<i64>>()
            .into_iter()
            .collect(),
    };
    let config_map = load_config_map(db, &involved_pools).await?;

    // 填充创建人/负责人名称 + 标签
    let user_ids: Vec<i64> = rows.iter()
        .flat_map(|item| [item.created_by, item.assigned_to])
        .flatten()
        .collect();
    let user_map = crate::modules::system::service::admin_service::build_admin_name_map(db, user_ids).await;
    let lead_ids: Vec<i64> = rows.iter().map(|l| l.id).collect();
    let tag_map = crate::modules::crm::service::lead_service::batch_query_lead_tags(db, &lead_ids).await?;

    let items: Vec<LeadListVO> = rows
        .into_iter()
        .map(|item| {
            // 池归属以 pool_id 为准，NULL=默认池
            let pool_id = item.pool_id;
            let eff_pool_id = pool_id.unwrap_or(DEFAULT_POOL_ID);
            let mut vo: LeadListVO = item.into();
            vo.created_by_name = vo.created_by.and_then(|id| user_map.get(&id).cloned());
            vo.assignee = vo.assigned_to.and_then(|id| user_map.get(&id).cloned());
            vo.tags = tag_map.get(&vo.id.unwrap_or_default()).cloned();
            // §5.9 脱敏：普通池成员打码，池管理员/超管明文（NULL 池归属按默认池判定）
            if !is_super && !managed_pools.contains(&eff_pool_id) {
                let mask_fields = config_map
                    .get(&eff_pool_id)
                    .and_then(|c| c.as_ref())
                    .and_then(|c| c.mask_fields.clone())
                    .unwrap_or_default();
                if !mask_fields.is_empty() {
                    apply_lead_mask(&mut vo, &mask_fields);
                }
            }
            vo
        })
        .collect();

    // 池级渲染参数（指定池时返回，供前端渲染领取按钮与脱敏标记）
    let (claim_mode, mask_fields) = match query.pool_id {
        Some(pid) => {
            let cfg = config_map.get(&pid).cloned().flatten();
            let claim_mode = cfg.as_ref().and_then(|c| c.claim_mode);
            let mask_fields = cfg.and_then(|c| c.mask_fields);
            (claim_mode, mask_fields)
        }
        None => (None, None),
    };

    Ok(PoolLeadPageVO { claim_mode, mask_fields, total, items })
}

/// 按脱敏字段配置对线索 VO 打码（§5.9）
pub(crate) fn apply_lead_mask(vo: &mut LeadListVO, fields: &[String]) {
    for field in fields {
        match field.as_str() {
            "mobile" => {
                if let Some(v) = vo.mobile.as_ref() {
                    vo.mobile = Some(mask_contact_value("mobile", v));
                }
            }
            "phone" => {
                if let Some(v) = vo.phone.as_ref() {
                    vo.phone = Some(mask_contact_value("phone", v));
                }
            }
            "email" => {
                if let Some(v) = vo.email.as_ref() {
                    vo.email = Some(mask_contact_value("email", v));
                }
            }
            _ => {}
        }
    }
}

/// 详情 VO 脱敏（§5.9）：公海详情 DTO 与列表同一打码规则（mobile/phone/email）
pub(crate) fn apply_lead_detail_mask(vo: &mut LeadDetailVO, fields: &[String]) {
    for field in fields {
        match field.as_str() {
            "mobile" => {
                if let Some(v) = vo.mobile.as_ref() {
                    vo.mobile = Some(mask_contact_value("mobile", v));
                }
            }
            "phone" => {
                if let Some(v) = vo.phone.as_ref() {
                    vo.phone = Some(mask_contact_value("phone", v));
                }
            }
            "email" => {
                if let Some(v) = vo.email.as_ref() {
                    vo.email = Some(mask_contact_value("email", v));
                }
            }
            _ => {}
        }
    }
}

/// 按查看者身份判定线索详情可见性与脱敏（§5.9）
///
/// 返回 `Ok(None)` = 明文；`Ok(Some(fields))` = 按 mask_fields 打码；`Err` = 无权查看。
/// 口径：
/// - 超管 / 池管理员 / 负责人本人：始终明文；
/// - 私海（有负责人）：既有详情明文口径不回退（负责人/主管/数据权限用户均可查看，
///   数据权限已在列表层约束，此处不因"非池成员"拒绝——否则会回归主管查看下属私海等场景）；
/// - 公海（无负责人）：普通池成员按池 mask_fields 打码；非成员且非池管理员 → 拒绝（池间隔离）。
/// `lead_pool_id` 为 NULL 时按默认池（§3.1 代码层映射）参与成员/管理员判定。
pub(crate) async fn lead_detail_access(
    db: &DbConn,
    lead_pool_id: Option<i64>,
    lead_owner: Option<i64>,
    viewer_id: i64,
) -> Result<Option<Vec<String>>> {
    // 超管：明文且全量可见
    if delete_guard_service::is_super_admin(db, viewer_id).await? {
        return Ok(None);
    }
    let pool_id = lead_pool_id.unwrap_or(DEFAULT_POOL_ID);
    // 池管理员：明文
    if is_pool_manager(db, pool_id, viewer_id).await? {
        return Ok(None);
    }
    // 负责人本人（私海视角）：始终明文
    if lead_owner == Some(viewer_id) {
        return Ok(None);
    }
    // 私海（有负责人）：保持既有详情明文口径，不做池隔离（见函数头注释）
    if lead_owner.is_some() {
        return Ok(None);
    }
    // 公海（无负责人）：普通池成员按池配置脱敏（mask_fields 为空则等效明文）
    if is_pool_member(db, pool_id, viewer_id).await? {
        let config = load_config_map(db, &[pool_id])
            .await?
            .get(&pool_id)
            .cloned()
            .flatten();
        let fields = config.and_then(|c| c.mask_fields).unwrap_or_default();
        return Ok(if fields.is_empty() { None } else { Some(fields) });
    }
    Err(Error::from("无权查看该线索".to_string()))
}

/// 联系方式脱敏：mobile/phone 保留前3后4（138****5678），email 保留首字符与域名（a***@x.com）
pub(crate) fn mask_contact_value(field: &str, value: &str) -> String {
    if field == "email" {
        if let Some(at) = value.find('@') {
            let mut masked = String::new();
            if let Some(c) = value.chars().next() {
                masked.push(c);
            }
            masked.push_str("***");
            masked.push_str(&value[at..]);
            return masked;
        }
        return "***".to_string();
    }
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= 7 {
        return "*".repeat(chars.len());
    }
    let prefix: String = chars[..3].iter().collect();
    let suffix: String = chars[chars.len() - 4..].iter().collect();
    format!("{}****{}", prefix, suffix)
}

/// DbErr 转业务错误（供本模块内部统一）
#[allow(dead_code)]
fn map_db_err(e: DbErr) -> Error {
    Error::from(e.to_string())
}
