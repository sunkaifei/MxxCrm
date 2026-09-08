//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 统一附件 URL 解析服务（附件URL与访问域名统一方案 v1.1 §5.1）
//!
//! 解析优先级：
//! 1. 网站设置 asset_domain（每站点可配，单站模式取默认站点）
//! 2. 配置中心 localStorage / config.ini [attach] static_url（全局兜底，兼容旧 CDN 配置）
//! 3. 空 → 原样返回相对路径（同源访问，局域网直启场景）
//!
//! 后端调用方：PDF 生成、站内信/webhook 消息卡片、cms_open_controller 前台出参、静态化渲染。
//! 管理后台列表/详情接口不调用（前端 resolveAssetUrl 拼接，见方案 §5.4）。

use sea_orm::DbConn;

use crate::core::errors::error::{Error, Result};
use crate::modules::website::service::website_service;

/// 判断是否已带协议前缀（无需处理）
fn has_scheme(path: &str) -> bool {
    path.starts_with("http://")
        || path.starts_with("https://")
        || path.starts_with("data:")
        || path.starts_with("//")
}

/// 获取当前生效的资源域名前缀（带尾斜杠；空串=同源相对路径）
///
/// 供列表场景**一次获取、本地拼接**（方案 §8.4：禁止每行查询站点表）；
/// 单条场景直接用 [`resolve_asset_url`]。
pub async fn asset_domain_prefix(db: &DbConn) -> Result<String> {
    // 1. 网站设置 asset_domain（单站模式取默认站点；空配置视为未设置）
    if let Ok(site) = website_service::find_default(db).await {
        if let Some(domain) = site.asset_domain {
            let domain = domain.trim().trim_end_matches('/').to_string();
            if !domain.is_empty() {
                return Ok(format!("{}/", domain));
            }
        }
    }

    // 2. 全局兜底：config localStorage / config.ini static_url
    let storage_url = super::attachment_service::upload_storage_url(db, &Some(1)).await?;
    if !storage_url.is_empty() {
        return Ok(format!("{}/", storage_url.trim_end_matches('/')));
    }

    // 3. 同源相对路径
    Ok(String::new())
}

/// 按"已解析前缀"拼接（列表批量场景配合 [`asset_domain_prefix`] 使用，无额外查询）
pub fn join_asset_url(prefix: &str, path: &str) -> String {
    if path.is_empty() || has_scheme(path) {
        return path.to_string();
    }
    if prefix.is_empty() {
        return path.to_string();
    }
    format!("{}{}", prefix, path.trim_start_matches('/'))
}

/// 统一附件 URL 解析
///
/// * `db`   数据库连接
/// * `path` 附件相对 URL（如 /upload/common/2026/09/11/xxx.jpg）
pub async fn resolve_asset_url(db: &DbConn, path: &str) -> Result<String> {
    if path.is_empty() {
        return Ok(String::new());
    }
    if has_scheme(path) {
        return Ok(path.to_string());
    }

    let prefix = asset_domain_prefix(db).await?;
    Ok(join_asset_url(&prefix, path))
}

/// 批量解析（用于列表出参；内部逐条调用，站点查询由调用方控制频率）
pub async fn resolve_asset_url_opt(
    db: &DbConn,
    path: &Option<String>,
) -> Result<Option<String>> {
    match path {
        Some(p) => Ok(Some(resolve_asset_url(db, p).await?)),
        None => Ok(None),
    }
}
