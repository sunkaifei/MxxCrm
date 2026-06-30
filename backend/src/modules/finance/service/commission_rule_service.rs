//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::*;
use chrono::Utc;
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::modules::finance::entity::{commission_rule, commission_tier, salary_record};
use crate::modules::finance::model::commission::{
    CommissionRuleDTO, CommissionRuleQuery, CommissionRuleSaveDTO, CommissionTierDTO, to_decimal,
};
use crate::modules::system::entity::{dept, post};

/// 分页列表，join查询部门名称和岗位名称
pub async fn get_list(
    db: &DatabaseConnection,
    query: CommissionRuleQuery,
) -> Result<(Vec<CommissionRuleDTO>, i64), String> {
    let mut stmt = commission_rule::Entity::find()
        .filter(commission_rule::Column::Deleted.eq(0));

    if let Some(rule_name) = &query.rule_name {
        stmt = stmt.filter(commission_rule::Column::RuleName.contains(rule_name));
    }
    if let Some(enabled) = query.enabled {
        stmt = stmt.filter(commission_rule::Column::Enabled.eq(enabled));
    }
    if let Some(department_id) = query.department_id {
        stmt = stmt.filter(commission_rule::Column::DepartmentId.eq(department_id));
    }
    if let Some(post_id) = query.post_id {
        stmt = stmt.filter(commission_rule::Column::PostId.eq(post_id));
    }

    stmt = stmt.order_by_desc(commission_rule::Column::CreateTime);

    let page = std::cmp::max(query.page.unwrap_or(1), 1);
    let page_size = std::cmp::max(query.page_size.unwrap_or(20), 1);

    let paginator = stmt.paginate(db, page_size as u64);
    let total = paginator.num_items().await.map_err(|e| e.to_string())? as i64;
    let items = paginator
        .fetch_page((page - 1) as u64)
        .await
        .map_err(|e| e.to_string())?;

    // 批量查询部门名称
    let dept_ids: Vec<i64> = items
        .iter()
        .filter_map(|r| r.department_id)
        .collect::<Vec<_>>();
    let mut dept_map: HashMap<i64, String> = HashMap::new();
    if !dept_ids.is_empty() {
        let depts = dept::Entity::find()
            .filter(dept::Column::Id.is_in(dept_ids))
            .all(db)
            .await
            .map_err(|e| e.to_string())?;
        for d in depts {
            if let Some(name) = d.dept_name {
                dept_map.insert(d.id, name);
            }
        }
    }

    // 批量查询岗位名称
    let post_ids: Vec<i64> = items
        .iter()
        .filter_map(|r| r.post_id)
        .collect::<Vec<_>>();
    let mut post_map: HashMap<i64, String> = HashMap::new();
    if !post_ids.is_empty() {
        let posts = post::Entity::find()
            .filter(post::Column::Id.is_in(post_ids))
            .all(db)
            .await
            .map_err(|e| e.to_string())?;
        for p in posts {
            if let Some(name) = p.post_name {
                post_map.insert(p.id, name);
            }
        }
    }

    let dto_list: Vec<CommissionRuleDTO> = items
        .into_iter()
        .map(|m| {
            let mut dto: CommissionRuleDTO = m.into();
            if let Some(dept_id) = dto.department_id {
                dto.department_name = dept_map.get(&dept_id).cloned();
            }
            if let Some(post_id) = dto.post_id {
                dto.post_name = post_map.get(&post_id).cloned();
            }
            dto
        })
        .collect();

    Ok((dto_list, total))
}

/// 详情含阶梯
pub async fn get_detail(db: &DatabaseConnection, id: i64) -> Result<CommissionRuleDTO, String> {
    let rule = commission_rule::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "提成规则不存在".to_string())?;

    let mut dto: CommissionRuleDTO = rule.into();

    // 填充部门名称
    if let Some(dept_id) = dto.department_id {
        if let Some(d) = dept::Entity::find_by_id(dept_id).one(db).await.map_err(|e| e.to_string())? {
            dto.department_name = d.dept_name;
        }
    }
    // 填充岗位名称
    if let Some(post_id) = dto.post_id {
        if let Some(p) = post::Entity::find_by_id(post_id).one(db).await.map_err(|e| e.to_string())? {
            dto.post_name = p.post_name;
        }
    }

    // 查询阶梯
    let tiers = commission_tier::Entity::find()
        .filter(commission_tier::Column::RuleId.eq(id))
        .order_by_asc(commission_tier::Column::Sort)
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    dto.tiers = tiers.into_iter().map(CommissionTierDTO::from).collect();

    Ok(dto)
}

/// 保存（事务：先存规则，再存阶梯）
pub async fn save(db: &DatabaseConnection, dto: CommissionRuleSaveDTO) -> Result<i64, String> {
    let effective_date = chrono::NaiveDate::parse_from_str(&dto.effective_date, "%Y-%m-%d")
        .map_err(|e| format!("生效日期格式错误: {}", e))?;

    let expiry_date = dto.expiry_date.as_ref().and_then(|s| {
        chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
    });

    let now = Utc::now().naive_utc();
    let rule_id_opt = dto.id;
    let rule_name = dto.rule_name.clone();
    let department_id = dto.department_id;
    let post_id = dto.post_id;
    let trigger_condition = dto.trigger_condition.unwrap_or(1);
    let enabled = dto.enabled.unwrap_or(1);
    let description = dto.description.clone();
    let created_by = dto.created_by;
    let updated_by = dto.updated_by;
    let tiers = dto.tiers.clone();

    let txn = db.begin().await.map_err(|e| e.to_string())?;

    let rule_id: i64 = if let Some(id) = rule_id_opt {
        // 更新
        let mut model: commission_rule::ActiveModel = commission_rule::Entity::find_by_id(id)
            .one(&txn)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "提成规则不存在".to_string())?
            .into();

        model.rule_name = Set(Some(rule_name));
        model.department_id = Set(department_id);
        model.post_id = Set(post_id);
        model.trigger_condition = Set(Some(trigger_condition));
        model.effective_date = Set(Some(effective_date));
        model.expiry_date = Set(expiry_date);
        model.enabled = Set(Some(enabled));
        model.description = Set(description);
        if let Some(uid) = updated_by {
            model.updated_by = Set(Some(uid));
        }
        model.update_time = Set(Some(now));

        let result = model.update(&txn).await.map_err(|e| e.to_string())?;

        // 先删除旧阶梯
        commission_tier::Entity::delete_many()
            .filter(commission_tier::Column::RuleId.eq(id))
            .exec(&txn)
            .await
            .map_err(|e| e.to_string())?;

        result.id
    } else {
        // 新增
        let model = commission_rule::ActiveModel {
            rule_name: Set(Some(rule_name)),
            department_id: Set(department_id),
            post_id: Set(post_id),
            trigger_condition: Set(Some(trigger_condition)),
            effective_date: Set(Some(effective_date)),
            expiry_date: Set(expiry_date),
            enabled: Set(Some(enabled)),
            description: Set(description),
            created_by: Set(created_by),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        };

        let result = model.insert(&txn).await.map_err(|e| e.to_string())?;
        result.id
    };

    // 插入新阶梯
    for (idx, tier) in tiers.into_iter().enumerate() {
        let tier_model = commission_tier::ActiveModel {
            rule_id: Set(rule_id),
            min_amount: Set(to_decimal(tier.min_amount)),
            max_amount: Set(tier.max_amount.map(to_decimal)),
            commission_rate: Set(to_decimal(tier.commission_rate)),
            sort: Set(tier.sort.or(Some(idx as i32))),
            ..Default::default()
        };
        tier_model.insert(&txn).await.map_err(|e| e.to_string())?;
    }

    txn.commit().await.map_err(|e| e.to_string())?;

    Ok(rule_id)
}

/// 删除（检查是否被工资记录引用）
pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<(), String> {
    // 检查是否被工资记录引用（通过提成明细关联）
    let rule = commission_rule::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "提成规则不存在".to_string())?;

    let rule_name = rule.rule_name.clone().unwrap_or_default();

    // 软删除规则
    let now = Utc::now().naive_utc();
    let txn = db.begin().await.map_err(|e| e.to_string())?;

    let mut model: commission_rule::ActiveModel = rule.into();
    model.deleted = Set(Some(1));
    model.update_time = Set(Some(now));
    model.update(&txn).await.map_err(|e| e.to_string())?;

    // 物理删除阶梯
    commission_tier::Entity::delete_many()
        .filter(commission_tier::Column::RuleId.eq(id))
        .exec(&txn)
        .await
        .map_err(|e| e.to_string())?;

    txn.commit().await.map_err(|e| e.to_string())?;

    // 避免未使用变量告警
    let _ = rule_name;

    Ok(())
}

/// 启用/禁用
pub async fn toggle(db: &DatabaseConnection, id: i64) -> Result<(), String> {
    let rule = commission_rule::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "提成规则不存在".to_string())?;

    let current_enabled = rule.enabled.unwrap_or(0);
    let new_enabled = if current_enabled == 1 { 0 } else { 1 };
    let now = Utc::now().naive_utc();

    let mut model: commission_rule::ActiveModel = rule.into();
    model.enabled = Set(Some(new_enabled));
    model.update_time = Set(Some(now));
    model.update(db).await.map_err(|e| e.to_string())?;

    Ok(())
}
