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
use crate::modules::crm::model::followup::{FollowupDetailVO, FollowupListQuery, FollowupListVO, FollowupModel, FollowupSaveDTO, FollowupSaveRequest, FollowupUpdateRequest, VisitCheckInRequest, VisitListQuery, VisitStatisticsVO};
use crate::modules::crm::model::work_log::WorkLogCreateDTO;
use crate::modules::crm::entity::{customer, followup, lead};
use crate::modules::crm::service::customer_service::get_accessible_user_ids;
use crate::modules::crm::service::work_log_service;
use crate::modules::system::entity::admin;
use crate::modules::system::service::admin_service::build_admin_name_map;
use crate::modules::system::service::role_service;
use chrono::Datelike;
use rust_decimal::prelude::FromPrimitive;
use sea_orm::DbConn;
use sea_orm::{ColumnTrait, EntityTrait, ActiveModelTrait, QueryFilter, Set, TransactionTrait, PaginatorTrait};

pub async fn insert(db: &DbConn, form_data: &FollowupSaveRequest, created_by: i64) -> Result<i64> {
    let txn = db.begin().await?;

    let mut dto: FollowupSaveDTO = form_data.clone().into();
    dto.created_by = Some(created_by);
    let result = FollowupModel::insert(&txn, &dto).await?;

    if let Some(lead_id) = form_data.lead_id {
        let mut lead_active: lead::ActiveModel = Default::default();

        if let Some(status_val) = form_data.lead_status {
            lead_active.status = Set(Some(status_val));
        }

        if let Some(next_date) = form_data.next_follow_date {
            let next_dt = chrono::NaiveDateTime::new(next_date, chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());
            lead_active.next_follow_at = Set(Some(next_dt));
        }

        lead_active.id = Set(lead_id);
        let _ = lead::Entity::update(lead_active).exec(&txn).await;
    }

    txn.commit().await?;

    // 工作日志埋点（客户跟进），不影响主业务
    let action_name = if form_data.lead_id.is_some() { "跟进线索" } else { "跟进客户" };
    let business_type = if form_data.lead_id.is_some() { "lead" } else { "customer" };
    let business_id = form_data.lead_id.or(form_data.customer_id).unwrap_or(0);
    let log_dto = WorkLogCreateDTO {
        user_id: created_by,
        user_name: None,
        action_type: Some(2),
        action_name: Some(action_name.to_string()),
        business_type: Some(business_type.to_string()),
        business_id: Some(business_id),
        business_title: None,
        description: form_data.content.clone(),
        result: Some(1),
        work_date: Some(chrono::Local::now().naive_local().date()),
    };
    let _ = work_log_service::insert(db, &log_dto).await;

    Ok(result)
}

pub async fn update(db: &DbConn, form_data: &FollowupUpdateRequest, _updated_by: i64) -> Result<i64> {
    let dto: FollowupSaveDTO = form_data.clone().into();
    let result = FollowupModel::update_by_id(&db, &form_data.id, &dto).await?;
    Ok(result)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let result = FollowupModel::batch_delete_by_ids(&db, &ids_vec).await?;
    Ok(result)
}

pub async fn find_by_id(db: &DbConn, id: i64) -> Result<FollowupDetailVO> {
    let result = FollowupModel::find_by_id(&db, id).await?;
    match result {
        Some(item) => {
            let mut vo: FollowupDetailVO = item.into();
            let created_by = vo.created_by;

            // 关联查询客户名称及客户负责人
            let mut customer_assignee_id: Option<i64> = None;
            if let Some(cid) = vo.customer_id {
                if let Some(c) = customer::Entity::find()
                    .filter(customer::Column::Deleted.eq(0))
                    .filter(customer::Column::Id.eq(cid))
                    .one(db).await?
                {
                    vo.customer_name = c.company_name.clone();
                    customer_assignee_id = c.assigned_to;
                }
            }

            // 合并客户负责人 + 创建人 一次 IN 查询用户名
            let user_ids: Vec<i64> = vec![customer_assignee_id, created_by].into_iter().flatten().collect();
            let name_map = build_admin_name_map(db, user_ids).await;
            if let Some(uid) = customer_assignee_id {
                vo.assignee_name = name_map.get(&uid).cloned();
            }
            if let Some(uid) = created_by {
                vo.created_by_name = name_map.get(&uid).cloned();
            }

            Ok(vo)
        }
        None => Err(Error::from("跟进记录不存在".to_string())),
    }
}

pub async fn list(db: &DbConn, query: &FollowupListQuery, current_user_id: i64) -> Result<ResultPage<Vec<FollowupListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let list_type = query.list_type.as_deref().unwrap_or("all");

    // 获取当前用户的数据权限范围
    let roles = role_service::select_by_admin_id(db, &Some(current_user_id)).await?;
    let data_scope = roles.iter().filter_map(|r| r.data_scope).min();

    // 当没有指定具体客户/线索/商机筛选时，按公司分组去重，每个公司只显示最新一条跟进
    let needs_grouping = query.customer_id.is_none()
        && query.lead_id.is_none()
        && query.opportunity_id.is_none();

    if needs_grouping {
        return list_grouped(db, query, current_user_id, data_scope, page, page_size, list_type).await;
    }

    let (list, total) = match list_type {
        "my" => {
            // 我的跟进：仅查询自己创建的跟进记录
            FollowupModel::select_in_page_by_creator_ids(
                &db, page, page_size,
                query.customer_id, query.lead_id, query.opportunity_id,
                query.only_customer, query.source_type,
                Some(vec![current_user_id]),
            ).await?
        }
        "subordinate" => {
            // 下属跟进：根据 data_scope 获取可见用户列表，排除自己
            let user_ids = match data_scope {
                Some(5) => {
                    // 仅本人数据权限的人，无法看到下属跟进
                    Vec::new()
                }
                Some(1) | None => {
                    // 全部数据权限：获取所有用户（排除自己）
                    let all_admins = admin::Entity::find()
                        .filter(admin::Column::Id.ne(current_user_id))
                        .all(db).await
                        .map_err(|e| Error::from(format!("查询用户列表失败: {}", e)))?;
                    all_admins.iter().map(|u| u.id).collect()
                }
                _ => {
                    get_accessible_user_ids(db, current_user_id, data_scope).await?
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|id| *id != current_user_id)
                        .collect::<Vec<_>>()
                }
            };
            // 即使 user_ids 为空（如 data_scope=5），也传 Some(vec![]) 让查询直接返回空结果
            FollowupModel::select_in_page_by_creator_ids(
                &db, page, page_size,
                query.customer_id, query.lead_id, query.opportunity_id,
                query.only_customer, query.source_type,
                Some(user_ids),
            ).await?
        }
        "todayFollow" => {
            // 今日跟进：按 data_scope 范围过滤 + 创建时间为今日
            let user_ids = get_accessible_user_ids(db, current_user_id, data_scope).await?;
            FollowupModel::select_today_follow_page(
                &db, page, page_size,
                query.customer_id, query.lead_id, query.opportunity_id,
                query.only_customer, query.source_type,
                user_ids,
            ).await?
        }
        _ => {
            // all：根据 data_scope 过滤
            match get_accessible_user_ids(db, current_user_id, data_scope).await? {
                None => {
                    // 全部数据 - 不过滤创建人
                    FollowupModel::select_in_page(
                        &db, page, page_size,
                        query.customer_id, query.lead_id, query.opportunity_id,
                        query.only_customer, query.source_type,
                    ).await?
                }
                Some(user_ids) => {
                    FollowupModel::select_in_page_by_creator_ids(
                        &db, page, page_size,
                        query.customer_id, query.lead_id, query.opportunity_id,
                        query.only_customer, query.source_type,
                        Some(user_ids),
                    ).await?
                }
            }
        }
    };

    let mut data: Vec<FollowupListVO> = list.into_iter().map(|item| item.into()).collect();
    fill_followup_vo_relations(db, &mut data).await?;
    Ok(ResultPage::new(data, total, page, page_size))
}

/// 按公司分组去重查询：每个公司只保留最新一条跟进记录
/// - 线索转客户后，原线索的跟进记录已继承 customer_id，与客户跟进记录合并为同一公司
/// - 未转客户的线索，按 lead_id 独立分组
async fn list_grouped(
    db: &DbConn,
    query: &FollowupListQuery,
    current_user_id: i64,
    data_scope: Option<i32>,
    page: i64,
    page_size: i64,
    list_type: &str,
) -> Result<ResultPage<Vec<FollowupListVO>>> {
    // 构建与普通查询一致的筛选参数
    let (creator_ids, time_range) = match list_type {
        "my" => (Some(vec![current_user_id]), None),
        "subordinate" => {
            let user_ids = match data_scope {
                Some(5) => Vec::new(),
                Some(1) | None => {
                    let all_admins = admin::Entity::find()
                        .filter(admin::Column::Id.ne(current_user_id))
                        .all(db).await
                        .map_err(|e| Error::from(format!("查询用户列表失败: {}", e)))?;
                    all_admins.iter().map(|u| u.id).collect()
                }
                _ => {
                    get_accessible_user_ids(db, current_user_id, data_scope).await?
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|id| *id != current_user_id)
                        .collect::<Vec<_>>()
                }
            };
            (Some(user_ids), None)
        }
        "todayFollow" => {
            let user_ids = get_accessible_user_ids(db, current_user_id, data_scope).await?;
            let today = chrono::Local::now().naive_local().date();
            let today_start = chrono::NaiveDateTime::new(today, chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());
            let today_end = chrono::NaiveDateTime::new(today, chrono::NaiveTime::from_hms_opt(23, 59, 59).unwrap());
            (user_ids, Some((today_start, today_end)))
        }
        _ => {
            // all：根据 data_scope 过滤
            let user_ids = get_accessible_user_ids(db, current_user_id, data_scope).await?;
            (user_ids, None)
        }
    };

    // 查询全部符合条件的跟进记录（不分页）
    let all_list = FollowupModel::select_all_internal(
        &db, None, None, None,
        query.only_customer, query.source_type,
        creator_ids, time_range,
    ).await?;

    // 按公司分组：COALESCE(customer_id, -lead_id)
    // - customer_id 不为空：按 customer_id 分组（含线索转客户后继承的记录）
    // - customer_id 为空：按 lead_id 分组（未转客户的线索）
    let mut grouped: std::collections::HashMap<i64, followup::Model> = std::collections::HashMap::new();
    for item in all_list {
        let key = item.customer_id.unwrap_or(-item.lead_id.unwrap_or(item.id));
        // 已按 create_time DESC 排序，第一条即最新，只保留最新记录
        grouped.entry(key).or_insert(item);
    }

    let mut data: Vec<FollowupListVO> = grouped.into_values()
        .map(|item| item.into())
        .collect();

    // 按创建时间降序排列
    data.sort_by(|a, b| b.create_time.cmp(&a.create_time));

    let total = data.len() as i64;

    // 内存分页
    let start = ((page - 1) * page_size) as usize;
    let mut data: Vec<FollowupListVO> = data.into_iter()
        .skip(start)
        .take(page_size as usize)
        .collect();

    fill_followup_vo_relations(db, &mut data).await?;
    Ok(ResultPage::new(data, total, page, page_size))
}

/// 填充 FollowupListVO 的关联字段：客户名称、客户负责人名称、回填 lead_id、创建人名称、线索名称
/// 优化：根据 source_type 分组收集 ID，只查询相关表，减少不必要的 IN 查询
async fn fill_followup_vo_relations(
    db: &DbConn,
    data: &mut Vec<FollowupListVO>,
) -> Result<()> {
    if data.is_empty() {
        return Ok(());
    }

    // 根据 source_type 分组收集 ID：
    // - source_type=1 (线索跟进)：只收集 lead_id，不收集 customer_id
    // - source_type=2/3 (客户/商机跟进)：只收集 customer_id，不收集 lead_id
    // - source_type=None 或其他：两者都收集（保险起见）
    let mut customer_ids_set: std::collections::HashSet<i64> = std::collections::HashSet::new();
    let mut lead_ids_set: std::collections::HashSet<i64> = std::collections::HashSet::new();
    let mut admin_ids_set: std::collections::HashSet<i64> = std::collections::HashSet::new();

    for vo in data.iter() {
        match vo.source_type {
            Some(1) => {
                // 线索跟进：只收集 lead_id
                if let Some(lid) = vo.lead_id {
                    lead_ids_set.insert(lid);
                }
            }
            Some(2) | Some(3) => {
                // 客户/商机跟进：只收集 customer_id
                if let Some(cid) = vo.customer_id {
                    customer_ids_set.insert(cid);
                }
            }
            _ => {
                // 未知类型：两者都收集
                if let Some(cid) = vo.customer_id {
                    customer_ids_set.insert(cid);
                }
                if let Some(lid) = vo.lead_id {
                    lead_ids_set.insert(lid);
                }
            }
        }
        // 跟进人ID（created_by）所有记录都需要
        if let Some(uid) = vo.created_by {
            admin_ids_set.insert(uid);
        }
    }

    // 批量查询客户表（仅 source_type != 1 时需要）
    let customers: std::collections::HashMap<i64, customer::Model> = if customer_ids_set.is_empty() {
        std::collections::HashMap::new()
    } else {
        let customer_ids: Vec<i64> = customer_ids_set.into_iter().collect();
        customer::Entity::find()
            .filter(customer::Column::Deleted.eq(0))
            .filter(customer::Column::Id.is_in(customer_ids))
            .all(db).await
            .map(|cs| cs.into_iter().map(|c| (c.id, c)).collect())
            .unwrap_or_default()
    };

    // 将客户负责人ID加入 admin_ids_set，统一一次查询
    for c in customers.values() {
        if let Some(aid) = c.assigned_to {
            admin_ids_set.insert(aid);
        }
    }

    // 批量查询线索表（仅 source_type == 1 或未知类型时需要）
    let leads_map: std::collections::HashMap<i64, lead::Model> = if lead_ids_set.is_empty() {
        std::collections::HashMap::new()
    } else {
        let lead_ids: Vec<i64> = lead_ids_set.into_iter().collect();
        lead::Entity::find()
            .filter(lead::Column::Deleted.eq(0))
            .filter(lead::Column::Id.is_in(lead_ids))
            .all(db).await
            .map(|ls| ls.into_iter().map(|l| (l.id, l)).collect())
            .unwrap_or_default()
    };

    // 批量查询 admin 用户名映射（跟进人 created_by + 客户负责人 assigned_to，一次 IN 查询）
    let admin_name_map = build_admin_name_map(db, admin_ids_set.into_iter().collect()).await;

    // 反查 lead_id：仅对 source_type != 1 且 lead_id 为空的客户跟进记录执行
    // 通过 lead.converted_to_customer_id 反查，使前端点击客户名称可打开线索详情
    let customer_ids_without_lead: Vec<i64> = data.iter()
        .filter(|vo| vo.source_type != Some(1))
        .filter(|vo| vo.lead_id.is_none())
        .filter_map(|vo| vo.customer_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let customer_to_lead: std::collections::HashMap<i64, i64> = if customer_ids_without_lead.is_empty() {
        std::collections::HashMap::new()
    } else {
        lead::Entity::find()
            .filter(lead::Column::Deleted.eq(0))
            .filter(lead::Column::ConvertedToCustomerId.is_in(customer_ids_without_lead))
            .all(db).await
            .map(|ls| ls.into_iter()
                .filter_map(|l| l.converted_to_customer_id.map(|cid| (cid, l.id)))
                .collect())
            .unwrap_or_default()
    };

    // 内存中填充字段
    for vo in data.iter_mut() {
        // 填充客户相关字段（仅 source_type != 1）
        if vo.source_type != Some(1) {
            if let Some(cid) = vo.customer_id {
                if let Some(c) = customers.get(&cid) {
                    vo.customer_name = c.company_name.clone();
                    // 客户的 assigned_to 即当前负责该客户的业务员
                    if let Some(assignee_id) = c.assigned_to {
                        vo.assignee_name = admin_name_map.get(&assignee_id).cloned();
                    }
                }
                // 客户跟进记录回填对应线索ID（线索转客户后关联）
                if vo.lead_id.is_none() {
                    if let Some(lid) = customer_to_lead.get(&cid) {
                        vo.lead_id = Some(*lid);
                        // 回填后顺便填充 lead_name（如果 leads_map 中有）
                        if let Some(l) = leads_map.get(lid) {
                            vo.lead_name = l.company_name.clone()
                                .or_else(|| l.contact_name.clone());
                        }
                    }
                }
            }
        }
        // 填充线索名称（source_type == 1）
        if vo.source_type == Some(1) {
            if let Some(lid) = vo.lead_id {
                if let Some(l) = leads_map.get(&lid) {
                    vo.lead_name = l.company_name.clone()
                        .or_else(|| l.contact_name.clone());
                }
            }
        }
        // 填充跟进人名称（所有记录）
        if let Some(uid) = vo.created_by {
            vo.created_by_name = admin_name_map.get(&uid).cloned();
        }
    }
    Ok(())
}

// ==================== 外勤拜访业务 ====================

/// 外勤拜访签到：创建一条 activity_type=2 的跟进记录，填入签到信息
///
/// - source_type 固定为 2（客户跟进）
/// - activity_type 固定为 2（拜访）
/// - check_in_time 设为当前时间
/// - 若 customer_id 存在，从 customer 表读取 latitude/longitude 存入 visit_customer_lat/lng
pub async fn visit_check_in(db: &DbConn, form_data: &VisitCheckInRequest, created_by: i64) -> Result<i64> {
    let txn = db.begin().await?;

    let now = chrono::Local::now().naive_local().to_owned();

    // 客户坐标快照：签到时从 customer 表读取 latitude/longitude
    let (customer_lat, customer_lng) = if let Some(cid) = form_data.customer_id {
        let c = customer::Entity::find()
            .filter(customer::Column::Deleted.eq(0))
            .filter(customer::Column::Id.eq(cid))
            .one(&txn).await
            .map_err(|e| Error::from(e.to_string()))?;
        if let Some(c) = c {
            (c.latitude, c.longitude)
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    let dto = FollowupSaveDTO {
        id: None,
        lead_id: form_data.lead_id,
        customer_id: form_data.customer_id,
        opportunity_id: form_data.opportunity_id,
        activity_type: Some(2),         // 拜访
        source_type: Some(2),           // 客户跟进
        content: form_data.content.clone(),
        next_follow_date: form_data.next_follow_date,
        duration_minutes: None,
        result: None,
        assigned_to: form_data.assigned_to,
        deleted: None,
        created_by: Some(created_by),
        create_time: Some(now),
        updated_by: None,
        update_time: Some(now),
        visit_address: form_data.visit_address.clone(),
        visit_latitude: form_data.visit_latitude.map(|v| rust_decimal::Decimal::from_f64(v).unwrap_or_default()),
        visit_longitude: form_data.visit_longitude.map(|v| rust_decimal::Decimal::from_f64(v).unwrap_or_default()),
        visit_accuracy: form_data.visit_accuracy.map(|v| rust_decimal::Decimal::from_f64(v).unwrap_or_default()),
        visit_photos: form_data.visit_photos.clone(),
        visit_distance: form_data.visit_distance.map(|v| rust_decimal::Decimal::from_f64(v).unwrap_or_default()),
        check_in_time: Some(now),
        check_out_time: None,
        visit_customer_lat: customer_lat,
        visit_customer_lng: customer_lng,
    };

    let result = FollowupModel::insert(&txn, &dto).await?;

    txn.commit().await?;

    // 工作日志埋点（外勤拜访），不影响主业务
    let business_type = if form_data.lead_id.is_some() { "lead" } else { "customer" };
    let business_id = form_data.lead_id.or(form_data.customer_id).unwrap_or(0);
    let log_dto = WorkLogCreateDTO {
        user_id: created_by,
        user_name: None,
        action_type: Some(2),
        action_name: Some("外勤拜访签到".to_string()),
        business_type: Some(business_type.to_string()),
        business_id: Some(business_id),
        business_title: None,
        description: form_data.content.clone(),
        result: Some(1),
        work_date: Some(chrono::Local::now().naive_local().date()),
    };
    let _ = work_log_service::insert(db, &log_dto).await;

    Ok(result)
}

/// 外勤拜访签退：更新指定拜访记录的 check_out_time
pub async fn visit_check_out(db: &DbConn, id: i64, _updated_by: i64) -> Result<i64> {
    // 先校验记录存在且为拜访类型
    let existing = FollowupModel::find_by_id(db, id).await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("拜访记录不存在".to_string()))?;

    if existing.activity_type != Some(2) {
        return Err(Error::from("该记录不是外勤拜访记录".to_string()));
    }

    if existing.check_out_time.is_some() {
        return Err(Error::from("该拜访记录已签退，不能重复签退".to_string()));
    }

    let now = chrono::Local::now().naive_local().to_owned();
    let result = FollowupModel::update_check_out_time(db, id, now).await
        .map_err(|e| Error::from(e.to_string()))?;

    if result == 0 {
        return Err(Error::from("签退失败，记录可能不存在或不是拜访类型".to_string()));
    }

    Ok(result)
}

/// 外勤拜访列表（筛选 activity_type=2 的跟进记录）
pub async fn visit_list(db: &DbConn, query: &VisitListQuery, current_user_id: i64) -> Result<ResultPage<Vec<FollowupListVO>>> {
    let page = query.page_num.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let list_type = query.list_type.as_deref().unwrap_or("all");

    // 获取当前用户的数据权限范围
    let roles = role_service::select_by_admin_id(db, &Some(current_user_id)).await?;
    let data_scope = roles.iter().filter_map(|r| r.data_scope).min();

    // 根据 list_type 确定创建人范围
    let creator_ids: Option<Vec<i64>> = match list_type {
        "my" => Some(vec![current_user_id]),
        "subordinate" => {
            let user_ids = match data_scope {
                Some(5) => Vec::new(),
                Some(1) | None => {
                    let all_admins = admin::Entity::find()
                        .filter(admin::Column::Id.ne(current_user_id))
                        .all(db).await
                        .map_err(|e| Error::from(format!("查询用户列表失败: {}", e)))?;
                    all_admins.iter().map(|u| u.id).collect()
                }
                _ => {
                    get_accessible_user_ids(db, current_user_id, data_scope).await?
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|id| *id != current_user_id)
                        .collect::<Vec<_>>()
                }
            };
            Some(user_ids)
        }
        _ => {
            // all：根据 data_scope 过滤
            get_accessible_user_ids(db, current_user_id, data_scope).await?
        }
    };

    let (list, total) = FollowupModel::select_visit_page(
        db, page, page_size,
        query.customer_id, query.lead_id,
        creator_ids, None,
    ).await
        .map_err(|e| Error::from(e.to_string()))?;

    let mut data: Vec<FollowupListVO> = list.into_iter().map(|item| item.into()).collect();
    fill_followup_vo_relations(db, &mut data).await?;
    Ok(ResultPage::new(data, total, page, page_size))
}

/// 外勤拜访详情（必须为 activity_type=2 的记录）
pub async fn visit_info(db: &DbConn, id: i64) -> Result<FollowupDetailVO> {
    let result = FollowupModel::find_by_id(db, id).await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("拜访记录不存在".to_string()))?;

    if result.activity_type != Some(2) {
        return Err(Error::from("该记录不是外勤拜访记录".to_string()));
    }

    let mut vo: FollowupDetailVO = result.into();
    let created_by = vo.created_by;

    // 关联查询客户名称及客户负责人
    let mut customer_assignee_id: Option<i64> = None;
    if let Some(cid) = vo.customer_id {
        if let Some(c) = customer::Entity::find()
            .filter(customer::Column::Deleted.eq(0))
            .filter(customer::Column::Id.eq(cid))
            .one(db).await?
        {
            vo.customer_name = c.company_name.clone();
            customer_assignee_id = c.assigned_to;
        }
    }

    // 合并客户负责人 + 创建人 一次 IN 查询用户名
    let user_ids: Vec<i64> = vec![customer_assignee_id, created_by].into_iter().flatten().collect();
    let name_map = build_admin_name_map(db, user_ids).await;
    if let Some(uid) = customer_assignee_id {
        vo.assignee_name = name_map.get(&uid).cloned();
    }
    if let Some(uid) = created_by {
        vo.created_by_name = name_map.get(&uid).cloned();
    }

    Ok(vo)
}

/// 外勤拜访统计（按人/按日统计）
pub async fn visit_statistics(db: &DbConn, current_user_id: i64) -> Result<VisitStatisticsVO> {
    // 获取当前用户的数据权限范围
    let roles = role_service::select_by_admin_id(db, &Some(current_user_id)).await?;
    let data_scope = roles.iter().filter_map(|r| r.data_scope).min();
    let creator_ids = get_accessible_user_ids(db, current_user_id, data_scope).await?;

    let creator_ids_ref = creator_ids.as_ref();

    // 总拜访次数
    let total_visits = FollowupModel::count_visits(db, creator_ids_ref, None).await
        .map_err(|e| Error::from(e.to_string()))?;

    // 今日拜访
    let today = chrono::Local::now().naive_local().date();
    let today_start = chrono::NaiveDateTime::new(today, chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    let today_end = chrono::NaiveDateTime::new(today, chrono::NaiveTime::from_hms_opt(23, 59, 59).unwrap());
    let today_visits = FollowupModel::count_visits(db, creator_ids_ref, Some((today_start, today_end))).await
        .map_err(|e| Error::from(e.to_string()))?;

    // 本周拜访（周一为一周开始）
    let weekday = today.weekday();
    let days_from_monday = weekday.num_days_from_monday() as i64;
    let monday = today - chrono::Duration::days(days_from_monday);
    let week_start = chrono::NaiveDateTime::new(monday, chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    let week_end = chrono::NaiveDateTime::new(today, chrono::NaiveTime::from_hms_opt(23, 59, 59).unwrap());
    let week_visits = FollowupModel::count_visits(db, creator_ids_ref, Some((week_start, week_end))).await
        .map_err(|e| Error::from(e.to_string()))?;

    // 本月拜访
    let month_start = chrono::NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
        .map(|d| chrono::NaiveDateTime::new(d, chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap()))
        .unwrap_or(today_start);
    let month_visits = FollowupModel::count_visits(db, creator_ids_ref, Some((month_start, today_end))).await
        .map_err(|e| Error::from(e.to_string()))?;

    // 拜访客户数（去重）
    let unique_customers = FollowupModel::count_unique_visit_customers(db, creator_ids_ref).await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(VisitStatisticsVO {
        total_visits,
        today_visits,
        week_visits,
        month_visits,
        unique_customers,
    })
}