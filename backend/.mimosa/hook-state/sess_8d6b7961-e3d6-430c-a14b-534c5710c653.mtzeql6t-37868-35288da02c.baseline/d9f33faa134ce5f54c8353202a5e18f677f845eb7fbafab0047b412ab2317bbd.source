//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! G-2.3: 静态化生成服务
//! 定时将动态页面渲染为 HTML 文件，Nginx 可直接读文件返回，显著降低数据库压力
//!
//! 实现策略：
//! - 在调度任务中调用 generate_all()，触发首页/栏目页/文章页静态化
//! - 渲染复用 cms_open_controller 的逻辑（站点+CMS数据+模板）
//! - 输出到 `static_output/{template_id}/` 目录，Nginx 配置 try_files 优先读静态文件
//! - 失败不阻塞其他页面，每页独立记录日志
//!
//! Nginx 配置示例：
//! ```
//! location / {
//!     try_files $uri $uri/ @dynamic;
//! }
//! location @dynamic {
//!     proxy_pass http://backend;
//! }
//! ```

use std::path::PathBuf;
use sea_orm::{DbConn, EntityTrait, ColumnTrait, QueryFilter};
use crate::core::errors::error::{Error, Result};
use crate::core::kit::template::get_template_a_with_cms;
use crate::core::web::tags::cms_tags::CmsTagData;
use crate::modules::articles::entity::category;
use crate::modules::articles::model::article::QueryPageRequest;
use crate::modules::articles::service::article_service;
use crate::modules::website::service::{website_service, template_user_data_service};
use minijinja::context;

/// 静态化输出根目录（相对于项目运行目录）
const STATIC_OUTPUT_DIR: &str = "static_output";

/// 获取静态化输出根目录
pub fn output_root() -> PathBuf {
    PathBuf::from(STATIC_OUTPUT_DIR)
}

/// 生成单个 HTML 文件
///
/// * `rel_path` 相对路径（如 "index.html"、"category/news.html"）
/// * `content` HTML 内容
fn write_html_file(rel_path: &str, content: &str) -> Result<()> {
    let mut full_path = output_root();
    full_path.push(rel_path);
    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::from(format!("创建目录失败 {:?}: {}", parent, e)))?;
    }
    std::fs::write(&full_path, content)
        .map_err(|e| Error::from(format!("写入文件失败 {:?}: {}", full_path, e)))?;
    Ok(())
}

/// 生成首页静态文件
///
/// 输出：`static_output/index.html`
pub async fn generate_index(db: &DbConn) -> Result<()> {
    let site = website_service::find_default(db).await?;
    let site_id = site.id.unwrap_or_default();
    let site_mode = site.site_mode.unwrap_or(1);

    let mut cms_data = CmsTagData::fetch(db).await.unwrap_or_default();
    cms_data.site_mode = site_mode;

    let template_id = site.template_id;
    let template_data = template_user_data_service::find_latest_by_template_and_type(
        db, &template_id, &Some(1)
    ).await?;
    let template_text = template_data.temptext.unwrap_or_default();

    let site_name = site.site_name.clone().unwrap_or_default();
    let site_domain = site.domain.clone().unwrap_or_default();
    let keywords = site.keywords.clone().unwrap_or_default();
    let description = site.description.clone().unwrap_or_default();

    let ctx = context!(
        site => &site,
        site_name => site_name,
        site_domain => site_domain,
        keywords => keywords,
        description => description,
        site_mode => site_mode,
        site_id => site_id,
    );

    let html = get_template_a_with_cms(&template_text, ctx, &cms_data)?;
    write_html_file("index.html", &html)
}

/// 生成所有栏目页静态文件
///
/// 输出：`static_output/category/{short_url}.html`
pub async fn generate_categories(db: &DbConn) -> Result<u64> {
    let site = website_service::find_default(db).await?;
    let site_id = site.id.unwrap_or_default();
    let site_mode = site.site_mode.unwrap_or(1);

    let mut cms_data = CmsTagData::fetch(db).await.unwrap_or_default();
    cms_data.site_mode = site_mode;

    let template_id = site.template_id;
    let template_data = template_user_data_service::find_latest_by_template_and_type(
        db, &template_id, &Some(2)
    ).await?;
    let template_text = template_data.temptext.unwrap_or_default();

    let site_name = site.site_name.clone().unwrap_or_default();
    let site_domain = site.domain.clone().unwrap_or_default();

    // 查询所有栏目（status=1 启用）
    let categories = category::Entity::find()
        .filter(category::Column::Status.eq(1))
        .all(db).await
        .map_err(|e| Error::from(format!("查询栏目失败: {}", e)))?;

    let mut count = 0u64;
    for cat in categories {
        let cat_id = cat.id;
        let cat_name = cat.category_name.clone().unwrap_or_default();
        let cat_desc = cat.description.clone().unwrap_or_default();
        let short_url = cat.short_url.clone().unwrap_or_else(|| cat_id.to_string());
        let breadcrumbs = cms_data.render_breadcrumbs_html(cat_id);

        // 查询栏目下文章
        let article_query = QueryPageRequest {
            title: None,
            page_num: Some(1),
            page_size: Some(20),
            category_id: Some(cat_id),
            website_id: Some(site_id),
            status: Some(2),
        };
        let article_page = match article_service::get_by_page(db, article_query).await {
            Ok(p) => p,
            Err(e) => {
                log::warn!("[静态化] 栏目 {} 文章查询失败: {}", cat_id, e);
                continue;
            }
        };

        let ctx = context!(
            site => &site,
            site_name => site_name,
            site_domain => site_domain,
            category => &cat,
            category_name => cat_name,
            category_description => cat_desc,
            breadcrumbs => breadcrumbs,
            articles => &article_page.items,
            list => &article_page.items,
            total => &article_page.total,
            site_mode => site_mode,
            site_id => site_id,
        );

        match get_template_a_with_cms(&template_text, ctx, &cms_data) {
            Ok(html) => {
                let rel = format!("category/{}.html", short_url);
                if write_html_file(&rel, &html).is_ok() {
                    count += 1;
                }
            }
            Err(e) => {
                log::warn!("[静态化] 栏目页 {} 渲染失败: {}", short_url, e);
            }
        }
    }
    Ok(count)
}

/// 生成所有文章详情页静态文件
///
/// 输出：`static_output/article/{short_url}.html`
pub async fn generate_articles(db: &DbConn) -> Result<u64> {
    let site = website_service::find_default(db).await?;
    let site_id = site.id.unwrap_or_default();
    let site_mode = site.site_mode.unwrap_or(1);

    let mut cms_data = CmsTagData::fetch(db).await.unwrap_or_default();
    cms_data.site_mode = site_mode;

    let template_id = site.template_id;
    let template_data = template_user_data_service::find_latest_by_template_and_type(
        db, &template_id, &Some(3)
    ).await?;
    let template_text = template_data.temptext.unwrap_or_default();

    let site_name = site.site_name.clone().unwrap_or_default();
    let site_domain = site.domain.clone().unwrap_or_default();

    // 查询已发布文章（每批 100 条，避免一次性加载过多）
    let article_query = QueryPageRequest {
        title: None,
        page_num: Some(1),
        page_size: Some(500),
        category_id: None,
        website_id: Some(site_id),
        status: Some(2),
    };
    let article_page = article_service::get_by_page(db, article_query).await?;

    let mut count = 0u64;
    for art in article_page.items {
        let short_url = art.short_url.clone().unwrap_or_else(|| art.id.clone().unwrap_or_default());
        let ctx = context!(
            site => &site,
            site_name => site_name,
            site_domain => site_domain,
            article => &art,
            site_mode => site_mode,
            site_id => site_id,
        );

        match get_template_a_with_cms(&template_text, ctx, &cms_data) {
            Ok(html) => {
                let rel = format!("article/{}.html", short_url);
                if write_html_file(&rel, &html).is_ok() {
                    count += 1;
                }
            }
            Err(e) => {
                log::warn!("[静态化] 文章页 {} 渲染失败: {}", short_url, e);
            }
        }
    }
    Ok(count)
}

/// 生成所有静态页面（首页 + 所有栏目页 + 所有文章页）
///
/// 供调度器调用，返回 (栏目数, 文章数)
pub async fn generate_all(db: &DbConn) -> Result<(u64, u64)> {
    log::info!("[静态化] 开始全站静态化生成");

    // 首页（失败不阻塞其他）
    if let Err(e) = generate_index(db).await {
        log::warn!("[静态化] 首页生成失败: {}", e);
    } else {
        log::info!("[静态化] 首页生成完成");
    }

    // 栏目页
    let cat_count = generate_categories(db).await.unwrap_or(0);
    log::info!("[静态化] 栏目页生成完成: {} 个", cat_count);

    // 文章页
    let art_count = generate_articles(db).await.unwrap_or(0);
    log::info!("[静态化] 文章页生成完成: {} 个", art_count);

    log::info!("[静态化] 全站静态化完成: 栏目 {} / 文章 {}", cat_count, art_count);
    Ok((cat_count, art_count))
}

/// 清空静态化输出目录（用于强制重新生成或关闭静态化）
pub fn clear_output() -> Result<()> {
    let root = output_root();
    if root.exists() {
        std::fs::remove_dir_all(&root)
            .map_err(|e| Error::from(format!("清空静态化目录失败: {}", e)))?;
        log::info!("[静态化] 已清空输出目录: {:?}", root);
    }
    Ok(())
}
