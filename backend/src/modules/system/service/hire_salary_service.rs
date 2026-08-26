//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use chrono::Local;
use rust_decimal::Decimal;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect};
use crate::core::errors::error::{Error, Result};
use crate::modules::system::entity::admin::{self, Entity as Admin};
use crate::modules::system::entity::admin_post_merge::{Column as PostMergeColumn, Entity as PostMergeEntity};
use crate::modules::system::entity::salary_band::{Column as BandColumn, Entity as BandEntity};
use crate::modules::system::entity::hire_salary_data::{self, Entity as HireSalaryData};
use crate::modules::system::entity::employee_salary::{self, Entity as EmployeeSalary};
use crate::modules::approval::entity::approval_instance::{Column as InstanceColumn, Entity as InstanceEntity};
use crate::modules::approval::model::approval::ApprovalProcessRequest;

/// 定薪环节（mxx_hr_hire_salary_data.stage）：1部门经理 2人事 3CEO 4财务
pub const STAGE_DEPT_MANAGER: i32 = 1;
pub const STAGE_HR: i32 = 2;
pub const STAGE_CEO: i32 = 3;
pub const STAGE_FINANCE: i32 = 4;

/// 试用期法定上限（月）：《劳动合同法》第十九条
/// 无固定期限合同或合同期限≥3年 → 6个月；≥1年且<3年 → 2个月；≥3个月且<1年 → 1个月；不足3个月不得约定试用期
pub fn max_probation_month(contract_type: Option<i16>, contract_months: Option<i32>) -> i32 {
    if contract_type == Some(2) {
        return 6;
    }
    match contract_months.unwrap_or(0) {
        m if m >= 36 => 6,
        m if m >= 12 => 2,
        m if m >= 3 => 1,
        _ => 0,
    }
}

/// 部门经理环节提交建议试用期时，按被审员工的劳动合同期限做法定上限校验
async fn validate_probation_by_contract(
    db: &impl ConnectionTrait,
    instance_id: i64,
    probation_months: Option<i32>,
) -> Result<()> {
    let months = match probation_months {
        Some(m) if m > 0 => m,
        _ => return Ok(()), // 未填或0视为无试用期，交由上层必填校验处理
    };
    // 审批实例 → 业务ID（员工ID）
    let instance = InstanceEntity::find()
        .filter(InstanceColumn::Id.eq(instance_id))
        .one(db)
        .await
        .map_err(|e| Error::from(format!("查询审批实例失败: {}", e)))?
        .ok_or_else(|| Error::from(format!("审批实例不存在: {}", instance_id)))?;
    let employee_id = instance
        .business_id
        .ok_or_else(|| Error::from("审批实例缺少业务关联ID"))?;
    // 员工档案 → 劳动合同信息
    let admin_model = Admin::find_by_id(employee_id)
        .one(db)
        .await
        .map_err(|e| Error::from(format!("查询员工档案失败: {}", e)))?
        .ok_or_else(|| Error::from(format!("员工档案不存在: {}", employee_id)))?;
    let max = max_probation_month(admin_model.contract_type, admin_model.contract_months);
    if max == 0 {
        return Err(Error::from(
            "该员工合同期限不足3个月，依法不得约定试用期",
        ));
    }
    if months > max {
        return Err(Error::from(format!(
            "试用期 {} 个月超过《劳动合同法》第十九条约定的上限（{} 个月）",
            months, max
        )));
    }
    Ok(())
}

/// 人事环节带宽评估一致性兜底：确保「带宽内/超带宽」结论与实际工资金额一致。
/// 薪资带宽仅作参照与是否特批的判定依据；判定金额以谈定工资优先、部门经理建议工资兜底。
/// 仅拦截“声称带宽内但金额越界”的正向矛盾（超带宽+特批原因属合法记录，不做反向限制），
/// 否则会误导 CEO 特批环节。
async fn validate_band_consistency(
    db: &impl ConnectionTrait,
    instance_id: i64,
    req: &ApprovalProcessRequest,
) -> Result<()> {
    if req.band_status != Some(1) {
        return Ok(());
    }
    let amount = match req.negotiated_salary.or(req.suggested_salary) {
        Some(v) => v,
        None => return Ok(()),
    };
    let instance = InstanceEntity::find()
        .filter(InstanceColumn::Id.eq(instance_id))
        .one(db)
        .await
        .map_err(|e| Error::from(format!("查询审批实例失败: {}", e)))?
        .ok_or_else(|| Error::from(format!("审批实例不存在: {}", instance_id)))?;
    let employee_id = instance
        .business_id
        .ok_or_else(|| Error::from("审批实例缺少业务关联ID"))?;
    let post = PostMergeEntity::find()
        .filter(PostMergeColumn::AdminId.eq(employee_id))
        .one(db)
        .await
        .map_err(|e| Error::from(format!("查询员工岗位失败: {}", e)))?;
    let post_id = match post.and_then(|p| p.post_id) {
        Some(pid) => pid,
        None => return Ok(()), // 无岗位归属则跳过带宽比对
    };
    let band = BandEntity::find()
        .filter(BandColumn::PostId.eq(post_id))
        .filter(BandColumn::Status.eq(1))
        .filter(BandColumn::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(format!("查询薪资带宽失败: {}", e)))?;
    if let Some(b) = band {
        if amount < b.min_salary || amount > b.max_salary {
            return Err(Error::from(
                "工资金额超出岗位薪资带宽，请将带宽评估改为「超带宽」并填写原因以便特批",
            ));
        }
    }
    Ok(())
}

/// 保存某环节填写的定薪数据（入职审批通过时，按当前节点写入）
/// 幂等：同一实例 + 节点重复写入时先删除旧记录再插入（覆盖最新值）
pub async fn save_stage(
    db: &impl ConnectionTrait,
    instance_id: i64,
    node_key: &str,
    stage: i32,
    req: &ApprovalProcessRequest,
) -> Result<()> {
    // 部门经理环节：建议试用期按劳动合同期限做法定上限校验（后端兜底，防绕过前端）
    if stage == STAGE_DEPT_MANAGER {
        validate_probation_by_contract(db, instance_id, req.probation_months).await?;
    }

    // 覆盖式写入：避免同一节点多次通过（会签/依次审批）产生多条脏数据
    HireSalaryData::delete_many()
        .filter(hire_salary_data::Column::InstanceId.eq(instance_id))
        .filter(hire_salary_data::Column::NodeKey.eq(node_key.to_string()))
        .exec(db)
        .await
        .map_err(|e| Error::from(format!("清理旧定薪环节数据失败: {}", e)))?;

    let now = Local::now().naive_local();
    let active = hire_salary_data::ActiveModel {
        instance_id: Set(instance_id),
        node_key: Set(Some(node_key.to_string())),
        stage: Set(stage),
        suggested_salary: Set(req.suggested_salary),
        probation_months: Set(req.probation_months),
        ability_assessment: Set(req.ability_assessment.clone()),
        band_status: Set(req.band_status),
        band_reason: Set(req.band_reason.clone()),
        probation_ratio: Set(req.probation_ratio),
        ceo_opinion: Set(req.ceo_opinion.clone()),
        final_salary: Set(req.final_salary),
        effective_date: Set(req.effective_date),
        approver_id: Set(Some(req.approver_id)),
        comment: Set(req.comment.clone()),
        create_time: Set(Option::from(now)),
        ..Default::default()
    };
    HireSalaryData::insert(active)
        .exec(db)
        .await
        .map_err(|e| Error::from(format!("保存定薪环节数据失败: {}", e)))?;
    Ok(())
}

/// 入职定薪审批完成（实例 status=3）后，按财务环节数据生成员工薪资档案
/// 在审批事务内调用，失败随事务整体回滚
pub async fn create_employee_salary_from_approval(
    db: &impl ConnectionTrait,
    instance_id: i64,
) -> Result<()> {
    // 1. 读取该实例全部定薪环节数据（按 stage 正序）
    let stages = HireSalaryData::find()
        .filter(hire_salary_data::Column::InstanceId.eq(instance_id))
        .order_by_asc(hire_salary_data::Column::Stage)
        .all(db)
        .await
        .map_err(|e| Error::from(format!("查询定薪环节数据失败: {}", e)))?;
    if stages.is_empty() {
        return Ok(());
    }

    // 2. 财务环节录入最终定薪
    let finance = stages.iter().find(|s| s.stage == STAGE_FINANCE);
    let final_salary: Option<Decimal> = finance.and_then(|f| f.final_salary);
    if final_salary.is_none() {
        return Err(Error::from("财务环节尚未录入最终定薪，无法生成薪资档案"));
    }

    // 3. 汇总试用期信息（部门经理填月数，人事填比例；缺失回退到财务/CEO环节携带值）
    let probation_months = stages
        .iter()
        .find(|s| s.probation_months.is_some())
        .and_then(|s| s.probation_months);
    let probation_ratio = stages
        .iter()
        .find(|s| s.probation_ratio.is_some())
        .and_then(|s| s.probation_ratio);
    let effective_date = finance.and_then(|f| f.effective_date);

    // 4. 幂等：同一审批实例已生成过档案则跳过（重复流转保护）
    let exist = EmployeeSalary::find()
        .filter(employee_salary::Column::ApprovalInstanceId.eq(instance_id))
        .count(db)
        .await
        .map_err(|e| Error::from(format!("查询员工薪资档案失败: {}", e)))?;
    if exist > 0 {
        return Ok(());
    }

    // 5. 员工ID = 审批实例 business_id
    let employee_id = InstanceEntity::find_by_id(instance_id)
        .one(db)
        .await
        .map_err(|e| Error::from(format!("查询审批实例失败: {}", e)))?
        .and_then(|i| i.business_id)
        .filter(|&id| id > 0)
        .ok_or_else(|| Error::from("审批实例缺少业务ID，无法生成薪资档案"))?;

    // 6. 生成档案
    let active = employee_salary::ActiveModel {
        employee_id: Set(employee_id),
        base_salary: Set(final_salary.unwrap_or_default()),
        probation_months: Set(probation_months),
        probation_ratio: Set(probation_ratio),
        effective_date: Set(effective_date),
        source: Set(Some(1)),
        approval_instance_id: Set(Some(instance_id)),
        status: Set(Some(1)),
        create_time: Set(Option::from(Local::now().naive_local())),
        ..Default::default()
    };
    EmployeeSalary::insert(active)
        .exec(db)
        .await
        .map_err(|e| Error::from(format!("生成员工薪资档案失败: {}", e)))?;
    Ok(())
}
