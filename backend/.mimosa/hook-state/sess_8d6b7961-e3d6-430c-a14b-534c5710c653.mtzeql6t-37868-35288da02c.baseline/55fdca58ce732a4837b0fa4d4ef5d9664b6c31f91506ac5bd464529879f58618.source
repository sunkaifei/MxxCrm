//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::{Error,Result};
use crate::core::kit::config;
use actix_web::HttpRequest;
use regex::Regex;

pub fn get_subdomain(req: &HttpRequest) -> Result<Option<String>> {
    // 获取 Host 头部
    let host_header = req.headers().get("host");
    if let Some(host) = host_header {
        if let Ok(host_str) = host.to_str() {
            // 分离域名和端口号
            let parts: Vec<&str> = host_str.split(':').collect();
            let domain_str = parts[0]; // 获取域名部分，不包括端口号
            // 正则表达式用于匹配合法的二级域名
            let re = Regex::new(r"^[a-zA-Z0-9]+([-.][a-zA-Z0-9]+)*.[a-zA-Z]{2,5}$").unwrap();
            if re.is_match(domain_str) {
                let domain_parts: Vec<&str> = domain_str.split('.').collect();
                if domain_parts.len() >= 2 { // 确保至少有两个部分（子域名和主域名）
                    let subdomain = domain_parts[0];
                    let main_domain = domain_parts[1..].join(".");
                    let saas_domain = config::section::<String>("server", "saas_domain", "".to_string());
                    if main_domain != saas_domain {
                        Err(Error::from("获取的二级域名格式错误".to_string()))?
                    };
                    Ok(Some(subdomain.to_string()))
                } else {
                    Err(Error::from("未获取到二级域名名称".to_string()))?
                }
            } else {
                Err(Error::from("获取的域名格式错误".to_string()))?
            }
        } else {
            Err(Error::from("未获取到host信息".to_string()))?
        }
    } else {
        Err(Error::from("未获取到header头部信息".to_string()))?
    }
}