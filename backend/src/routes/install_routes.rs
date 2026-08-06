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

use crate::modules::install::controller;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/install")
            .route("/license", web::get().to(controller::get_license))
            .route("/license-accept", web::post().to(controller::license_accept))
            .route("/env-check", web::get().to(controller::env_check))
            .route("/status", web::get().to(controller::status))
            .route("/test-connection", web::post().to(controller::test_connection))
            .route("/create-database", web::post().to(controller::create_database))
            .route("/import", web::post().to(controller::import_database))
            .route("/import-progress", web::get().to(controller::import_progress))
            .route("/complete", web::post().to(controller::complete_install)),
    );
}