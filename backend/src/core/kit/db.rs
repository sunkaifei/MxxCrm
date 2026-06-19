//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use std::time::Duration;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use crate::core::kit::{app,config};
pub use sea_orm;

// 数据库连接类型别名
pub type DbConn = DatabaseConnection;


// 数据库连接
pub async fn connect() -> Result<DatabaseConnection, DbErr> {
    let db_url = config::section::<String>("db", "url", "".to_string());

    let max_connections = config::section::<u32>("db", "max_connections", 100);
    let min_connections = config::section::<u32>("db", "min_connections", 20);

    let connect_timeout = config::section::<i64>("db", "connect_timeout", 20);
    let acquire_timeout = config::section::<i64>("db", "acquire_timeout", 20);
    let idle_timeout = config::section::<i64>("db", "idle_timeout", 20);
    let max_lifetime = config::section::<i64>("db", "max_lifetime", 20);

    let logging = config::section::<bool>("db", "logging", false);

    let db_logging_level = config::section::<String>("db", "logging_level", "info".into());
    let logging_level = app::get_log_level(db_logging_level);

    let mut opt = ConnectOptions::new(db_url);
    opt.max_connections(max_connections)
        .min_connections(min_connections)
        .connect_timeout(Duration::from_secs(connect_timeout as u64))
        .acquire_timeout(Duration::from_secs(acquire_timeout as u64))
        .idle_timeout(Duration::from_secs(idle_timeout as u64))
        .max_lifetime(Duration::from_secs(max_lifetime as u64))
        .sqlx_logging(logging)
        .sqlx_logging_level(logging_level);
    
    let db = Database::connect(opt);

    db.await
}
