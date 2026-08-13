use std::collections::HashSet;

use actix_web::dev::ServiceRequest;
use actix_web::{error, web, Error, Result};
use actix_web_grants::GrantsMiddleware;

use crate::core::kit::config;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::kit::global::AppState;
use crate::modules::articles::controller::admin::{article_admin_controller, article_field_admin_controller, category_admin_controller, comment_admin_controller, label_admin_controller};
use crate::modules::search::controller::admin::search_admin_controller;
use crate::modules::statistics::controller::admin::statistics_admin_controller as sys_statistics_admin_controller;
use crate::modules::statistics::controller::admin::performance_plan_controller;
use crate::modules::system::controller::admin::{config_admin_controller, dept_admin_controller, ip_admin_controller, menu_admin_controller, notice_admin_controller, post_admin_controller, region_admin_controller, area_admin_controller, role_admin_controller, system_admin_controller, system_dict_controller, system_log_admin_controller, tag_admin_controller, edit_log_admin_controller, mail_controller, admin_preference_controller, scheduler_controller, pdf_controller, setting_admin_controller, integration_config_controller};
use crate::modules::approval::controller::admin::approval_controller;
use crate::modules::upload::controller::admin::attachment_admin_controller;
use crate::modules::website::controller::admin::{my_template_admin_controller, website_admin_controller, template_admin_controller, template_category_admin_controller, website_links_admin_controller, template_data_admin_controller, website_media_admin_controller, content_model_admin_controller, content_model_field_admin_controller, template_var_admin_controller, template_revision_admin_controller, website_banner_admin_controller, website_block_admin_controller, website_page_admin_controller, leave_msg_admin_controller, navigation_admin_controller, website_user_admin_controller, website_order_admin_controller, website_refund_admin_controller, website_notification_config_admin_controller};
use crate::modules::shop::controller::admin::shop_admin_controller;
use crate::modules::shop::controller::admin::category_controller;
use crate::modules::shop::controller::admin::audit_controller;
use crate::modules::finance::controller::admin::{member_fee_admin_controller, payment_admin_controller, refund_admin_controller, statistics_admin_controller as finance_statistics_admin_controller, commission_rule_controller, salary_controller, payment_controller as finance_payment_controller, expense_controller as finance_expense_controller, tax_controller, insurance_controller, bank_export_controller, payslip_controller, team_commission_controller, attendance_controller, salary_item_controller, salary_adjustment_controller, commission_pool_controller};
use crate::modules::ai::controller::admin::{ai_config_controller, background_check_controller};
use crate::modules::crm::controller::admin::{customer_controller as crm_customer_controller, lead_controller, contact_controller, opportunity_controller, contract_controller, followup_controller, customer_edit_log_controller, todo_controller, visit_controller, work_log_controller};
use crate::modules::product::controller::admin::{product_controller, category_controller as product_category_controller, spec_controller, sku_template_controller, brand_controller, unit_conversion_controller};
use crate::modules::purchase::controller::admin::{purchase_order_controller, supplier_controller, purchase_requisition_controller, purchase_receipt_controller, purchase_return_controller, purchase_stock_plan_controller, purchase_report_controller, supplier_brand_controller, supplier_product_controller};
use crate::modules::production::controller::admin::{production_plan_controller, production_order_controller};
use crate::modules::sale::controller::admin::{invoice_controller, order_controller as sale_order_controller, order_item_controller, payment_controller as sale_payment_controller, quotation_controller, refund_controller, shipment_controller, delivery_controller, card_pool_controller, entitlement_controller, online_payment_controller, logistics_controller, delivery_notification_controller, tax_invoice_controller, exchange_controller, download_link_controller};
use crate::modules::inventory::controller::admin::{warehouse_controller, inventory_controller, inbound_controller, outbound_controller, inventory_report_controller, quality_check_controller, batch_controller, stock_snapshot_controller, inventory_suggestion_controller, stocktake_controller, warehouse_area_controller, transfer_controller, alert_controller, serial_number_controller, bin_location_controller};
use crate::modules::company::controller::admin::company_controller;
use crate::modules::company::controller::admin::code_rule_controller;
use crate::modules::crm::controller::admin::customer_360_controller;
use crate::modules::crm::controller::admin::electronic_signature_controller;
use crate::modules::crm::controller::admin::payment_plan_auto_controller;
use crate::modules::crm::controller::admin::service_ticket_controller;
use crate::modules::statistics::controller::admin::subscription_stats_controller;
use crate::modules::statistics::controller::admin::customer_ltv_controller;
use crate::modules::statistics::controller::admin::abc_analysis_controller;
use crate::modules::system::controller::admin::exchange_rate_controller;
use crate::modules::message::controller::admin::notification_admin_controller;
use crate::modules::message::controller::admin::my_notification_controller;
use crate::modules::message::controller::admin::chat_admin_controller;
use crate::modules::message::websocket;
use crate::modules::system::service::permission_cache_service;

async fn extract(req: &ServiceRequest) -> Result<HashSet<String>, Error> {
    let path = req.path();

    let exclude_urls = config::section::<String>("server", "permission_exclude_urls", "".to_string());
    let exclude_list: Vec<&str> = exclude_urls.split(',').collect();

    if exclude_list.iter().any(|url| path.starts_with(url.trim())) {
        return Ok(HashSet::new());
    }

    let token = req
        .headers()
        .get("Authorization")
        .map(|v| v.to_str().unwrap_or_default().to_string())
        .unwrap_or_default()
        .split("Bearer ")
        .collect::<Vec<&str>>()
        .pop()
        .unwrap_or_default()
        .to_string();
    let jwt_token_e = JWTToken::verify(&config::section::<String>("server", "jwt_secret_admin", "".to_string()), &token);

    match jwt_token_e {
        Ok(jwt_data) => {
            // v2.0: 权限从缓存读取（缓存miss时回查DB），实现权限实时生效
            let user_id = jwt_data.id.unwrap_or_default();
            if user_id > 0 {
                // v1.1: 统一会话校验（单设备精确匹配 / 多设备检查 token 集合）
                // 确保踢人、改密、禁用在两种模式下都能即时生效
                // v1.2: 缓存未命中时降级查 DB session 表（mem 模式重启不丢登录态）
                let db = req.app_data::<web::Data<AppState>>().map(|s| s.db.clone());
                if !permission_cache_service::validate_session_with_db(user_id, &token, db.as_ref()).await {
                    return Err(error::ErrorUnauthorized("登录状态已失效，请重新登录"));
                }

                // 从请求中获取数据库连接
                if let Some(app_state) = req.app_data::<web::Data<AppState>>() {
                    let permissions = permission_cache_service::get_or_load_permissions(&app_state.db, user_id).await;
                    let set: HashSet<String> = permissions.into_iter().collect();
                    Ok(set)
                } else {
                    // 无法获取DB连接，降级为从JWT读取（兼容旧逻辑）
                    log::warn!("[extract] 无法获取AppState, 降级使用JWT内权限, user_id={}", user_id);
                    let set: HashSet<String> = jwt_data.permissions.into_iter().collect();
                    Ok(set)
                }
            } else {
                Err(error::ErrorUnauthorized("无效的用户身份"))
            }
        },
        Err(_err) => {
            Err(error::ErrorUnauthorized("Authorization Not Found"))
        }
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/system")
            .wrap(crate::core::web::demo_guard::DemoGuard::new())
            .wrap(GrantsMiddleware::with_extractor(extract))
            // ============ 所有 controller 通过 register 集中注册 ============
            // System Admin Management（登录、注销、注册、用户管理、权限码）
            .configure(system_admin_controller::register)
            // System Setting Management（系统设置、在线会话管理 v1.1）
            .configure(setting_admin_controller::register)
            // Role Management
            .configure(role_admin_controller::register)
            // Menu Management
            .configure(menu_admin_controller::register)
            // Dict Management
            .configure(system_dict_controller::register)
            // Dept Management
            .configure(dept_admin_controller::register)
            // Post Management
            .configure(post_admin_controller::register)
            // Config Management
            .configure(config_admin_controller::register)
            // Third-party Integration Config Management（第三方接口统一配置中心）
            .configure(integration_config_controller::register)
            // Region Management
            .configure(region_admin_controller::register)
            // Area Management
            .configure(area_admin_controller::register)
            // Website Management
            .configure(website_admin_controller::register)
            // Article Management
            .configure(article_admin_controller::register)
            // Category Management (Article)
            .configure(category_admin_controller::register)
            // IP Address Management
            .configure(ip_admin_controller::register)
            // Attachment Management (含 attachment/category 子路由，注册在 /attachment scope 内)
            .configure(attachment_admin_controller::register)
            // System Log Management
            .configure(system_log_admin_controller::register)
            // Edit Log Management
            .configure(edit_log_admin_controller::register)
            // Template Category Management
            .configure(template_category_admin_controller::register)
            // Template Management (含 template/data 子路由，嵌套在 /template scope 内)
            .configure(template_admin_controller::register)
            // My Template Management
            .configure(my_template_admin_controller::register)
            // Website Links Management
            .configure(website_links_admin_controller::register)
            // Website Media Management (含 media/category 子路由，注册在 /website/media scope 内)
            .configure(website_media_admin_controller::register)
            // CMS Enhancement: Content Model Management
            .configure(content_model_admin_controller::register)
            .configure(content_model_field_admin_controller::register)
            // CMS Enhancement: Template Variables & Revisions
            .configure(template_var_admin_controller::register)
            .configure(template_revision_admin_controller::register)
            // CMS Enhancement: Website Banner/Block/Page Management
            .configure(website_banner_admin_controller::register)
            .configure(website_block_admin_controller::register)
            .configure(website_page_admin_controller::register)
            // CMS: Leave Message Management (留言管理)
            .configure(leave_msg_admin_controller::register)
            // CMS: Navigation Management (导航管理)
            .configure(navigation_admin_controller::register)
            // CMS: Website User Management (前台用户管理)
            .configure(website_user_admin_controller::register)
            // CMS: Website Order/Delivery Management (网站订单/发货管理)
            .configure(website_order_admin_controller::register)
            // CMS: Website Refund Management (网站退款管理)
            .configure(website_refund_admin_controller::register)
            // CMS: Website Notification Config Management (网站通知配置管理)
            .configure(website_notification_config_admin_controller::register)
            // Label Management
            .configure(label_admin_controller::register)
            // Comment Management (文章评论管理)
            .configure(comment_admin_controller::register)
            // Article Custom Field Management (G-2.1: 文章自定义字段管理)
            .configure(article_field_admin_controller::register)
            // Notice Management
            .configure(notice_admin_controller::register)
            // Tag Management
            .configure(tag_admin_controller::register)
            // Search Management
            .configure(search_admin_controller::register)
            // Performance Plan Management (MUST be before sys_statistics_admin_controller:
            // /statistics/performance/plan scope is more specific than /statistics/performance,
            // actix-web matches scopes in registration order, first match wins)
            .configure(performance_plan_controller::register)
            // Data Analysis Statistics Management
            .configure(sys_statistics_admin_controller::register)
            // Shop Management
            .configure(shop_admin_controller::register)
            // Shop Category Management
            .configure(category_controller::register)
            // Audit Management
            .configure(audit_controller::register)
            // Member Fee Management
            .configure(member_fee_admin_controller::register)
            // Payment Record Management
            .configure(payment_admin_controller::register)
            // Refund Record Management
            .configure(refund_admin_controller::register)
            // Finance Statistics Management
            .configure(finance_statistics_admin_controller::register)
            // Commission Rule Management
            .configure(commission_rule_controller::register)
            // Salary Management
            .configure(salary_controller::register)
            // Scheduler Job Management (定时任务管理)
            .configure(scheduler_controller::register)
            // Payment Management
            .configure(finance_payment_controller::register)
            // Finance Expense Management（费用申请）
            .configure(finance_expense_controller::register)
            // Finance Tax Management（个税管理）
            .configure(tax_controller::register)
            // Finance Social Insurance Management（社保公积金）
            .configure(insurance_controller::register)
            // Finance Bank Export Management（银行代发）
            .configure(bank_export_controller::register)
            // Finance Payslip Management（工资条下发）
            .configure(payslip_controller::register)
            // Finance Team Commission（团队提成）
            .configure(team_commission_controller::register)
            // Finance Commission Pool（团建资金池）
            .configure(commission_pool_controller::register)
            // Finance Attendance（考勤扣款）
            .configure(attendance_controller::register)
            // Finance Salary Item（工资项目自定义）
            .configure(salary_item_controller::register)
            // Finance Salary Adjustment（调薪记录）
            .configure(salary_adjustment_controller::register)
            // CRM Customer Management（含 customer_edit_log，注册在 /customer scope 内）
            .configure(crm_customer_controller::register)
            // CRM Lead Management
            .configure(lead_controller::register)
            // CRM Contact Management
            .configure(contact_controller::register)
            // CRM Opportunity Management
            .configure(opportunity_controller::register)
            // CRM Contract Management（含 payment-plan，注册在 /contract scope 内）
            .configure(contract_controller::register)
            // CRM Followup Management
            .configure(followup_controller::register)
            // CRM Visit Management (外勤拜访签到)
            .configure(visit_controller::register)
            // CRM Todo Center
            .configure(todo_controller::register)
            // CRM Work Log
            .configure(work_log_controller::register)
            // AI Config Management —— 已废弃，AI 配置统一到「第三方接口配置」integration_config (category=ai)
            // .configure(ai_config_controller::register)
            // Background Check Management
            .configure(background_check_controller::register)
            // Product Category Management
            .configure(product_category_controller::register)
            // Product Management
            .configure(product_controller::register)
            // Product Spec Management
            .configure(spec_controller::register)
            // SKU Template Management
            .configure(sku_template_controller::register)
            // Inventory Warehouse Management
            .configure(warehouse_controller::register)
            // Inventory Report Management (must be before /inventory to avoid scope prefix conflict)
            .configure(inventory_report_controller::register)
            // Inventory Quality Check Management (must be before /inventory)
            .configure(quality_check_controller::register)
            // Inventory Stocktake Management (must be before /inventory)
            .configure(stocktake_controller::register)
            // Inventory Transfer Management (must be before /inventory)
            .configure(transfer_controller::register)
            // Inventory Alert Rule Management (must be before /inventory which registers /alert)
            .configure(alert_controller::register)
            // Inventory Stock Management
            .configure(inventory_controller::register)
            // Inventory Inbound Management
            .configure(inbound_controller::register)
            // Inventory Outbound Management
            .configure(outbound_controller::register)
            // Inventory Batch Management
            .configure(batch_controller::register)
            // Inventory Stock Snapshot Management
            .configure(stock_snapshot_controller::register)
            // Inventory Suggestion Management
            .configure(inventory_suggestion_controller::register)
            // Inventory Warehouse Area Management
            .configure(warehouse_area_controller::register)
            // Sale Order Management
            .configure(sale_order_controller::register)
            // Sale Order Item Management
            .configure(order_item_controller::register)
            // Sale Refund Management
            .configure(refund_controller::register)
            // Sale Payment Management
            .configure(sale_payment_controller::register)
            // Sale Quotation Management
            .configure(quotation_controller::register)
            // Sale Invoice Management
            .configure(invoice_controller::register)
            // Sale Shipment Management
            .configure(shipment_controller::register)
            // Sale Virtual Product Delivery Management (07-virtual-product)
            .configure(delivery_controller::register)
            // Sale Card Pool Management (07-virtual-product)
            .configure(card_pool_controller::register)
            // Sale Entitlement Management (07-virtual-product)
            .configure(entitlement_controller::register)
            // === 08-CRM能力补齐 P0-P3 ===
            // P0-1: 在线支付
            .configure(online_payment_controller::register)
            // P1-2: 物流追踪
            .configure(logistics_controller::register)
            // P1-5: 交付通知
            .configure(delivery_notification_controller::register)
            // P2-9: 金税开票
            .configure(tax_invoice_controller::register)
            // P2-10: 换货流程
            .configure(exchange_controller::register)
            // P3-16: 下载链接防盗
            .configure(download_link_controller::register)
            // P1-4: 客户360视图
            .configure(customer_360_controller::register)
            // P1-6: 电子签约
            .configure(electronic_signature_controller::register)
            // P2-7: 回款计划自动生成
            .configure(payment_plan_auto_controller::register)
            // P3-15: 服务工单
            .configure(service_ticket_controller::register)
            // P1-3: MRR/ARR 订阅统计
            .configure(subscription_stats_controller::register)
            // P2-11: 客户LTV分析
            .configure(customer_ltv_controller::register)
            // P3-14: ABC分类分析
            .configure(abc_analysis_controller::register)
            // P2-12: 序列号管理
            .configure(serial_number_controller::register)
            // P3-13: 库位管理
            .configure(bin_location_controller::register)
            // P2-8: 汇率管理
            .configure(exchange_rate_controller::register)
            // Purchase Order Management
            .configure(purchase_order_controller::register)
            // Supplier Management
            .configure(supplier_controller::register)
            // Purchase Requisition Management
            .configure(purchase_requisition_controller::register)
            // Purchase Receiving Management
            .configure(purchase_receipt_controller::register)
            // Purchase Return Management
            .configure(purchase_return_controller::register)
            // Purchase Stock Plan Management
            .configure(purchase_stock_plan_controller::register)
            // Purchase Report Management
            .configure(purchase_report_controller::register)
            // Supplier Brand Association
            .configure(supplier_brand_controller::register)
            // Supplier Product Association
            .configure(supplier_product_controller::register)
            // Product Brand Management
            .configure(brand_controller::register)
            // Product Unit Conversion Management
            .configure(unit_conversion_controller::register)
            // Production Plan Management
            .configure(production_plan_controller::register)
            // Production Order Management
            .configure(production_order_controller::register)
            // Approval Flow + Instance Management
            .configure(approval_controller::register)
            // Company Info Management
            .configure(company_controller::register)
            // Company Code Rule Management
            .configure(code_rule_controller::register)
            // Message Notification Management
            .configure(notification_admin_controller::register)
            // My Notification (admin user's own notifications)
            .configure(my_notification_controller::register)
            // Chat Management (admin user's chat)
            .configure(chat_admin_controller::register)
            // Mail Management (邮箱配置/模板/发送/日志)
            .configure(mail_controller::register)
            // Admin Preference Management (快捷导航/仪表盘布局)
            .configure(admin_preference_controller::register)
            // PDF Template & Generator Management (PDF模板管理/生成/下载)
            .configure(pdf_controller::register)
    );

    // logout 路由独立注册（不在 /api/system scope 下，避免前缀冲突）
    cfg.service(
        web::resource("/api/auth/logout").route(web::delete().to(system_admin_controller::logout)),
    );

    // WebSocket 路由（独立 scope，不经过 GrantsMiddleware，握手时通过 query token 认证）
    cfg.service(
        web::resource("/ws/message")
            .route(web::get().to(websocket::ws_handler))
    );
}
