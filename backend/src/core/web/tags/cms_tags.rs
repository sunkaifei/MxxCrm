//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!


use std::collections::HashMap;
use minijinja::Value;
use sea_orm::*;
use serde_json::json;
use crate::core::errors::error::Result;
use crate::modules::website::entity::{
    template_var, website_banner, website_block, website_page, content_model,
    navigation, website_links, template_data, website_media,
};
use crate::modules::articles::entity::category;
use crate::modules::articles::model::article::{ArticleListVO, QueryPageRequest};
use crate::modules::articles::service::article_service;
use crate::modules::product::model::product::ProductListQuery;
use crate::modules::product::service::product_service;

/// CMS 标签预取数据
///
/// 由于 minijinja 的 `add_function` 需要 `Fn` 闭包（同步），而数据库查询是异步的，
/// 因此采用预取数据模式：在渲染模板前先查询所有 CMS 数据放入该结构体，
/// 再通过闭包将数据暴露给模板标签函数使用。
#[derive(Default, Clone)]
pub struct CmsTagData {
    /// 模板变量（key -> value）
    pub template_vars: HashMap<String, String>,
    /// Banner 列表，按 position 分组
    pub banners: HashMap<String, Vec<Value>>,
    /// 区块内容，按 block_code 索引
    pub blocks: HashMap<String, Value>,
    /// 自定义页面，按 page_code 索引
    pub pages: HashMap<String, Value>,
    /// 内容模型列表
    pub content_models: Vec<Value>,
    /// 导航列表，按 nav_type 分组（header/footer）
    pub navigations: HashMap<String, Vec<Value>>,
    /// 友情链接列表，按 link_type 分组
    pub links: HashMap<String, Vec<Value>>,
    /// 栏目列表（用于面包屑、栏目导航等），按 id 索引
    pub categories: HashMap<i64, category::Model>,
    /// 预取的最新文章列表（默认 100 条，供 get_articles 标签使用）
    pub articles: Vec<ArticleListVO>,
    /// 预取的产品列表（默认 100 条，供 get_products 标签使用）
    pub products: Vec<Value>,
    /// 当前站点模式：1=展示型 2=交易型 3=混合型
    pub site_mode: i32,
    /// 模板片段（type_id=4 的模板数据），按 name 索引
    /// 供 include_template 标签使用
    pub template_fragments: HashMap<String, String>,
    /// 媒体库（最近 200 条），按 id 索引，供 get_media / media_url 等标签使用
    pub media: HashMap<i64, Value>,
    /// 媒体列表（按 category_id 分组），供 get_media_list / get_media_gallery 标签使用
    pub media_by_category: HashMap<i64, Vec<Value>>,
}

impl CmsTagData {
    /// 从数据库预取所有 CMS 数据
    ///
    /// 每个数据分区独立加载，单个分区失败不影响其他分区，
    /// 确保即使某张表缺失或字段不匹配，核心页面仍能渲染。
    pub async fn fetch(db: &DatabaseConnection) -> Result<Self> {
        let mut data = Self::default();

        // 1. 预取模板变量（失败不影响其他数据）
        if let Ok(vars) = template_var::Entity::find()
            .filter(template_var::Column::Deleted.eq(0))
            .filter(template_var::Column::Status.eq(1))
            .all(db).await
        {
            for v in vars {
                if let (Some(k), Some(val)) = (v.var_key, v.var_value) {
                    data.template_vars.insert(k, val);
                }
            }
        }

        // 2. 预取 Banners（按 position 分组，按 sort 升序）
        if let Ok(banners) = website_banner::Entity::find()
            .filter(website_banner::Column::Deleted.eq(0))
            .filter(website_banner::Column::Status.eq(1))
            .order_by_asc(website_banner::Column::Sort)
            .all(db).await
        {
            let mut banner_map: HashMap<String, Vec<Value>> = HashMap::new();
            for b in banners {
                let banner_value = json!({
                    "id": b.id,
                    "title": b.title,
                    "imageUrl": b.image_url,
                    "linkUrl": b.link_url,
                    "altText": b.alt_text,
                    "position": b.position,
                    "target": b.target,
                    "sort": b.sort,
                });
                let pos = b.position.unwrap_or_else(|| "home_top".to_string());
                banner_map.entry(pos).or_default().push(Value::from_serialize(&banner_value));
            }
            data.banners = banner_map;
        }

        // 3. 预取区块
        if let Ok(blocks) = website_block::Entity::find()
            .filter(website_block::Column::Deleted.eq(0))
            .filter(website_block::Column::Status.eq(1))
            .all(db).await
        {
            for blk in blocks {
                if let Some(code) = &blk.block_code {
                    let block_value = json!({
                        "id": blk.id,
                        "blockCode": blk.block_code,
                        "blockName": blk.block_name,
                        "blockType": blk.block_type,
                        "content": blk.content,
                        "imageUrl": blk.image_url,
                        "linkUrl": blk.link_url,
                    });
                    data.blocks.insert(code.clone(), Value::from_serialize(&block_value));
                }
            }
        }

        // 4. 预取页面
        if let Ok(pages) = website_page::Entity::find()
            .filter(website_page::Column::Deleted.eq(0))
            .filter(website_page::Column::Status.eq(1))
            .all(db).await
        {
            for p in pages {
                if let Some(code) = &p.page_code {
                    let page_value = json!({
                        "id": p.id,
                        "pageCode": p.page_code,
                        "pageName": p.page_name,
                        "pageTitle": p.page_title,
                        "pageContent": p.page_content,
                        "seoKeywords": p.seo_keywords,
                        "seoDescription": p.seo_description,
                    });
                    data.pages.insert(code.clone(), Value::from_serialize(&page_value));
                }
            }
        }

        // 5. 预取内容模型（按 sort 升序）
        if let Ok(models) = content_model::Entity::find()
            .filter(content_model::Column::Deleted.eq(0))
            .filter(content_model::Column::Status.eq(1))
            .order_by_asc(content_model::Column::Sort)
            .all(db).await
        {
            for m in models {
                let model_value = json!({
                    "id": m.id,
                    "modelCode": m.model_code,
                    "modelName": m.model_name,
                    "modelIcon": m.model_icon,
                    "description": m.description,
                });
                data.content_models.push(Value::from_serialize(&model_value));
            }
        }

        // 6. 预取导航（按 nav_type 分组，按 sort 升序）
        if let Ok(navs) = navigation::Entity::find()
            .order_by_asc(navigation::Column::Sort)
            .all(db).await
        {
            let mut nav_map: HashMap<String, Vec<Value>> = HashMap::new();
            for n in navs {
                // 根据 is_new_window_open 派生 target 字段（实体无 target 列）
                let target = if n.is_new_window_open.unwrap_or(0) == 1 { "_blank" } else { "_self" };
                let nav_value = json!({
                    "id": n.id,
                    "websiteId": n.website_id,
                    "parentId": n.parent_id,
                    "name": n.name,
                    "webUrl": n.web_url,
                    "value": n.value,
                    "dataType": n.data_type,
                    "navType": n.nav_type,
                    "sort": n.sort,
                    "isShow": n.is_show,
                    "isNewWindowOpen": n.is_new_window_open,
                    "target": target,
                });
                let nav_type = n.nav_type.unwrap_or_else(|| "header".to_string());
                nav_map.entry(nav_type).or_default().push(Value::from_serialize(&nav_value));
            }
            data.navigations = nav_map;
        }

        // 7. 预取友情链接（按 link_type 分组，按 sort 升序）
        if let Ok(links) = website_links::Entity::find()
            .filter(website_links::Column::Deleted.eq(0))
            .filter(website_links::Column::Status.eq(1))
            .order_by_asc(website_links::Column::Sort)
            .all(db).await
        {
            let mut link_map: HashMap<String, Vec<Value>> = HashMap::new();
            for l in links {
                let link_value = json!({
                    "id": l.id,
                    "websiteId": l.website_id,
                    "linkType": l.link_type,
                    "linkName": l.link_name,
                    "linkUrl": l.link_url,
                    "linkLogo": l.link_logo,
                    "status": l.status,
                    "sort": l.sort,
                });
                let link_type = format!("{}", l.link_type.unwrap_or(0));
                link_map.entry(link_type).or_default().push(Value::from_serialize(&link_value));
            }
            data.links = link_map;
        }

        // 8. 预取栏目（用于面包屑、栏目导航等），按 id 索引
        if let Ok(cats) = category::Entity::find().all(db).await {
            for c in cats {
                data.categories.insert(c.id, c);
            }
        }

        // 9. 预取最新文章（100 条，供 get_articles 标签使用）
        // 失败不阻塞渲染，仅影响文章列表标签
        let article_query = QueryPageRequest {
            title: None,
            page_num: Some(1),
            page_size: Some(100),
            category_id: None,
            website_id: None,
            status: Some(2), // 已发布
        };
        if let Ok(article_page) = article_service::get_by_page(db, article_query).await {
            data.articles = article_page.items;
        }

        // 10. 预取产品（100 条，供 get_products 标签使用）
        let product_query = ProductListQuery {
            keywords: None,
            category_id: None,
            warehouse_id: None,
            brand_id: None,
            is_active: Some(true),
            page_num: Some(1),
            page_size: Some(100),
        };
        if let Ok((products, _total, _pages)) = product_service::get_list(db, &product_query).await {
            data.products = products.into_iter()
                .map(|p| Value::from_serialize(&p))
                .collect();
        }

        // 11. 预取模板片段（type_id=4，供 include_template 标签使用）
        // 失败不阻塞渲染
        if let Ok(fragments) = template_data::Entity::find()
            .filter(template_data::Column::TypeId.eq(4))
            .filter(template_data::Column::Deleted.eq(0))
            .filter(template_data::Column::Status.eq(1))
            .all(db).await
        {
            for f in fragments {
                if let Some(name) = f.name.clone() {
                    data.template_fragments.insert(name, f.temptext.unwrap_or_default());
                }
            }
        }

        // 12. 预取媒体库（最近 200 条，供 get_media / media_url 等标签使用）
        // 失败不阻塞渲染
        if let Ok(medias) = website_media::Entity::find()
            .filter(website_media::Column::Deleted.eq(0))
            .filter(website_media::Column::Status.eq(1))
            .order_by_desc(website_media::Column::Id)
            .limit(200)
            .all(db).await
        {
            let mut by_cat: HashMap<i64, Vec<Value>> = HashMap::new();
            for m in medias {
                let mid = m.id;
                let media_value = json!({
                    "id": m.id,
                    "originalName": m.original_name,
                    "storageName": m.storage_name,
                    "filePath": m.file_path,
                    "fileUrl": m.file_url,
                    "fileExt": m.file_ext,
                    "fileSize": m.file_size,
                    "fileType": m.file_type,
                    "mimeType": m.mime_type,
                    "width": m.width,
                    "height": m.height,
                    "thumbSmall": m.thumb_small,
                    "thumbMedium": m.thumb_medium,
                    "thumbLarge": m.thumb_large,
                    "altText": m.alt_text,
                    "title": m.title,
                    "caption": m.caption,
                    "categoryId": m.category_id,
                });
                let cat_id = m.category_id.unwrap_or(0);
                let media_val = Value::from_serialize(&media_value);
                by_cat.entry(cat_id).or_default().push(media_val.clone());
                data.media.insert(mid, media_val);
            }
            data.media_by_category = by_cat;
        }

        Ok(data)
    }

    /// 转换为 minijinja context Value
    #[allow(dead_code)]
    pub fn to_value(&self) -> Value {
        let mut ctx: HashMap<&str, Value> = HashMap::new();
        ctx.insert("tpl_vars", Value::from_serialize(&self.template_vars));
        ctx.insert("banners", Value::from_serialize(&self.banners));
        ctx.insert("blocks", Value::from_serialize(&self.blocks));
        ctx.insert("pages", Value::from_serialize(&self.pages));
        ctx.insert("content_models", Value::from_serialize(&self.content_models));
        ctx.insert("navigations", Value::from_serialize(&self.navigations));
        ctx.insert("links", Value::from_serialize(&self.links));
        Value::from_serialize(&ctx)
    }

    /// 获取模板变量值
    #[allow(dead_code)]
    pub fn get_var(&self, key: &str) -> String {
        self.template_vars.get(key).cloned().unwrap_or_default()
    }

    /// 获取指定位置的 Banners
    #[allow(dead_code)]
    pub fn get_banners(&self, position: &str) -> Vec<Value> {
        self.banners.get(position).cloned().unwrap_or_default()
    }

    /// 获取指定 code 的区块
    #[allow(dead_code)]
    pub fn get_block(&self, code: &str) -> Option<&Value> {
        self.blocks.get(code)
    }

    /// 获取指定 code 的页面
    #[allow(dead_code)]
    pub fn get_page(&self, code: &str) -> Option<&Value> {
        self.pages.get(code)
    }

    /// 获取指定类型的导航
    #[allow(dead_code)]
    pub fn get_navigations(&self, nav_type: &str) -> Vec<Value> {
        self.navigations.get(nav_type).cloned().unwrap_or_default()
    }

    /// 获取指定类型的友情链接
    #[allow(dead_code)]
    pub fn get_links(&self, link_type: &str) -> Vec<Value> {
        self.links.get(link_type).cloned().unwrap_or_default()
    }

    /// 获取文章列表（从预取数据中过滤）
    ///
    /// 参数：
    /// - `category_id`: 栏目 ID，None 或 0 表示不限栏目
    /// - `limit`: 返回条数，默认 10
    /// - `page`: 页码，默认 1（基于 limit 分页）
    pub fn get_articles(&self, category_id: Option<i64>, limit: Option<usize>, page: Option<usize>) -> Vec<Value> {
        let limit = limit.unwrap_or(10).min(50);
        let page = page.unwrap_or(1).max(1);
        let offset = (page - 1) * limit;

        let filtered: Vec<&ArticleListVO> = self.articles.iter()
            .filter(|a| {
                match category_id {
                    Some(cid) if cid > 0 => a.category_id == Some(cid),
                    _ => true,
                }
            })
            .collect();

        filtered.into_iter()
            .skip(offset)
            .take(limit)
            .map(|a| Value::from_serialize(a))
            .collect()
    }

    /// 获取推荐文章（isrecommend=1）
    pub fn get_recommend_articles(&self, limit: Option<usize>) -> Vec<Value> {
        let limit = limit.unwrap_or(5).min(20);
        self.articles.iter()
            .filter(|a| a.isrecommend.unwrap_or(0) == 1)
            .take(limit)
            .map(|a| Value::from_serialize(a))
            .collect()
    }

    /// 获取产品列表（从预取数据中过滤）
    ///
    /// 参数：
    /// - `category_id`: 栏目 ID，None 或 0 表示不限栏目
    /// - `limit`: 返回条数，默认 10
    /// - `page`: 页码，默认 1
    /// - `order`: 排序方式 "new"=最新（默认）
    pub fn get_products(&self, category_id: Option<i64>, limit: Option<usize>, page: Option<usize>, _order: Option<String>) -> Vec<Value> {
        let limit = limit.unwrap_or(10).min(50);
        let page = page.unwrap_or(1).max(1);
        let offset = (page - 1) * limit;

        let filtered: Vec<&Value> = self.products.iter()
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
            .collect();

        filtered.into_iter()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect()
    }

    /// 获取栏目树（按 parent_id 过滤，默认返回顶级栏目）
    pub fn get_categories(&self, parent_id: Option<i64>) -> Vec<Value> {
        let pid = parent_id.unwrap_or(0);
        self.categories.values()
            .filter(|c| {
                if pid == 0 {
                    c.parent_id.unwrap_or(0) == 0
                } else {
                    c.parent_id == Some(pid)
                }
            })
            .map(|c| {
                Value::from_serialize(&json!({
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
    }

    /// 根据栏目 id 构建面包屑路径（从根到当前栏目）
    /// 返回 Vec<(id, name, short_url)>，已按从根到当前的顺序排列
    /// 防止循环引用：最多向上回溯 20 层
    pub fn get_breadcrumbs(&self, category_id: i64) -> Vec<(i64, String, String)> {
        let mut chain: Vec<(i64, String, String)> = Vec::new();
        let mut current_id = Some(category_id);
        let mut depth = 0;
        let mut visited = std::collections::HashSet::new();

        while let Some(cid) = current_id {
            if depth >= 20 || visited.contains(&cid) {
                break;
            }
            visited.insert(cid);
            match self.categories.get(&cid) {
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
        chain
    }

    /// 根据栏目 id 渲染面包屑 HTML（DEDECMS 风格）
    /// 输出形如：<a href="/">首页</a> &gt; <a href="/category/news">新闻</a> &gt; 行业动态
    pub fn render_breadcrumbs_html(&self, category_id: i64) -> String {
        let chain = self.get_breadcrumbs(category_id);
        let mut parts: Vec<String> = Vec::new();
        parts.push(r#"<a href="/" class="breadcrumb-home">首页</a>"#.to_string());
        let total = chain.len();
        for (idx, (_id, name, short_url)) in chain.iter().enumerate() {
            if name.is_empty() {
                continue;
            }
            let is_last = idx == total.saturating_sub(1);
            // HTML 转义简单处理
            let esc_name = escape_html(name);
            if is_last || short_url.is_empty() {
                parts.push(format!(r#"<span class="breadcrumb-current">{}</span>"#, esc_name));
            } else {
                let esc_url = escape_html(short_url);
                parts.push(format!(
                    r#"<a href="/category/{}" class="breadcrumb-link">{}</a>"#,
                    esc_url, esc_name
                ));
            }
        }
        parts.join(" &gt; ")
    }
}

/// 简单 HTML 转义（用于面包屑输出，避免 XSS）
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

// ============================================================
// 表单标签（cart_button / lead_form）
// ============================================================
//
// 这两个标签是纯 HTML 生成器，不依赖数据库查询，
// 可直接作为 minijinja 的 `Fn` 闭包注册。
// 渲染逻辑根据 `site_mode` 分流：
//   - 展示型 (1)：渲染"立即咨询"按钮 + lead_form 弹窗
//   - 交易型 (2)：渲染"加入购物车"/"立即购买"按钮
//   - 混合型 (3)：两者都渲染

/// 购物车/咨询按钮：根据站点模式渲染不同按钮
///
/// 模板调用：{{ cart_button(product_id, site_mode) }}
///
/// - site_mode=1（展示型）：渲染"立即咨询"按钮，点击展开 lead_form
/// - site_mode=2（交易型）：渲染"加入购物车"+"立即购买"按钮
/// - site_mode=3（混合型）：渲染全部按钮
pub fn cart_button_html(product_id: i64, site_mode: Option<i32>) -> String {
    let mode = site_mode.unwrap_or(1);
    let pid = product_id.to_string();
    let mut html = String::new();
    html.push_str(r#"<div class="cms-cart-button" data-product-id=""#);
    html.push_str(&pid);
    html.push_str(r#"">"#);

    match mode {
        1 => {
            // 展示型：仅"立即咨询"
            html.push_str(r#"<button type="button" class="btn btn-lead" onclick="document.getElementById('lead-form-"#);
            html.push_str(&pid);
            html.push_str(r#"').style.display='block'">立即咨询</button>"#);
        }
        2 => {
            // 交易型：加购物车 + 立即购买
            html.push_str(r#"<button type="button" class="btn btn-cart" onclick="cmsAddCart("#);
            html.push_str(&pid);
            html.push_str(r#"')">加入购物车</button>"#);
            html.push_str(r#"<button type="button" class="btn btn-buy" onclick="cmsBuyNow("#);
            html.push_str(&pid);
            html.push_str(r#"')">立即购买</button>"#);
        }
        3 => {
            // 混合型：咨询 + 加购物车 + 立即购买
            html.push_str(r#"<button type="button" class="btn btn-lead" onclick="document.getElementById('lead-form-"#);
            html.push_str(&pid);
            html.push_str(r#"').style.display='block'">立即咨询</button>"#);
            html.push_str(r#"<button type="button" class="btn btn-cart" onclick="cmsAddCart("#);
            html.push_str(&pid);
            html.push_str(r#"')">加入购物车</button>"#);
            html.push_str(r#"<button type="button" class="btn btn-buy" onclick="cmsBuyNow("#);
            html.push_str(&pid);
            html.push_str(r#"')">立即购买</button>"#);
        }
        _ => {
            html.push_str(r#"<button type="button" class="btn btn-lead" onclick="document.getElementById('lead-form-"#);
            html.push_str(&pid);
            html.push_str(r#"').style.display='block'">立即咨询</button>"#);
        }
    }

    html.push_str("</div>");
    html
}

/// 线索/咨询表单：渲染表单 HTML
///
/// 模板调用：{{ lead_form(product_id=p.id) }} 或 {{ lead_form() }}
///
/// 提交到 POST /api/open/lead/submit，字段：name / phone / email / content / product_id
pub fn lead_form_html(product_id: Option<i64>) -> String {
    let pid = product_id.unwrap_or(0);
    let mut html = String::new();

    // 表单容器（默认隐藏，由 cart_button 触发显示）
    html.push_str(&format!(r#"<div id="lead-form-{}" class="cms-lead-form" style="display:none">"#, pid));
    html.push_str(r#"<div class="lead-form-inner">"#);
    html.push_str(r#"<h3 class="lead-form-title">在线咨询</h3>"#);
    html.push_str(r#"<form action="/api/open/lead/submit" method="post" class="lead-form">"#);

    // 隐藏字段：产品 ID
    if pid > 0 {
        html.push_str(&format!(r#"<input type="hidden" name="product_id" value="{}">"#, pid));
    }

    // 姓名
    html.push_str(r#"<div class="form-row"><label>姓名 <span class="req">*</span></label>"#);
    html.push_str(r#"<input type="text" name="name" required placeholder="请输入您的姓名"></div>"#);

    // 电话
    html.push_str(r#"<div class="form-row"><label>电话 <span class="req">*</span></label>"#);
    html.push_str(r#"<input type="tel" name="phone" required placeholder="请输入联系电话"></div>"#);

    // 邮箱
    html.push_str(r#"<div class="form-row"><label>邮箱</label>"#);
    html.push_str(r#"<input type="email" name="email" placeholder="请输入邮箱（选填）"></div>"#);

    // 留言内容
    html.push_str(r#"<div class="form-row"><label>留言内容</label>"#);
    html.push_str(r#"<textarea name="content" rows="3" placeholder="请输入您的需求（选填）"></textarea></div>"#);

    // 提交按钮
    html.push_str(r#"<div class="form-row"><button type="submit" class="btn btn-submit">提交咨询</button>"#);
    html.push_str(r#"<button type="button" class="btn btn-cancel" onclick="this.closest('.cms-lead-form').style.display='none'">取消</button></div>"#);

    html.push_str(r#"</form></div></div>"#);
    html
}
