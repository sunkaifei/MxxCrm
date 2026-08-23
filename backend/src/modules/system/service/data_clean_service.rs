//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 数据初始化（一键清除业务链路数据）服务
//!
//! 场景：系统上架前清理测试数据，给企业一个干净的初始环境。
//!
//! 设计要点：
//! 1. 仅清理【业务数据表】，系统核心表（用户/角色/部门/菜单/配置/审批流定义等）一律保留
//! 2. 保留基础配置类数据：SKU/规格模板（sku_template*）、财务配置（税率/社保/工资项）、
//!    内容模型字段定义、站点装修配置（banner/页面/SEO）等，企业上架后无需重复配置
//! 3. 三重防护：仅超管（user_type=1）可执行 + 当前超管登录密码校验（bcrypt）+ 一次性确认码
//! 4. 执行前强制触发一次完整备份（pg_dump），备份失败则中止清除
//! 5. 业务上传目录 storage/upload/ 下的 product/contract/invoice/pdf/... 一并清空，avatar/、common/ 保留
//! 6. 操作记录：审计事件 + 前置备份记录（由 run_backup 写入，operate_type=0 普通备份，可直接用于数据恢复）
//!
//! 清除策略：单条 `TRUNCATE TABLE ... RESTART IDENTITY` 覆盖全部业务表。
//! 经核验，库内全部 51 个外键均为"业务表 → mxx_system_admin"（创建人/负责人），
//! 无任何系统表被业务表引用，因此无需 CASCADE，不会误伤保留表。

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use sea_orm::{ConnectionTrait, DatabaseConnection};

/// 一次性确认码缓存：admin_id -> (确认码, 过期时间)，5 分钟有效
static CONFIRM_CACHE: OnceLock<Mutex<Vec<(i64, String, Instant)>>> = OnceLock::new();

fn confirm_cache() -> &'static Mutex<Vec<(i64, String, Instant)>> {
    CONFIRM_CACHE.get_or_init(|| Mutex::new(Vec::new()))
}

/// 业务数据表清单（按模块分组注释，排除了需保留的基础配置表）
pub static CLEAN_TABLES: &[&str] = &[
    // ===== CRM 客户/商机/合同链路 =====
    "mxx_crm_lead",
    "mxx_crm_customer",
    "mxx_crm_customer_assign_history",
    "mxx_crm_customer_contact_merge",
    "mxx_crm_customer_edit_log",
    "mxx_crm_customer_financial",
    "mxx_crm_contact",
    "mxx_crm_contact_edit_log",
    "mxx_crm_opportunity",
    "mxx_crm_followup",
    "mxx_crm_contract",
    "mxx_crm_contract_approval_log",
    "mxx_crm_contract_commission_member",
    "mxx_crm_contract_payment_plan",
    "mxx_crm_electronic_signature",
    "mxx_crm_service_ticket",
    "mxx_crm_service_ticket_log",
    "mxx_crm_mail_log",
    "mxx_crm_company_background_check",
    "mxx_work_log",
    // ===== 销售：报价/订单/回款/发货/发票 =====
    "mxx_sale_quotation",
    "mxx_sale_quotation_item",
    "mxx_sale_quotation_approval",
    "mxx_sale_order",
    "mxx_sale_order_item",
    "mxx_sale_order_delivery",
    "mxx_sale_payment",
    "mxx_sale_payment_application",
    "mxx_sale_refund",
    "mxx_sale_refund_item",
    "mxx_sale_refund_payment",
    "mxx_sale_shipment",
    "mxx_sale_shipment_item",
    "mxx_sale_invoice",
    "mxx_sale_invoice_edit_log",
    "mxx_sale_invoice_bak_20260817",
    "mxx_sale_tax_invoice",
    "mxx_sale_card_pool",
    "mxx_sale_entitlement",
    "mxx_sale_exchange",
    "mxx_sale_exchange_item",
    "mxx_sale_logistics_tracking",
    "mxx_sale_online_payment",
    // ===== 采购 =====
    "mxx_purchase_supplier",
    "mxx_purchase_supplier_brand",
    "mxx_purchase_supplier_product",
    "mxx_purchase_requisition",
    "mxx_purchase_requisition_item",
    "mxx_purchase_po",
    "mxx_purchase_po_item",
    "mxx_purchase_item",
    "mxx_purchase_receipt",
    "mxx_purchase_receipt_item",
    "mxx_purchase_return",
    "mxx_purchase_return_item",
    "mxx_purchase_stock_plan",
    "mxx_purchase_approval_record",
    // ===== 库存 =====
    "mxx_inventory_warehouse",
    "mxx_inventory_warehouse_area",
    "mxx_inventory_bin_location",
    "mxx_inventory_stock",
    "mxx_inventory_stock_bin",
    "mxx_inventory_stock_freeze",
    "mxx_inventory_stock_log",
    "mxx_inventory_stock_snapshot",
    "mxx_inventory_batch",
    "mxx_inventory_serial_number",
    "mxx_inventory_inbound",
    "mxx_inventory_inbound_item",
    "mxx_inventory_outbound",
    "mxx_inventory_outbound_item",
    "mxx_inventory_transfer",
    "mxx_inventory_transfer_item",
    "mxx_inventory_stocktake",
    "mxx_inventory_stocktake_item",
    "mxx_inventory_quality_check",
    "mxx_inventory_doc_change_log",
    // ===== 产品资料（规格模板 sku_template* 保留）=====
    "mxx_product",
    "mxx_product_brand",
    "mxx_product_category",
    "mxx_product_sku",
    "mxx_product_spec",
    "mxx_product_spec_value",
    // ===== 财务：收支/工资/佣金/考勤 =====
    "mxx_finance_payment",
    "mxx_finance_expense",
    "mxx_finance_expense_item",
    "mxx_finance_payslip",
    "mxx_finance_salary_record",
    "mxx_finance_salary_item",
    "mxx_finance_salary_item_value",
    "mxx_finance_salary_adjustment",
    "mxx_finance_salary_confirm",
    "mxx_finance_salary_calc_log",
    "mxx_finance_salary_tax_detail",
    "mxx_finance_commission_allocation",
    "mxx_finance_commission_detail",
    "mxx_finance_commission_pool",
    "mxx_finance_commission_pool_log",
    "mxx_finance_commission_result",
    "mxx_finance_commission_rule",
    "mxx_finance_commission_rule_member",
    "mxx_finance_commission_tier",
    "mxx_finance_attendance_record",
    "mxx_finance_bank_payment_file",
    "mxx_finance_statistics",
    // ===== 财务杂项 / 人事 =====
    "mxx_member_fee",
    "mxx_payment_record",
    "mxx_refund_record",
    "mxx_sms_verification",
    "mxx_hr_resume",
    "mxx_hr_emergency_contact",
    "mxx_hr_profile_log",
    // 员工社保/个税配置（按员工粒度，属业务数据；全局税率/社保政策/工资项配置保留）
    "mxx_finance_employee_insurance_config",
    "mxx_finance_employee_tax_config",
    "mxx_system_email",
    // ===== 审批实例（流程定义 approval_flow* 保留）=====
    "mxx_system_approval_instance",
    "mxx_system_approval_log",
    "mxx_system_approval_cc",
    // ===== CMS 文章 / 模板 / 内容模型（定义保留）=====
    "mxx_article",
    "mxx_article_revision",
    "mxx_article_field_value",
    "mxx_article_label_merge",
    "mxx_article_tag_merge",
    "mxx_template_data",
    "mxx_template_user_data",
    "mxx_content_model_data",
    // ===== 消息 / 聊天 / 通知 =====
    "mxx_chat_message",
    "mxx_chat_session",
    "mxx_chat_session_member",
    "mxx_system_notification",
    "mxx_user_online",
    // ===== 统计缓存 =====
    "mxx_statistics_access_record",
    "mxx_statistics_agg_batch",
    "mxx_statistics_daily_contract",
    "mxx_statistics_daily_customer",
    "mxx_statistics_daily_employee",
    "mxx_statistics_daily_payment",
    "mxx_statistics_performance_plan",
    "mxx_statistics_performance_target",
    "mxx_statistics_plan_approval_log",
    "mxx_statistics_plan_approval_node",
    "mxx_statistics_plan_monthly_target",
    "mxx_statistics_source",
    // ===== 商城/官网：交易/会员/商品（站点配置保留）=====
    "mxx_website_member",
    "mxx_website_member_address",
    "mxx_website_member_wallet",
    "mxx_website_member_transaction_log",
    "mxx_website_user",
    "mxx_website_order",
    "mxx_website_order_item",
    "mxx_website_cart",
    "mxx_website_cart_item",
    "mxx_website_refund",
    "mxx_website_delivery",
    "mxx_website_favorite",
    "mxx_website_comment",
    "mxx_website_leave_msg",
    "mxx_website_form_submission",
    "mxx_website_visit_log",
    "mxx_website_search_log",
    "mxx_website_ai_chat_log",
    "mxx_website_product",
    "mxx_website_product_sku",
    "mxx_website_product_category",
    // ===== 附件台账 / 编辑日志 / PDF 产物（业务附件记录与生成物）=====
    "mxx_attachment",
    "mxx_system_edit_log",
    "mxx_system_pdf_record",
    "mxx_system_pdf_download_log",
];

/// 预览分组（前端按组展示）
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanGroupVO {
    pub name: String,
    pub tables: Vec<CleanTableVO>,
    pub rows: i64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanTableVO {
    pub table_name: String,
    pub rows: i64,
}

/// 预览结果：表清单 + 行数 + 一次性确认码
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanPreviewVO {
    pub groups: Vec<CleanGroupVO>,
    pub total_tables: usize,
    pub total_rows: i64,
    /// 一次性确认码，5 分钟有效，执行时必须回传
    pub confirm_code: String,
}

/// 分组：表名前缀 -> 中文名称
fn group_name(table: &str) -> (&'static str, String) {
    let module = if table.starts_with("mxx_crm_") || table == "mxx_work_log" {
        "客户关系"
    } else if table.starts_with("mxx_sale_") {
        "销售管理"
    } else if table.starts_with("mxx_purchase_") {
        "采购管理"
    } else if table.starts_with("mxx_inventory_") {
        "库存管理"
    } else if table.starts_with("mxx_product_") {
        "产品资料"
    } else if table.starts_with("mxx_finance_") || matches!(table, "mxx_member_fee" | "mxx_payment_record" | "mxx_refund_record" | "mxx_sms_verification") {
        "财务管理"
    } else if table.starts_with("mxx_hr_") {
        "人事档案"
    } else if table.starts_with("mxx_system_approval_") {
        "审批实例"
    } else if table.starts_with("mxx_article_") || table.starts_with("mxx_template") || table.starts_with("mxx_content_model") {
        "内容管理"
    } else if table.starts_with("mxx_chat_") || table == "mxx_system_notification" || table == "mxx_user_online" {
        "消息通知"
    } else if table.starts_with("mxx_statistics_") {
        "统计缓存"
    } else if table.starts_with("mxx_website_") {
        "商城业务"
    } else {
        "其他"
    };
    (module, table.to_string())
}

/// 生成 6 位随机确认码
fn gen_confirm_code() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    // 简单线性同余，生成 6 位数字
    let mut x = seed ^ (seed >> 21) ^ (seed >> 11) ^ (seed >> 32);
    x = x ^ (x << 13);
    x ^= x >> 17;
    x ^= x << 5;
    format!("{:06}", x % 1_000_000)
}

/// 清理过期的确认码缓存
fn prune_cache() {
    let now = Instant::now();
    if let Ok(mut guard) = confirm_cache().lock() {
        guard.retain(|(_, _, at)| now.duration_since(*at) < Duration::from_secs(300));
    }
}

/// 预览：统计各业务表行数并生成确认码
pub async fn preview(db: &DatabaseConnection, admin_id: i64) -> Result<CleanPreviewVO, String> {
    prune_cache();

    // 构建表名集合（过滤数据库中不存在的表，兼容实体已有但表未建的模块）
    let rows = db
        .query_all_raw(sea_orm::Statement::from_string(
            sea_orm::DbBackend::Postgres,
            "SELECT tablename FROM pg_tables WHERE schemaname = 'public'".to_string(),
        ))
        .await
        .map_err(|e| format!("查询数据表清单失败: {}", e))?;
    let mut exist: HashSet<String> = HashSet::new();
    for row in rows {
        if let Some(name) = row.try_get_by_index::<String>(0).ok() {
            exist.insert(name);
        }
    }
    let tables: Vec<&str> = CLEAN_TABLES
        .iter()
        .copied()
        .filter(|t| exist.contains(*t))
        .collect();
    if tables.is_empty() {
        return Err("未匹配到任何业务数据表，已中止".to_string());
    }

    // 精确统计行数（单条 SQL 批量 count）
    let mut total_rows: i64 = 0;
    let mut group_map: Vec<(String, Vec<CleanTableVO>)> = Vec::new();
    for table in &tables {
        let cnt_sql = format!("SELECT count(*)::bigint FROM \"{}\"", table);
        let cnt = db
            .query_one_raw(sea_orm::Statement::from_string(sea_orm::DbBackend::Postgres, cnt_sql))
            .await
            .map_err(|e| format!("统计表 {} 行数失败: {}", table, e))?
            .and_then(|r| r.try_get_by_index::<i64>(0).ok())
            .unwrap_or(0);
        total_rows += cnt;
        let (name, _) = group_name(table);
        match group_map.iter_mut().find(|(n, _)| n == name) {
            Some((_, list)) => list.push(CleanTableVO {
                table_name: table.to_string(),
                rows: cnt,
            }),
            None => group_map.push((
                name.to_string(),
                vec![CleanTableVO {
                    table_name: table.to_string(),
                    rows: cnt,
                }],
            )),
        }
    }

    let groups: Vec<CleanGroupVO> = group_map
        .into_iter()
        .map(|(name, tables)| {
            let rows: i64 = tables.iter().map(|t| t.rows).sum();
            CleanGroupVO { name, tables, rows }
        })
        .collect();

    // 生成确认码并绑定到当前超管（5 分钟有效）
    let confirm_code = gen_confirm_code();
    bind_confirm_code(admin_id, &confirm_code);

    Ok(CleanPreviewVO {
        groups,
        total_tables: tables.len(),
        total_rows,
        confirm_code,
    })
}

/// 绑定确认码到指定超管（controller 调用）
pub fn bind_confirm_code(admin_id: i64, code: &str) {
    prune_cache();
    if let Ok(mut guard) = confirm_cache().lock() {
        guard.retain(|(id, _, _)| *id != admin_id);
        guard.push((admin_id, code.to_string(), Instant::now()));
    }
}

/// 校验确认码（一次性：校验成功后即失效）
fn take_confirm_code(admin_id: i64, code: &str) -> bool {
    prune_cache();
    let mut ok = false;
    if let Ok(mut guard) = confirm_cache().lock() {
        let now = Instant::now();
        let idx = guard
            .iter()
            .position(|(id, c, at)| *id == admin_id && c == code && now.duration_since(*at) < Duration::from_secs(300));
        if let Some(i) = idx {
            guard.remove(i);
            ok = true;
        }
    }
    ok
}

/// 数据初始化执行结果
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanExecuteVO {
    pub cleared_tables: usize,
    pub cleared_rows: i64,
    pub removed_files: usize,
    pub backup_message: String,
}

/// 清空业务上传目录：遍历 storage/upload/ 根目录，仅保留 avatar/（用户头像）与 common/（公共图片），
/// 其余子目录（product/contract/invoice/quotation/payment/pdf/...）全部删除，避免业务附件与生成物残留
fn clear_business_upload() -> Result<usize, String> {
    let upload_root = crate::core::kit::config::section::<String>(
        "attach",
        "upload_path",
        "./storage/upload/".to_string(),
    );
    let path = std::path::Path::new(&upload_root);
    if !path.exists() {
        return Ok(0);
    }
    let mut removed = 0usize;
    let entries = std::fs::read_dir(path).map_err(|e| format!("读取上传目录失败: {}", e))?;
    for entry in entries.flatten() {
        let p = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "avatar" || name == "common" {
            continue; // 保留：用户头像、公共图片
        }
        if p.is_file() {
            if std::fs::remove_file(&p).is_ok() {
                removed += 1;
            }
        } else if p.is_dir() {
            if std::fs::remove_dir_all(&p).is_ok() {
                removed += 1;
            }
        }
    }
    Ok(removed)
}

/// 统计待清业务表当前总行数（TRUNCATE 前调用，用于返回真实清除行数）
async fn count_rows(db: &DatabaseConnection, tables: &[&str]) -> Result<i64, String> {
    let mut total: i64 = 0;
    for table in tables {
        let sql = format!("SELECT count(*)::bigint FROM \"{}\"", table);
        match db
            .query_one_raw(sea_orm::Statement::from_string(sea_orm::DbBackend::Postgres, sql))
            .await
        {
            Ok(Some(row)) => total += row.try_get_by_index::<i64>(0).unwrap_or(0),
            _ => { /* 单表统计失败不影响整体 */ }
        }
    }
    Ok(total)
}

/// 执行数据初始化（危险操作，需前置校验通过后才调用）
///
/// 顺序：前置备份 → TRUNCATE 业务表 → 清空业务上传目录 → 清统计缓存 → 返回结果
pub async fn execute(
    db: &DatabaseConnection,
    admin_id: i64,
    confirm_code: &str,
) -> Result<CleanExecuteVO, String> {
    // 1) 校验确认码（一次性）
    if !take_confirm_code(admin_id, confirm_code) {
        return Err("确认码不正确或已过期，请重新获取确认码".to_string());
    }

    // 2) 前置强制备份（备份失败则中止，绝不无备份清除）
    let backup_message = match crate::modules::system::service::backup_service::run_backup(db).await {
        Ok(msg) => msg,
        Err(e) => {
            log::error!("[数据初始化] 前置备份失败，已中止清除: {}", e);
            return Err(format!("前置备份失败，已中止清除（请检查备份配置后重试）: {}", e));
        }
    };

    // 过滤出实际存在的表，构建 TRUNCATE
    let rows = db
        .query_all_raw(sea_orm::Statement::from_string(
            sea_orm::DbBackend::Postgres,
            "SELECT tablename FROM pg_tables WHERE schemaname = 'public'".to_string(),
        ))
        .await
        .map_err(|e| format!("查询数据表清单失败: {}", e))?;
    let mut exist: HashSet<String> = HashSet::new();
    for row in rows {
        if let Some(name) = row.try_get_by_index::<String>(0).ok() {
            exist.insert(name);
        }
    }
    let tables: Vec<&str> = CLEAN_TABLES
        .iter()
        .copied()
        .filter(|t| exist.contains(*t))
        .collect();
    if tables.is_empty() {
        return Err("未匹配到任何业务数据表，已中止".to_string());
    }

    let table_sql = tables
        .iter()
        .map(|t| format!("\"{}\"", t))
        .collect::<Vec<_>>()
        .join(", ");
    let truncate_sql = format!("TRUNCATE TABLE {} RESTART IDENTITY", table_sql);
    // 3) TRUNCATE 前统计真实行数（供结果与审计展示，避免"表数量当行数"的语义错误）
    let cleared_rows = count_rows(db, &tables).await?;
    db.execute_unprepared(&truncate_sql)
        .await
        .map_err(|e| format!("业务数据清理失败: {}", e))?;

    // 4) 清空业务上传目录（avatar/common 保留）
    let removed_files = clear_business_upload().unwrap_or(0);

    // 5) 清除统计缓存，避免工作台/报表展示清除前的旧数据
    crate::modules::statistics::service::stats_cache::invalidate_all_stats_cache().await;

    log::warn!(
        "[数据初始化] 管理员(id={}) 已清除 {} 张业务表（{} 行记录），并清空业务上传目录（{} 个文件/目录）",
        admin_id,
        tables.len(),
        cleared_rows,
        removed_files
    );

    Ok(CleanExecuteVO {
        cleared_tables: tables.len(),
        cleared_rows,
        removed_files,
        backup_message,
    })
}
