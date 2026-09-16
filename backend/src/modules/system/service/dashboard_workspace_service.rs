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
use crate::modules::system::service::{
    admin_service, dashboard_card_service, data_scope_service, notice_service, role_service,
    subordinate_service,
};

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
///
/// ## 数据视角（方案 v2.1 §7.2.1「接通既有统一层」）
///
/// 本函数**不再让数据卡各自硬编码 `user_id` 过滤**（"我的审批/待办"类任务卡
/// 除外——其口径是任务队列，与数据视角正交）。scope 解析委托既有统一层
/// `data_scope_service` / `subordinate_service`。
///
/// **兼容硬承诺**：不传 `scope` 时进入遗留模式（`ScopeCtx.legacy`），
/// 各数据卡严格保持改造前取数行为（sales_performance 连超管也仅本人、
/// payment_reminder 超管全部/他人本人），保证既有验收逐字段一致。
///
/// `scope` 取值与既有列表页 Tab 一致：`my` / `subordinate` / `all`。
pub async fn get_workspace_summary(
    db: &DbConn,
    user_id: i64,
    scope_req: Option<&str>,
    time_range_req: Option<&str>,
) -> Result<WorkspaceSummaryVO> {
    let visible_cards = dashboard_card_service::get_visible_cards(db, user_id)
        .await
        .unwrap_or_default();
    let visible: Vec<String> = visible_cards
        .iter()
        .filter_map(|c| c.card_code.clone())
        .collect();
    let is_admin = admin_service::get_by_detail(db, &Some(user_id))
        .await
        .map(|a| a.user_type == Some(1))
        .unwrap_or(false);

    // 解析数据视角（含越权静默降级），得到本次生效范围
    let scope_ctx = resolve_scope(db, user_id, is_admin, scope_req).await?;

    let has = |code: &str| visible.iter().any(|c| c == code);

    // 卡片配置（card_config JSON）解析：当前支持 timeRange / displayForm，
    // 卡级配置覆盖全局 time_range_req
    let card_config_of = |code: &str| -> Option<serde_json::Value> {
        visible_cards
            .iter()
            .find(|c| c.card_code.as_deref() == Some(code))
            .and_then(|c| c.card_config.as_deref())
            .and_then(|s| serde_json::from_str(s).ok())
    };
    let sp_config = card_config_of(CARD_SALES_PERFORMANCE);
    let sp_time_range = sp_config
        .as_ref()
        .and_then(|c| c.get("timeRange"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| time_range_req.map(String::from));

    let stock_alert = build_stock_alert(db, &scope_ctx, has(CARD_STOCK_ALERT)).await;
    let stock_doc_todo = build_stock_doc_todo(db, &scope_ctx, has(CARD_STOCK_DOC_TODO)).await;
    let purchase_approval = build_purchase_approval(db, &scope_ctx, has(CARD_PURCHASE_APPROVAL)).await;
    let payment_reminder = build_payment_reminder(db, &scope_ctx, has(CARD_PAYMENT_REMINDER)).await;
    let payslip_stat = build_payslip_stat(db, has(CARD_PAYSLIP_STAT)).await;
    let hr_todo = build_hr_todo(db, &scope_ctx, has(CARD_HR_TODO)).await;
    let sales_performance = build_sales_performance(db, &scope_ctx, has(CARD_SALES_PERFORMANCE), sp_time_range.as_deref()).await;
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
        scope: scope_ctx.scope.clone(),
        scope_applied: scope_ctx.applied,
        scope_options: scope_ctx.options.clone(),
    })
}

/// 数据视角上下文（方案 v2.1 §7.2.1）
///
/// 由既有 `data_scope_service` 统一产出，各卡**只消费不判断**。
#[derive(Debug, Clone)]
pub struct ScopeCtx {
    /// 本次实际生效的视角：my / subordinate / all
    pub scope: String,
    /// 请求的视角是否被成功应用（false = 已静默降级）
    pub applied: bool,
    /// 当前用户可用的视角列表
    pub options: Vec<String>,
    /// 当前用户本人 ID（"my" 口径直接使用）
    pub user_id: i64,
    /// 可见用户 ID 集合；`None` = 不限（全部数据权限）
    pub user_ids: Option<Vec<i64>>,
    /// 是否为「未传 scope」的遗留模式
    ///
    /// **为真时各数据卡严格保持改造前的取数行为**（验收硬承诺：
    /// "不传 scope 与改造前逐字段一致"）。这是因为旧实现中各卡默认口径
    /// 并不一致（如 sales_performance 连超管也只看本人，而 payment_reminder
    /// 超管看全部），无法用单一 scope 复原，故遗留模式下各卡走自己的旧逻辑。
    pub legacy: bool,
}

impl ScopeCtx {
    /// 是否不限范围（全量）
    pub fn is_unbounded(&self) -> bool {
        self.user_ids.is_none()
    }
}

/// 解析数据视角（复用既有统一层，不新造范围语义）
///
/// ## 可用视角判定（与 `use-data-scope-tabs.ts` 前端判定对齐）
/// - 超管（user_type=1）：`all` / `subordinate` / `my` 全可用
/// - data_scope = 1（全部）：`all` / `my`
/// - data_scope = 2/3/4（部门级）：`subordinate` / `my`
/// - data_scope = 5 / 未设置：仅 `my`
///
/// ## 静默降级（吸收 B2）
/// 请求的视角不可用时，降级为**该用户可用列表中的第一个**并置 `applied=false`，
/// **不返回错误**，避免用户切视角就弹错误框。
async fn resolve_scope(
    db: &DbConn,
    user_id: i64,
    is_admin: bool,
    requested: Option<&str>,
) -> Result<ScopeCtx> {
    // 数据权限五档（既有统一层的判定依据，此处仅用于推导"可用视角列表"）
    let data_scope = if is_admin {
        1
    } else {
        let roles = role_service::select_by_admin_id(db, &Some(user_id)).await?;
        let scopes: Vec<i32> = roles.iter().filter_map(|r| r.data_scope).collect();
        // 多角色：取最宽（数值最小即最宽，1 > 2 > 3/4 > 5），对齐统一层"任一为 1 即全量"的策略
        scopes.iter().copied().min().unwrap_or(5)
    };

    // 可用视角：优先级 all > subordinate > my
    let mut options: Vec<String> = Vec::new();
    let can_all = is_admin || data_scope == 1;
    let can_subordinate = is_admin || matches!(data_scope, 2 | 3 | 4);
    if can_all {
        options.push("all".to_string());
    }
    if can_subordinate {
        options.push("subordinate".to_string());
    }
    options.push("my".to_string());

    // 请求视角解析 + 降级
    let req = requested.map(|s| s.trim().to_ascii_lowercase());
    let (scope, applied, legacy) = match req.as_deref() {
        // 不传 = 遗留模式（各卡保持改造前行为），保证既有验收不变。
        // 生效视角仅作描述性返回：超管历史上看全部，其余人仅本人。
        None => (
            if is_admin { "all" } else { "my" }.to_string(),
            true,
            true,
        ),
        Some(s) if s.is_empty() => ("my".to_string(), true, false),
        Some(s) if options.iter().any(|o| o == s) => (s.to_string(), true, false),
        // 越权/未知 → 静默降级到可用列表首项
        Some(_) => (
            options.first().cloned().unwrap_or_else(|| "my".to_string()),
            false,
            false,
        ),
    };

    // 按生效视角计算可见用户集合（委托既有统一层）
    //
    // 遗留模式的 user_ids 语义 = 旧 payment_reminder 的手写规则：
    // 超管 None（不限）/ 其他人 Some([本人])——数据卡在遗留模式下各走旧逻辑，
    // 该集合仅保证 is_unbounded() 判定与旧行为一致。
    let user_ids: Option<Vec<i64>> = if legacy {
        if is_admin {
            None
        } else {
            Some(vec![user_id])
        }
    } else {
        match scope.as_str() {
            // 仅本人：不查库，直接本人（零额外开销）
            "my" => Some(vec![user_id]),
            // 下属：既有 subordinate_service（数据权限 ∪ 汇报线，已剔除本人）
            "subordinate" => subordinate_service::get_subordinate_scope_ids(db, user_id).await?,
            // 全部：走统一层（全量权限返回 None 表示不限）
            _ => data_scope_service::get_accessible_user_ids(db, user_id).await?,
        }
    };

    Ok(ScopeCtx {
        scope,
        applied,
        options,
        user_id,
        user_ids,
        legacy,
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

async fn build_stock_alert(db: &DbConn, ctx: &ScopeCtx, visible: bool) -> StockAlertSummary {
    if !visible {
        return StockAlertSummary::default();
    }
    // 库存预警为仓储公共数据（无负责人维度），沿用既有的 user_id 参数语义：
    // 传本人即可（底层按仓库权限过滤，与视角无关）
    match inventory_service::get_alert_list(db, None, None, None, 1, SUMMARY_ITEM_LIMIT as u64, None, ctx.user_id).await {
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
///
/// **任务型口径，不随数据视角变化**（与采购审批/人事待办同理）：
/// "待办"语义是"流程里停着的单"，对仓储/制单岗位是工作队列而非个人数据，
/// 旧实现即为全局视角（底层 `get_list` 在 scope=None 时不过滤用户），
/// 本方案保持该行为不变。
async fn build_stock_doc_todo(db: &DbConn, ctx: &ScopeCtx, visible: bool) -> StockDocTodoSummary {
    if !visible {
        return StockDocTodoSummary::default();
    }
    let mut inbound_total = 0i64;
    let mut outbound_total = 0i64;
    let mut items: Vec<StockDocTodoItem> = Vec::new();

    for st in [0i32, 1] {
        let in_query = InboundListQuery {
            scope: None,
            page_num: 1,
            page_size: 20,
            inbound_no: None,
            inbound_type: None,
            warehouse_id: None,
            status: Some(st),
        };
        // user_id 传 0：与旧实现一致；底层 scope=None 路径不使用该参数（全局视角）
        if let Ok(vo) = inbound_service::get_list(db, &in_query, 0).await {
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
            scope: None,
            page_num: 1,
            page_size: 20,
            outbound_no: None,
            outbound_type: None,
            warehouse_id: None,
            status: Some(st),
        };
        if let Ok(vo) = outbound_service::get_list(db, &out_query, 0).await {
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

async fn build_purchase_approval(db: &DbConn, ctx: &ScopeCtx, visible: bool) -> PurchaseApprovalSummary {
    if !visible {
        return PurchaseApprovalSummary::default();
    }
    // "待我审批"是任务型口径（签核流到我这儿的），与数据视角正交：
    // 无论何种视角，都应展示"需要我处理的审批"，故固定传本人
    match purchase_requisition_service::get_my_approval_list(db, ctx.user_id, 1, SUMMARY_ITEM_LIMIT as i64).await {
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
///
/// 视角处理（方案 v2.1 §7.2.1）：**本条是"接通既有统一层"的核心示例**。
/// 原实现手写 `if is_admin { None } else { Some(vec![user_id]) }`，
/// 只区分"超管看全部 / 其他人看自己"，无法支持"主管看下属"。
/// 现改为统一消费 `ScopeCtx`：`my` 看自己、`subordinate` 看下属、`all` 看全部。
async fn build_payment_reminder(db: &DbConn, ctx: &ScopeCtx, visible: bool) -> PaymentReminderSummary {
    if !visible {
        return PaymentReminderSummary::default();
    }
    // 统一层产出：None = 不限（全部数据权限），Some(ids) = 限定负责人集合
    let owner: Option<Vec<i64>> = if ctx.is_unbounded() {
        None
    } else {
        Some(ctx.user_ids.clone().unwrap_or_default())
    };
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
///
/// 与采购审批同理，"待我审批"是任务型口径，固定传本人（不随数据视角变化）
async fn build_hr_todo(db: &DbConn, ctx: &ScopeCtx, visible: bool) -> HrTodoSummary {
    if !visible {
        return HrTodoSummary::default();
    }
    let user_id = ctx.user_id;
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
///
/// 视角处理（方案 v2.1 §7.2.1）：**本条是硬编码 `user_id` 最集中的地方（原 4 处）**。
/// 原实现全部 `AssignedTo.eq(user_id)` / `CreatedBy.eq(user_id)`，
/// 主管无法看团队业绩。现统一改为按 `ScopeCtx.user_ids` 过滤。
/// 统计时间边界：month=本月（既有行为）/ quarter=本季 / year=本年；未知值回落 month
fn resolve_time_bounds(range: &str) -> (chrono::NaiveDateTime, chrono::NaiveDateTime) {
    use chrono::{Datelike, Months};
    let today = chrono::Local::now().date_naive();
    let start_day = match range {
        "quarter" => {
            let month_start = today.with_day(1).unwrap_or(today);
            let quarter_first_month = ((month_start.month() - 1) / 3) * 3 + 1;
            today
                .with_month(quarter_first_month)
                .and_then(|d| d.with_day(1))
                .unwrap_or(month_start)
        }
        "year" => today
            .with_month(1)
            .and_then(|d| d.with_day(1))
            .unwrap_or(today),
        // month 与未知值均回落本月（兼容既有口径）
        _ => today.with_day(1).unwrap_or(today),
    };
    let end_day = match range {
        "quarter" => start_day + Months::new(3),
        "year" => start_day + Months::new(12),
        _ => start_day + Months::new(1),
    };
    (
        start_day.and_hms_opt(0, 0, 0).unwrap_or_default(),
        end_day.and_hms_opt(0, 0, 0).unwrap_or_default(),
    )
}

async fn build_sales_performance(
    db: &DbConn,
    ctx: &ScopeCtx,
    visible: bool,
    time_range: Option<&str>,
) -> SalesPerformanceSummary {
    if !visible {
        return SalesPerformanceSummary::default();
    }
    let (start_dt, end_dt) = resolve_time_bounds(time_range.unwrap_or("month"));

    // 负责人集合：
    // - 遗留模式（未传 scope）：严格保持旧行为——**所有角色（含超管）只看本人**
    //   （旧实现 4 处全部 .eq(user_id)，超管也不例外）；
    // - 显式视角：按统一层集合过滤，None = 全量不过滤。
    let owner_ids: Option<Vec<i64>> = if ctx.legacy {
        Some(vec![ctx.user_id])
    } else {
        ctx.user_ids.clone()
    };

    // 四张表的负责人字段名不同（AssignedTo / CreatedBy），故分别构造查询；
    // 仅当 owner_ids 为 Some 时才追加 is_in 条件（None 表示全量，不过滤）
    let mut q_customer = customer::Entity::find()
        .filter(customer::Column::Deleted.eq(0))
        .filter(customer::Column::CreateTime.gte(start_dt))
        .filter(customer::Column::CreateTime.lt(end_dt));
    let mut q_followup = followup::Entity::find()
        .filter(followup::Column::Deleted.eq(0))
        .filter(followup::Column::CreateTime.gte(start_dt))
        .filter(followup::Column::CreateTime.lt(end_dt));
    let mut q_opportunity = opportunity::Entity::find()
        .filter(opportunity::Column::Deleted.eq(0))
        .filter(opportunity::Column::CreateTime.gte(start_dt))
        .filter(opportunity::Column::CreateTime.lt(end_dt));
    let mut q_contract = contract::Entity::find()
        .filter(contract::Column::Deleted.eq(0))
        .filter(contract::Column::SignDate.gte(start_dt))
        .filter(contract::Column::SignDate.lt(end_dt))
        .filter(contract::Column::Status.is_in([
            ContractStatus::Signed,
            ContractStatus::Executing,
            ContractStatus::Completed,
        ]));

    if let Some(ids) = &owner_ids {
        q_customer = q_customer.filter(customer::Column::AssignedTo.is_in(ids.clone()));
        q_followup = q_followup.filter(followup::Column::CreatedBy.is_in(ids.clone()));
        q_opportunity = q_opportunity.filter(opportunity::Column::AssignedTo.is_in(ids.clone()));
        q_contract = q_contract.filter(contract::Column::AssignedTo.is_in(ids.clone()));
    }

    let new_customers = q_customer.count(db).await.unwrap_or(0) as i64;
    let follow_ups = q_followup.count(db).await.unwrap_or(0) as i64;
    let new_opportunities = q_opportunity.count(db).await.unwrap_or(0) as i64;

    let deal_amount = match q_contract.all(db).await {
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
