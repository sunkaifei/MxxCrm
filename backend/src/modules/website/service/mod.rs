//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

pub mod website_service;
pub mod template_service;
pub mod website_links_service;
pub mod template_category_service;
pub mod template_data_service;
pub mod template_user_data_service;
pub mod website_media_service;
pub mod website_media_category_service;
pub mod template_var_service;
pub mod template_revision_service;
pub mod website_banner_service;
pub mod website_block_service;
pub mod website_page_service;
pub mod image_process_service;

pub mod content_model_service;
pub mod content_model_field_service;
pub mod dynamic_table_service;

pub mod leave_msg_service;
pub mod navigation_service;

// 交易型模块（阶段5-7）
pub mod website_user_service;
pub mod website_cart_service;
pub mod website_order_service;
pub mod website_delivery_service;
pub mod website_refund_service;
pub mod website_notification_config_service;
pub mod static_generate_service;
pub mod content_collector_service;