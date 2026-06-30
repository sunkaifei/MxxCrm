//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use ini::Ini;
use std::path;
use std::str::FromStr;
use std::sync::LazyLock;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "config/"]
pub struct Config;

static GLOBAL_CONF: LazyLock<Ini> = LazyLock::new(|| {
    let config_paths = vec![
        "production_config.ini",
        "config/production_config.ini",
        "../config/production_config.ini",
        "Config.ini",
        "config/Config.ini",
        "../config/Config.ini",
    ];

    for config_path in config_paths {
        let abs_path = std::env::current_dir().unwrap().join(config_path);
        log::info!("[配置] 尝试加载配置文件: {} (绝对路径: {})", config_path, abs_path.display());
        if path::Path::new(&abs_path).exists() {
            log::info!("[配置] 找到配置文件: {}", abs_path.display());
            match Ini::load_from_file(&abs_path) {
                Ok(ini) => {
                    log::info!("[配置] 成功加载配置文件: {}", abs_path.display());
                    return ini;
                },
                Err(e) => {
                    log::error!("[配置] 加载配置文件失败: {}, 错误: {:?}", abs_path.display(), e);
                    continue;
                }
            }
        } else {
            log::info!("[配置] 配置文件不存在: {}", abs_path.display());
        }
    }

    log::info!("[配置] 使用内置的默认配置");
    let conf = match Config::get("Config.ini") {
        Some(v) => v.data.into_owned(),
        None => {
            log::error!("[配置] 无法找到内置的配置文件");
            return Ini::new();
        }
    };
    let conf_str = std::str::from_utf8(conf.as_ref()).unwrap_or("");

    match Ini::load_from_str(conf_str) {
        Ok(ini) => {
            log::info!("[配置] 成功加载内置的配置");
            ini
        },
        Err(e) => {
            log::error!("[配置] 加载内置配置失败: {:?}", e);
            Ini::new()
        }
    }
});

/// 初始化
pub fn new() -> Ini {
    let conf = Ini::new();
    conf
}

pub fn load_from_str(data: &str) -> Ini {
    Ini::load_from_str(data).unwrap_or_default()
}

pub fn load_from_file(file: &str) -> Ini {
    Ini::load_from_file(file).unwrap_or_default()
}

// env
pub fn section<B: FromStr>(section: &str, key: &str, def_val: B) -> B 
    where <B as FromStr>::Err: std::fmt::Debug, B: std::fmt::Debug
{
    let ini = &GLOBAL_CONF;

    match ini.get_from(Some(section), key){
        Some(data) => {
            match data.parse::<B>() {
                Ok(val) => {
                    log::debug!("[配置] 读取配置: {} -> {} = {:?}", section, key, val);
                    val
                },
                Err(e) => {
                    log::warn!("[配置] 解析配置失败: {} -> {} = \"{}\", 错误: {:?}, 使用默认值: {:?}", section, key, data, e, def_val);
                    def_val
                }
            }
        },
        None => {
            log::warn!("[配置] 未找到配置: {} -> {}, 使用默认值: {:?}", section, key, def_val);
            def_val
        }
    }
}

