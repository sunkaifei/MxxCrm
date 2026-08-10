use chrono::{Local, NaiveDate};
use rust_decimal::Decimal;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};

use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::approval::model::approval::ApprovalModel;
use crate::modules::approval::entity::approval_cc::{Column as CcColumn, Entity as CcEntity};
use crate::modules::crm::entity::contract::{Column as ContractColumn, Entity as ContractEntity};
use crate::modules::crm::entity::contract_payment_plan::{
    Column as PlanColumn, Entity as PlanEntity,
};
use crate::modules::crm::entity::customer::{Column as CustomerColumn, Entity as CustomerEntity};
use crate::modules::crm::entity::lead::{Column as LeadColumn, Entity as LeadEntity};
use crate::modules::crm::entity::opportunity::{
    Column as OppColumn, Entity as OppEntity,
};
use crate::modules::crm::model::todo::*;
use crate::modules::statistics::entity::performance_plan::{
    Column as PerfPlanColumn, Entity as PerfPlanEntity,
};

pub struct TodoService;

impl TodoService {
    /// 待办汇总统计
    pub async fn summary(db: &DatabaseConnection, user_id: i64) -> Result<TodoSummaryVO> {
        let today = Local::now().naive_local().date();
        let now = Local::now().naive_local();

        // 逾期跟进（客户+线索）
        let overdue_customer = CustomerEntity::find()
            .filter(CustomerColumn::NextFollowAt.is_not_null())
            .filter(CustomerColumn::NextFollowAt.lt(now))
            .filter(CustomerColumn::AssignedTo.eq(user_id))
            .filter(CustomerColumn::Deleted.eq(0))
            .count(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        let overdue_lead = LeadEntity::find()
            .filter(LeadColumn::NextFollowAt.is_not_null())
            .filter(LeadColumn::NextFollowAt.lt(now))
            .filter(LeadColumn::AssignedTo.eq(user_id))
            .filter(LeadColumn::Deleted.eq(0))
            .count(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        // 今日待跟进
        let today_start = today.and_hms_opt(0, 0, 0).unwrap();
        let today_end = today.and_hms_opt(23, 59, 59).unwrap();

        let today_customer = CustomerEntity::find()
            .filter(CustomerColumn::NextFollowAt.is_not_null())
            .filter(CustomerColumn::NextFollowAt.between(today_start, today_end))
            .filter(CustomerColumn::AssignedTo.eq(user_id))
            .filter(CustomerColumn::Deleted.eq(0))
            .count(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        let today_lead = LeadEntity::find()
            .filter(LeadColumn::NextFollowAt.is_not_null())
            .filter(LeadColumn::NextFollowAt.between(today_start, today_end))
            .filter(LeadColumn::AssignedTo.eq(user_id))
            .filter(LeadColumn::Deleted.eq(0))
            .count(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        // 待我审批
        let pending_approval = Self::count_pending_approval(db, user_id).await?;

        // 待回款提醒（7天内到期且未完成）
        let payment_deadline = today + chrono::Duration::days(7);
        let pending_payment = PlanEntity::find()
            .filter(PlanColumn::PlanDate.is_not_null())
            .filter(PlanColumn::PlanDate.lte(payment_deadline))
            .filter(PlanColumn::Status.is_in(vec![0, 1]))
            .count(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        // 即将到期合同（30天内）
        let contract_deadline = today + chrono::Duration::days(30);
        let expiring_contract = ContractEntity::find()
            .filter(ContractColumn::EndDate.is_not_null())
            .filter(ContractColumn::EndDate.between(today, contract_deadline))
            .filter(ContractColumn::Status.is_in(vec![2, 3]))
            .filter(ContractColumn::Deleted.eq(0))
            .count(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        // 停滞商机（超过30天未更新且未成交）
        let stagnant_threshold = now - chrono::Duration::days(30);
        let stagnant_opportunity = OppEntity::find()
            .filter(OppColumn::UpdateTime.lt(stagnant_threshold))
            .filter(OppColumn::Stage.ne(5))
            .filter(OppColumn::Deleted.eq(0))
            .count(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        // 待我审批的销售计划（当前用户为 current_approver_id 且状态为待审批）
        let pending_plan_approval = PerfPlanEntity::find()
            .filter(PerfPlanColumn::CurrentApproverId.eq(user_id))
            .filter(PerfPlanColumn::Status.eq(1))
            .filter(PerfPlanColumn::Deleted.eq(0))
            .count(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        // 未读抄送数
        let unread_cc = CcEntity::find()
            .filter(CcColumn::UserId.eq(user_id))
            .filter(CcColumn::IsRead.eq(0))
            .filter(CcColumn::Deleted.eq(0))
            .count(db)
            .await
            .map_err(|e| Error::from(e.to_string()))? as i64;

        Ok(TodoSummaryVO {
            overdue_follow_up: (overdue_customer + overdue_lead) as i64,
            today_follow_up: (today_customer + today_lead) as i64,
            pending_approval,
            pending_payment: pending_payment as i64,
            expiring_contract: expiring_contract as i64,
            stagnant_opportunity: stagnant_opportunity as i64,
            pending_plan_approval: pending_plan_approval as i64,
            unread_cc,
        })
    }

    /// 审批待办列表
    pub async fn approval_list(
        db: &DatabaseConnection,
        user_id: i64,
        query: &ApprovalTodoQuery,
    ) -> Result<ApprovalTodoResult> {
        ApprovalModel::find_instance_list_filtered(
            db,
            user_id,
            query.business_type.as_deref(),
            query.status,
            query.business_title.as_deref(),
            query.page_num,
            query.page_size,
        )
        .await
    }

    /// 跟进待办列表
    pub async fn follow_up_list(
        db: &DatabaseConnection,
        user_id: i64,
        query: &FollowUpTodoQuery,
    ) -> Result<ResultPage<Vec<FollowUpTodoVO>>> {
        let today = Local::now().naive_local().date();
        let now = Local::now().naive_local();
        let item_type = query.item_type.as_deref().unwrap_or("all");
        let range_type = query.range_type.as_deref().unwrap_or("all");

        let mut items: Vec<FollowUpTodoVO> = Vec::new();

        // 查询客户
        if item_type == "all" || item_type == "customer" {
            let mut q = CustomerEntity::find()
                .filter(CustomerColumn::NextFollowAt.is_not_null())
                .filter(CustomerColumn::AssignedTo.eq(user_id))
                .filter(CustomerColumn::Deleted.eq(0));

            q = Self::apply_follow_up_range_customer(q, range_type, now, today);

            let customers = q
                .order_by_desc(CustomerColumn::NextFollowAt)
                .all(db)
                .await
                .map_err(|e| Error::from(e.to_string()))?;

            for c in customers {
                let nfa = c.next_follow_at;
                let overdue_days = Self::calc_overdue_days(nfa, today);
                let name = c
                    .company_name
                    .clone()
                    .or(c.short_name.clone())
                    .or(c.person_name.clone())
                    .unwrap_or_default();
                items.push(FollowUpTodoVO {
                    id: c.id,
                    item_type: "customer".to_string(),
                    name,
                    owner_user_id: c.assigned_to,
                    owner_user_name: None,
                    next_follow_at: nfa.map(|t| t.to_string()),
                    overdue_days,
                });
            }
        }

        // 查询线索
        if item_type == "all" || item_type == "lead" {
            let mut q = LeadEntity::find()
                .filter(LeadColumn::NextFollowAt.is_not_null())
                .filter(LeadColumn::AssignedTo.eq(user_id))
                .filter(LeadColumn::Deleted.eq(0));

            q = Self::apply_follow_up_range_lead(q, range_type, now, today);

            let leads = q
                .order_by_desc(LeadColumn::NextFollowAt)
                .all(db)
                .await
                .map_err(|e| Error::from(e.to_string()))?;

            for l in leads {
                let nfa = l.next_follow_at;
                let overdue_days = Self::calc_overdue_days(nfa, today);
                items.push(FollowUpTodoVO {
                    id: l.id,
                    item_type: "lead".to_string(),
                    name: l.title.clone().unwrap_or_default(),
                    owner_user_id: l.assigned_to,
                    owner_user_name: None,
                    next_follow_at: nfa.map(|t| t.to_string()),
                    overdue_days,
                });
            }
        }

        // 排序：逾期天数多的在前
        items.sort_by(|a, b| b.overdue_days.cmp(&a.overdue_days));

        let total = items.len() as u64;
        let page_size = query.page_size;
        let page_num = query.page_num;
        let start = ((page_num - 1) * page_size) as usize;
        let end = (start + page_size as usize).min(items.len());
        let paged: Vec<FollowUpTodoVO> = if start < end {
            items[start..end].to_vec()
        } else {
            vec![]
        };

        Ok(ResultPage {
            items: paged,
            total: total as i64,
            current_page: page_num as i64,
            page_size: page_size as i64,
            total_pages: ((total as f64) / (page_size as f64)).ceil() as i64,
        })
    }

    /// 待回款提醒
    pub async fn payment_list(
        db: &DatabaseConnection,
        _user_id: i64,
        query: &PaymentTodoQuery,
    ) -> Result<ResultPage<Vec<PaymentTodoVO>>> {
        let today = Local::now().naive_local().date();
        let days = query.days.unwrap_or(7);
        let deadline = today + chrono::Duration::days(days as i64);

        let paginator = PlanEntity::find()
            .filter(PlanColumn::PlanDate.is_not_null())
            .filter(PlanColumn::PlanDate.lte(deadline))
            .filter(PlanColumn::Status.is_in(vec![0, 1]))
            .order_by_asc(PlanColumn::PlanDate)
            .paginate(db, query.page_size);

        let total = paginator
            .num_items()
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        let items = paginator
            .fetch_page(query.page_num - 1)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        let list: Vec<PaymentTodoVO> = items
            .into_iter()
            .map(|p| {
                let remaining_days = p
                    .plan_date
                    .map(|d| (d - today).num_days() as i32)
                    .unwrap_or(0);
                PaymentTodoVO {
                    id: p.id,
                    contract_id: p.contract_id,
                    contract_title: None,
                    stage_name: p.stage_name,
                    plan_amount: p.plan_amount,
                    received_amount: p.received_amount,
                    plan_date: p.plan_date.map(|d| d.to_string()),
                    remaining_days,
                    status: p.status,
                }
            })
            .collect();

        Ok(ResultPage {
            items: list,
            total: total as i64,
            current_page: query.page_num as i64,
            page_size: query.page_size as i64,
            total_pages: ((total as f64) / (query.page_size as f64)).ceil() as i64,
        })
    }

    /// 合同到期提醒
    pub async fn contract_list(
        db: &DatabaseConnection,
        _user_id: i64,
        query: &ContractTodoQuery,
    ) -> Result<ResultPage<Vec<ContractTodoVO>>> {
        let today = Local::now().naive_local().date();
        let days = query.days.unwrap_or(30);
        let deadline = today + chrono::Duration::days(days as i64);

        let paginator = ContractEntity::find()
            .filter(ContractColumn::EndDate.is_not_null())
            .filter(ContractColumn::EndDate.between(today, deadline))
            .filter(ContractColumn::Status.is_in(vec![2, 3]))
            .filter(ContractColumn::Deleted.eq(0))
            .order_by_asc(ContractColumn::EndDate)
            .paginate(db, query.page_size);

        let total = paginator
            .num_items()
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        let items = paginator
            .fetch_page(query.page_num - 1)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        let list: Vec<ContractTodoVO> = items
            .into_iter()
            .map(|c| {
                let remaining_days = c
                    .end_date
                    .map(|d| (d - today).num_days() as i32)
                    .unwrap_or(0);
                ContractTodoVO {
                    id: c.id,
                    contract_no: c.contract_no,
                    title: c.title,
                    customer_name: None,
                    end_date: c.end_date.map(|d| d.to_string()),
                    amount: c.amount,
                    remaining_days,
                    status: c.status.map(|s| s as i32),
                    assigned_to: c.assigned_to,
                }
            })
            .collect();

        Ok(ResultPage {
            items: list,
            total: total as i64,
            current_page: query.page_num as i64,
            page_size: query.page_size as i64,
            total_pages: ((total as f64) / (query.page_size as f64)).ceil() as i64,
        })
    }

    /// 停滞商机
    pub async fn opportunity_list(
        db: &DatabaseConnection,
        _user_id: i64,
        query: &OpportunityTodoQuery,
    ) -> Result<ResultPage<Vec<OpportunityTodoVO>>> {
        let now = Local::now().naive_local();
        let days = query.days.unwrap_or(30);
        let threshold = now - chrono::Duration::days(days as i64);

        let paginator = OppEntity::find()
            .filter(OppColumn::UpdateTime.lt(threshold))
            .filter(OppColumn::Stage.ne(5))
            .filter(OppColumn::Deleted.eq(0))
            .order_by_asc(OppColumn::UpdateTime)
            .paginate(db, query.page_size);

        let total = paginator
            .num_items()
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        let items = paginator
            .fetch_page(query.page_num - 1)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        const STAGE_NAMES: [&str; 5] = [
            "初步沟通", "需求确认", "方案沟通", "已报价", "成交/丢单",
        ];

        let list: Vec<OpportunityTodoVO> = items
            .into_iter()
            .map(|o| {
                let stagnant_days = o
                    .update_time
                    .map(|t| (now - t).num_days() as i32)
                    .unwrap_or(0);
                let stage_name = o
                    .stage
                    .and_then(|s| {
                        if s >= 1 && s <= 5 {
                            Some(STAGE_NAMES[(s - 1) as usize].to_string())
                        } else {
                            None
                        }
                    });
                OpportunityTodoVO {
                    id: o.id,
                    title: o.title,
                    customer_name: None,
                    stage: o.stage,
                    stage_name,
                    expected_close_date: o.expected_close_date.map(|d| d.to_string()),
                    update_time: o.update_time.map(|t| t.to_string()),
                    stagnant_days,
                    assigned_to: o.assigned_to,
                }
            })
            .collect();

        Ok(ResultPage {
            items: list,
            total: total as i64,
            current_page: query.page_num as i64,
            page_size: query.page_size as i64,
            total_pages: ((total as f64) / (query.page_size as f64)).ceil() as i64,
        })
    }

    // ============ Private helpers ============

    async fn count_pending_approval(db: &DatabaseConnection, user_id: i64) -> Result<i64> {
        let page = ApprovalModel::find_instance_list_filtered(
            db, user_id, None, None, None, 1, 1,
        )
        .await?;
        Ok(page.total)
    }

    fn apply_follow_up_range_customer(
        mut q: sea_orm::Select<CustomerEntity>,
        range_type: &str,
        now: chrono::NaiveDateTime,
        today: NaiveDate,
    ) -> sea_orm::Select<CustomerEntity> {
        match range_type {
            "overdue" => {
                q = q.filter(CustomerColumn::NextFollowAt.lt(now));
            }
            "today" => {
                let start = today.and_hms_opt(0, 0, 0).unwrap();
                let end = today.and_hms_opt(23, 59, 59).unwrap();
                q = q.filter(CustomerColumn::NextFollowAt.between(start, end));
            }
            _ => {
                let end = today.and_hms_opt(23, 59, 59).unwrap();
                q = q.filter(CustomerColumn::NextFollowAt.lte(end));
            }
        }
        q
    }

    fn apply_follow_up_range_lead(
        mut q: sea_orm::Select<LeadEntity>,
        range_type: &str,
        now: chrono::NaiveDateTime,
        today: NaiveDate,
    ) -> sea_orm::Select<LeadEntity> {
        match range_type {
            "overdue" => {
                q = q.filter(LeadColumn::NextFollowAt.lt(now));
            }
            "today" => {
                let start = today.and_hms_opt(0, 0, 0).unwrap();
                let end = today.and_hms_opt(23, 59, 59).unwrap();
                q = q.filter(LeadColumn::NextFollowAt.between(start, end));
            }
            _ => {
                let end = today.and_hms_opt(23, 59, 59).unwrap();
                q = q.filter(LeadColumn::NextFollowAt.lte(end));
            }
        }
        q
    }

    fn calc_overdue_days(nfa: Option<chrono::NaiveDateTime>, today: NaiveDate) -> i32 {
        nfa.map(|t| {
            let d = t.date();
            (today - d).num_days() as i32
        })
        .unwrap_or(0)
    }
}
