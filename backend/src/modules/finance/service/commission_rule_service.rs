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
use sea_orm::sea_query::Expr;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use std::collections::HashMap;

use crate::modules::finance::entity::{commission_rule, commission_rule_member, commission_tier};
use crate::modules::finance::model::commission_rule::{
    CommissionRuleDetailVO, CommissionRuleListVO, CommissionRuleMemberVO, CommissionRuleQuery,
    CommissionRuleSaveDTO, CommissionTierVO,
};
use crate::modules::system::entity::{dept, post};

/// 分页列表，join查询部门名称和岗位名称
pub async fn get_list(
    db: &DatabaseConnection,
    query: CommissionRuleQuery,
) -> Result<(Vec<CommissionRuleListVO>, i64), String> {
    let page = std::cmp::max(query.page.unwrap_or(1), 1);
    let page_size = std::cmp::max(query.page_size.unwrap_or(20), 1);

    // 查询数据
    let (items, total) = crate::modules::finance::model::commission_rule::CommissionRuleModel::select_in_page(
        db,
        page,
        page_size,
        query.rule_name,
        query.rule_type,
        query.enabled,
        query.department_id,
        query.post_id,
    )
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

    // 转换为VO并填充名称
    let vo_list: Vec<CommissionRuleListVO> = items
        .into_iter()
        .map(|m| {
            let mut vo = CommissionRuleListVO::from(m);
            if let Some(dept_id) = vo.department_id {
                vo.department_name = dept_map.get(&dept_id).cloned();
            }
            if let Some(post_id) = vo.post_id {
                vo.post_name = post_map.get(&post_id).cloned();
            }
            vo
        })
        .collect();

    Ok((vo_list, total))
}

/// 详情含阶梯和成员
pub async fn get_detail(db: &DatabaseConnection, id: i64) -> Result<CommissionRuleDetailVO, String> {
    let rule = crate::modules::finance::model::commission_rule::CommissionRuleModel::find_by_id(db, id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "提成规则不存在".to_string())?;

    let mut vo = CommissionRuleDetailVO {
        id: rule.id,
        rule_name: rule.rule_name,
        rule_type: rule.rule_type,
        apply_scope: rule.apply_scope,
        department_id: rule.department_id,
        department_name: None,
        post_id: rule.post_id,
        post_name: None,
        commission_target_type: rule.commission_target_type,
        priority: rule.priority,
        is_default: rule.is_default,
        calc_base_type: rule.calc_base_type,
        trigger_condition: rule.trigger_condition,
        effective_date: rule.effective_date.map(|d| d.format("%Y-%m-%d").to_string()),
        expiry_date: rule.expiry_date.map(|d| d.format("%Y-%m-%d").to_string()),
        enabled: rule.enabled,
        description: rule.description,
        created_by: rule.created_by,
        create_time: rule.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        updated_by: rule.updated_by,
        update_time: rule.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        tiers: Vec::new(),
        members: Vec::new(),
    };

    // 填充部门名称
    if let Some(dept_id) = vo.department_id {
        if let Some(d) = dept::Entity::find_by_id(dept_id).one(db).await.map_err(|e| e.to_string())? {
            vo.department_name = d.dept_name;
        }
    }
    // 填充岗位名称
    if let Some(post_id) = vo.post_id {
        if let Some(p) = post::Entity::find_by_id(post_id).one(db).await.map_err(|e| e.to_string())? {
            vo.post_name = p.post_name;
        }
    }

    // 查询阶梯
    let tiers = commission_tier::Entity::find()
        .filter(commission_tier::Column::RuleId.eq(id))
        .order_by_asc(commission_tier::Column::Sort)
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    vo.tiers = tiers
        .into_iter()
        .map(|t| CommissionTierVO {
            id: Some(t.id),
            rule_id: Some(t.rule_id),
            min_amount: t.min_amount.to_f64().unwrap_or_default(),
            max_amount: t.max_amount.map(|d| d.to_f64().unwrap_or_default()),
            commission_rate: t.commission_rate.to_f64().unwrap_or_default(),
            sort: t.sort,
        })
        .collect();

    // 查询成员分配
    let members = commission_rule_member::Entity::find()
        .filter(commission_rule_member::Column::RuleId.eq(id))
        .order_by_asc(commission_rule_member::Column::Sort)
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    vo.members = members
        .into_iter()
        .map(|m| CommissionRuleMemberVO {
            id: m.id,
            rule_id: m.rule_id,
            member_type: m.member_type,
            role_name: m.role_name,
            member_name: m.member_name,
            distribution_type: m.distribution_type,
            fixed_rate: m.fixed_rate.to_f64().unwrap_or_default(),
            default_ratio: Some(m.default_ratio.to_f64().unwrap_or_default()),
            required: Some(m.required),
            sort: m.sort,
        })
        .collect();

    Ok(vo)
}

/// 保存（事务：先存规则，再存阶梯和成员）
pub async fn save(
    db: &DatabaseConnection,
    dto: CommissionRuleSaveDTO,
    user_id: i64,
) -> Result<i64, String> {
    let txn = db.begin().await.map_err(|e| e.to_string())?;

    let is_default = dto.is_default.unwrap_or(0);

    if is_default == 1 {
        commission_rule::Entity::update_many()
            .col_expr(commission_rule::Column::IsDefault, Expr::value(0))
            .filter(commission_rule::Column::Deleted.eq(0))
            .filter(commission_rule::Column::IsDefault.eq(1))
            .exec(&txn)
            .await
            .map_err(|e| e.to_string())?;
    }

    // 保存规则
    let rule_id = if let Some(id) = dto.id {
        // 更新
        crate::modules::finance::model::commission_rule::CommissionRuleModel::update_by_id(
            &txn, id, &dto, Some(user_id),
        )
        .await
        .map_err(|e| e.to_string())?;

        // 删除旧阶梯
        commission_tier::Entity::delete_many()
            .filter(commission_tier::Column::RuleId.eq(id))
            .exec(&txn)
            .await
            .map_err(|e| e.to_string())?;

        // 删除旧成员
        commission_rule_member::Entity::delete_many()
            .filter(commission_rule_member::Column::RuleId.eq(id))
            .exec(&txn)
            .await
            .map_err(|e| e.to_string())?;

        id
    } else {
        // 新增
        crate::modules::finance::model::commission_rule::CommissionRuleModel::insert(
            &txn, &dto, Some(user_id),
        )
        .await
        .map_err(|e| e.to_string())?
    };

    // 插入新阶梯
    use rust_decimal::Decimal;
    
    for (idx, tier) in dto.tiers.iter().enumerate() {
        let tier_model = commission_tier::ActiveModel {
            rule_id: Set(rule_id),
            min_amount: Set(Decimal::from_f64(tier.min_amount).unwrap_or_default()),
            max_amount: Set(tier.max_amount.and_then(|v| Decimal::from_f64(v))),
            commission_rate: Set(Decimal::from_f64(tier.commission_rate).unwrap_or_default()),
            sort: Set(tier.sort.or(Some(idx as i32))),
            ..Default::default()
        };
        tier_model.insert(&txn).await.map_err(|e| e.to_string())?;
    }

    // 插入新成员
    for member in dto.members.iter() {
        let member_model = commission_rule_member::ActiveModel {
            rule_id: Set(rule_id),
            member_type: Set(member.member_type),
            role_name: Set(member.role_name.clone()),
            member_name: Set(member.member_name.clone()),
            distribution_type: Set(member.distribution_type),
            fixed_rate: Set(Decimal::from_f64(member.fixed_rate).unwrap_or_default()),
            default_ratio: Set(Decimal::from_f64(member.default_ratio.unwrap_or(0.0)).unwrap_or_default()),
            required: Set(member.required.unwrap_or(0)),
            sort: Set(member.sort),
            create_time: Set(Some(chrono::Utc::now().naive_utc())),
            update_time: Set(Some(chrono::Utc::now().naive_utc())),
            ..Default::default()
        };
        member_model.insert(&txn).await.map_err(|e| e.to_string())?;
    }

    txn.commit().await.map_err(|e| e.to_string())?;

    Ok(rule_id)
}

/// 删除（软删除）
pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<(), String> {
    let txn = db.begin().await.map_err(|e| e.to_string())?;

    // 软删除规则
    crate::modules::finance::model::commission_rule::CommissionRuleModel::delete_by_id(&txn, id)
        .await
        .map_err(|e| e.to_string())?;

    // 物理删除阶梯
    commission_tier::Entity::delete_many()
        .filter(commission_tier::Column::RuleId.eq(id))
        .exec(&txn)
        .await
        .map_err(|e| e.to_string())?;

    // 物理删除成员
    commission_rule_member::Entity::delete_many()
        .filter(commission_rule_member::Column::RuleId.eq(id))
        .exec(&txn)
        .await
        .map_err(|e| e.to_string())?;

    txn.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

/// 启用/禁用切换
pub async fn toggle(db: &DatabaseConnection, id: i64) -> Result<(), String> {
    crate::modules::finance::model::commission_rule::CommissionRuleModel::toggle_enabled(db, id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 获取默认方案
pub async fn get_default_plan(db: &DatabaseConnection) -> Result<Option<CommissionRuleDetailVO>, String> {
    let rule = crate::modules::finance::model::commission_rule::CommissionRuleModel::find_default(db)
        .await
        .map_err(|e| e.to_string())?;

    match rule {
        Some(r) => {
            let vo = get_detail(db, r.id).await?;
            Ok(Some(vo))
        }
        None => Ok(None),
    }
}

/// 设置默认方案
pub async fn set_default(db: &DatabaseConnection, id: i64) -> Result<(), String> {
    let txn = db.begin().await.map_err(|e| e.to_string())?;

    let rule = crate::modules::finance::model::commission_rule::CommissionRuleModel::find_by_id(&txn, id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "提成规则不存在".to_string())?;

    if rule.enabled.unwrap_or(0) != 1 {
        return Err("只有启用状态的规则才能设为默认".to_string());
    }

    commission_rule::Entity::update_many()
        .col_expr(commission_rule::Column::IsDefault, Expr::value(0))
        .filter(commission_rule::Column::Deleted.eq(0))
        .filter(commission_rule::Column::IsDefault.eq(1))
        .exec(&txn)
        .await
        .map_err(|e| e.to_string())?;

    let mut model: commission_rule::ActiveModel = rule.into();
    model.is_default = Set(Some(1));
    model.update_time = Set(Some(chrono::Utc::now().naive_utc()));
    model.update(&txn).await.map_err(|e| e.to_string())?;

    txn.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

/// 获取启用的规则列表（下拉选择用）
pub async fn list_options(db: &DatabaseConnection) -> Result<Vec<CommissionRuleListVO>, String> {
    let items = crate::modules::finance::model::commission_rule::CommissionRuleModel::list_enabled_options(db)
        .await
        .map_err(|e| e.to_string())?;

    let vo_list: Vec<CommissionRuleListVO> = items
        .into_iter()
        .map(CommissionRuleListVO::from)
        .collect();

    Ok(vo_list)
}
