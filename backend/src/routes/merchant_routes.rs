//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::web;
use crate::modules::shop::controller as shop_controller;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/merchant")
            .service(shop_controller::merchant::shop_controller::get_shop_info)
            .service(shop_controller::merchant::shop_controller::update_shop)
            .service(shop_controller::merchant::spu_controller::create_spu)
            .service(shop_controller::merchant::spu_controller::update_spu)
            .service(shop_controller::merchant::spu_controller::get_spu_list)
            .service(shop_controller::merchant::spu_controller::get_spu_detail)
            .service(shop_controller::merchant::spu_controller::offline_spu)
            .service(shop_controller::merchant::spu_controller::batch_update_sku_stock)
            .service(shop_controller::merchant::order_controller::get_order_list)
            .service(shop_controller::merchant::order_controller::get_order_detail)
            .service(shop_controller::merchant::order_controller::deliver_order)
            .service(shop_controller::merchant::refund_controller::get_refund_list)
            .service(shop_controller::merchant::refund_controller::get_refund_detail)
            .service(shop_controller::merchant::refund_controller::agree_refund)
            .service(shop_controller::merchant::refund_controller::refuse_refund)
            .service(shop_controller::merchant::settlement_controller::get_settlement_list)
            .service(shop_controller::merchant::settlement_controller::get_settlement_detail)
    );
}