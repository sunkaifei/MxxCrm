//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
use std::sync::LazyLock;
use crate::modules::system::service::cache_service::CacheService;

pub mod elasticsearch;
pub mod template;
pub mod db;
pub mod app;
pub mod redis;
pub mod global;
pub mod config;
pub mod column;
pub mod jwt_util;
pub mod user_auth;

pub static CONTEXT: LazyLock<ServiceContext> = LazyLock::new(|| ServiceContext::default());

pub struct ServiceContext {
    pub cache_service: CacheService,
}
impl Default for ServiceContext {
    fn default() -> Self {
        ServiceContext {
            cache_service: CacheService::new().unwrap(),
        }
    }
}