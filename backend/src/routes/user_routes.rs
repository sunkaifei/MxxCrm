//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{web, Error, Result};
use actix_web::dev::ServiceRequest;

use crate::core::kit::config;
use crate::core::kit::jwt_util::JWTToken;
use crate::modules::message::controller::user::chat_controller;
use crate::modules::finance::controller::user::{payment_user_controller, member_fee_user_controller};
use crate::modules::system::controller::user::region_user_controller;
use crate::modules::shop::controller as shop_controller;

pub async fn user_auth_middleware(req: &ServiceRequest) -> Result<i64, Error> {
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

    let jwt_token = JWTToken::verify(&config::section::<String>("server", "jwt_secret_user", "".to_string()), &token)
        .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid token"))?;

    Ok(jwt_token.id.unwrap_or_default())
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/user")
            .service(payment_user_controller::list)
            .service(payment_user_controller::detail)
            .service(payment_user_controller::create_payment)
            .service(payment_user_controller::create_member_experience_order)
            // Member Fee
            .service(member_fee_user_controller::get_my_member_info)
            .service(member_fee_user_controller::purchase_member)
            // Region
            .service(region_user_controller::get_region_tree)
            // Chat
            .service(chat_controller::send_message_handler)
            .service(chat_controller::get_session_list_handler)
            .service(chat_controller::get_chat_messages_handler)
            .service(chat_controller::mark_read_handler)
            .service(chat_controller::delete_session_handler)
            .service(chat_controller::search_users_handler)
            .service(chat_controller::get_unread_count_handler)
            .service(chat_controller::start_session_handler)
            // Mall - Shop
            .service(
                web::scope("/mall")
                    .service(shop_controller::user::spu_controller::get_spu_list)
                    .service(shop_controller::user::spu_controller::get_spu_detail)
                    .service(shop_controller::user::cart_controller::add_cart)
                    .service(shop_controller::user::cart_controller::get_cart_list)
                    .service(shop_controller::user::cart_controller::update_cart)
                    .service(shop_controller::user::cart_controller::delete_cart)
                    .service(shop_controller::user::order_controller::create_order)
                    .service(shop_controller::user::order_controller::pay_order)
                    .service(shop_controller::user::order_controller::get_order_list)
                    .service(shop_controller::user::order_controller::get_order_detail)
                    .service(shop_controller::user::order_controller::cancel_order)
                    .service(shop_controller::user::order_controller::confirm_receive)
                    .service(shop_controller::user::review_controller::add_review)
                    .service(shop_controller::user::review_controller::get_review_list)
                    .service(shop_controller::user::review_controller::get_review_stats)
                    .service(shop_controller::user::refund_controller::apply_refund)
                    .service(shop_controller::user::refund_controller::get_refund_detail)
                    .service(shop_controller::user::refund_controller::cancel_refund)
                    .service(shop_controller::user::refund_controller::get_refund_list)
            )
    );
}
