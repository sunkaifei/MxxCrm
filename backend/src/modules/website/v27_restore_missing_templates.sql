-- ============================================================
-- v27_restore_missing_templates.sql
-- 恢复缺失的关键模板：type_id 1(首页) 2(栏目页) 3(文章详情) 6(自定义页面) 8(产品详情)
-- ============================================================

BEGIN;

-- type_id=1 首页模板
INSERT INTO mxx_template_user_data (id, template_id, type_id, name, temptext, sort, status, deleted, create_time)
SELECT 1888888888888888901, 1, 1, '首页模板',
'<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ site.site_name }} - 首页</title>
    <meta name="keywords" content="{{ site.keywords }}">
    <meta name="description" content="{{ site.description }}">
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: "Microsoft YaHei", Arial, sans-serif; color: #333; background: #f5f5f5; }
        .header { background: #fff; border-bottom: 2px solid #2563eb; padding: 15px 0; }
        .container { max-width: 1200px; margin: 0 auto; padding: 0 15px; }
        .header .container { display: flex; justify-content: space-between; align-items: center; }
        .logo { font-size: 24px; font-weight: bold; color: #2563eb; text-decoration: none; }
        .nav-menu { display: flex; gap: 25px; list-style: none; }
        .nav-menu a { color: #555; text-decoration: none; font-size: 15px; }
        .nav-menu a:hover { color: #2563eb; }
        .banner { background: linear-gradient(135deg, #2563eb, #7c3aed); color: #fff; padding: 60px 0; text-align: center; }
        .banner h1 { font-size: 36px; margin-bottom: 15px; }
        .banner p { font-size: 18px; opacity: 0.9; }
        .section { padding: 40px 0; }
        .section-title { text-align: center; font-size: 28px; margin-bottom: 30px; color: #1a202c; }
        .article-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 25px; }
        .article-card { background: #fff; border-radius: 8px; overflow: hidden; box-shadow: 0 2px 8px rgba(0,0,0,0.08); transition: transform 0.2s; }
        .article-card:hover { transform: translateY(-4px); }
        .article-card img { width: 100%; height: 200px; object-fit: cover; }
        .article-info { padding: 20px; }
        .article-info h3 { font-size: 18px; margin-bottom: 10px; }
        .article-info h3 a { color: #1a202c; text-decoration: none; }
        .article-info p { color: #718096; font-size: 14px; line-height: 1.6; }
        .article-meta { margin-top: 10px; font-size: 13px; color: #a0aec0; }
        .product-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 20px; }
        .product-card { background: #fff; border-radius: 8px; overflow: hidden; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }
        .product-card img { width: 100%; height: 180px; object-fit: cover; }
        .product-info { padding: 15px; }
        .product-info h3 { font-size: 16px; margin-bottom: 8px; }
        .product-info h3 a { color: #333; text-decoration: none; }
        .product-price { color: #e53e3e; font-size: 18px; font-weight: bold; }
        .footer { background: #2d3748; color: #cbd5e0; padding: 30px 0; margin-top: 40px; }
        .footer a { color: #cbd5e0; text-decoration: none; }
        .footer-links { display: flex; gap: 30px; list-style: none; justify-content: center; margin-bottom: 15px; }
        .footer-copyright { text-align: center; font-size: 14px; color: #718096; }
    </style>
</head>
<body>
    <div class="header">
        <div class="container">
            <a href="/" class="logo">{{ site.site_name }}</a>
            <ul class="nav-menu">
                <li><a href="/">首页</a></li>
                <li><a href="/product">产品中心</a></li>
                {% for c in categories %}
                <li><a href="/category/{{ c.short_url }}">{{ c.category_name }}</a></li>
                {% endfor %}
            </ul>
        </div>
    </div>

    <div class="banner">
        <div class="container">
            <h1>{{ site.site_name }}</h1>
            <p>{{ site.description }}</p>
        </div>
    </div>

    {% if articles %}
    <div class="section">
        <div class="container">
            <h2 class="section-title">最新资讯</h2>
            <div class="article-grid">
                {% for a in articles %}
                <div class="article-card">
                    <a href="/article/{{ a.short_url }}">
                        <img src="{{ a.title_image || '' }}/static/default/images/no-image.svg" alt="{{ a.title }}">
                    </a>
                    <div class="article-info">
                        <h3><a href="/article/{{ a.short_url }}">{{ a.title }}</a></h3>
                        <p>{{ a.description }}</p>
                        <div class="article-meta">{{ a.create_time }}</div>
                    </div>
                </div>
                {% endfor %}
            </div>
        </div>
    </div>
    {% endif %}

    {% if products %}
    <div class="section" style="background: #fff;">
        <div class="container">
            <h2 class="section-title">热门产品</h2>
            <div class="product-grid">
                {% for p in products %}
                <div class="product-card">
                    <a href="/product/{{ p.id }}">
                        <img src="{{ p.image_url || '' }}/static/default/images/no-image.svg" alt="{{ p.name }}">
                    </a>
                    <div class="product-info">
                        <h3><a href="/product/{{ p.id }}">{{ p.name }}</a></h3>
                        {% if p.sale_price %}
                        <p class="product-price">¥{{ p.sale_price }}</p>
                        {% endif %}
                    </div>
                </div>
                {% endfor %}
            </div>
        </div>
    </div>
    {% endif %}

    <div class="footer">
        <div class="container">
            <ul class="footer-links">
                <li><a href="/">首页</a></li>
                {% for c in categories %}
                <li><a href="/category/{{ c.short_url }}">{{ c.category_name }}</a></li>
                {% endfor %}
            </ul>
            <div class="footer-copyright">
                <p>{{ site.copyright || site.site_name }}</p>
                <p>{{ site.icp }}</p>
            </div>
        </div>
    </div>
</body>
</html>',
1, 1, 0, CURRENT_TIMESTAMP
WHERE NOT EXISTS (SELECT 1 FROM mxx_template_user_data WHERE template_id = 1 AND type_id = 1);

-- type_id=2 栏目页/分类页
INSERT INTO mxx_template_user_data (id, template_id, type_id, name, temptext, sort, status, deleted, create_time)
SELECT 1888888888888888902, 1, 2, '栏目页',
'<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ category.category_name }} - {{ site.site_name }}</title>
    <meta name="keywords" content="{{ category.seo_keywords || category.category_name }}">
    <meta name="description" content="{{ category.seo_description || category.description }}">
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: "Microsoft YaHei", Arial, sans-serif; color: #333; background: #f5f5f5; }
        .header { background: #fff; border-bottom: 2px solid #2563eb; padding: 15px 0; }
        .container { max-width: 1200px; margin: 0 auto; padding: 0 15px; }
        .header .container { display: flex; justify-content: space-between; align-items: center; }
        .logo { font-size: 24px; font-weight: bold; color: #2563eb; text-decoration: none; }
        .nav-menu { display: flex; gap: 25px; list-style: none; }
        .nav-menu a { color: #555; text-decoration: none; font-size: 15px; }
        .nav-menu a:hover { color: #2563eb; }
        .breadcrumb { padding: 15px 0; color: #999; font-size: 14px; }
        .breadcrumb a { color: #666; text-decoration: none; }
        .article-list { background: #fff; border-radius: 8px; padding: 20px; }
        .article-item { display: flex; gap: 20px; padding: 20px 0; border-bottom: 1px solid #eee; }
        .article-item:last-child { border-bottom: none; }
        .article-item img { width: 200px; height: 130px; object-fit: cover; border-radius: 6px; }
        .article-content h3 { font-size: 18px; margin-bottom: 8px; }
        .article-content h3 a { color: #1a202c; text-decoration: none; }
        .article-content p { color: #718096; font-size: 14px; line-height: 1.6; margin-bottom: 8px; }
        .article-meta { font-size: 13px; color: #a0aec0; }
        .pagination { text-align: center; padding: 30px 0; }
        .pagination a, .pagination span { display: inline-block; padding: 8px 14px; margin: 0 3px; border: 1px solid #ddd; border-radius: 4px; text-decoration: none; color: #666; }
        .pagination .current { background: #2563eb; color: #fff; border-color: #2563eb; }
        .footer { background: #2d3748; color: #cbd5e0; padding: 30px 0; margin-top: 40px; }
        .footer a { color: #cbd5e0; text-decoration: none; }
        .footer-links { display: flex; gap: 30px; list-style: none; justify-content: center; margin-bottom: 15px; }
        .footer-copyright { text-align: center; font-size: 14px; color: #718096; }
    </style>
</head>
<body>
    <div class="header">
        <div class="container">
            <a href="/" class="logo">{{ site.site_name }}</a>
            <ul class="nav-menu">
                <li><a href="/">首页</a></li>
                <li><a href="/product">产品中心</a></li>
                {% for c in categories %}
                <li><a href="/category/{{ c.short_url }}">{{ c.category_name }}</a></li>
                {% endfor %}
            </ul>
        </div>
    </div>

    <div class="container">
        <div class="breadcrumb">
            <a href="/">首页</a> &gt; {{ category.category_name }}
        </div>

        <div class="article-list">
            {% if articles %}
            {% for a in articles %}
            <div class="article-item">
                {% if a.title_image %}
                <img src="{{ a.title_image }}" alt="{{ a.title }}">
                {% endif %}
                <div class="article-content">
                    <h3><a href="/article/{{ a.short_url }}">{{ a.title }}</a></h3>
                    <p>{{ a.description }}</p>
                    <div class="article-meta">{{ a.create_time }} {% if a.author %}· {{ a.author }}{% endif %}</div>
                </div>
            </div>
            {% endfor %}
            {% else %}
            <p style="text-align: center; padding: 40px; color: #999;">暂无文章</p>
            {% endif %}
        </div>

        {% if pagination %}
        <div class="pagination">
            {% if pagination.has_prev %}
            <a href="/category/{{ category.short_url }}?page={{ pagination.prev_page }}">上一页</a>
            {% endif %}
            {% for p in pagination.pages %}
            {% if p == pagination.current_page %}
            <span class="current">{{ p }}</span>
            {% else %}
            <a href="/category/{{ category.short_url }}?page={{ p }}">{{ p }}</a>
            {% endif %}
            {% endfor %}
            {% if pagination.has_next %}
            <a href="/category/{{ category.short_url }}?page={{ pagination.next_page }}">下一页</a>
            {% endif %}
        </div>
        {% endif %}
    </div>

    <div class="footer">
        <div class="container">
            <ul class="footer-links">
                <li><a href="/">首页</a></li>
                {% for c in categories %}
                <li><a href="/category/{{ c.short_url }}">{{ c.category_name }}</a></li>
                {% endfor %}
            </ul>
            <div class="footer-copyright">
                <p>{{ site.copyright || site.site_name }}</p>
            </div>
        </div>
    </div>
</body>
</html>',
1, 1, 0, CURRENT_TIMESTAMP
WHERE NOT EXISTS (SELECT 1 FROM mxx_template_user_data WHERE template_id = 1 AND type_id = 2);

-- type_id=3 文章详情页
INSERT INTO mxx_template_user_data (id, template_id, type_id, name, temptext, sort, status, deleted, create_time)
SELECT 1888888888888888903, 1, 3, '文章详情页',
'<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ article.title }} - {{ site.site_name }}</title>
    <meta name="keywords" content="{{ article.seo_keywords || article.title }}">
    <meta name="description" content="{{ article.seo_description || article.description }}">
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: "Microsoft YaHei", Arial, sans-serif; color: #333; background: #f5f5f5; }
        .header { background: #fff; border-bottom: 2px solid #2563eb; padding: 15px 0; }
        .container { max-width: 1200px; margin: 0 auto; padding: 0 15px; }
        .header .container { display: flex; justify-content: space-between; align-items: center; }
        .logo { font-size: 24px; font-weight: bold; color: #2563eb; text-decoration: none; }
        .nav-menu { display: flex; gap: 25px; list-style: none; }
        .nav-menu a { color: #555; text-decoration: none; font-size: 15px; }
        .nav-menu a:hover { color: #2563eb; }
        .breadcrumb { padding: 15px 0; color: #999; font-size: 14px; }
        .breadcrumb a { color: #666; text-decoration: none; }
        .article-detail { background: #fff; border-radius: 8px; padding: 40px; }
        .article-title { font-size: 28px; margin-bottom: 15px; color: #1a202c; }
        .article-meta { color: #a0aec0; font-size: 14px; margin-bottom: 25px; padding-bottom: 15px; border-bottom: 1px solid #eee; }
        .article-content { line-height: 1.8; font-size: 16px; }
        .article-content img { max-width: 100%; height: auto; }
        .article-nav { display: flex; justify-content: space-between; margin-top: 40px; padding-top: 20px; border-top: 1px solid #eee; }
        .article-nav a { color: #2563eb; text-decoration: none; }
        .footer { background: #2d3748; color: #cbd5e0; padding: 30px 0; margin-top: 40px; }
        .footer a { color: #cbd5e0; text-decoration: none; }
        .footer-links { display: flex; gap: 30px; list-style: none; justify-content: center; margin-bottom: 15px; }
        .footer-copyright { text-align: center; font-size: 14px; color: #718096; }
    </style>
</head>
<body>
    <div class="header">
        <div class="container">
            <a href="/" class="logo">{{ site.site_name }}</a>
            <ul class="nav-menu">
                <li><a href="/">首页</a></li>
                <li><a href="/product">产品中心</a></li>
                {% for c in categories %}
                <li><a href="/category/{{ c.short_url }}">{{ c.category_name }}</a></li>
                {% endfor %}
            </ul>
        </div>
    </div>

    <div class="container">
        <div class="breadcrumb">
            <a href="/">首页</a>
            {% if category %}&gt; <a href="/category/{{ category.short_url }}">{{ category.category_name }}</a>{% endif %}
            &gt; 正文
        </div>

        <div class="article-detail">
            <h1 class="article-title">{{ article.title }}</h1>
            <div class="article-meta">
                {% if article.author %}作者: {{ article.author }}{% endif %}
                {% if article.create_time %} | 发布时间: {{ article.create_time }}{% endif %}
                {% if article.count_view is defined %} | 浏览: {{ article.count_view }}{% endif %}
            </div>
            <div class="article-content">
                {{ article.content | safe }}
            </div>
            <div class="article-nav">
                <div>
                    {% if prev_article %}
                    <a href="/article/{{ prev_article.short_url }}">上一篇: {{ prev_article.title }}</a>
                    {% else %}
                    <span style="color: #999;">已是第一篇</span>
                    {% endif %}
                </div>
                <div>
                    {% if next_article %}
                    <a href="/article/{{ next_article.short_url }}">下一篇: {{ next_article.title }}</a>
                    {% else %}
                    <span style="color: #999;">已是最后一篇</span>
                    {% endif %}
                </div>
            </div>
        </div>
    </div>

    <div class="footer">
        <div class="container">
            <ul class="footer-links">
                <li><a href="/">首页</a></li>
                {% for c in categories %}
                <li><a href="/category/{{ c.short_url }}">{{ c.category_name }}</a></li>
                {% endfor %}
            </ul>
            <div class="footer-copyright">
                <p>{{ site.copyright || site.site_name }}</p>
            </div>
        </div>
    </div>
</body>
</html>',
1, 1, 0, CURRENT_TIMESTAMP
WHERE NOT EXISTS (SELECT 1 FROM mxx_template_user_data WHERE template_id = 1 AND type_id = 3);

-- type_id=6 自定义页面
INSERT INTO mxx_template_user_data (id, template_id, type_id, name, temptext, sort, status, deleted, create_time)
SELECT 1888888888888888906, 1, 6, '自定义页面',
'<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ page.page_title || page.page_name }} - {{ site.site_name }}</title>
    <meta name="keywords" content="{{ page.seo_keywords }}">
    <meta name="description" content="{{ page.seo_description }}">
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: "Microsoft YaHei", Arial, sans-serif; color: #333; background: #f5f5f5; }
        .header { background: #fff; border-bottom: 2px solid #2563eb; padding: 15px 0; }
        .container { max-width: 1200px; margin: 0 auto; padding: 0 15px; }
        .header .container { display: flex; justify-content: space-between; align-items: center; }
        .logo { font-size: 24px; font-weight: bold; color: #2563eb; text-decoration: none; }
        .nav-menu { display: flex; gap: 25px; list-style: none; }
        .nav-menu a { color: #555; text-decoration: none; font-size: 15px; }
        .nav-menu a:hover { color: #2563eb; }
        .breadcrumb { padding: 15px 0; color: #999; font-size: 14px; }
        .breadcrumb a { color: #666; text-decoration: none; }
        .page-content { background: #fff; border-radius: 8px; padding: 40px; min-height: 400px; line-height: 1.8; }
        .page-content h1 { color: #2563eb; margin-bottom: 20px; }
        .page-content h2 { margin-top: 25px; margin-bottom: 15px; }
        .page-content p { margin-bottom: 15px; }
        .footer { background: #2d3748; color: #cbd5e0; padding: 30px 0; margin-top: 40px; }
        .footer a { color: #cbd5e0; text-decoration: none; }
        .footer-links { display: flex; gap: 30px; list-style: none; justify-content: center; margin-bottom: 15px; }
        .footer-copyright { text-align: center; font-size: 14px; color: #718096; }
    </style>
</head>
<body>
    <div class="header">
        <div class="container">
            <a href="/" class="logo">{{ site.site_name }}</a>
            <ul class="nav-menu">
                <li><a href="/">首页</a></li>
                <li><a href="/product">产品中心</a></li>
                {% for c in categories %}
                <li><a href="/category/{{ c.short_url }}">{{ c.category_name }}</a></li>
                {% endfor %}
            </ul>
        </div>
    </div>

    <div class="container">
        <div class="breadcrumb">
            <a href="/">首页</a> &gt; {{ page.page_name }}
        </div>
        <div class="page-content">
            {{ page.page_content | safe }}
        </div>
    </div>

    <div class="footer">
        <div class="container">
            <ul class="footer-links">
                <li><a href="/">首页</a></li>
                {% for c in categories %}
                <li><a href="/category/{{ c.short_url }}">{{ c.category_name }}</a></li>
                {% endfor %}
            </ul>
            <div class="footer-copyright">
                <p>{{ site.copyright || site.site_name }}</p>
            </div>
        </div>
    </div>
</body>
</html>',
1, 1, 0, CURRENT_TIMESTAMP
WHERE NOT EXISTS (SELECT 1 FROM mxx_template_user_data WHERE template_id = 1 AND type_id = 6);

-- type_id=8 产品详情页
INSERT INTO mxx_template_user_data (id, template_id, type_id, name, temptext, sort, status, deleted, create_time)
SELECT 1888888888888888908, 1, 8, '产品详情页',
'<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ product.name }} - {{ site.site_name }}</title>
    <meta name="keywords" content="{{ product.name }}">
    <meta name="description" content="{{ product.description }}">
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: "Microsoft YaHei", Arial, sans-serif; color: #333; background: #f5f5f5; }
        .header { background: #fff; border-bottom: 2px solid #2563eb; padding: 15px 0; }
        .container { max-width: 1200px; margin: 0 auto; padding: 0 15px; }
        .header .container { display: flex; justify-content: space-between; align-items: center; }
        .logo { font-size: 24px; font-weight: bold; color: #2563eb; text-decoration: none; }
        .nav-menu { display: flex; gap: 25px; list-style: none; }
        .nav-menu a { color: #555; text-decoration: none; font-size: 15px; }
        .nav-menu a:hover { color: #2563eb; }
        .breadcrumb { padding: 15px 0; color: #999; font-size: 14px; }
        .breadcrumb a { color: #666; text-decoration: none; }
        .product-detail { background: #fff; border-radius: 8px; padding: 30px; display: flex; gap: 40px; }
        .product-image { flex: 0 0 400px; }
        .product-image img { width: 100%; border-radius: 8px; }
        .product-info { flex: 1; }
        .product-info h1 { font-size: 26px; margin-bottom: 15px; }
        .product-price { color: #e53e3e; font-size: 28px; font-weight: bold; margin: 20px 0; }
        .product-desc { color: #718096; line-height: 1.8; margin-bottom: 20px; }
        .product-attrs { margin-top: 20px; }
        .product-attrs dl { display: flex; margin-bottom: 10px; }
        .product-attrs dt { width: 100px; color: #999; }
        .product-attrs dd { flex: 1; }
        .btn-contact { display: inline-block; background: #2563eb; color: #fff; padding: 12px 30px; border-radius: 6px; text-decoration: none; font-size: 16px; margin-top: 20px; }
        .footer { background: #2d3748; color: #cbd5e0; padding: 30px 0; margin-top: 40px; }
        .footer a { color: #cbd5e0; text-decoration: none; }
        .footer-links { display: flex; gap: 30px; list-style: none; justify-content: center; margin-bottom: 15px; }
        .footer-copyright { text-align: center; font-size: 14px; color: #718096; }
    </style>
</head>
<body>
    <div class="header">
        <div class="container">
            <a href="/" class="logo">{{ site.site_name }}</a>
            <ul class="nav-menu">
                <li><a href="/">首页</a></li>
                <li><a href="/product">产品中心</a></li>
                {% for c in categories %}
                <li><a href="/category/{{ c.short_url }}">{{ c.category_name }}</a></li>
                {% endfor %}
            </ul>
        </div>
    </div>

    <div class="container">
        <div class="breadcrumb">
            <a href="/">首页</a> &gt; <a href="/product">产品中心</a> &gt; {{ product.name }}
        </div>
        <div class="product-detail">
            <div class="product-image">
                <img src="{{ product.image_url || '' }}/static/default/images/no-image.svg" alt="{{ product.name }}">
            </div>
            <div class="product-info">
                <h1>{{ product.name }}</h1>
                {% if product.sale_price %}
                <div class="product-price">¥{{ product.sale_price }}</div>
                {% endif %}
                <div class="product-desc">{{ product.description }}</div>
                <div class="product-attrs">
                    {% if product.model %}<dl><dt>型号:</dt><dd>{{ product.model }}</dd></dl>{% endif %}
                    {% if product.brand %}<dl><dt>品牌:</dt><dd>{{ product.brand }}</dd></dl>{% endif %}
                    {% if product.unit %}<dl><dt>单位:</dt><dd>{{ product.unit }}</dd></dl>{% endif %}
                </div>
                <a href="/page/contact" class="btn-contact">咨询此产品</a>
            </div>
        </div>
    </div>

    <div class="footer">
        <div class="container">
            <ul class="footer-links">
                <li><a href="/">首页</a></li>
                {% for c in categories %}
                <li><a href="/category/{{ c.short_url }}">{{ c.category_name }}</a></li>
                {% endfor %}
            </ul>
            <div class="footer-copyright">
                <p>{{ site.copyright || site.site_name }}</p>
            </div>
        </div>
    </div>
</body>
</html>',
1, 1, 0, CURRENT_TIMESTAMP
WHERE NOT EXISTS (SELECT 1 FROM mxx_template_user_data WHERE template_id = 1 AND type_id = 8);

COMMIT;

-- 验证
SELECT type_id, name, status FROM mxx_template_user_data WHERE template_id = 1 AND deleted = 0 ORDER BY type_id;
