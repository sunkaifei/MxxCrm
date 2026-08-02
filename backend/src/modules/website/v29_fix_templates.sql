-- v29: 修复文章详情模板和栏目/搜索模板
-- 问题1: minijinja context! 宏中 article 变量名冲突，改用 field
-- 问题2: ArticleDetailVO/ArticleListVO 有 rename_all=camelCase，字段名需用 camelCase
-- 问题3: 搜索页 category 是 minijinja Value，不支持 .field 访问，改用扁平变量
-- 问题4: site.site_name 为 None，改用扁平变量 site_name

-- =====================================================
-- 1. 修复文章详情模板 (type_id=3)
-- =====================================================
UPDATE mxx_template_user_data SET temptext = '<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ field.title }} - {{ site_name }}</title>
    <meta name="keywords" content="{{ field.title }}">
    <meta name="description" content="{{ field.description }}">
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
            <a href="/" class="logo">{{ site_name }}</a>
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
            <a href="/">首页</a> &gt; 正文
        </div>

        <div class="article-detail">
            <h1 class="article-title">{{ field.title }}</h1>
            <div class="article-meta">
                {% if field.author %}作者: {{ field.author }}{% endif %}
                {% if field.createTime %} | 发布时间: {{ field.createTime }}{% endif %}
            </div>
            <div class="article-content">
                {{ field.content | safe }}
            </div>
            <div class="article-nav">
                <div>
                    {% if prev_article %}
                    <a href="/article/{{ prev_article.shortUrl }}">上一篇: {{ prev_article.title }}</a>
                    {% else %}
                    <span style="color: #999;">已是第一篇</span>
                    {% endif %}
                </div>
                <div>
                    {% if next_article %}
                    <a href="/article/{{ next_article.shortUrl }}">下一篇: {{ next_article.title }}</a>
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
                <p>{{ site_name }}</p>
            </div>
        </div>
    </div>
</body>
</html>' WHERE type_id = 3 AND deleted=0;

-- =====================================================
-- 2. 修复栏目/搜索列表模板 (type_id=2)
-- 使用扁平变量 category_name, category_description 代替 category.xxx
-- 使用 site_name 代替 site.site_name
-- 使用 camelCase 字段名 (shortUrl, createTime) 因 ArticleListVO 有 rename_all=camelCase
-- =====================================================
UPDATE mxx_template_user_data SET temptext = '<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ category_name }} - {{ site_name }}</title>
    <meta name="keywords" content="{{ category_name }}">
    <meta name="description" content="{{ category_description }}">
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
            <a href="/" class="logo">{{ site_name }}</a>
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
            <a href="/">首页</a> &gt; {{ category_name }}
        </div>

        <div class="article-list">
            {% if list %}
            {% for a in list %}
            <div class="article-item">
                <div class="article-content">
                    <h3><a href="/article/{{ a.shortUrl }}">{{ a.title }}</a></h3>
                    <p>{{ a.title }}</p>
                    <div class="article-meta">{{ a.createTime }}</div>
                </div>
            </div>
            {% endfor %}
            {% else %}
            <p style="text-align: center; padding: 40px; color: #999;">暂无文章</p>
            {% endif %}
        </div>

        {% if total and total > 10 %}
        <div class="pagination">
            <span class="current">1</span>
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
                <p>{{ site_name }}</p>
            </div>
        </div>
    </div>
</body>
</html>' WHERE type_id = 2 AND deleted=0;
