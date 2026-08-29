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

use chrono::{Datelike, Months, NaiveDate};
use rust_decimal::Decimal;

use crate::core::errors::error::Result;
use crate::core::r#enum::contract_status_enum::ContractStatus;
use crate::modules::approval::service::approval_service::ApprovalService;
use crate::modules::crm::entity::{contract, customer, followup, opportunity};
use crate::modules::crm::model::contract_payment_plan::PaymentPlanModel;
use crate::modules::finance::service::payslip_service;
use crate::modules::inventory::model::inbound::InboundListQuery;
use crate::modules::inventory::model::outbound::OutboundListQuery;
use crate::modules::inventory::service::{inbound_service, inventory_service, outbound_service};
use crate::modules::purchase::service::purchase_requisition_service;
use crate::modules::system::model::dashboard_card::DashboardCardModel;
use crate::modules::system::model::dashboard_workspace::*;
use crate::modules::system::model::notice::ListQuery;
use crate::modules::system::service::{admin_service, dashboard_card_service, notice_service};

pub const CARD_STOCK_ALERT: &str = "workspace_stock_alert";
pub const CARD_STOCK_DOC_TODO: &str = "workspace_stock_doc_todo";
pub const CARD_PURCHASE_APPROVAL: &str = "workspace_purchase_approval";
pub const CARD_PAYMENT_REMINDER: &str = "workspace_payment_reminder";
pub const CARD_PAYSLIP_STAT: &str = "workspace_payslip_stat";
pub const CARD_HR_TODO: &str = "workspace_hr_todo";
pub const CARD_SALES_PERFORMANCE: &str = "workspace_sales_performance";
pub const CARD_ANNOUNCEMENT: &str = "workspace_announcement";

const SUMMARY_ITEM_LIMIT: usize = 6;

/// 工作台聚合摘要（方案 6 节 #4）：一次返回各卡摘要，防首屏请求线性膨胀
///
/// - 仅返回当前用户可见卡片的摘要，不可见卡片返回空数据（数据兜底，手册硬性规则）
/// - 单卡数据源异常时降级为空摘要，不影响其余卡片
pub async fn get_workspace_summary(db: &DbConn, user_id: i64) -> Result<WorkspaceSummaryVO> {
    let visible = get_visible_codes(db, user_id).await;
    let is_admin = admin_service::get_by_detail(db, &Some(user_id))
        .await
        .map(|a| a.user_type == Some(1))
        .unwrap_or(false);

    let has = |code: &str| visible.iter().any(|c| c == code);

    let stock_alert = build_stock_alert(db, has(CARD_STOCK_ALERT)).await;
    let stock_doc_todo = build_stock_doc_todo(db, has(CARD_STOCK_DOC_TODO)).await;
    let purchase_approval = build_purchase_approval(db, user_id, has(CARD_PURCHASE_APPROVAL)).await;
    let payment_reminder = build_payment_reminder(db, user_id, is_admin, has(CARD_PAYMENT_REMINDER)).await;
    let payslip_stat = build_payslip_stat(db, has(CARD_PAYSLIP_STAT)).await;
    let hr_todo = build_hr_todo(db, user_id, has(CARD_HR_TODO)).await;
    let sales_performance = build_sales_performance(db, user_id, has(CARD_SALES_PERFORMANCE)).await;
    let announcement = build_announcement(db, user_id, has(CARD_ANNOUNCEMENT)).await;

    Ok(WorkspaceSummaryVO {
        stock_alert,
        stock_doc_todo,
        purchase_approval,
        payment_reminder,
        payslip_stat,
        hr_todo,
        sales_performance,
        announcement,
    })
}

/// 当前用户可见卡片编码集合（get_visible_cards 异常时降级为全部启用卡，保证降级方向为"全部可见"）
async fn get_visible_codes(db: &DbConn, user_id: i64) -> Vec<String> {
    match dashboard_card_service::get_visible_cards(db, user_id).await {
        Ok(cards) => cards.into_iter().filter_map(|c| c.card_code).collect(),
        Err(_) => match DashboardCardModel::find_all_enabled(db).await {
            Ok(all) => all.into_iter().filter_map(|c| c.card_code).collect(),
            Err(_) => vec![],
        },
    }
}

async fn build_stock_alert(db: &DbConn, visible: bool) -> StockAlertSummary {
    if !visible {
        return StockAlertSummary::default();
    }
    match inventory_service::get_alert_list(db, None, None, None, 1, SUMMARY_ITEM_LIMIT as u64).await {
        Ok(data) => StockAlertSummary {
            total: data.total as i64,
            items: data
                .items
                .into_iter()
                .map(|it| StockAlertItem {
                    product_name: it.product_name,
                    warehouse_name: it.warehouse_name,
                    quantity: it.quantity,
                    alert_type: it.alert_type,
                    alert_min_quantity: it.alert_min_quantity,
                    alert_max_quantity: it.alert_max_quantity,
                })
                .collect(),
        },
        Err(_) => StockAlertSummary::default(),
    }
}

/// 待办出入库单：status 0=草稿 1=审核中（各查一次后合并，total 相加）
async fn build_stock_doc_todo(db: &DbConn, visible: bool) -> StockDocTodoSummary {
    if !visible {
        return StockDocTodoSummary::default();
    }
    let mut inbound_total = 0i64;
    let mut outbound_total = 0i64;
    let mut items: Vec<StockDocTodoItem> = Vec::new();

    for st in [0i32, 1] {
        let in_query = InboundListQuery {
            page_num: 1,
            page_size: 20,
            inbound_no: None,
            inbound_type: None,
            warehouse_id: None,
            status: Some(st),
        };
        if let Ok(vo) = inbound_service::get_list(db, &in_query).await {
            inbound_total += vo.total as i64;
            for it in vo.list {
                items.push(StockDocTodoItem {
                    biz_type: Some("inbound".to_string()),
                    id: Some(it.id),
                    order_no: it.inbound_no,
                    status: it.status,
                    total_quantity: it.total_quantity,
                    total_amount: it.total_amount,
                    created_by_name: it.created_by_name,
                    create_time: it.create_time,
                });
            }
        }

        let out_query = OutboundListQuery {
            page_num: 1,
            page_size: 20,
            outbound_no: None,
            outbound_type: None,
            warehouse_id: None,
            status: Some(st),
        };
        if let Ok(vo) = outbound_service::get_list(db, &out_query).await {
            outbound_total += vo.total as i64;
            for it in vo.list {
                items.push(StockDocTodoItem {
                    biz_type: Some("outbound".to_string()),
                    id: Some(it.id),
                    order_no: it.outbound_no,
                    status: it.status,
                    total_quantity: it.total_quantity,
                    total_amount: it.total_amount,
                    created_by_name: it.created_by_name,
                    create_time: it.create_time,
                });
            }
        }
    }

    items.sort_by(|a, b| b.create_time.cmp(&a.create_time));
    items.truncate(SUMMARY_ITEM_LIMIT);
    StockDocTodoSummary {
        inbound_total,
        outbound_total,
        items,
    }
}

async fn build_purchase_approval(db: &DbConn, user_id: i64, visible: bool) -> PurchaseApprovalSummary {
    if !visible {
        return PurchaseApprovalSummary::default();
    }
    match purchase_requisition_service::get_my_approval_list(db, user_id, 1, SUMMARY_ITEM_LIMIT as i64).await {
        Ok((list, total)) => PurchaseApprovalSummary {
            total: total as i64,
            items: list
                .into_iter()
                .map(|it| PurchaseApprovalItem {
                    id: it.id,
                    pr_no: it.pr_no,
                    title: it.title,
                    status: it.status,
                    total_amount: it.total_amount,
                    urgency: it.urgency,
                    create_time: it.create_time,
                })
                .collect(),
        },
        Err(_) => PurchaseApprovalSummary::default(),
    }
}

/// 待收款提醒：进行中的回款计划（status 0=未开始 1=部分回款），按 plan_date 升序
async fn build_payment_reminder(db: &DbConn, user_id: i64, is_admin: bool, visible: bool) -> PaymentReminderSummary {
    if !visible {
        return PaymentReminderSummary::default();
    }
    let owner: Option<Vec<i64>> = if is_admin { None } else { Some(vec![user_id]) };
    let mut total = 0i64;
    let mut rows: Vec<Vec<crate::modules::crm::entity::contract_payment_plan::Model>> = Vec::new();

    for st in [0i32, 1] {
        match PaymentPlanModel::select_in_page_by_owner_user_ids(db, 1, 20, None, Some(st), None, owner.clone()).await {
            Ok((list, cnt)) => {
                total += cnt;
                rows.push(list);
            }
            Err(_) => {}
        }
    }

    let mut flat: Vec<crate::modules::crm::entity::contract_payment_plan::Model> = rows.into_iter().flatten().collect();
    flat.sort_by_key(|r| r.plan_date.map(|d| (0i8, d)).unwrap_or((1i8, NaiveDate::MIN)));
    flat.truncate(SUMMARY_ITEM_LIMIT);

    PaymentReminderSummary {
        total,
        items: flat
            .into_iter()
            .map(|m| PaymentReminderItem {
                id: Some(m.id),
                contract_id: m.contract_id,
                stage_name: m.stage_name,
                plan_amount: m.plan_amount,
                received_amount: m.received_amount,
                plan_date: m.plan_date,
                status: m.status,
            })
            .collect(),
    }
}

async fn build_payslip_stat(db: &DbConn, visible: bool) -> PayslipStatSummary {
    if !visible {
        return PayslipStatSummary::default();
    }
    let now = chrono::Local::now();
    match payslip_service::get_read_statistics(db, now.year(), now.month() as i32).await {
        Ok(s) => PayslipStatSummary {
            sent_count: s.sent_count,
            read_count: s.read_count,
            unread_count: s.unread_count,
            total_count: s.total_count,
        },
        Err(_) => PayslipStatSummary::default(),
    }
}

/// 人事待办：待我审批的入职（business_type=user）与离职（resign）在途实例（status 1/2）
async fn build_hr_todo(db: &DbConn, user_id: i64, visible: bool) -> HrTodoSummary {
    if !visible {
        return HrTodoSummary::default();
    }
    let mut total = 0i64;
    let mut onboarding = 0i64;
    let mut resign = 0i64;
    let mut items: Vec<HrTodoItem> = Vec::new();

    for bt in ["user", "resign"] {
        for st in [1i32, 2] {
            if let Ok(page) = ApprovalService::find_instance_list_filtered(db, user_id, Some(bt), Some(st), None, 1, 10).await {
                total += page.total;
                if bt == "user" {
                    onboarding += page.total;
                } else {
                    resign += page.total;
                }
                for vo in page.items {
                    items.push(HrTodoItem {
                        id: vo.id,
                        business_type: vo.business_type,
                        business_title: vo.business_title,
                        submitter_name: vo.submitter_name,
                        submitted_at: vo.submitted_at,
                    });
                }
            }
        }
    }

    items.sort_by(|a, b| b.submitted_at.cmp(&a.submitted_at));
    items.truncate(SUMMARY_ITEM_LIMIT);
    HrTodoSummary {
        total,
        onboarding,
        resign,
        items,
    }
}

/// 销售业绩：当前用户当月新增客户/跟进/商机与成交合同金额（签署后状态）
async fn build_sales_performance(db: &DbConn, user_id: i64, visible: bool) -> SalesPerformanceSummary {
    if !visible {
        return SalesPerformanceSummary::default();
    }
    let today = chrono::Local::now().date_naive();
    let month_start = today.with_day(1).unwrap_or(today);
    let next_month_start = month_start + Months::new(1);
    let start_dt = month_start.and_hms_opt(0, 0, 0).unwrap_or_default();
    let end_dt = next_month_start.and_hms_opt(0, 0, 0).unwrap_or_default();

    let new_customers = customer::Entity::find()
        .filter(customer::Column::Deleted.eq(0))
        .filter(customer::Column::AssignedTo.eq(user_id))
        .filter(customer::Column::CreateTime.gte(start_dt))
        .filter(customer::Column::CreateTime.lt(end_dt))
        .count(db)
        .await
        .unwrap_or(0) as i64;

    let follow_ups = followup::Entity::find()
        .filter(followup::Column::Deleted.eq(0))
        .filter(followup::Column::CreatedBy.eq(user_id))
        .filter(followup::Column::CreateTime.gte(start_dt))
        .filter(followup::Column::CreateTime.lt(end_dt))
        .count(db)
        .await
        .unwrap_or(0) as i64;

    let new_opportunities = opportunity::Entity::find()
        .filter(opportunity::Column::Deleted.eq(0))
        .filter(opportunity::Column::AssignedTo.eq(user_id))
        .filter(opportunity::Column::CreateTime.gte(start_dt))
        .filter(opportunity::Column::CreateTime.lt(end_dt))
        .count(db)
        .await
        .unwrap_or(0) as i64;

    let deal_amount = match contract::Entity::find()
        .filter(contract::Column::Deleted.eq(0))
        .filter(contract::Column::AssignedTo.eq(user_id))
        .filter(contract::Column::SignDate.gte(month_start))
        .filter(contract::Column::SignDate.lt(next_month_start))
        .filter(contract::Column::Status.is_in([
            ContractStatus::Signed,
            ContractStatus::Executing,
            ContractStatus::Completed,
        ]))
        .all(db)
        .await
    {
        Ok(rows) => rows.iter().filter_map(|c| c.total_amount).sum::<Decimal>(),
        Err(_) => Decimal::ZERO,
    };

    SalesPerformanceSummary {
        new_customers,
        follow_ups,
        new_opportunities,
        deal_amount,
    }
}

async fn build_announcement(db: &DbConn, user_id: i64, visible: bool) -> AnnouncementSummary {
    if !visible {
        return AnnouncementSummary::default();
    }
    let mut summary = AnnouncementSummary::default();

    let query = ListQuery {
        title: None,
        user_id: Some(user_id),
        is_read: None,
        status: None,
        page_num: Some(1),
        page_size: Some(SUMMARY_ITEM_LIMIT as i64),
    };
    if let Ok(page) = notice_service::get_by_my_page(db, query).await {
        summary.total = page.total;
        summary.items = page
            .items
            .into_iter()
            .map(|n| AnnouncementItem {
                id: n.id,
                title: n.title,
                level: n.level,
                publish_name: n.publish_name,
                publish_time: n.publish_time,
                is_read: n.is_read,
            })
            .collect();
    }

    let unread_query = ListQuery {
        title: None,
        user_id: Some(user_id),
        is_read: Some(0),
        status: None,
        page_num: Some(1),
        page_size: Some(1),
    };
    if let Ok(page) = notice_service::get_by_my_page(db, unread_query).await {
        summary.unread = page.total;
    }

    summary
}
