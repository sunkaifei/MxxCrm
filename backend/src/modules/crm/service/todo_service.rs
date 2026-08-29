use chrono::{Datelike, Local, NaiveDate};
use rust_decimal::Decimal;
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};

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
use crate::modules::sale::entity::invoice::{Column as InvoiceColumn, Entity as InvoiceEntity};
use crate::modules::sale::entity::order::{Column as OrderColumn, Entity as OrderEntity};
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

        // 待回款提醒（7天内到期且未完成，仅我名下客户的回款计划）
        let payment_deadline = today + chrono::Duration::days(7);
        let pending_payment = PlanEntity::find()
            .filter(PlanColumn::PlanDate.is_not_null())
            .filter(PlanColumn::PlanDate.lte(payment_deadline))
            .filter(PlanColumn::Status.is_in(vec![0, 1]))
            .filter(PlanColumn::OwnerUserId.eq(user_id))
            .count(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        // 即将到期合同（30天内，仅我负责的）
        let contract_deadline = today + chrono::Duration::days(30);
        let expiring_contract = ContractEntity::find()
            .filter(ContractColumn::EndDate.is_not_null())
            .filter(ContractColumn::EndDate.between(today, contract_deadline))
            .filter(ContractColumn::Status.is_in(vec![2, 3]))
            .filter(ContractColumn::Deleted.eq(0))
            .filter(ContractColumn::AssignedTo.eq(user_id))
            .count(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        // 停滞商机（超过30天未更新且未成交，仅我负责的）
        let stagnant_threshold = now - chrono::Duration::days(30);
        let stagnant_opportunity = OppEntity::find()
            .filter(OppColumn::UpdateTime.lt(stagnant_threshold))
            .filter(OppColumn::Stage.ne(5))
            .filter(OppColumn::Deleted.eq(0))
            .filter(OppColumn::AssignedTo.eq(user_id))
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

        // 查询线索（排除已转化/无效线索：转化或判定无效后不应残留跟进待办；无状态的线索保留）
        if item_type == "all" || item_type == "lead" {
            let mut q = LeadEntity::find()
                .filter(LeadColumn::NextFollowAt.is_not_null())
                .filter(LeadColumn::AssignedTo.eq(user_id))
                .filter(LeadColumn::Deleted.eq(0))
                .filter(
                    Condition::any()
                        .add(LeadColumn::Status.is_null())
                        .add(LeadColumn::Status.is_not_in(vec![3, 4])),
                );

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
        user_id: i64,
        query: &PaymentTodoQuery,
    ) -> Result<ResultPage<Vec<PaymentTodoVO>>> {
        let today = Local::now().naive_local().date();
        let days = query.days.unwrap_or(7);
        let deadline = today + chrono::Duration::days(days as i64);

        // 仅我名下客户的回款计划（待办=我需要处理的事，管理视角看全量走列表页）
        let mut finder = PlanEntity::find()
            .filter(PlanColumn::PlanDate.is_not_null())
            .filter(PlanColumn::PlanDate.lte(deadline))
            .filter(PlanColumn::Status.is_in(vec![0, 1]))
            .filter(PlanColumn::OwnerUserId.eq(user_id))
            .order_by_asc(PlanColumn::PlanDate);

        let paginator = finder.paginate(db, query.page_size);

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

    /// 合同到期提醒（仅我负责的合同）
    pub async fn contract_list(
        db: &DatabaseConnection,
        user_id: i64,
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
            .filter(ContractColumn::AssignedTo.eq(user_id))
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

    /// 停滞商机（仅我负责的商机）
    pub async fn opportunity_list(
        db: &DatabaseConnection,
        user_id: i64,
        query: &OpportunityTodoQuery,
    ) -> Result<ResultPage<Vec<OpportunityTodoVO>>> {
        let now = Local::now().naive_local();
        let days = query.days.unwrap_or(30);
        let threshold = now - chrono::Duration::days(days as i64);

        let paginator = OppEntity::find()
            .filter(OppColumn::UpdateTime.lt(threshold))
            .filter(OppColumn::Stage.ne(5))
            .filter(OppColumn::Deleted.eq(0))
            .filter(OppColumn::AssignedTo.eq(user_id))
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

    /// 任务日历：返回指定月份（缺省当月）用户名下日程型任务明细（跟进提醒/待回款/合同到期）
    ///
    /// - 跟进提醒：客户/线索 next_follow_at 落在月内（线索排除已转化/无效，对齐 follow_up_list）
    /// - 待回款：回款计划 plan_date 落在月内且未完成（status 0/1，仅我名下）
    /// - 合同到期：合同 end_date 落在月内且执行中（status 2/3，仅我负责）
    /// - 商机预计成交：expected_close_date 落在月内且未终态（排除 stage 5/6，仅我负责）
    /// - 发票到期：due_date 落在月内且有效（排除 4=作废/5=红冲，仅我名下）
    /// - 订单付款到期：payment_due_date 落在月内且未付清（pay_status 1=未支付/2=部分支付，仅我名下）
    /// 一次返回整月明细，前端按日期分组打点与点击过滤（月度数据量小，无需分页）
    pub async fn calendar_tasks(
        db: &DatabaseConnection,
        user_id: i64,
        query: &CalendarTaskQuery,
    ) -> Result<Vec<CalendarTaskVO>> {
        let today = Local::now().naive_local().date();
        // 解析月份：YYYY-MM，缺省当月
        let (year, month) = match query.month.as_deref().map(str::trim) {
            Some(m) if !m.is_empty() => {
                let parts: Vec<&str> = m.split('-').collect();
                if parts.len() != 2 {
                    return Err(Error::from("month 格式应为 YYYY-MM"));
                }
                let y: i32 = parts[0]
                    .parse()
                    .map_err(|_| Error::from("month 年份无效"))?;
                let mo: u32 = parts[1]
                    .parse()
                    .map_err(|_| Error::from("month 月份无效"))?;
                if !(1..=12).contains(&mo) {
                    return Err(Error::from("month 月份应在 1-12 之间"));
                }
                (y, mo)
            }
            _ => (today.year(), today.month()),
        };
        let month_start = NaiveDate::from_ymd_opt(year, month, 1)
            .ok_or_else(|| Error::from("month 无效"))?;
        let next_month_first = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
        };
        let month_start_dt = month_start.and_hms_opt(0, 0, 0).unwrap();
        let next_month_start_dt = next_month_first.and_hms_opt(0, 0, 0).unwrap();

        let mut items: Vec<CalendarTaskVO> = Vec::new();

        // 跟进提醒：客户（next_follow_at 落在月内，仅我名下）
        let customers = CustomerEntity::find()
            .filter(CustomerColumn::NextFollowAt.is_not_null())
            .filter(CustomerColumn::NextFollowAt.gte(month_start_dt))
            .filter(CustomerColumn::NextFollowAt.lt(next_month_start_dt))
            .filter(CustomerColumn::AssignedTo.eq(user_id))
            .filter(CustomerColumn::Deleted.eq(0))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        for c in customers {
            let d = match c.next_follow_at.map(|t| t.date()) {
                Some(d) => d,
                None => continue,
            };
            let name = c
                .company_name
                .clone()
                .or(c.short_name.clone())
                .or(c.person_name.clone())
                .unwrap_or_default();
            items.push(CalendarTaskVO {
                date: d.to_string(),
                task_type: "followUp".to_string(),
                title: if name.is_empty() {
                    "客户跟进".to_string()
                } else {
                    name
                },
                business_id: c.id,
                amount: None,
            });
        }

        // 跟进提醒：线索（排除已转化/无效线索，对齐 follow_up_list 过滤）
        let leads = LeadEntity::find()
            .filter(LeadColumn::NextFollowAt.is_not_null())
            .filter(LeadColumn::NextFollowAt.gte(month_start_dt))
            .filter(LeadColumn::NextFollowAt.lt(next_month_start_dt))
            .filter(LeadColumn::AssignedTo.eq(user_id))
            .filter(LeadColumn::Deleted.eq(0))
            .filter(
                Condition::any()
                    .add(LeadColumn::Status.is_null())
                    .add(LeadColumn::Status.is_not_in(vec![3, 4])),
            )
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        for l in leads {
            let d = match l.next_follow_at.map(|t| t.date()) {
                Some(d) => d,
                None => continue,
            };
            let title = l.title.clone().unwrap_or_default();
            items.push(CalendarTaskVO {
                date: d.to_string(),
                task_type: "followUp".to_string(),
                title: if title.is_empty() {
                    "线索跟进".to_string()
                } else {
                    title
                },
                business_id: l.id,
                amount: None,
            });
        }

        // 待回款：回款计划（plan_date 落在月内且未完成 status 0/1，仅我名下）
        let plans = PlanEntity::find()
            .filter(PlanColumn::PlanDate.is_not_null())
            .filter(PlanColumn::PlanDate.gte(month_start))
            .filter(PlanColumn::PlanDate.lt(next_month_first))
            .filter(PlanColumn::Status.is_in(vec![0, 1]))
            .filter(PlanColumn::OwnerUserId.eq(user_id))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        for p in plans {
            let d = match p.plan_date {
                Some(d) => d,
                None => continue,
            };
            items.push(CalendarTaskVO {
                date: d.to_string(),
                task_type: "payment".to_string(),
                title: p
                    .stage_name
                    .clone()
                    .unwrap_or_else(|| "回款计划".to_string()),
                business_id: p.contract_id.unwrap_or(0),
                amount: p.plan_amount,
            });
        }

        // 合同到期：end_date 落在月内且执行中（status 2/3，仅我负责）
        let contracts = ContractEntity::find()
            .filter(ContractColumn::EndDate.is_not_null())
            .filter(ContractColumn::EndDate.gte(month_start))
            .filter(ContractColumn::EndDate.lt(next_month_first))
            .filter(ContractColumn::Status.is_in(vec![2, 3]))
            .filter(ContractColumn::Deleted.eq(0))
            .filter(ContractColumn::AssignedTo.eq(user_id))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        for ct in contracts {
            let d = match ct.end_date {
                Some(d) => d,
                None => continue,
            };
            let title = ct
                .title
                .clone()
                .or(ct.contract_no.clone())
                .unwrap_or_else(|| "合同".to_string());
            items.push(CalendarTaskVO {
                date: d.to_string(),
                task_type: "contract".to_string(),
                title,
                business_id: ct.id,
                amount: ct.amount,
            });
        }

        // 商机预计成交：expected_close_date 落在月内且未到终态（stage 5=成交/丢单、6=作废，仅我负责）
        let opps = OppEntity::find()
            .filter(OppColumn::ExpectedCloseDate.is_not_null())
            .filter(OppColumn::ExpectedCloseDate.gte(month_start))
            .filter(OppColumn::ExpectedCloseDate.lt(next_month_first))
            .filter(
                Condition::any()
                    .add(OppColumn::Stage.is_null())
                    .add(OppColumn::Stage.is_not_in(vec![5, 6])),
            )
            .filter(OppColumn::AssignedTo.eq(user_id))
            .filter(OppColumn::Deleted.eq(0))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        for o in opps {
            let d = match o.expected_close_date {
                Some(d) => d,
                None => continue,
            };
            let title = o
                .title
                .clone()
                .or(o.opportunity_no.clone())
                .unwrap_or_else(|| "商机预计成交".to_string());
            items.push(CalendarTaskVO {
                date: d.to_string(),
                task_type: "opportunity".to_string(),
                title,
                business_id: o.id,
                amount: o.amount,
            });
        }

        // 发票到期：due_date 落在月内且有效（排除 4=作废、5=红冲，仅我名下）
        let invoices = InvoiceEntity::find()
            .filter(InvoiceColumn::DueDate.is_not_null())
            .filter(InvoiceColumn::DueDate.gte(month_start))
            .filter(InvoiceColumn::DueDate.lt(next_month_first))
            .filter(
                Condition::any()
                    .add(InvoiceColumn::Status.is_null())
                    .add(InvoiceColumn::Status.is_not_in(vec![4, 5])),
            )
            .filter(InvoiceColumn::OwnerUserId.eq(user_id))
            .filter(InvoiceColumn::Deleted.eq(0))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        for inv in invoices {
            let d = match inv.due_date {
                Some(d) => d,
                None => continue,
            };
            let title = inv
                .invoice_no
                .clone()
                .or(inv.customer_name.clone())
                .unwrap_or_else(|| "发票到期".to_string());
            items.push(CalendarTaskVO {
                date: d.to_string(),
                task_type: "invoice".to_string(),
                title,
                business_id: inv.id,
                amount: inv.amount,
            });
        }

        // 订单付款到期：payment_due_date 落在月内且未付清（pay_status 1=未支付/2=部分支付，仅我名下）
        let orders = OrderEntity::find()
            .filter(OrderColumn::PaymentDueDate.is_not_null())
            .filter(OrderColumn::PaymentDueDate.gte(month_start))
            .filter(OrderColumn::PaymentDueDate.lt(next_month_first))
            .filter(
                Condition::any()
                    .add(OrderColumn::PayStatus.is_null())
                    .add(OrderColumn::PayStatus.is_in(vec![1, 2])),
            )
            .filter(OrderColumn::OwnerUserId.eq(user_id))
            .filter(OrderColumn::Deleted.eq(0))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        for od in orders {
            let d = match od.payment_due_date {
                Some(d) => d,
                None => continue,
            };
            let title = od
                .order_no
                .clone()
                .or(od.customer_name.clone())
                .unwrap_or_else(|| "订单付款到期".to_string());
            items.push(CalendarTaskVO {
                date: d.to_string(),
                task_type: "order".to_string(),
                title,
                business_id: od.id,
                amount: od.total_amount,
            });
        }

        // 按日期升序，同日按类型稳定排序
        items.sort_by(|a, b| {
            a.date
                .cmp(&b.date)
                .then_with(|| a.task_type.cmp(&b.task_type))
        });
        Ok(items)
    }


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
