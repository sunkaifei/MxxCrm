-- 修复文章详情模板 (type_id=3)
-- 使用 field 代替 article (minijinja context! 宏的变量名冲突)
-- 使用 site_name 扁平变量代替 site.site_name
-- 使用 camelCase 字段名 (createTime, shortUrl 等) 因 ArticleDetailVO 有 rename_all=camelCase
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
