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
use std::sync::Mutex;
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
pub mod sensitive;
pub mod scheduler;
pub mod install;
pub mod json_util;

pub static CONTEXT: LazyLock<ServiceContext> = LazyLock::new(|| ServiceContext::default());

pub struct ServiceContext {
    pub cache_service: CacheService,
    /// 全局数据库连接（启动时注入，供无法通过请求上下文获取 db 的场景使用）
    pub db: Mutex<Option<sea_orm::DatabaseConnection>>,
}
impl Default for ServiceContext {
    fn default() -> Self {
        ServiceContext {
            cache_service: CacheService::new().unwrap(),
            db: Mutex::new(None),
        }
    }
}

impl ServiceContext {
    /// 启动时注入数据库连接
    pub fn set_db(&self, db: sea_orm::DatabaseConnection) {
        if let Ok(mut guard) = self.db.lock() {
            *guard = Some(db);
        }
    }

    /// 获取数据库连接的克隆（如果已注入）
    pub fn get_db(&self) -> Option<sea_orm::DatabaseConnection> {
        self.db.lock().ok().and_then(|g| g.clone())
    }
}