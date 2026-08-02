//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::Result;
use crate::core::web::tags::common_tags::{filter_html, none_default, to_json_filter};
use crate::core::web::tags::format_time_tags::format_time;
use crate::core::web::tags::cms_tags::{CmsTagData, cart_button_html, lead_form_html};
use crate::modules::articles::entity::category;
use crate::modules::articles::model::article::ArticleListVO;
use minijinja::{path_loader, Environment, Value};
use serde::Serialize;
use std::collections::HashMap;


pub fn get_template(name: &str, ctx: Value) -> Result<String> {
    let mut env = Environment::new();
    env.set_auto_escape_callback(|_| minijinja::AutoEscape::Html);
    env.set_loader(path_loader("templates"));
    // 注册自定义过滤器
    env.add_filter("to_json", to_json_filter);
    env.add_filter("default", none_default);
    env.add_function("format_time", format_time);
    env.add_function("filter_html",filter_html);
    //env.add_function("lang", lang_function);
    let tpl = env.get_template(name)?;
    //log::info!("===========tpl=========={:?}",tpl);
    Ok(tpl.render(ctx).unwrap_or_default())
}


pub fn get_template_a(template_content: &str, ctx: Value) -> Result<String> {
    let mut env = Environment::new();
    // 注册自定义过滤器
    env.add_filter("to_json", to_json_filter);
    env.add_filter("default", none_default);
    env.add_function("format_time", format_time);
    env.add_function("filter_html", filter_html);

    // 添加模板字符串
    let r = env.render_str(template_content,ctx)?;
    Ok(r)
}


/// 注册 CMS 标签函数到 minijinja 环境
///
/// 由于 `add_function` 需要 `Fn` 闭包（同步），数据库查询无法直接在模板标签中完成，
/// 因此通过预取的 `CmsTagData` 将数据以闭包形式暴露给模板：
/// - `tpl_var(key)` 获取模板变量值
/// - `banners(position)` 获取指定位置的 Banner 列表
/// - `block(code)` 获取指定 code 的区块
/// - `page(code)` 获取指定 code 的页面
/// - `content_models()` 获取内容模型列表
/// - `navigations(nav_type)` 获取指定类型的导航列表
/// - `links(link_type)` 获取指定类型的友情链接
fn register_cms_functions(env: &mut Environment, cms_data: &CmsTagData) {
    let tpl_vars = cms_data.template_vars.clone();
    env.add_function("tpl_var", move |key: String| -> String {
        tpl_vars.get(&key).cloned().unwrap_or_default()
    });

    let banners_data = cms_data.banners.clone();
    env.add_function("get_banners", move |position: String| -> Vec<Value> {
        banners_data.get(&position).cloned().unwrap_or_default()
    });

    let blocks_data = cms_data.blocks.clone();
    env.add_function("get_block", move |code: String| -> Option<Value> {
        blocks_data.get(&code).cloned()
    });

    let pages_data = cms_data.pages.clone();
    env.add_function("get_page", move |code: String| -> Option<Value> {
        pages_data.get(&code).cloned()
    });

    let models_data = cms_data.content_models.clone();
    env.add_function("content_models", move || -> Vec<Value> {
        models_data.clone()
    });

    let navs_data = cms_data.navigations.clone();
    env.add_function("get_navigations", move |nav_type: Option<String>| -> Vec<Value> {
        let nt = nav_type.unwrap_or_else(|| "header".to_string());
        navs_data.get(&nt).cloned().unwrap_or_default()
    });

    let links_data = cms_data.links.clone();
    env.add_function("get_links", move |link_type: Option<i32>| -> Vec<Value> {
        let lt = format!("{}", link_type.unwrap_or(0));
        links_data.get(&lt).cloned().unwrap_or_default()
    });

    // 面包屑：返回面包屑数组 [{id, name, shortUrl}]
    let breadcrumbs_data = cms_data.categories.clone();
    env.add_function("get_breadcrumbs", move |category_id: i64| -> Vec<Value> {
        let mut chain: Vec<(i64, String, String)> = Vec::new();
        let mut current_id = Some(category_id);
        let mut depth = 0;
        let mut visited = std::collections::HashSet::new();
        while let Some(cid) = current_id {
            if depth >= 20 || visited.contains(&cid) {
                break;
            }
            visited.insert(cid);
            match breadcrumbs_data.get(&cid) {
                Some(c) => {
                    let name = c.category_name.clone().unwrap_or_default();
                    let short_url = c.short_url.clone().unwrap_or_default();
                    chain.push((cid, name, short_url));
                    current_id = c.parent_id;
                    depth += 1;
                }
                None => break,
            }
        }
        chain.reverse();
        chain.into_iter()
            .map(|(id, name, short_url)| {
                minijinja::Value::from_serialize(&serde_json::json!({
                    "id": id,
                    "name": name,
                    "shortUrl": short_url,
                }))
            })
            .collect()
    });

    // 面包屑 HTML：直接输出渲染好的面包屑 HTML 字符串
    let breadcrumbs_html_data = cms_data.categories.clone();
    env.add_function("get_breadcrumbs_html", move |category_id: Option<i64>| -> String {
        let cid = match category_id {
            Some(v) if v > 0 => v,
            _ => return String::new(),
        };
        // 复用 CmsTagData 的渲染逻辑
        let tmp = CmsTagData {
            categories: breadcrumbs_html_data.clone(),
            ..Default::default()
        };
        tmp.render_breadcrumbs_html(cid)
    });

    // 文章列表标签：从预取数据中过滤
    let articles_data = cms_data.articles.clone();
    env.add_function("get_articles", move |category_id: Option<i64>, limit: Option<usize>, page: Option<usize>| -> Vec<Value> {
        let limit = limit.unwrap_or(10).min(50);
        let page = page.unwrap_or(1).max(1);
        let offset = (page - 1) * limit;

        articles_data.iter()
            .filter(|a| {
                match category_id {
                    Some(cid) if cid > 0 => a.category_id == Some(cid),
                    _ => true,
                }
            })
            .skip(offset)
            .take(limit)
            .map(|a| Value::from_serialize(a))
            .collect()
    });

    // 推荐文章标签
    let recommend_data = cms_data.articles.clone();
    env.add_function("get_recommend_articles", move |limit: Option<usize>| -> Vec<Value> {
        let limit = limit.unwrap_or(5).min(20);
        recommend_data.iter()
            .filter(|a| a.isrecommend.unwrap_or(0) == 1)
            .take(limit)
            .map(|a| Value::from_serialize(a))
            .collect()
    });

    // 产品列表标签：从预取数据中过滤
    let products_data = cms_data.products.clone();
    env.add_function("get_products", move |category_id: Option<i64>, limit: Option<usize>, page: Option<usize>, order: Option<String>| -> Vec<Value> {
        let limit = limit.unwrap_or(10).min(50);
        let page = page.unwrap_or(1).max(1);
        let offset = (page - 1) * limit;
        let _ = order; // 预取数据已按最新排序，order 参数暂不重新排序

        products_data.iter()
            .filter(|p| {
                match category_id {
                    Some(cid) if cid > 0 => {
                        p.get_attr("categoryId").ok()
                            .and_then(|v| v.as_i64())
                            .map(|id| id == cid)
                            .unwrap_or(false)
                    },
                    _ => true,
                }
            })
            .skip(offset)
            .take(limit)
            .cloned()
            .collect()
    });

    // 栏目树标签
    let categories_data = cms_data.categories.clone();
    env.add_function("get_categories", move |parent_id: Option<i64>| -> Vec<Value> {
        let pid = parent_id.unwrap_or(0);
        categories_data.values()
            .filter(|c| {
                if pid == 0 {
                    c.parent_id.unwrap_or(0) == 0
                } else {
                    c.parent_id == Some(pid)
                }
            })
            .map(|c| {
                Value::from_serialize(&serde_json::json!({
                    "id": c.id,
                    "categoryName": c.category_name,
                    "shortUrl": c.short_url,
                    "parentId": c.parent_id,
                    "description": c.description,
                    "isShow": c.is_show,
                    "sort": c.sort,
                }))
            })
            .collect()
    });

    // 站点模式（供 cart_button 等标签使用）
    let site_mode = cms_data.site_mode;
    env.add_function("get_site_mode", move || -> i32 { site_mode });

    // 购物车/咨询按钮：根据站点模式渲染不同按钮
    env.add_function("cart_button", move |product_id: i64, site_mode_param: Option<i32>| -> String {
        cart_button_html(product_id, site_mode_param.or(Some(site_mode)))
    });

    // 线索/咨询表单：渲染表单 HTML
    env.add_function("lead_form", move |product_id: Option<i64>| -> String {
        lead_form_html(product_id)
    });

    // TPL-10: include_template 标签
    // 模板调用：{{ include_template("header.html") }}
    // 返回片段内容（type_id=4 模板）作为安全字符串，不转义
    let fragments = cms_data.template_fragments.clone();
    env.add_function("include_template", move |name: String| -> String {
        fragments.get(&name).cloned().unwrap_or_default()
    });

    // MED-10: 媒体标签
    // get_media(id) 获取单个媒体
    let media_data = cms_data.media.clone();
    env.add_function("get_media", move |id: i64| -> Option<Value> {
        media_data.get(&id).cloned()
    });

    // media_url(id) 仅返回 URL，便于 <img src="{{ media_url(1) }}">
    let media_url_data = cms_data.media.clone();
    env.add_function("media_url", move |id: i64| -> String {
        media_url_data.get(&id)
            .and_then(|v| v.get_attr("fileUrl").ok())
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default()
    });

    // get_media_list(category_id, limit) 按分类获取媒体列表
    let media_list_data = cms_data.media_by_category.clone();
    env.add_function("get_media_list", move |category_id: Option<i64>, limit: Option<usize>| -> Vec<Value> {
        let limit = limit.unwrap_or(20).min(100);
        match category_id {
            Some(cid) => media_list_data.get(&cid).cloned().unwrap_or_default()
                .into_iter().take(limit).collect(),
            None => {
                // 未指定分类时返回所有
                media_list_data.values().flat_map(|v| v.iter().cloned())
                    .take(limit).collect()
            }
        }
    });

    // get_media_gallery(category_id, limit) 获取图廊（仅图片，file_type=1）
    let media_gallery_data = cms_data.media_by_category.clone();
    env.add_function("get_media_gallery", move |category_id: Option<i64>, limit: Option<usize>| -> Vec<Value> {
        let limit = limit.unwrap_or(20).min(100);
        let filter_img = |v: &Value| -> bool {
            v.get_attr("fileType").ok()
                .and_then(|t| t.as_i64())
                .map(|t| t == 1)
                .unwrap_or(false)
        };
        match category_id {
            Some(cid) => media_gallery_data.get(&cid).cloned().unwrap_or_default()
                .into_iter().filter(|v| filter_img(v)).take(limit).collect(),
            None => {
                media_gallery_data.values().flat_map(|v| v.iter().cloned())
                    .filter(|v| filter_img(v)).take(limit).collect()
            }
        }
    });

    // ===== G-1.7: 相关文章标签（同分类，排除当前文章） =====
    let related_data = cms_data.articles.clone();
    env.add_function("get_related_articles", move |article_id: Option<i64>, category_id: Option<i64>, limit: Option<usize>| -> Vec<Value> {
        let limit = limit.unwrap_or(5).min(20);
        let aid = article_id.unwrap_or(0);
        related_data.iter()
            .filter(|a| {
                // 排除当前文章
                if aid > 0 {
                    if let Some(id) = &a.id {
                        if id.parse::<i64>().unwrap_or(0) == aid {
                            return false;
                        }
                    }
                }
                // 同分类
                match category_id {
                    Some(cid) if cid > 0 => a.category_id == Some(cid),
                    _ => true,
                }
            })
            .take(limit)
            .map(|a| Value::from_serialize(a))
            .collect()
    });

    // ===== G-1.7: 文章搜索标签（按标题关键词过滤预取数据） =====
    let search_data = cms_data.articles.clone();
    env.add_function("search_articles", move |keyword: String, limit: Option<usize>| -> Vec<Value> {
        let limit = limit.unwrap_or(10).min(50);
        let kw = keyword.to_lowercase();
        search_data.iter()
            .filter(|a| {
                a.title.as_ref()
                    .map(|t| t.to_lowercase().contains(&kw))
                    .unwrap_or(false)
            })
            .take(limit)
            .map(|a| Value::from_serialize(a))
            .collect()
    });

    // ===== G-1.13: Open Graph 标签生成（纯函数） =====
    env.add_function("og_tags", move |title: String, description: Option<String>, image: Option<String>, url: Option<String>| -> String {
        let desc = description.unwrap_or_default();
        let img = image.unwrap_or_default();
        let u = url.unwrap_or_default();
        format!(
            r#"<meta property="og:title" content="{}"/>
<meta property="og:description" content="{}"/>
<meta property="og:image" content="{}"/>
<meta property="og:url" content="{}"/>
<meta property="og:type" content="article"/>
<meta name="twitter:card" content="summary_large_image"/>"#,
            html_escape(&title),
            html_escape(&desc),
            html_escape(&img),
            html_escape(&u)
        )
    });

    // ===== G-1.13: HTML 站点地图标签（基于预取栏目和文章） =====
    let sitemap_cats = cms_data.categories.clone();
    let sitemap_articles = cms_data.articles.clone();
    env.add_function("get_sitemap_html", move || -> String {
        let mut html = String::from(r#"<ul class="sitemap-list">"#);
        // 栏目
        let mut top_cats: Vec<&category::Model> = sitemap_cats.values()
            .filter(|c| c.parent_id.unwrap_or(0) == 0)
            .collect();
        top_cats.sort_by_key(|c| c.sort.unwrap_or(0));
        for cat in top_cats {
            let cat_id = cat.id;
            let name = cat.category_name.clone().unwrap_or_default();
            let url = cat.short_url.clone().unwrap_or_default();
            html.push_str(&format!(
                r#"<li><a href="/category/{}">{}</a>"#,
                html_escape(&url),
                html_escape(&name)
            ));
            // 该栏目下的文章
            let arts: Vec<&ArticleListVO> = sitemap_articles.iter()
                .filter(|a| a.category_id == Some(cat_id))
                .collect();
            if !arts.is_empty() {
                html.push_str("<ul>");
                for art in arts {
                    let title = art.title.clone().unwrap_or_default();
                    let short = art.short_url.clone().unwrap_or_default();
                    html.push_str(&format!(
                        r#"<li><a href="/article/{}">{}</a></li>"#,
                        html_escape(&short),
                        html_escape(&title)
                    ));
                }
                html.push_str("</ul>");
            }
            html.push_str("</li>");
        }
        html.push_str("</ul>");
        html
    });

    // ===== M-9: 内容模型标签 =====
    let models_data = cms_data.content_models.clone();
    env.add_function("get_model", move |model_code: String| -> Option<Value> {
        models_data.iter()
            .find(|m| {
                m.get_attr("modelCode").ok()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .map(|c| c == model_code)
                    .unwrap_or(false)
            })
            .cloned()
    });

    // get_model_articles(model_code, limit)：按模型编码过滤文章
    // 注：栏目表无 model_id 字段，此处按 content_type=1（文章类型）降级返回最新文章
    let model_articles_data = cms_data.articles.clone();
    env.add_function("get_model_articles", move |_model_code: String, limit: Option<usize>| -> Vec<Value> {
        let limit = limit.unwrap_or(10).min(50);
        model_articles_data.iter()
            .take(limit)
            .map(|a| Value::from_serialize(a))
            .collect()
    });

    // 注册工具标签（纯函数，无 DB 依赖）
    env.add_filter("truncate", truncate_filter);
    env.add_function("time_ago", time_ago_function);
    env.add_function("pagination", pagination_function);
}

/// HTML 转义辅助函数（用于 og_tags 等标签输出）
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// 渲染模板（带 CMS 数据）
///
/// 与 `get_template` 相同，但额外注册了 CMS 标签函数，
/// 模板中可直接使用 `tpl_var`、`banners`、`block`、`page`、`content_models` 等标签。
pub fn get_template_with_cms(name: &str, ctx: Value, cms_data: &CmsTagData) -> Result<String> {
    let mut env = Environment::new();
    env.set_auto_escape_callback(|_| minijinja::AutoEscape::Html);
    env.set_loader(path_loader("templates"));
    env.add_filter("to_json", to_json_filter);
    env.add_filter("default", none_default);
    env.add_function("format_time", format_time);
    env.add_function("filter_html", filter_html);

    // 注册 CMS 标签函数
    register_cms_functions(&mut env, cms_data);

    let tpl = env.get_template(name)?;
    Ok(tpl.render(ctx).unwrap_or_default())
}

/// 渲染模板字符串（带 CMS 数据）
///
/// 与 `get_template_a` 相同，但额外注册了 CMS 标签函数，
/// 适用于从数据库读取模板内容字符串并渲染的场景。
pub fn get_template_a_with_cms(template_content: &str, ctx: Value, cms_data: &CmsTagData) -> Result<String> {
    let mut env = Environment::new();
    env.add_filter("to_json", to_json_filter);
    env.add_filter("default", none_default);
    env.add_function("format_time", format_time);
    env.add_function("filter_html", filter_html);

    // 注册 CMS 标签函数
    register_cms_functions(&mut env, cms_data);

    let r = env.render_str(template_content, ctx)?;
    Ok(r)
}


//pub fn get_website()



// 定义全局标签插入函数
#[allow(dead_code)]
fn render_template(template: &str, context: &HashMap<&str, String>) -> String {
    let mut rendered = template.to_string();
    for (key, value) in context {
        let placeholder = format!("{{{{ {} }}}}", key); // 假设使用 {{ key }} 作为占位符
        rendered = rendered.replace(&placeholder, value);
    }
    rendered
}


// ============================================================
// 工具标签函数（纯函数，无 DB 依赖）
// ============================================================

/// 截断字符串过滤器
/// 模板调用：{{ text|truncate(100) }}
fn truncate_filter(s: String, length: Option<usize>) -> String {
    let max_len = length.unwrap_or(100);
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_len {
        s
    } else {
        let truncated: String = chars[..max_len].iter().collect();
        format!("{}...", truncated)
    }
}

/// 相对时间函数（"3小时前"）
/// 模板调用：{{ time_ago(article.create_time) }}
fn time_ago_function(time_str: Option<String>) -> String {
    let time_str = match time_str {
        Some(s) if !s.is_empty() => s,
        _ => return String::new(),
    };

    // 尝试解析时间字符串
    let parse_formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%d",
    ];

    let parsed = parse_formats.iter().find_map(|fmt| {
        chrono::NaiveDateTime::parse_from_str(&time_str, fmt).ok()
            .or_else(|| chrono::NaiveDate::parse_from_str(&time_str, fmt).ok().map(|d| d.and_hms_opt(0, 0, 0).unwrap_or_default()))
    });

    let target = match parsed {
        Some(dt) => dt,
        None => return time_str,
    };

    let now = chrono::Local::now().naive_local();
    let diff = now.signed_duration_since(target);

    if diff.num_seconds() < 0 {
        return time_str;
    }

    let secs = diff.num_seconds();
    let mins = diff.num_minutes();
    let hours = diff.num_hours();
    let days = diff.num_days();

    if secs < 60 {
        "刚刚".to_string()
    } else if mins < 60 {
        format!("{}分钟前", mins)
    } else if hours < 24 {
        format!("{}小时前", hours)
    } else if days < 30 {
        format!("{}天前", days)
    } else if days < 365 {
        format!("{}个月前", days / 30)
    } else {
        format!("{}年前", days / 365)
    }
}

/// 分页数据函数
/// 模板调用：{% set pg = pagination(1, 100, 10) %}
/// 返回：{ current, total, page_size, total_pages, has_prev, has_next, prev_page, next_page }
#[derive(Serialize)]
struct PaginationData {
    current: u64,
    total: u64,
    page_size: u64,
    total_pages: u64,
    has_prev: bool,
    has_next: bool,
    prev_page: u64,
    next_page: u64,
    pages: Vec<u64>,
}

fn pagination_function(page: Option<u64>, total: Option<u64>, page_size: Option<u64>) -> Value {
    let current = page.unwrap_or(1).max(1);
    let total = total.unwrap_or(0);
    let page_size = page_size.unwrap_or(10).max(1);
    let total_pages = if total == 0 { 1 } else { (total + page_size - 1) / page_size };

    // 生成页码列表（最多显示 7 页）
    let mut pages = Vec::new();
    let start = if current > 4 { current - 3 } else { 1 };
    let end = (start + 6).min(total_pages);
    for i in start..=end {
        pages.push(i);
    }

    let pg = PaginationData {
        current,
        total,
        page_size,
        total_pages,
        has_prev: current > 1,
        has_next: current < total_pages,
        prev_page: if current > 1 { current - 1 } else { 1 },
        next_page: if current < total_pages { current + 1 } else { total_pages },
        pages,
    };

    Value::from_serialize(&pg)
}




