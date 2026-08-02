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
use serde::{Deserialize, Serialize};
use rust_decimal::prelude::ToPrimitive;

use crate::modules::finance::entity::commission_rule;

/// 提成规则列表VO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommissionRuleListVO {
    pub id: i64,
    pub rule_name: Option<String>,
    pub rule_type: Option<i32>,
    pub apply_scope: Option<i32>,
    pub department_id: Option<i64>,
    pub department_name: Option<String>,
    pub post_id: Option<i64>,
    pub post_name: Option<String>,
    pub commission_target_type: Option<i32>,
    pub priority: Option<i32>,
    pub is_default: Option<i32>,
    pub calc_base_type: Option<i32>,
    pub trigger_condition: Option<i32>,
    /// P2-3: 产品线维度
    pub product_line: Option<String>,
    /// P2-3: 区域编码维度
    pub region_code: Option<String>,
    /// P2-3: 客户类型维度
    pub customer_type: Option<String>,
    pub effective_date: Option<String>,
    pub expiry_date: Option<String>,
    pub enabled: Option<i32>,
    pub description: Option<String>,
    pub created_by: Option<i64>,
    pub create_time: Option<String>,
    pub updated_by: Option<i64>,
    pub update_time: Option<String>,
    /// 提成性质: 1=个人提成 2=管理分润 3=团队激励奖金 4=团建资金池 5=总提成再分配 6=利润提成
    pub commission_category: i16,
    /// 受益岗位: 1=销售本人 2=直属主管 3=部门经理 4=总监 5=总经理 6=自定义岗位
    pub beneficiary_role: i16,
    /// 计算方式: 1=按比例 2=固定金额(达标后) 3=阶梯累进 4=超额递增
    pub calc_method: i16,
    /// 达标门槛(calc_method=2时使用)
    pub bonus_target: Option<f64>,
    /// 固定奖金金额(calc_method=2时使用)
    pub bonus_fixed_amount: Option<f64>,
    /// 单笔提成封顶(NULL=不封顶)
    pub commission_cap: Option<f64>,
    /// 月度提成保底(NULL=不保底)
    pub commission_floor: Option<f64>,
    /// 客户分类筛选: new=仅新客户 old=仅老客户 NULL=全部
    pub customer_category: Option<String>,
    /// 递延发放月数: 0=随当月发 N=分N个月递延
    pub defer_months: i32,
    /// 关联资金池ID(category=4时使用)
    pub pool_id: Option<i64>,
    /// 计算基数字段: payment_amount/contract_amount/net_amount/profit
    pub calc_base_field: Option<String>,
    /// 阶梯模式: 0=单档命中 1=累进 2=超额递增
    pub tier_mode: Option<i32>,
}

impl From<commission_rule::Model> for CommissionRuleListVO {
    fn from(model: commission_rule::Model) -> Self {
        Self {
            id: model.id,
            rule_name: model.rule_name,
            rule_type: model.rule_type,
            apply_scope: model.apply_scope,
            department_id: model.department_id,
            department_name: None,
            post_id: model.post_id,
            post_name: None,
            commission_target_type: model.commission_target_type,
            priority: model.priority,
            is_default: model.is_default,
            calc_base_type: model.calc_base_type,
            trigger_condition: model.trigger_condition,
            product_line: model.product_line,
            region_code: model.region_code,
            customer_type: model.customer_type,
            effective_date: model.effective_date.map(|d| d.format("%Y-%m-%d").to_string()),
            expiry_date: model.expiry_date.map(|d| d.format("%Y-%m-%d").to_string()),
            enabled: model.enabled,
            description: model.description,
            created_by: model.created_by,
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            updated_by: model.updated_by,
            update_time: model.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            commission_category: model.commission_category,
            beneficiary_role: model.beneficiary_role,
            calc_method: model.calc_method,
            bonus_target: model.bonus_target.and_then(|d| d.to_f64()),
            bonus_fixed_amount: model.bonus_fixed_amount.and_then(|d| d.to_f64()),
            commission_cap: model.commission_cap.and_then(|d| d.to_f64()),
            commission_floor: model.commission_floor.and_then(|d| d.to_f64()),
            customer_category: model.customer_category,
            defer_months: model.defer_months,
            pool_id: model.pool_id,
            calc_base_field: model.calc_base_field,
            tier_mode: model.tier_mode,
        }
    }
}

/// 提成规则详情VO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommissionRuleDetailVO {
    pub id: i64,
    pub rule_name: Option<String>,
    pub rule_type: Option<i32>,
    pub apply_scope: Option<i32>,
    pub department_id: Option<i64>,
    pub department_name: Option<String>,
    pub post_id: Option<i64>,
    pub post_name: Option<String>,
    pub commission_target_type: Option<i32>,
    pub priority: Option<i32>,
    pub is_default: Option<i32>,
    pub calc_base_type: Option<i32>,
    pub trigger_condition: Option<i32>,
    /// P2-3: 产品线维度
    pub product_line: Option<String>,
    /// P2-3: 区域编码维度
    pub region_code: Option<String>,
    /// P2-3: 客户类型维度
    pub customer_type: Option<String>,
    pub effective_date: Option<String>,
    pub expiry_date: Option<String>,
    pub enabled: Option<i32>,
    pub description: Option<String>,
    pub created_by: Option<i64>,
    pub create_time: Option<String>,
    pub updated_by: Option<i64>,
    pub update_time: Option<String>,
    pub tiers: Vec<CommissionTierVO>,
    pub members: Vec<CommissionRuleMemberVO>,
    /// 提成性质: 1=个人提成 2=管理分润 3=团队激励奖金 4=团建资金池 5=总提成再分配 6=利润提成
    pub commission_category: i16,
    /// 受益岗位: 1=销售本人 2=直属主管 3=部门经理 4=总监 5=总经理 6=自定义岗位
    pub beneficiary_role: i16,
    /// 计算方式: 1=按比例 2=固定金额(达标后) 3=阶梯累进 4=超额递增
    pub calc_method: i16,
    /// 达标门槛(calc_method=2时使用)
    pub bonus_target: Option<f64>,
    /// 固定奖金金额(calc_method=2时使用)
    pub bonus_fixed_amount: Option<f64>,
    /// 单笔提成封顶(NULL=不封顶)
    pub commission_cap: Option<f64>,
    /// 月度提成保底(NULL=不保底)
    pub commission_floor: Option<f64>,
    /// 客户分类筛选: new=仅新客户 old=仅老客户 NULL=全部
    pub customer_category: Option<String>,
    /// 递延发放月数: 0=随当月发 N=分N个月递延
    pub defer_months: i32,
    /// 关联资金池ID(category=4时使用)
    pub pool_id: Option<i64>,
    /// 计算基数字段: payment_amount/contract_amount/net_amount/profit
    pub calc_base_field: Option<String>,
    /// 阶梯模式: 0=单档命中 1=累进 2=超额递增
    pub tier_mode: Option<i32>,
}

/// 提成阶梯VO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommissionTierVO {
    pub id: Option<i64>,
    pub rule_id: Option<i64>,
    pub min_amount: f64,
    pub max_amount: Option<f64>,
    pub commission_rate: f64,
    pub sort: Option<i32>,
}

/// 提成规则成员VO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommissionRuleMemberVO {
    pub id: i64,
    pub rule_id: i64,
    pub member_type: i32,
    pub role_name: Option<String>,
    pub member_name: String,
    pub distribution_type: i32,
    pub fixed_rate: f64,
    pub default_ratio: Option<f64>,
    pub required: Option<i32>,
    pub sort: i32,
}

/// 提成规则保存DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommissionRuleSaveDTO {
    pub id: Option<i64>,
    pub rule_name: String,
    pub rule_type: Option<i32>,
    pub apply_scope: Option<i32>,
    pub department_id: Option<i64>,
    pub post_id: Option<i64>,
    pub calc_base_type: Option<i32>,
    pub trigger_condition: Option<i32>,
    pub commission_target_type: Option<i32>,
    pub priority: Option<i32>,
    /// P2-3: 产品线维度
    pub product_line: Option<String>,
    /// P2-3: 区域编码维度
    pub region_code: Option<String>,
    /// P2-3: 客户类型维度
    pub customer_type: Option<String>,
    pub effective_date: String,
    pub expiry_date: Option<String>,
    pub is_default: Option<i32>,
    pub enabled: Option<i32>,
    pub description: Option<String>,
    pub tiers: Vec<CommissionTierSaveDTO>,
    pub members: Vec<CommissionRuleMemberSaveDTO>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    /// 提成性质: 1=个人提成 2=管理分润 3=团队激励奖金 4=团建资金池 5=总提成再分配 6=利润提成
    pub commission_category: Option<i16>,
    /// 受益岗位: 1=销售本人 2=直属主管 3=部门经理 4=总监 5=总经理 6=自定义岗位
    pub beneficiary_role: Option<i16>,
    /// 计算方式: 1=按比例 2=固定金额(达标后) 3=阶梯累进 4=超额递增
    pub calc_method: Option<i16>,
    /// 达标门槛(calc_method=2时使用)
    pub bonus_target: Option<f64>,
    /// 固定奖金金额(calc_method=2时使用)
    pub bonus_fixed_amount: Option<f64>,
    /// 单笔提成封顶(NULL=不封顶)
    pub commission_cap: Option<f64>,
    /// 月度提成保底(NULL=不保底)
    pub commission_floor: Option<f64>,
    /// 客户分类筛选: new=仅新客户 old=仅老客户 NULL=全部
    pub customer_category: Option<String>,
    /// 递延发放月数: 0=随当月发 N=分N个月递延
    pub defer_months: Option<i32>,
    /// 关联资金池ID(category=4时使用)
    pub pool_id: Option<i64>,
    /// 计算基数字段: payment_amount/contract_amount/net_amount/profit
    pub calc_base_field: Option<String>,
    /// 阶梯模式: 0=单档命中 1=累进 2=超额递增
    pub tier_mode: Option<i32>,
}

/// 阶梯保存DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommissionTierSaveDTO {
    pub id: Option<i64>,
    pub min_amount: f64,
    pub max_amount: Option<f64>,
    pub commission_rate: f64,
    pub sort: Option<i32>,
}

/// 成员保存DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommissionRuleMemberSaveDTO {
    pub id: Option<i64>,
    pub member_type: i32,
    pub role_name: Option<String>,
    pub member_name: String,
    pub distribution_type: i32,
    pub fixed_rate: f64,
    pub default_ratio: Option<f64>,
    pub required: Option<i32>,
    pub sort: i32,
}

/// 提成规则查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommissionRuleQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub rule_name: Option<String>,
    pub rule_type: Option<i32>,
    pub enabled: Option<i32>,
    pub department_id: Option<i64>,
    pub post_id: Option<i64>,
    /// P2-3: 产品线筛选
    pub product_line: Option<String>,
    /// P2-3: 区域编码筛选
    pub region_code: Option<String>,
    /// P2-3: 客户类型筛选
    pub customer_type: Option<String>,
}

/// 提成规则数据模型操作类
pub struct CommissionRuleModel;

impl CommissionRuleModel {
    /// 根据ID查询
    pub async fn find_by_id<C: ConnectionTrait>(
        db: &C,
        id: i64,
    ) -> Result<Option<commission_rule::Model>, DbErr> {
        commission_rule::Entity::find_by_id(id)
            .filter(commission_rule::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 分页查询列表
    pub async fn select_in_page<C: ConnectionTrait>(
        db: &C,
        page: i64,
        page_size: i64,
        rule_name: Option<String>,
        rule_type: Option<i32>,
        enabled: Option<i32>,
        department_id: Option<i64>,
        post_id: Option<i64>,
        product_line: Option<String>,
        region_code: Option<String>,
        customer_type: Option<String>,
    ) -> Result<(Vec<commission_rule::Model>, i64), DbErr> {
        let mut stmt = commission_rule::Entity::find()
            .filter(commission_rule::Column::Deleted.eq(0));

        if let Some(name) = rule_name {
            stmt = stmt.filter(commission_rule::Column::RuleName.contains(name));
        }
        if let Some(rt) = rule_type {
            stmt = stmt.filter(commission_rule::Column::RuleType.eq(rt));
        }
        if let Some(e) = enabled {
            stmt = stmt.filter(commission_rule::Column::Enabled.eq(e));
        }
        if let Some(dept_id) = department_id {
            stmt = stmt.filter(commission_rule::Column::DepartmentId.eq(dept_id));
        }
        if let Some(post_id) = post_id {
            stmt = stmt.filter(commission_rule::Column::PostId.eq(post_id));
        }
        // P2-3: 新增三个维度筛选
        if let Some(pl) = product_line {
            stmt = stmt.filter(commission_rule::Column::ProductLine.eq(pl));
        }
        if let Some(rc) = region_code {
            stmt = stmt.filter(commission_rule::Column::RegionCode.eq(rc));
        }
        if let Some(ct) = customer_type {
            stmt = stmt.filter(commission_rule::Column::CustomerType.eq(ct));
        }

        stmt = stmt.order_by_desc(commission_rule::Column::Id);

        let paginator = stmt.paginate(db, page_size as u64);
        let total = paginator.num_items().await? as i64;
        let items = paginator.fetch_page((page - 1) as u64).await?;

        Ok((items, total))
    }

    /// 新增规则
    pub async fn insert<C: ConnectionTrait>(
        db: &C,
        dto: &CommissionRuleSaveDTO,
        created_by: Option<i64>,
    ) -> Result<i64, DbErr> {
        let now = chrono::Utc::now().naive_utc();
        let effective_date = chrono::NaiveDate::parse_from_str(&dto.effective_date, "%Y-%m-%d")
            .map_err(|_| DbErr::Custom("生效日期格式错误".to_string()))?;
        let expiry_date = dto.expiry_date.as_ref().and_then(|s| {
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
        });

        use rust_decimal::prelude::FromPrimitive;
        use rust_decimal::Decimal;
        let model = commission_rule::ActiveModel {
            rule_name: Set(Some(dto.rule_name.clone())),
            rule_type: Set(dto.rule_type),
            apply_scope: Set(dto.apply_scope),
            department_id: Set(dto.department_id),
            post_id: Set(dto.post_id),
            calc_base_type: Set(dto.calc_base_type),
            trigger_condition: Set(Some(dto.trigger_condition.unwrap_or(1))),
            commission_target_type: Set(dto.commission_target_type),
            priority: Set(dto.priority),
            product_line: Set(dto.product_line.clone()),
            region_code: Set(dto.region_code.clone()),
            customer_type: Set(dto.customer_type.clone()),
            effective_date: Set(Some(effective_date)),
            expiry_date: Set(expiry_date),
            is_default: Set(dto.is_default),
            enabled: Set(Some(dto.enabled.unwrap_or(1))),
            description: Set(dto.description.clone()),
            created_by: Set(created_by),
            create_time: Set(Some(now)),
            updated_by: Set(None),
            update_time: Set(Some(now)),
            deleted: Set(Some(0)),
            // v2 新增字段
            commission_category: Set(dto.commission_category.unwrap_or(1)),
            beneficiary_role: Set(dto.beneficiary_role.unwrap_or(1)),
            calc_method: Set(dto.calc_method.unwrap_or(1)),
            bonus_target: Set(dto.bonus_target.and_then(|v| Decimal::from_f64(v))),
            bonus_fixed_amount: Set(dto.bonus_fixed_amount.and_then(|v| Decimal::from_f64(v))),
            commission_cap: Set(dto.commission_cap.and_then(|v| Decimal::from_f64(v))),
            commission_floor: Set(dto.commission_floor.and_then(|v| Decimal::from_f64(v))),
            customer_category: Set(dto.customer_category.clone()),
            defer_months: Set(dto.defer_months.unwrap_or(0)),
            pool_id: Set(dto.pool_id),
            calc_base_field: Set(dto.calc_base_field.clone()),
            tier_mode: Set(dto.tier_mode),
            ..Default::default()
        };

        let result = commission_rule::Entity::insert(model).exec(db).await?;
        Ok(result.last_insert_id)
    }

    /// 更新规则
    pub async fn update_by_id<C: ConnectionTrait>(
        db: &C,
        id: i64,
        dto: &CommissionRuleSaveDTO,
        updated_by: Option<i64>,
    ) -> Result<i64, DbErr> {
        let now = chrono::Utc::now().naive_utc();
        let effective_date = chrono::NaiveDate::parse_from_str(&dto.effective_date, "%Y-%m-%d")
            .map_err(|_| DbErr::Custom("生效日期格式错误".to_string()))?;
        let expiry_date = dto.expiry_date.as_ref().and_then(|s| {
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
        });

        let model: commission_rule::ActiveModel = commission_rule::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("提成规则不存在".to_string()))?
            .into();

        use rust_decimal::prelude::FromPrimitive;
        use rust_decimal::Decimal;
        let mut model = model;
        model.rule_name = Set(Some(dto.rule_name.clone()));
        model.rule_type = Set(dto.rule_type);
        model.apply_scope = Set(dto.apply_scope);
        model.department_id = Set(dto.department_id);
        model.post_id = Set(dto.post_id);
        model.calc_base_type = Set(dto.calc_base_type);
        model.trigger_condition = Set(dto.trigger_condition);
        model.commission_target_type = Set(dto.commission_target_type);
        model.priority = Set(dto.priority);
        model.product_line = Set(dto.product_line.clone());
        model.region_code = Set(dto.region_code.clone());
        model.customer_type = Set(dto.customer_type.clone());
        model.effective_date = Set(Some(effective_date));
        model.expiry_date = Set(expiry_date);
        model.is_default = Set(dto.is_default);
        model.enabled = Set(dto.enabled);
        model.description = Set(dto.description.clone());
        model.updated_by = Set(updated_by);
        model.update_time = Set(Some(now));
        // v2 新增字段
        model.commission_category = Set(dto.commission_category.unwrap_or(1));
        model.beneficiary_role = Set(dto.beneficiary_role.unwrap_or(1));
        model.calc_method = Set(dto.calc_method.unwrap_or(1));
        model.bonus_target = Set(dto.bonus_target.and_then(|v| Decimal::from_f64(v)));
        model.bonus_fixed_amount = Set(dto.bonus_fixed_amount.and_then(|v| Decimal::from_f64(v)));
        model.commission_cap = Set(dto.commission_cap.and_then(|v| Decimal::from_f64(v)));
        model.commission_floor = Set(dto.commission_floor.and_then(|v| Decimal::from_f64(v)));
        model.customer_category = Set(dto.customer_category.clone());
        model.defer_months = Set(dto.defer_months.unwrap_or(0));
        model.pool_id = Set(dto.pool_id);
        model.calc_base_field = Set(dto.calc_base_field.clone());
        model.tier_mode = Set(dto.tier_mode);

        let result = model.update(db).await?;
        Ok(result.id)
    }

    /// 软删除
    pub async fn delete_by_id<C: ConnectionTrait>(
        db: &C,
        id: i64,
    ) -> Result<i64, DbErr> {
        let now = chrono::Utc::now().naive_utc();
        
        let model: commission_rule::ActiveModel = commission_rule::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("提成规则不存在".to_string()))?
            .into();

        let mut model = model;
        model.deleted = Set(Some(1));
        model.update_time = Set(Some(now));
        
        model.update(db).await?;
        Ok(1)
    }

    /// 切换启用状态
    pub async fn toggle_enabled<C: ConnectionTrait>(
        db: &C,
        id: i64,
    ) -> Result<i64, DbErr> {
        let now = chrono::Utc::now().naive_utc();
        
        let rule = commission_rule::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("提成规则不存在".to_string()))?;
        
        let current_enabled = rule.enabled.unwrap_or(0);
        let new_enabled = if current_enabled == 1 { 0 } else { 1 };
        
        let mut model: commission_rule::ActiveModel = rule.into();
        model.enabled = Set(Some(new_enabled));
        model.update_time = Set(Some(now));
        
        model.update(db).await?;
        Ok(1)
    }

    /// 查询默认方案
    pub async fn find_default<C: ConnectionTrait>(
        db: &C,
    ) -> Result<Option<commission_rule::Model>, DbErr> {
        commission_rule::Entity::find()
            .filter(commission_rule::Column::IsDefault.eq(1))
            .filter(commission_rule::Column::Enabled.eq(1))
            .filter(commission_rule::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 查询所有启用的规则列表（用于下拉选择）
    pub async fn list_enabled_options<C: ConnectionTrait>(
        db: &C,
    ) -> Result<Vec<commission_rule::Model>, DbErr> {
        commission_rule::Entity::find()
            .filter(commission_rule::Column::Enabled.eq(1))
            .filter(commission_rule::Column::Deleted.eq(0))
            .order_by_desc(commission_rule::Column::Id)
            .all(db)
            .await
    }
}
