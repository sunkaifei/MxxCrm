use std::collections::HashSet;

use actix_web::dev::ServiceRequest;
use actix_web::{error, web, Error, Result};
use actix_web_grants::GrantsMiddleware;

use crate::core::kit::config;
use crate::core::kit::jwt_util::JWTToken;
use crate::modules::articles::controller::admin::{article_admin_controller, category_admin_controller, label_admin_controller};
use crate::modules::search::controller::admin::search_admin_controller;
use crate::modules::statistics::controller::admin::statistics_admin_controller as sys_statistics_admin_controller;
use crate::modules::statistics::controller::admin::performance_plan_controller;
use crate::modules::system::controller::admin::{config_admin_controller, dept_admin_controller, ip_admin_controller, menu_admin_controller, notice_admin_controller, post_admin_controller, region_admin_controller, area_admin_controller, role_admin_controller, system_admin_controller, system_dict_controller, system_log_admin_controller, tag_admin_controller};
use crate::modules::upload::controller::admin::{attachment_admin_controller, attachment_category_admin_controller};
use crate::modules::website::controller::admin::{my_template_admin_controller, website_admin_controller, template_admin_controller, template_category_admin_controller, website_links_admin_controller, template_data_admin_controller};
use crate::modules::shop::controller::admin::shop_admin_controller;
use crate::modules::shop::controller::admin::category_controller;
use crate::modules::shop::controller::admin::audit_controller;
use crate::modules::finance::controller::admin::{member_fee_admin_controller, payment_admin_controller, refund_admin_controller, statistics_admin_controller as finance_statistics_admin_controller};
use crate::modules::crm::controller::admin::{customer_controller as crm_customer_controller, lead_controller, contact_controller, opportunity_controller, contract_controller, followup_controller};
use crate::modules::product::controller::admin::{product_controller, category_controller as product_category_controller, spec_controller, sku_template_controller};
use crate::modules::purchase::controller::admin::{purchase_order_controller, supplier_controller};
use crate::modules::sale::controller::admin::{invoice_controller, order_controller as sale_order_controller, order_item_controller, payment_controller as sale_payment_controller, quotation_controller};
use crate::modules::inventory::controller::admin::{warehouse_controller, inventory_controller};

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
        Ok(data) => {
            let set: HashSet<String> = data.permissions.into_iter().collect();
            Ok(set)
        },
        Err(_err) => {
            Err(error::ErrorUnauthorized("Authorization Not Found"))
        }
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(system_admin_controller::post_login)
        .service(system_admin_controller::logout)
        .service(
            web::scope("/api/system")
                .wrap(GrantsMiddleware::with_extractor(extract))
                // System Admin Management
                .service(system_admin_controller::save_admin)
                .service(system_admin_controller::admin_batch_delete)
                .service(system_admin_controller::admin_soft_delete)
                .service(system_admin_controller::update_user_role)
                .service(system_admin_controller::admin_update)
                .service(system_admin_controller::update_password)
                .service(system_admin_controller::update_my_password)
                .service(system_admin_controller::update_admin_status)
                .service(system_admin_controller::get_user_info)
                .service(system_admin_controller::get_user_detail)
                .service(system_admin_controller::admin_list)
                .service(system_admin_controller::get_auth_codes)
                // Role Management
                .service(role_admin_controller::role_insert)
                .service(role_admin_controller::update_role_menus)
                .service(role_admin_controller::bath_delete_role)
                .service(role_admin_controller::update_role)
                .service(role_admin_controller::get_role_menu_list_by_role_id)
                .service(role_admin_controller::role_options)
                .service(role_admin_controller::role_list)
                .service(role_admin_controller::get_by_detail)
                // Menu Management
                .service(menu_admin_controller::menu_list)
                .service(menu_admin_controller::add_menu)
                .service(menu_admin_controller::menu_delete)
                .service(menu_admin_controller::menu_update)
                .service(menu_admin_controller::menu_detail)
                .service(menu_admin_controller::get_menu_options)
                .service(menu_admin_controller::get_user_menu)
                // Dict Management
                .service(system_dict_controller::save_dict)
                .service(system_dict_controller::batch_delete)
                .service(system_dict_controller::update_dict)
                .service(system_dict_controller::get_dict_page)
                .service(system_dict_controller::get_dict_detail)
                .service(system_dict_controller::save_dict_data)
                .service(system_dict_controller::batch_delete_data)
                .service(system_dict_controller::update_dict_data)
                .service(system_dict_controller::get_dict_data_list_by_code)
                .service(system_dict_controller::get_dict_data_list)
                .service(system_dict_controller::get_dict_data_detail)
                // Dept Management
                .service(dept_admin_controller::save_dept)
                .service(dept_admin_controller::dept_update)
                .service(dept_admin_controller::get_dept_tree)
                .service(dept_admin_controller::get_dept_options)
                .service(dept_admin_controller::get_by_detail)
                .service(dept_admin_controller::dept_list)
                .service(dept_admin_controller::dept_batch_delete)
                // Post Management
                .service(post_admin_controller::save_post)
                .service(post_admin_controller::bath_delete_post)
                .service(post_admin_controller::update_post)
                .service(post_admin_controller::get_by_detail)
                .service(post_admin_controller::get_by_page)
                // Config Management
                .service(config_admin_controller::insert_config)
                .service(config_admin_controller::update_config)
                .service(config_admin_controller::batch_delete)
                .service(config_admin_controller::get_by_page)
                .service(config_admin_controller::get_by_detail)
                // Region Management
                .service(region_admin_controller::save_region)
                .service(region_admin_controller::batch_delete)
                .service(region_admin_controller::update_by_id)
                .service(region_admin_controller::get_detail)
                .service(region_admin_controller::get_region_tree)
                // Area Management
                .service(area_admin_controller::get_area_tree)
                .service(area_admin_controller::get_cascader_data)
                .service(area_admin_controller::get_countries)
                .service(area_admin_controller::get_provinces)
                .service(area_admin_controller::get_children)
                .service(area_admin_controller::get_detail)
                .service(area_admin_controller::get_by_page)
                .service(area_admin_controller::insert)
                .service(area_admin_controller::update)
                .service(area_admin_controller::batch_delete)
                // Website Management
                .service(website_admin_controller::add_site)
                .service(website_admin_controller::batch_delete)
                .service(website_admin_controller::update_site)
                .service(website_admin_controller::update_by_status)
                .service(website_admin_controller::update_by_default)
                .service(website_admin_controller::get_by_detail)
                .service(website_admin_controller::get_by_page)
                // Article Management
                .service(article_admin_controller::save_article)
                .service(article_admin_controller::batch_delete)
                .service(article_admin_controller::update_article_detail)
                .service(article_admin_controller::get_article_detail)
                .service(article_admin_controller::get_article_list)
                // Category Management (Article)
                .service(category_admin_controller::save_category)
                .service(category_admin_controller::batch_delete)
                .service(category_admin_controller::update_category)
                .service(category_admin_controller::category_list_tree)
                .service(category_admin_controller::category_option)
                .service(category_admin_controller::get_category_detail)
                // IP Address Management
                .service(ip_admin_controller::ip_address_page)
                // Attachment Management
                .service(attachment_admin_controller::upload_attachment)
                .service(attachment_admin_controller::delete_attachment)
                .service(attachment_admin_controller::batch_move)
                .service(attachment_admin_controller::update)
                .service(attachment_admin_controller::get_detail)
                .service(attachment_admin_controller::get_page_list)
                .service(attachment_category_admin_controller::save_category)
                .service(attachment_category_admin_controller::batch_delete_by_ids)
                .service(attachment_category_admin_controller::update_category)
                .service(attachment_category_admin_controller::get_by_tree)
                .service(attachment_category_admin_controller::get_by_detail)
                .service(attachment_category_admin_controller::get_by_list)
                // System Log Management
                .service(system_log_admin_controller::get_by_page)
                // Template Category Management
                .service(template_category_admin_controller::add)
                .service(template_category_admin_controller::batch_delete)
                .service(template_category_admin_controller::update_by_id)
                .service(template_category_admin_controller::get_by_detail)
                .service(template_category_admin_controller::get_by_options)
                .service(template_category_admin_controller::select_by_parent)
                .service(template_category_admin_controller::get_by_list)
                // Template Management
                .service(template_admin_controller::add)
                .service(template_admin_controller::batch_delete)
                .service(template_admin_controller::update_by_id)
                .service(template_admin_controller::get_by_detail)
                .service(template_admin_controller::get_by_options)
                .service(template_admin_controller::get_by_page)
                // Template Data Management
                .service(template_data_admin_controller::add)
                .service(template_data_admin_controller::batch_delete)
                .service(template_data_admin_controller::update_by_id)
                .service(template_data_admin_controller::get_by_detail)
                .service(template_data_admin_controller::get_by_page)
                // My Template Management
                .service(my_template_admin_controller::add)
                .service(my_template_admin_controller::batch_delete)
                .service(my_template_admin_controller::update_by_id)
                .service(my_template_admin_controller::get_by_tree)
                .service(my_template_admin_controller::get_by_detail)
                .service(my_template_admin_controller::get_by_page)
                .service(my_template_admin_controller::get_buy_by_page)
                // Website Links Management
                .service(website_links_admin_controller::add_links)
                .service(website_links_admin_controller::batch_delete)
                .service(website_links_admin_controller::update_by_id)
                .service(website_links_admin_controller::get_by_detail)
                .service(website_links_admin_controller::get_by_page)
                // Label Management
                .service(label_admin_controller::add)
                .service(label_admin_controller::batch_delete)
                .service(label_admin_controller::update_by_id)
                .service(label_admin_controller::get_by_detail)
                // Notice Management
                .service(notice_admin_controller::add_notice)
                .service(notice_admin_controller::batch_delete)
                .service(notice_admin_controller::update_by_id)
                .service(notice_admin_controller::revoke_notice)
                .service(notice_admin_controller::publish_notice)
                .service(notice_admin_controller::user_read_all)
                .service(notice_admin_controller::get_by_detail)
                .service(notice_admin_controller::get_by_user_detail)
                .service(notice_admin_controller::get_by_my_page)
                .service(notice_admin_controller::get_by_page)
                .service(label_admin_controller::get_by_page)
                // Tag Management
                .service(tag_admin_controller::save_tag)
                .service(tag_admin_controller::update_tag)
                .service(tag_admin_controller::delete_tag)
                .service(tag_admin_controller::batch_delete_tag)
                .service(tag_admin_controller::get_tag_detail)
                .service(tag_admin_controller::get_tag_list)
                .service(tag_admin_controller::get_tag_statistics)
                .service(tag_admin_controller::add_tags_to_entity)
                .service(tag_admin_controller::remove_tags_from_entity)
                .service(tag_admin_controller::get_tags_by_entity)
                .service(tag_admin_controller::batch_tag_entity)
                .service(tag_admin_controller::get_all_tags)
                .service(tag_admin_controller::get_tag_group_list)
                // Search Management
                .service(search_admin_controller::create_index)
                .service(search_admin_controller::delete_index)
                // Data Analysis Statistics Management
                .service(sys_statistics_admin_controller::get_performance_target)
                .service(sys_statistics_admin_controller::save_performance_target)
                .service(sys_statistics_admin_controller::get_monthly_performance)
                .service(sys_statistics_admin_controller::get_performance_ranking)
                .service(sys_statistics_admin_controller::get_customer_type_stats)
                .service(sys_statistics_admin_controller::get_customer_source_stats)
                .service(sys_statistics_admin_controller::get_customer_industry_stats)
                .service(sys_statistics_admin_controller::get_customer_funnel)
                .service(sys_statistics_admin_controller::get_employee_customer_count)
                .service(sys_statistics_admin_controller::get_employee_follow_up)
                .service(sys_statistics_admin_controller::get_employee_conversion)
                .service(sys_statistics_admin_controller::get_contract_ranking)
                .service(sys_statistics_admin_controller::get_contract_type_distribution)
                .service(sys_statistics_admin_controller::get_contract_status_analysis)
                .service(sys_statistics_admin_controller::get_payment_completion)
                .service(sys_statistics_admin_controller::get_payment_monthly_trend)
                .service(sys_statistics_admin_controller::get_payment_status_analysis)
                .service(sys_statistics_admin_controller::get_payment_ranking)
                // Performance Plan Management
                .service(performance_plan_controller::create_plan)
                .service(performance_plan_controller::submit_plan)
                .service(performance_plan_controller::approve_plan)
                .service(performance_plan_controller::reject_plan)
                .service(performance_plan_controller::modify_plan)
                .service(performance_plan_controller::get_plan_list)
                .service(performance_plan_controller::get_plan_detail)
                .service(performance_plan_controller::get_plan_modify_detail)
                // Shop Management
                .service(shop_admin_controller::save_shop)
                .service(shop_admin_controller::batch_delete_shop)
                .service(shop_admin_controller::update_shop)
                .service(shop_admin_controller::get_shop_detail)
                .service(shop_admin_controller::get_shop_list)
                // Shop Category Management
                .service(category_controller::save)
                .service(category_controller::update)
                .service(category_controller::delete)
                .service(category_controller::tree)
                // Audit Management
                .service(audit_controller::audit_apply)
                .service(audit_controller::audit_spu)
                // Member Fee Management
                .service(member_fee_admin_controller::list)
                .service(member_fee_admin_controller::detail)
                .service(member_fee_admin_controller::create)
                .service(member_fee_admin_controller::update)
                .service(member_fee_admin_controller::delete)
                // Payment Record Management
                .service(payment_admin_controller::list)
                .service(payment_admin_controller::detail)
                .service(payment_admin_controller::create)
                .service(payment_admin_controller::update)
                .service(payment_admin_controller::delete)
                // Refund Record Management
                .service(refund_admin_controller::list)
                .service(refund_admin_controller::detail)
                .service(refund_admin_controller::create)
                .service(refund_admin_controller::update)
                .service(refund_admin_controller::delete)
                // Finance Statistics Management
                .service(finance_statistics_admin_controller::summary)
                .service(finance_statistics_admin_controller::list)
                .service(finance_statistics_admin_controller::generate_daily)
                // CRM Customer Management
                .service(crm_customer_controller::customer_insert)
                .service(crm_customer_controller::customer_update)
                .service(crm_customer_controller::bath_delete_customer)
                .service(crm_customer_controller::customer_info)
                .service(crm_customer_controller::customer_list)
                .service(crm_customer_controller::customer_contacts)
                // CRM Lead Management
                .service(lead_controller::lead_insert)
                .service(lead_controller::lead_update)
                .service(lead_controller::bath_delete_lead)
                .service(lead_controller::lead_info)
                .service(lead_controller::lead_list)
                .service(lead_controller::lead_pool_list)
                .service(lead_controller::lead_pool_info)
                .service(lead_controller::bath_delete_lead_pool)
                .service(lead_controller::lead_update_status)
                .service(lead_controller::lead_add_to_pool)
                .service(lead_controller::lead_claim)
                // CRM Contact Management
                .service(contact_controller::contact_insert)
                .service(contact_controller::contact_update)
                .service(contact_controller::bath_delete_contact)
                .service(contact_controller::contact_info)
                .service(contact_controller::contact_list)
                .service(contact_controller::contact_bind)
                .service(contact_controller::contact_unbind)
                .service(contact_controller::contact_set_role)
                // CRM Opportunity Management
                .service(opportunity_controller::opportunity_insert)
                .service(opportunity_controller::opportunity_update)
                .service(opportunity_controller::bath_delete_opportunity)
                .service(opportunity_controller::opportunity_info)
                .service(opportunity_controller::opportunity_list)
                // CRM Contract Management
                .service(contract_controller::contract_insert)
                .service(contract_controller::contract_update)
                .service(contract_controller::bath_delete_contract)
                .service(contract_controller::contract_info)
                .service(contract_controller::contract_list)
                // CRM Followup Management
                .service(followup_controller::followup_insert)
                .service(followup_controller::followup_update)
                .service(followup_controller::bath_delete_followup)
                .service(followup_controller::followup_info)
                .service(followup_controller::followup_list)
                // Product Category Management
                .service(product_category_controller::category_insert)
                .service(product_category_controller::category_update)
                .service(product_category_controller::batch_delete_category)
                .service(product_category_controller::info_category)
                .service(product_category_controller::list_category)
                // Product Management
                .service(product_controller::product_insert)
                .service(product_controller::product_update)
                .service(product_controller::batch_delete_product)
                .service(product_controller::product_info)
                .service(product_controller::product_list)
                // Product Spec Management
                .service(spec_controller::get_product_specs)
                .service(spec_controller::save_product_specs)
                .service(spec_controller::generate_skus)
                .service(spec_controller::batch_save_skus)
                // SKU Template Management
                .service(sku_template_controller::template_list)
                .service(sku_template_controller::template_info)
                .service(sku_template_controller::template_save)
                .service(sku_template_controller::template_update)
                .service(sku_template_controller::template_delete)
                .service(sku_template_controller::template_spec_save)
                // Inventory Warehouse Management
                .service(warehouse_controller::warehouse_insert)
                .service(warehouse_controller::warehouse_update)
                .service(warehouse_controller::batch_delete_warehouse)
                .service(warehouse_controller::warehouse_info)
                .service(warehouse_controller::warehouse_list)
                // Inventory Stock Management
                .service(inventory_controller::inventory_list)
                .service(inventory_controller::inventory_info)
                // Sale Order Management
                .service(sale_order_controller::order_insert)
                .service(sale_order_controller::order_update)
                .service(sale_order_controller::batch_delete_order)
                .service(sale_order_controller::order_info)
                .service(sale_order_controller::order_list)
                // Sale Order Item Management
                .service(order_item_controller::order_item_insert)
                .service(order_item_controller::order_item_update)
                .service(order_item_controller::bath_delete_order_item)
                .service(order_item_controller::order_item_info)
                .service(order_item_controller::order_item_list)
                // Sale Payment Management
                .service(sale_payment_controller::payment_insert)
                .service(sale_payment_controller::payment_update)
                .service(sale_payment_controller::bath_delete_payment)
                .service(sale_payment_controller::payment_info)
                .service(sale_payment_controller::payment_list)
                // Sale Quotation Management
                .service(quotation_controller::quotation_insert)
                .service(quotation_controller::quotation_update)
                .service(quotation_controller::bath_delete_quotation)
                .service(quotation_controller::quotation_info)
                .service(quotation_controller::quotation_list)
                .service(quotation_controller::quotation_send)
                .service(quotation_controller::quotation_accept)
                .service(quotation_controller::quotation_reject)
                // Sale Invoice Management
                .service(invoice_controller::invoice_insert)
                .service(invoice_controller::invoice_update)
                .service(invoice_controller::bath_delete_invoice)
                .service(invoice_controller::invoice_info)
                .service(invoice_controller::invoice_list)
                // Purchase Order Management
                .service(purchase_order_controller::purchase_order_insert)
                .service(purchase_order_controller::purchase_order_update)
                .service(purchase_order_controller::batch_delete_purchase_order)
                .service(purchase_order_controller::purchase_order_info)
                .service(purchase_order_controller::purchase_order_list)
                // Supplier Management
                .service(supplier_controller::supplier_insert)
                .service(supplier_controller::supplier_update)
                .service(supplier_controller::bath_delete_supplier)
                .service(supplier_controller::supplier_info)
                .service(supplier_controller::supplier_list)
        );
}