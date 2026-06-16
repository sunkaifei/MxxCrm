//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::kit::config;

// 是否是调试模式
pub fn is_debug() -> bool {
    config::section::<bool>("app", "debug", false)
}

// 管理员ID
pub fn get_admin_id() -> u32 {
    config::section::<u32>("app", "admin_id", 0)
}

// 获取日志等级
pub fn get_log_level(name: String) -> log::LevelFilter {
    match name.as_str() {
        "debug" => log::LevelFilter::Debug,
        "error" => log::LevelFilter::Error,
        "info" => log::LevelFilter::Info,
        "trace" => log::LevelFilter::Trace,
        "warn" => log::LevelFilter::Warn,
        _ => log::LevelFilter::Info,
    }
}

pub fn upload_path(name: String) -> String {
    let path = config::section::<String>("attach", "upload_path", "./storage/upload".to_string());

    format!("{}{}", path, name)
}

pub fn upload_url(name: String) -> String {
    let static_url = config::section::<String>("attach", "static_url", "https://static.s88.cn/".to_string());
    let upload_url = config::section::<String>("attach", "upload_url", "/upload/".to_string());

    format!("{}{}{}", static_url.trim_end_matches('/'), upload_url, name)
}

// 附件路径
pub fn attach_path(name: String) -> String {
    let path = config::section::<String>("attach", "attach_path", "./storage/attach".to_string());

    format!("{}{}", path, name)
}



