//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究。
//!

use serde::Serialize;

/// 单个标签的元数据，用于前端模板编辑器的标签文档面板
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct TemplateTag {
    /// 标签名称（函数名或变量名）
    pub name: &'static str,
    /// 分类
    pub category: &'static str,
    /// 语法（完整调用形式）
    pub syntax: &'static str,
    /// 简短说明
    pub description: &'static str,
    /// 完整示例代码片段（点击「插入」时会写入编辑器）
    pub example: &'static str,
    /// 参数说明（可选）
    pub params: Option<&'static [(&'static str, &'static str)]>,
}

/// 返回所有可用的模板标签元数据
///
/// 与 `template.rs::register_cms_functions` 中注册的标签保持一致，
/// 新增标签时需同步更新此函数。
pub fn get_all_template_tags() -> Vec<TemplateTag> {
    vec![
        // ── 模板变量 ──
        TemplateTag {
            name: "tpl_var",
            category: "模板变量",
            syntax: "{{ tpl_var(\"key\") }}",
            description: "取后台「模板变量」表中指定 key 的值（如客服电话、备案号等）",
            example: "{{ tpl_var(\"phone\") }}",
            params: Some(&[
                ("key", "变量 key，如 phone / email / icp / logo_text"),
            ]),
        },
        // ── 导航 ──
        TemplateTag {
            name: "get_navigations",
            category: "导航",
            syntax: "{{ get_navigations(nav_type) }}",
            description: "取导航列表，nav_type 可选 header（顶部）或 footer（底部）",
            example: "{% for nav in get_navigations(\"header\") %}\n  <a href=\"{{ nav.webUrl }}\">{{ nav.name }}</a>\n{% endfor %}",
            params: Some(&[
                ("nav_type", "导航类型：header / footer，默认 header"),
            ]),
        },
        // ── Banner / 区块 / 页面 ──
        TemplateTag {
            name: "get_banners",
            category: "Banner与区块",
            syntax: "{{ get_banners(position) }}",
            description: "取指定 position 的 Banner 列表",
            example: "{% for b in get_banners(\"home_top\") %}\n  <a href=\"{{ b.linkUrl }}\"><img src=\"{{ b.imageUrl }}\" alt=\"{{ b.altText }}\"></a>\n{% endfor %}",
            params: Some(&[
                ("position", "Banner 位置标识，如 home_top / home_banner"),
            ]),
        },
        TemplateTag {
            name: "get_block",
            category: "Banner与区块",
            syntax: "{{ get_block(code) }}",
            description: "取指定 code 的区块内容（自定义 HTML 片段）",
            example: "{{ get_block(\"about_us\") }}",
            params: Some(&[
                ("code", "区块标识 code"),
            ]),
        },
        TemplateTag {
            name: "get_page",
            category: "Banner与区块",
            syntax: "{{ get_page(code) }}",
            description: "取指定 code 的自定义页面内容",
            example: "{{ get_page(\"contact\") }}",
            params: Some(&[
                ("code", "页面标识 code"),
            ]),
        },
        TemplateTag {
            name: "content_models",
            category: "Banner与区块",
            syntax: "{{ content_models() }}",
            description: "取所有内容模型列表",
            example: "{% for m in content_models() %}\n  <a href=\"/model/{{ m.modelCode }}\">{{ m.modelName }}</a>\n{% endfor %}",
            params: None,
        },
        // ── 友情链接 ──
        TemplateTag {
            name: "get_links",
            category: "友情链接",
            syntax: "{{ get_links(link_type) }}",
            description: "取指定类型的友情链接列表",
            example: "{% for l in get_links(0) %}\n  <a href=\"{{ l.linkUrl }}\" target=\"_blank\">{{ l.linkName }}</a>\n{% endfor %}",
            params: Some(&[
                ("link_type", "链接类型，数字，默认 0"),
            ]),
        },
        // ── 面包屑 ──
        TemplateTag {
            name: "get_breadcrumbs",
            category: "面包屑",
            syntax: "{{ get_breadcrumbs(category_id) }}",
            description: "返回面包屑数组 [{id, name, shortUrl}]，需配合 for 循环渲染",
            example: "{% for b in get_breadcrumbs(category.id) %}\n  <a href=\"/category/{{ b.shortUrl }}\">{{ b.name }}</a>\n{% endfor %}",
            params: Some(&[
                ("category_id", "栏目 ID"),
            ]),
        },
        TemplateTag {
            name: "get_breadcrumbs_html",
            category: "面包屑",
            syntax: "{{ get_breadcrumbs_html(category_id) }}",
            description: "直接返回面包屑 HTML 字符串（无需循环）",
            example: "{{ get_breadcrumbs_html(category.id) }}",
            params: Some(&[
                ("category_id", "栏目 ID"),
            ]),
        },
        // ── 文章 ──
        TemplateTag {
            name: "get_articles",
            category: "文章",
            syntax: "{{ get_articles(category_id, limit, page) }}",
            description: "取指定栏目的文章列表，limit 上限 50",
            example: "{% for a in get_articles(category.id, 10, 1) %}\n  <a href=\"/article/{{ a.id }}\">{{ a.title }}</a>\n{% endfor %}",
            params: Some(&[
                ("category_id", "栏目 ID"),
                ("limit", "每页数量，默认 10，上限 50"),
                ("page", "页码，默认 1"),
            ]),
        },
        TemplateTag {
            name: "get_recommend_articles",
            category: "文章",
            syntax: "{{ get_recommend_articles(limit) }}",
            description: "取推荐文章列表，limit 上限 20",
            example: "{% for a in get_recommend_articles(5) %}\n  <a href=\"/article/{{ a.id }}\">{{ a.title }}</a>\n{% endfor %}",
            params: Some(&[
                ("limit", "数量，默认 5，上限 20"),
            ]),
        },
        TemplateTag {
            name: "get_related_articles",
            category: "文章",
            syntax: "{{ get_related_articles(article_id, category_id, limit) }}",
            description: "取相关文章（同分类，排除当前文章）",
            example: "{% for a in get_related_articles(article.id, article.categoryId, 5) %}\n  <a href=\"/article/{{ a.id }}\">{{ a.title }}</a>\n{% endfor %}",
            params: Some(&[
                ("article_id", "当前文章 ID"),
                ("category_id", "栏目 ID"),
                ("limit", "数量，默认 5"),
            ]),
        },
        TemplateTag {
            name: "search_articles",
            category: "文章",
            syntax: "{{ search_articles(keyword, limit) }}",
            description: "按标题关键词搜索文章",
            example: "{% for a in search_articles(keyword, 10) %}\n  <a href=\"/article/{{ a.id }}\">{{ a.title }}</a>\n{% endfor %}",
            params: Some(&[
                ("keyword", "关键词"),
                ("limit", "数量，默认 10"),
            ]),
        },
        TemplateTag {
            name: "get_article_labels",
            category: "文章",
            syntax: "{{ get_article_labels(article_id) }}",
            description: "取文章的所有标签",
            example: "{% for l in get_article_labels(article.id) %}\n  <span class=\"tag\">{{ l.name }}</span>\n{% endfor %}",
            params: Some(&[
                ("article_id", "文章 ID"),
            ]),
        },
        TemplateTag {
            name: "get_articles_by_label",
            category: "文章",
            syntax: "{{ get_articles_by_label(tag_id, limit) }}",
            description: "按标签 ID 取文章列表",
            example: "{% for a in get_articles_by_label(1, 10) %}\n  <a href=\"/article/{{ a.id }}\">{{ a.title }}</a>\n{% endfor %}",
            params: Some(&[
                ("tag_id", "标签 ID"),
                ("limit", "数量，默认 10"),
            ]),
        },
        // ── 产品 / 栏目 ──
        TemplateTag {
            name: "get_products",
            category: "产品与栏目",
            syntax: "{{ get_products(category_id, limit, page, order) }}",
            description: "取产品列表，order 可选 new/price_asc/price_desc",
            example: "{% for p in get_products(category.id, 12, 1, \"new\") %}\n  <a href=\"/product/{{ p.id }}\">{{ p.name }}</a>\n{% endfor %}",
            params: Some(&[
                ("category_id", "栏目 ID"),
                ("limit", "每页数量，默认 10，上限 50"),
                ("page", "页码，默认 1"),
                ("order", "排序：new / price_asc / price_desc"),
            ]),
        },
        TemplateTag {
            name: "get_categories",
            category: "产品与栏目",
            syntax: "{{ get_categories(parent_id) }}",
            description: "取栏目树，parent_id=0 返回顶级栏目",
            example: "{% for c in get_categories(0) %}\n  <a href=\"/category/{{ c.shortUrl }}\">{{ c.categoryName }}</a>\n{% endfor %}",
            params: Some(&[
                ("parent_id", "父栏目 ID，0 表示顶级"),
            ]),
        },
        // ── 站点 / 电商 ──
        TemplateTag {
            name: "get_site_mode",
            category: "站点与电商",
            syntax: "{{ get_site_mode() }}",
            description: "取站点模式：1=展示型 2=交易型 3=混合型",
            example: "{% if get_site_mode() == 2 %}\n  <!-- 交易型，显示购物车 -->\n{% endif %}",
            params: None,
        },
        TemplateTag {
            name: "cart_button",
            category: "站点与电商",
            syntax: "{{ cart_button(product_id, site_mode) }}",
            description: "根据站点模式渲染「立即咨询/加购物车/立即购买」按钮 HTML",
            example: "{{ cart_button(product.id, site_mode) }}",
            params: Some(&[
                ("product_id", "产品 ID"),
                ("site_mode", "站点模式，1/2/3"),
            ]),
        },
        TemplateTag {
            name: "lead_form",
            category: "站点与电商",
            syntax: "{{ lead_form(product_id) }}",
            description: "渲染在线咨询表单 HTML（POST 到 /api/open/lead/submit）",
            example: "{{ lead_form(product.id) }}",
            params: Some(&[
                ("product_id", "产品 ID"),
            ]),
        },
        // ── 模板片段 ──
        TemplateTag {
            name: "include_template",
            category: "模板片段",
            syntax: "{{ include_template(name) }}",
            description: "引入 type_id=4 的模板片段（如页头/页脚）",
            example: "{{ include_template(\"header.html\") }}",
            params: Some(&[
                ("name", "模板片段名称"),
            ]),
        },
        // ── 媒体 ──
        TemplateTag {
            name: "get_media",
            category: "媒体",
            syntax: "{{ get_media(id) }}",
            description: "取单个媒体对象（含 url/title/alt 等）",
            example: "{% set m = get_media(1) %}\n<img src=\"{{ m.imageUrl }}\" alt=\"{{ m.title }}\">",
            params: Some(&[
                ("id", "媒体 ID"),
            ]),
        },
        TemplateTag {
            name: "media_url",
            category: "媒体",
            syntax: "{{ media_url(id) }}",
            description: "仅返回媒体 URL，配合 <img src> 使用更简洁",
            example: "<img src=\"{{ media_url(1) }}\" alt=\"\">",
            params: Some(&[
                ("id", "媒体 ID"),
            ]),
        },
        TemplateTag {
            name: "get_media_list",
            category: "媒体",
            syntax: "{{ get_media_list(category_id, limit) }}",
            description: "按分类取媒体列表，limit 上限 100",
            example: "{% for m in get_media_list(1, 20) %}\n  <img src=\"{{ m.imageUrl }}\" alt=\"{{ m.title }}\">\n{% endfor %}",
            params: Some(&[
                ("category_id", "媒体分类 ID"),
                ("limit", "数量，默认 20，上限 100"),
            ]),
        },
        TemplateTag {
            name: "get_media_gallery",
            category: "媒体",
            syntax: "{{ get_media_gallery(category_id, limit) }}",
            description: "取图廊（仅 file_type=1 的图片）",
            example: "{% for m in get_media_gallery(1, 20) %}\n  <img src=\"{{ m.imageUrl }}\" alt=\"{{ m.title }}\">\n{% endfor %}",
            params: Some(&[
                ("category_id", "媒体分类 ID"),
                ("limit", "数量，默认 20"),
            ]),
        },
        // ── SEO / 工具 ──
        TemplateTag {
            name: "og_tags",
            category: "SEO与工具",
            syntax: "{{ og_tags(title, description, image, url) }}",
            description: "生成 Open Graph + Twitter Card meta 标签 HTML",
            example: "{{ og_tags(article.title, article.description, article.titleImage, canonical_url) }}",
            params: Some(&[
                ("title", "标题"),
                ("description", "描述"),
                ("image", "图片 URL"),
                ("url", "页面 URL"),
            ]),
        },
        TemplateTag {
            name: "get_sitemap_html",
            category: "SEO与工具",
            syntax: "{{ get_sitemap_html() }}",
            description: "生成 HTML 站点地图",
            example: "{{ get_sitemap_html() }}",
            params: None,
        },
        TemplateTag {
            name: "get_model",
            category: "SEO与工具",
            syntax: "{{ get_model(model_code) }}",
            description: "按 model_code 取内容模型",
            example: "{% set m = get_model(\"news\") %}\n{{ m.modelName }}",
            params: Some(&[
                ("model_code", "模型标识 code"),
            ]),
        },
        TemplateTag {
            name: "get_model_articles",
            category: "SEO与工具",
            syntax: "{{ get_model_articles(model_code, limit) }}",
            description: "按模型取文章（实际降级为最新文章）",
            example: "{% for a in get_model_articles(\"news\", 10) %}\n  <a href=\"/article/{{ a.id }}\">{{ a.title }}</a>\n{% endfor %}",
            params: Some(&[
                ("model_code", "模型标识 code"),
                ("limit", "数量，默认 10"),
            ]),
        },
        // ── 过滤器 ──
        TemplateTag {
            name: "| default",
            category: "过滤器",
            syntax: "{{ value | default(\"默认值\") }}",
            description: "值为空时显示默认值",
            example: "{{ site_name | default(\"MxxCRM\") }}",
            params: Some(&[
                ("默认值", "为空时的替代值"),
            ]),
        },
        TemplateTag {
            name: "| truncate",
            category: "过滤器",
            syntax: "{{ text | truncate(100) }}",
            description: "截断字符串并追加 ...",
            example: "{{ article.description | truncate(100) }}",
            params: Some(&[
                ("length", "最大长度"),
            ]),
        },
        TemplateTag {
            name: "| to_json",
            category: "过滤器",
            syntax: "{{ value | to_json }}",
            description: "把值序列化为 JSON 字符串",
            example: "{{ article | to_json }}",
            params: None,
        },
        TemplateTag {
            name: "| format_time",
            category: "过滤器",
            syntax: "{{ time | format_time(\"%Y-%m-%d\") }}",
            description: "时间格式化（chrono 风格格式）",
            example: "{{ article.createTime | format_time(\"%Y-%m-%d %H:%M\") }}",
            params: Some(&[
                ("format", "时间格式，如 %Y-%m-%d"),
            ]),
        },
        TemplateTag {
            name: "time_ago",
            category: "过滤器",
            syntax: "{{ time_str | time_ago }}",
            description: "相对时间，如「3小时前」「2天前」",
            example: "{{ article.createTime | time_ago }}",
            params: None,
        },
        // ── 分页 ──
        TemplateTag {
            name: "pagination",
            category: "分页",
            syntax: "{{ pagination(page, total, page_size) }}",
            description: "返回分页数据 {current, total, page_size, total_pages, has_prev, has_next, prev_page, next_page, pages}",
            example: "{% set pg = pagination(current_page, total, 10) %}\n{% if pg.has_prev %}<a href=\"?page={{ pg.prev_page }}\">上一页</a>{% endif %}\n{% if pg.has_next %}<a href=\"?page={{ pg.next_page }}\">下一页</a>{% endif %}",
            params: Some(&[
                ("page", "当前页码"),
                ("total", "总条数"),
                ("page_size", "每页数量"),
            ]),
        },
        // ── 上下文变量 ──
        TemplateTag {
            name: "site_name",
            category: "上下文变量",
            syntax: "{{ site_name }}",
            description: "站点名称（控制器注入，非函数）",
            example: "<title>{{ site_name }}</title>",
            params: None,
        },
        TemplateTag {
            name: "site_domain",
            category: "上下文变量",
            syntax: "{{ site_domain }}",
            description: "站点域名（控制器注入）",
            example: "<span>{{ site_domain }}</span>",
            params: None,
        },
        TemplateTag {
            name: "keywords",
            category: "上下文变量",
            syntax: "{{ keywords }}",
            description: "站点 SEO 关键词（控制器注入）",
            example: "<meta name=\"keywords\" content=\"{{ keywords }}\">",
            params: None,
        },
        TemplateTag {
            name: "description",
            category: "上下文变量",
            syntax: "{{ description }}",
            description: "站点 SEO 描述（控制器注入）",
            example: "<meta name=\"description\" content=\"{{ description }}\">",
            params: None,
        },
        TemplateTag {
            name: "canonical_url",
            category: "上下文变量",
            syntax: "{{ canonical_url }}",
            description: "当前页面规范 URL（控制器注入）",
            example: "<link rel=\"canonical\" href=\"{{ canonical_url }}\">",
            params: None,
        },
        TemplateTag {
            name: "site_mode",
            category: "上下文变量",
            syntax: "{{ site_mode }}",
            description: "站点模式（控制器注入）：1=展示型 2=交易型 3=混合型",
            example: "{% if site_mode == 2 %}<!-- 交易型 -->{% endif %}",
            params: None,
        },
        TemplateTag {
            name: "category",
            category: "上下文变量",
            syntax: "{{ category }}",
            description: "当前栏目对象（栏目/文章/产品页注入），含 id/categoryName/shortUrl 等",
            example: "<h1>{{ category.categoryName }}</h1>",
            params: None,
        },
        TemplateTag {
            name: "article",
            category: "上下文变量",
            syntax: "{{ article }}",
            description: "当前文章对象（文章详情页注入），含 id/title/content/categoryId 等",
            example: "<h1>{{ article.title }}</h1>\n<div>{{ article.content }}</div>",
            params: None,
        },
        TemplateTag {
            name: "product",
            category: "上下文变量",
            syntax: "{{ product }}",
            description: "当前产品对象（产品详情页注入），含 id/name/price/content 等",
            example: "<h1>{{ product.name }}</h1>\n<p>价格：¥{{ product.price }}</p>",
            params: None,
        },
        TemplateTag {
            name: "page",
            category: "上下文变量",
            syntax: "{{ page }}",
            description: "当前页码（列表页注入）",
            example: "当前第 {{ page }} 页",
            params: None,
        },
        TemplateTag {
            name: "keyword",
            category: "上下文变量",
            syntax: "{{ keyword }}",
            description: "搜索关键词（搜索页注入）",
            example: "搜索关键词：{{ keyword }}",
            params: None,
        },
    ]
}
