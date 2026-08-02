//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究。
//!

use actix_web::{HttpRequest, HttpResponse, web};
use actix_web::http::header::ContentType;
use minijinja::context;
use serde_json::json;
use crate::core::errors::error::{Error, Result};
use crate::core::kit::template::{get_template, get_template_a_with_cms};
use crate::core::web::entity::common::QueryUrl;
use crate::core::kit::global::AppState;
use crate::core::web::tags::cms_tags::CmsTagData;
use crate::modules::articles::entity::category;
use crate::modules::articles::model::article::QueryPageRequest;
use crate::modules::articles::model::category::CategoryModel;
use crate::modules::articles::service::article_service;
use crate::modules::product::model::product::ProductListQuery;
use crate::modules::product::service::product_service;
use crate::modules::website::service::{template_user_data_service, website_service};

/// 读取站点 + 预取 CMS 标签数据（所有页面通用）
async fn prepare_site_and_cms(state: &web::Data<AppState>) -> Result<(crate::modules::website::model::website::SiteDetailVO, i64, CmsTagData)> {
    let db = &state.db;
    let site = website_service::find_default(db).await?;
    let site_id = site.id.unwrap_or_default();
    let mut cms_data = CmsTagData::fetch(db).await.unwrap_or_default();
    // 注入站点模式，供 cart_button 等标签使用
    cms_data.site_mode = site.site_mode.unwrap_or(1);
    Ok((site, site_id, cms_data))
}

/// G-2.4: 根据 User-Agent 检测是否为移动端
/// 简单 UA 关键词匹配，覆盖主流移动设备（iPhone/Android/iPad/Windows Phone 等）
fn is_mobile_request(req: &HttpRequest) -> bool {
    if let Some(ua) = req.headers().get("user-agent").and_then(|v| v.to_str().ok()) {
        let ua_lower = ua.to_lowercase();
        const MOBILE_KEYWORDS: &[&str] = &[
            "mobile", "android", "iphone", "ipod", "ipad", "windows phone",
            "blackberry", "opera mini", "mobile safari", "ucbrowser",
            "micromessenger", "huawei", "xiaomi", "oppo", "vivo",
        ];
        MOBILE_KEYWORDS.iter().any(|kw| ua_lower.contains(kw))
    } else {
        false
    }
}

/// G-2.4: 根据请求选择有效的 template_id
/// 移动端且站点配置了 mobile_template_id（非 0）时使用 mobile_template_id，否则使用 template_id
fn select_template_id(site: &crate::modules::website::model::website::SiteDetailVO, req: &HttpRequest) -> Option<i64> {
    if is_mobile_request(req) {
        // 移动端：优先 mobile_template_id，为空或 0 时回退 template_id
        match site.mobile_template_id {
            Some(mid) if mid > 0 => Some(mid),
            _ => site.template_id,
        }
    } else {
        site.template_id
    }
}

/// 构建 canonical URL（基于站点绑定域名 + 路径）
fn build_canonical_url(site: &crate::modules::website::model::website::SiteDetailVO, path: &str) -> String {
    let domain = site.bind_domain.clone()
        .filter(|d| !d.is_empty())
        .or_else(|| site.domain.clone())
        .unwrap_or_default();
    let domain = domain.trim_end_matches('/');
    let path = path.trim_start_matches('/');
    if domain.is_empty() {
        format!("/{}", path)
    } else if domain.starts_with("http") {
        format!("{}/{}", domain, path)
    } else {
        format!("https://{}/{}", domain, path)
    }
}

/// 获取站点导航栏目（is_show=1）
async fn get_nav_categories(db: &sea_orm::DbConn, site_id: i64) -> Vec<category::Model> {
    let categories = CategoryModel::find_all(db, site_id).await.unwrap_or_default();
    categories.into_iter()
        .filter(|c| c.is_show == Some(1))
        .collect()
}

/// 渲染模板并返回 HTML 响应
fn render_html(template_text: &str, ctx: minijinja::Value, cms_data: &CmsTagData) -> Result<HttpResponse> {
    let rendered = get_template_a_with_cms(template_text, ctx, cms_data)?;
    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
}

/// 获取指定 type_id 的模板内容
async fn get_template_text(db: &sea_orm::DbConn, template_id: &Option<i64>, type_id: i32) -> Result<String> {
    let template_data = template_user_data_service::find_latest_by_template_and_type(
        db, template_id, &Some(type_id)
    ).await?;
    Ok(template_data.temptext.unwrap_or_default())
}

/// CMS 首页 — 单站模式，直接取默认站点渲染
pub async fn cms_index(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    let (site, site_id, cms_data) = prepare_site_and_cms(&state).await?;
    let nav_categories = get_nav_categories(db, site_id).await;
    let site_mode = site.site_mode.unwrap_or(1);
    // G-2.4: 根据 User-Agent 选择 PC/移动端模板
    let effective_template_id = select_template_id(&site, &req);
    let template_text = get_template_text(db, &effective_template_id, 1).await?;

    // 同时注入扁平变量（site_name/keywords/description/site_domain）
    // 兼容使用扁平变量的模板
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
        categories => &nav_categories,
        site_mode => site_mode,
        site_id => site_id,
        canonical_url => build_canonical_url(&site, "/"),
    );
    render_html(&template_text, ctx, &cms_data)
}

/// 栏目页 — 根据短链接匹配栏目，支持封面模式和列表模式
pub async fn category_page(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<QueryUrl>
) -> Result<HttpResponse> {
    let db = &state.db;
    let (site, site_id, cms_data) = prepare_site_and_cms(&state).await?;
    let short_url = path.short_url.clone();
    // G-2.4: 根据 User-Agent 选择 PC/移动端模板
    let effective_template_id = select_template_id(&site, &req);

    let category = CategoryModel::find_by_short_url(db, short_url.unwrap_or_default()).await?
        .ok_or_else(|| Error::from("栏目不存在"))?;

    let category_id = category.id;
    let nav_categories = get_nav_categories(db, site_id).await;

    // 扁平变量兼容
    let site_name = site.site_name.clone().unwrap_or_default();
    let site_domain = site.domain.clone().unwrap_or_default();
    let category_name = category.category_name.clone().unwrap_or_default();
    let category_description = category.description.clone().unwrap_or_default();
    // 面包屑 HTML（DEDECMS 风格，已转义）
    let breadcrumbs = cms_data.render_breadcrumbs_html(category_id);

    if category.page_type == Some(1) && category.page_template_data_id.is_some() {
        // 封面模式
        let template_data = template_user_data_service::get_by_detail(db, &category.page_template_data_id).await?;
        let children = CategoryModel::find_all(db, site_id).await.unwrap_or_default();
        let sub_categories: Vec<category::Model> = children.into_iter()
            .filter(|c| c.parent_id == Some(category_id))
            .collect();

        let ctx = context!(
            site => &site,
            site_name => site_name,
            site_domain => site_domain,
            category => &category,
            category_name => category_name,
            category_description => category_description,
            breadcrumbs => &breadcrumbs,
            children => &sub_categories,
            categories => &nav_categories,
            site_id => site_id,
            canonical_url => build_canonical_url(&site, &format!("/category/{}", category.short_url.clone().unwrap_or_default())),
        );
        let template_text = template_data.temptext.unwrap_or_default();
        render_html(&template_text, ctx, &cms_data)
    } else {
        // 列表模式：分页查询文章
        let query_data = QueryPageRequest {
            title: None,
            page_num: Some(1),
            page_size: Some(10),
            category_id: Option::from(category_id),
            website_id: Option::from(site_id),
            status: Some(2),
        };
        let article_page = article_service::get_by_page(db, query_data).await?;

        let ctx = context!(
            site => &site,
            site_name => site_name,
            site_domain => site_domain,
            category => &category,
            category_name => category_name,
            category_description => category_description,
            breadcrumbs => &breadcrumbs,
            list => &article_page.items,
            total => &article_page.total,
            page => &1,
            categories => &nav_categories,
            site_id => site_id,
            canonical_url => build_canonical_url(&site, &format!("/category/{}", category.short_url.clone().unwrap_or_default())),
        );
        let template_text = get_template_text(db, &effective_template_id, 2).await?;
        render_html(&template_text, ctx, &cms_data)
    }
}

/// 文章详情
pub async fn article_detail(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<QueryUrl>
) -> Result<HttpResponse> {
    let db = &state.db;
    let (site, site_id, cms_data) = prepare_site_and_cms(&state).await?;
    let short_url = path.short_url.clone();
    let nav_categories = get_nav_categories(db, site_id).await;
    // G-2.4: 根据 User-Agent 选择 PC/移动端模板
    let effective_template_id = select_template_id(&site, &req);

    // 扁平变量兼容
    let site_name = site.site_name.clone().unwrap_or_default();
    let site_domain = site.domain.clone().unwrap_or_default();

    if let Some(article) = article_service::get_by_short_url(db, &site.id, &short_url).await? {
        // 浏览量自增（异步执行，不阻塞渲染）
        let article_id_str = article.id.clone().unwrap_or_default();
        let article_id: i64 = article_id_str.parse().unwrap_or(0);
        if article_id > 0 {
            let _ = article_service::increment_view_count(db, article_id).await;
        }

        // 查询上一篇/下一篇文章
        let category_id = article.category_id;
        let (prev_article, next_article) = article_service::find_prev_next(db, category_id, article_id).await.unwrap_or((None, None));

        // 面包屑：基于文章所属栏目构建
        let breadcrumbs = category_id
            .map(|cid| cms_data.render_breadcrumbs_html(cid))
            .unwrap_or_default();

        let ctx = context!(
            site => &site,
            site_name => site_name,
            site_domain => site_domain,
            article => &article,
            field => &article,
            prev_article => &prev_article,
            next_article => &next_article,
            breadcrumbs => &breadcrumbs,
            categories => &nav_categories,
            site_id => site_id,
            canonical_url => build_canonical_url(&site, &format!("/article/{}", short_url.clone().unwrap_or_default())),
        );
        let template_text = get_template_text(db, &effective_template_id, 3).await?;
        render_html(&template_text, ctx, &cms_data)
    } else {
        let ctx = context!(error => "查询出现错误，请联系管理员");
        let out = get_template("default/404.html", ctx)?;
        Ok(HttpResponse::Ok().content_type(ContentType::html()).body(out))
    }
}

/// 产品列表页
pub async fn product_list(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let db = &state.db;
    let (site, site_id, cms_data) = prepare_site_and_cms(&state).await?;
    let nav_categories = get_nav_categories(db, site_id).await;
    // G-2.4: 根据 User-Agent 选择 PC/移动端模板
    let effective_template_id = select_template_id(&site, &req);

    // 解析查询参数
    let query_params = web::Query::<std::collections::HashMap<String, String>>::from_query(req.query_string())
        .unwrap_or_else(|_| web::Query(std::collections::HashMap::new()));
    let category_id = query_params.get("category_id")
        .and_then(|s| s.parse::<i64>().ok());
    let keyword = query_params.get("keyword").cloned();
    let order = query_params.get("order").cloned().unwrap_or_else(|| "new".to_string());
    let page_num = query_params.get("page")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(1);

    let query = ProductListQuery {
        keywords: keyword,
        category_id,
        is_active: Some(true),
        page_num: Some(page_num),
        page_size: Some(12),
    };
    let (products, total, _total_pages) = product_service::get_list(db, &query).await?;

    let ctx = context!(
        site => &site,
        products => &products,
        total => &total,
        current_page => &page_num,
        current_category_id => &category_id,
        order => &order,
        categories => &nav_categories,
        site_id => site_id,
        site_mode => site.site_mode.unwrap_or(1),
        canonical_url => build_canonical_url(&site, "/product"),
    );
    let template_text = get_template_text(db, &effective_template_id, 7).await?;
    render_html(&template_text, ctx, &cms_data)
}

/// 产品详情页
pub async fn product_detail(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<QueryUrl>
) -> Result<HttpResponse> {
    let db = &state.db;
    let (site, site_id, cms_data) = prepare_site_and_cms(&state).await?;
    let nav_categories = get_nav_categories(db, site_id).await;
    // G-2.4: 根据 User-Agent 选择 PC/移动端模板
    let effective_template_id = select_template_id(&site, &req);

    let product_id = path.short_url.clone()
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or_else(|| Error::from("产品ID不能为空"))?;

    let product = product_service::get_detail(db, product_id).await?;

    let ctx = context!(
        site => &site,
        product => &product,
        categories => &nav_categories,
        site_id => site_id,
        site_mode => site.site_mode.unwrap_or(1),
        canonical_url => build_canonical_url(&site, &format!("/product/{}", product_id)),
    );
    let template_text = get_template_text(db, &effective_template_id, 8).await?;
    render_html(&template_text, ctx, &cms_data)
}

/// 搜索页
pub async fn search(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let db = &state.db;
    let (site, site_id, cms_data) = prepare_site_and_cms(&state).await?;
    let nav_categories = get_nav_categories(db, site_id).await;
    // G-2.4: 根据 User-Agent 选择 PC/移动端模板
    let effective_template_id = select_template_id(&site, &req);

    let query_params = web::Query::<std::collections::HashMap<String, String>>::from_query(req.query_string())
        .unwrap_or_else(|_| web::Query(std::collections::HashMap::new()));
    let keyword = query_params.get("keyword").cloned().unwrap_or_default();
    let page_num = query_params.get("page")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(1);

    // 搜索文章
    let query_data = QueryPageRequest {
        title: Some(keyword.clone()),
        page_num: Some(page_num),
        page_size: Some(10),
        category_id: None,
        website_id: Option::from(site_id),
        status: Some(2),
    };
    let article_page = article_service::get_by_page(db, query_data).await?;

    // 搜索产品
    let product_query = ProductListQuery {
        keywords: Some(keyword.clone()),
        category_id: None,
        is_active: Some(true),
        page_num: Some(1),
        page_size: Some(10),
    };
    let (products, _p_total, _) = product_service::get_list(db, &product_query).await?;

    // 扁平变量兼容（搜索页复用列表模板 type_id=2，需传 list 而非 articles）
    let site_name = site.site_name.clone().unwrap_or_default();
    let site_domain = site.domain.clone().unwrap_or_default();
    let category_name = format!("搜索: {}", keyword);
    let category_description = format!("包含 \"{}\" 的搜索结果", keyword);
    let category_ctx = context!(
        category_name => &category_name,
        description => &category_description,
        short_url => "",
    );

    let ctx = context!(
        site => &site,
        site_name => site_name,
        site_domain => site_domain,
        keyword => &keyword,
        category => &category_ctx,
        category_name => category_name,
        category_description => category_description,
        articles => &article_page.items,
        list => &article_page.items,
        total => &article_page.total,
        products => &products,
        current_page => &page_num,
        categories => &nav_categories,
        site_id => site_id,
        canonical_url => build_canonical_url(&site, "/search"),
    );
    let template_text = get_template_text(db, &effective_template_id, 2).await?;
    render_html(&template_text, ctx, &cms_data)
}

/// 自定义页面
pub async fn custom_page(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<QueryUrl>
) -> Result<HttpResponse> {
    let db = &state.db;
    let (site, site_id, cms_data) = prepare_site_and_cms(&state).await?;
    let nav_categories = get_nav_categories(db, site_id).await;
    // G-2.4: 根据 User-Agent 选择 PC/移动端模板
    let effective_template_id = select_template_id(&site, &req);
    let page_code = path.short_url.clone().unwrap_or_default();

    // 从 CMS 标签数据中获取自定义页面
    if let Some(page_data) = cms_data.get_page(&page_code) {
        let site_name = site.site_name.clone().unwrap_or_default();
        let site_domain = site.domain.clone().unwrap_or_default();

        // type_id=6 模板原为栏目封面页，引用 category.category_name / children。
        // 自定义页面复用该模板，需构造 category 对象（从 page 派生）与空 children，
        // 使模板变量解析不报 "undefined value"。
        let page_name = page_data.get_attr("pageName").ok()
            .and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default();
        let page_title = page_data.get_attr("pageTitle").ok()
            .and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default();
        let seo_description = page_data.get_attr("seoDescription").ok()
            .and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default();
        let display_name = if page_title.is_empty() { page_name } else { page_title };
        // 使用 minijinja context! 构造，确保属性访问（category.category_name）可用
        let category_ctx = context!(
            category_name => &display_name,
            description => &seo_description,
        );
        let empty_children: Vec<serde_json::Value> = Vec::new();

        let ctx = context!(
            site => &site,
            site_name => site_name,
            site_domain => site_domain,
            page => &page_data,
            category => &category_ctx,
            children => &empty_children,
            categories => &nav_categories,
            site_id => site_id,
            canonical_url => build_canonical_url(&site, &format!("/page/{}", path.short_url.clone().unwrap_or_default())),
        );
        let template_text = get_template_text(db, &effective_template_id, 6).await?;
        render_html(&template_text, ctx, &cms_data)
    } else {
        let ctx = context!(error => "页面不存在");
        let out = get_template("default/404.html", ctx)?;
        Ok(HttpResponse::Ok().content_type(ContentType::html()).body(out))
    }
}

/// HTML 站点地图页
pub async fn sitemap(
    state: web::Data<AppState>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    let db = &state.db;
    let (site, site_id, cms_data) = prepare_site_and_cms(&state).await?;
    let nav_categories = get_nav_categories(db, site_id).await;

    // 获取所有栏目
    let all_categories = CategoryModel::find_all(db, site_id).await.unwrap_or_default();

    // 获取已发布文章
    let query_data = QueryPageRequest {
        title: None,
        page_num: Some(1),
        page_size: Some(1000),
        category_id: None,
        website_id: Option::from(site_id),
        status: Some(2),
    };
    let article_page = article_service::get_by_page(db, query_data).await?;

    // 获取产品
    let product_query = ProductListQuery {
        keywords: None,
        category_id: None,
        is_active: Some(true),
        page_num: Some(1),
        page_size: Some(1000),
    };
    let (products, _p_total, _) = product_service::get_list(db, &product_query).await?;

    let ctx = context!(
        site => &site,
        categories => &all_categories,
        articles => &article_page.items,
        products => &products,
        nav_categories => &nav_categories,
        site_id => site_id,
    );

    // 直接渲染一个内联的站点地图模板
    let sitemap_template = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <title>{{ site.site_name }} - 站点地图</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 960px; margin: 0 auto; padding: 20px; }
        h1 { color: #333; border-bottom: 2px solid #2563eb; padding-bottom: 10px; }
        h2 { color: #2563eb; margin-top: 30px; }
        ul { list-style: none; padding-left: 20px; }
        li { margin: 5px 0; }
        a { color: #666; text-decoration: none; }
        a:hover { color: #2563eb; text-decoration: underline; }
        .section { margin-bottom: 30px; }
    </style>
</head>
<body>
    <h1>{{ site.site_name }} 站点地图</h1>

    <div class="section">
        <h2>导航页面</h2>
        <ul>
            <li><a href="/">首页</a></li>
            {% for c in nav_categories %}
            <li><a href="/category/{{ c.short_url }}">{{ c.category_name }}</a></li>
            {% endfor %}
            <li><a href="/product">产品中心</a></li>
        </ul>
    </div>

    <div class="section">
        <h2>产品列表</h2>
        <ul>
            {% for p in products %}
            <li><a href="/product/{{ p.id }}">{{ p.name }}</a></li>
            {% endfor %}
        </ul>
    </div>

    <div class="section">
        <h2>文章列表</h2>
        <ul>
            {% for a in articles %}
            <li><a href="/article/{{ a.short_url }}">{{ a.title }}</a></li>
            {% endfor %}
        </ul>
    </div>
</body>
</html>"#;

    render_html(sitemap_template, ctx, &cms_data)
}

/// XML 站点地图（供搜索引擎抓取）
pub async fn sitemap_xml(
    state: web::Data<AppState>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    let db = &state.db;
    let (site, site_id, _cms_data) = prepare_site_and_cms(&state).await?;
    let site_domain = site.domain.clone().unwrap_or_default();

    // 获取已发布文章
    let query_data = QueryPageRequest {
        title: None,
        page_num: Some(1),
        page_size: Some(1000),
        category_id: None,
        website_id: Option::from(site_id),
        status: Some(2),
    };
    let article_page = article_service::get_by_page(db, query_data).await?;

    // 获取栏目
    let all_categories = CategoryModel::find_all(db, site_id).await.unwrap_or_default();

    // 获取产品
    let product_query = ProductListQuery {
        keywords: None,
        category_id: None,
        is_active: Some(true),
        page_num: Some(1),
        page_size: Some(1000),
    };
    let (products, _p_total, _) = product_service::get_list(db, &product_query).await?;

    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");

    // 首页
    xml.push_str(&format!("  <url>\n    <loc>{}/</loc>\n    <changefreq>daily</changefreq>\n    <priority>1.0</priority>\n  </url>\n", site_domain));

    // 栏目页
    for c in &all_categories {
        if c.is_show == Some(1) {
            if let Some(short_url) = &c.short_url {
                xml.push_str(&format!("  <url>\n    <loc>{}/category/{}</loc>\n    <changefreq>daily</changefreq>\n    <priority>0.8</priority>\n  </url>\n", site_domain, short_url));
            }
        }
    }

    // 文章页
    for a in &article_page.items {
        if let Some(short_url) = &a.short_url {
            if !short_url.is_empty() {
                xml.push_str(&format!("  <url>\n    <loc>{}/article/{}</loc>\n    <changefreq>weekly</changefreq>\n    <priority>0.6</priority>\n  </url>\n", site_domain, short_url));
            }
        }
    }

    // 产品页
    for p in &products {
        let pid = p.id.unwrap_or_default();
        xml.push_str(&format!("  <url>\n    <loc>{}/product/{}</loc>\n    <changefreq>weekly</changefreq>\n    <priority>0.6</priority>\n  </url>\n", site_domain, pid));
    }

    xml.push_str("</urlset>");

    Ok(HttpResponse::Ok()
        .content_type("application/xml; charset=utf-8")
        .body(xml))
}

/// robots.txt
pub async fn robots_txt(
    state: web::Data<AppState>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    let (site, _site_id, _cms_data) = prepare_site_and_cms(&state).await?;
    let site_domain = site.domain.clone().unwrap_or_default();

    // 优先使用后台配置的自定义 robots.txt 内容
    let robots = if let Some(custom) = &site.robots_content {
        if !custom.trim().is_empty() {
            // 替换 {domain} 占位符为站点域名
            custom.replace("{domain}", &site_domain)
        } else {
            format!(
                "User-agent: *\nAllow: /\nDisallow: /api/\nSitemap: {}/sitemap.xml\n",
                site_domain
            )
        }
    } else {
        format!(
            "User-agent: *\nAllow: /\nDisallow: /api/\nSitemap: {}/sitemap.xml\n",
            site_domain
        )
    };

    Ok(HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(robots))
}
