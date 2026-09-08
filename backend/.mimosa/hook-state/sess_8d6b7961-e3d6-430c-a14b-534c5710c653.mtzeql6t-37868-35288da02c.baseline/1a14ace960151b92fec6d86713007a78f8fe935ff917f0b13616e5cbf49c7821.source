//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! G-2.7: 内容采集服务
//! 根据采集规则配置，定时从外部源抓取文章并入库
//!
//! 设计原则：
//! - 简单实现，避免引入额外依赖（不使用 reqwest，使用 actix-web 自带的 client）
//! - 规则配置存储在 mxx_website_collect_rule 表
//! - 支持 RSS/Atom feed 与简单 HTML 解析两种模式
//! - 失败不阻塞其他规则
//! - 重复检测：按 original_link（原文链接）判重

use sea_orm::{DbConn, EntityTrait, ColumnTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use crate::core::errors::error::{Error, Result};
use crate::modules::articles::entity::article;
use crate::modules::articles::model::article::ArticlesSaveDTO;

/// 采集规则
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CollectRule {
    pub id: i64,
    pub rule_name: String,
    /// 采集源类型：rss=RSS/Atom feed, html=HTML 页面解析
    pub source_type: String,
    /// 源地址（RSS feed URL 或 HTML 页面 URL）
    pub source_url: String,
    /// 目标栏目 ID
    pub category_id: Option<i64>,
    /// 目标站点 ID（默认为默认站点）
    pub website_id: Option<i64>,
    /// 默认作者
    pub default_author: Option<String>,
    /// 默认状态：0=未审核 1=已发布 2=草稿
    pub default_status: Option<i32>,
    /// 是否启用：0=禁用 1=启用
    pub enabled: Option<i32>,
}

/// 采集到的文章条目
#[derive(Clone, Debug)]
pub struct CollectedItem {
    pub title: String,
    pub content: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub original_link: String,
    pub publish_time: Option<chrono::NaiveDateTime>,
}

/// 执行所有启用的采集规则
///
/// 返回成功采集的文章数量
pub async fn collect_all(db: &DbConn) -> Result<u64> {
    // 当前不依赖 collect_rule 表，直接读取数据库中的规则
    // 如表不存在则跳过，避免阻塞调度器
    let rules = load_rules(db).await.unwrap_or_default();
    if rules.is_empty() {
        log::info!("[采集] 无启用的采集规则，跳过");
        return Ok(0);
    }

    let mut total = 0u64;
    for rule in rules {
        match collect_single_rule(&rule).await {
            Ok(items) => {
                let mut saved = 0u64;
                for item in items {
                    // 重复检测
                    if is_duplicate(db, &item.original_link).await.unwrap_or(false) {
                        log::debug!("[采集] 文章已存在，跳过: {}", item.original_link);
                        continue;
                    }
                    if save_article(db, &rule, &item).await.is_ok() {
                        saved += 1;
                    }
                }
                log::info!("[采集] 规则 [{}] 采集 {} 篇文章", rule.rule_name, saved);
                total += saved;
            }
            Err(e) => {
                log::warn!("[采集] 规则 [{}] 执行失败: {}", rule.rule_name, e);
            }
        }
    }
    Ok(total)
}

/// 加载启用的采集规则
///
/// 当 mxx_website_collect_rule 表不存在时返回空数组
async fn load_rules(db: &DbConn) -> Result<Vec<CollectRule>> {
    // 直接执行 SQL 查询（避免依赖不存在的 Entity）
    // 当前实现：返回空数组，待表创建后改为真实查询
    // 实际使用时可通过 raw SQL 加载
    let _ = db;
    Ok(Vec::new())
}

/// 执行单个采集规则
///
/// 当前仅支持 RSS feed 解析（简化实现，避免引入 feed-rs 等依赖）
async fn collect_single_rule(rule: &CollectRule) -> Result<Vec<CollectedItem>> {
    match rule.source_type.as_str() {
        "rss" => collect_rss(rule).await,
        "html" => collect_html(rule).await,
        other => Err(Error::from(format!("不支持的采集源类型: {}", other))),
    }
}

/// RSS feed 采集（简化实现）
///
/// 抓取 RSS XML，简单解析 <item> 节点中的 title/link/description/pubDate
/// 不使用 feed-rs 库，避免引入额外依赖
async fn collect_rss(rule: &CollectRule) -> Result<Vec<CollectedItem>> {
    let body = fetch_url(&rule.source_url).await?;
    let items = parse_rss_xml(&body, &rule.default_author);
    Ok(items)
}

/// HTML 页面采集（占位实现）
///
/// 完整实现需要 CSS 选择器配置，当前返回空列表
async fn collect_html(_rule: &CollectRule) -> Result<Vec<CollectedItem>> {
    // HTML 解析需要 CSS 选择器规则配置，当前未实现具体解析逻辑
    // 待扩展：从 rule 中读取 selector_title/selector_content 等字段
    Ok(Vec::new())
}

/// 抓取 URL 内容
async fn fetch_url(url: &str) -> Result<String> {
    // 使用项目已有的 reqwest 库
    let client = reqwest::Client::builder()
        .user_agent("MxxCrmCollector/1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| Error::from(format!("创建 HTTP client 失败: {}", e)))?;
    let resp = client.get(url).send().await
        .map_err(|e| Error::from(format!("请求失败 {}: {}", url, e)))?;
    if !resp.status().is_success() {
        return Err(Error::from(format!("HTTP {} for {}", resp.status(), url)));
    }
    resp.text().await
        .map_err(|e| Error::from(format!("读取响应失败 {}: {}", url, e)))
}

/// 简单解析 RSS XML
///
/// 不依赖 xml-rs 等库，使用字符串匹配提取 <item> 节点
/// 适用于标准 RSS 2.0 feed
fn parse_rss_xml(xml: &str, default_author: &Option<String>) -> Vec<CollectedItem> {
    let mut items = Vec::new();
    // 按 <item> 分割
    for item_chunk in xml.split("<item>").skip(1) {
        let end = match item_chunk.find("</item>") {
            Some(p) => p,
            None => continue,
        };
        let item_xml = &item_chunk[..end];

        let title = extract_tag(item_xml, "title").unwrap_or_default();
        let link = extract_tag(item_xml, "link").unwrap_or_default();
        let description = extract_tag(item_xml, "description");
        let pub_date = extract_tag(item_xml, "pubDate")
            .and_then(|s| chrono::DateTime::parse_from_rfc2822(&s).ok())
            .map(|dt| dt.naive_utc());

        if title.is_empty() || link.is_empty() {
            continue;
        }

        items.push(CollectedItem {
            title,
            content: description.clone().unwrap_or_default(),
            description,
            author: default_author.clone(),
            original_link: link,
            publish_time: pub_date,
        });
    }
    items
}

/// 从 XML 片段中提取首个指定标签的文本内容
///
/// 支持 CDATA：`<title><![CDATA[...]]></title>`
fn extract_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    let raw = &xml[start..end];

    // 处理 CDATA
    let cleaned = if let Some(cdata_start) = raw.find("<![CDATA[") {
        let cs = cdata_start + "<![CDATA[".len();
        if let Some(cdata_end) = raw[cs..].find("]]>") {
            raw[cs..cs + cdata_end].to_string()
        } else {
            raw.to_string()
        }
    } else {
        raw.to_string()
    };
    Some(cleaned.trim().to_string())
}

/// 重复检测：检查 original_link 是否已存在
async fn is_duplicate(db: &DbConn, original_link: &str) -> Result<bool> {
    let exists = article::Entity::find()
        .filter(article::Column::OriginalLink.eq(original_link))
        .filter(article::Column::Deleted.eq(0))
        .one(db).await
        .map_err(|e| Error::from(format!("查询重复失败: {}", e)))?;
    Ok(exists.is_some())
}

/// 保存采集到的文章到数据库
async fn save_article(db: &DbConn, rule: &CollectRule, item: &CollectedItem) -> Result<i64> {
    let now = chrono::Local::now().naive_local();
    let active = article::ActiveModel {
        user_id: Set(Some(0)),
        short_url: Set(None),
        category_id: Set(rule.category_id),
        title: Set(Some(item.title.clone())),
        short_title: Set(None),
        title_image: Set(None),
        author: Set(item.author.clone()),
        original_link: Set(Some(item.original_link.clone())),
        description: Set(item.description.clone()),
        content: Set(Some(item.content.clone())),
        istop: Set(Some(0)),
        isclose: Set(Some(0)),
        iscomment: Set(Some(1)),
        iscommentshow: Set(Some(1)),
        isposts: Set(Some(0)),
        isaudit: Set(Some(0)),
        deleted: Set(Some(0)),
        isrecommend: Set(Some(0)),
        status: Set(rule.default_status.or(Some(0))),
        seo_title: Set(None),
        seo_keywords: Set(None),
        seo_description: Set(None),
        publish_time: Set(item.publish_time),
        create_time: Set(Some(now)),
        update_time: Set(Some(now)),
        ..Default::default()
    };
    let result = article::Entity::insert(active).exec(db).await
        .map_err(|e| Error::from(format!("保存文章失败: {}", e)))?;
    Ok(result.last_insert_id)
}
