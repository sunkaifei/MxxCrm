-- v26_fix_custom_page_template.sql
-- 修复 type_id=6 模板：支持自定义页面（page.pageContent）与栏目封面（children）两种场景
-- 执行方式：psql -h 115.190.210.106 -U postgres -d mxxcrm_data -f v26_fix_custom_page_template.sql

BEGIN;

UPDATE mxx_template_user_data SET temptext = '<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ category.category_name }} · {{ site_name }}</title>
    <link rel="stylesheet" href="/static/default/css/bootstrap.min.css">
    <style>
        body{font-family:system-ui,-apple-system,sans-serif;color:#0f172a;background:#fff;}
        .hero{background:linear-gradient(135deg,#1e293b,#334155);color:#fff;padding:5rem 0 3rem;margin-top:56px;}
        .hero h1{font-weight:800;}
        .hero p{opacity:.85;max-width:600px;}
        .section{padding:4rem 0;}
        .sub-card{border:1px solid #e2e8f0;border-radius:12px;overflow:hidden;background:#fff;transition:transform .3s ease;margin-bottom:1.5rem;height:100%;}
        .sub-card:hover{transform:translateY(-3px);box-shadow:0 12px 28px -14px rgba(15,23,42,.18);}
        .sub-card .card-body{padding:1.5rem;}
        .sub-card .card-body h4{font-size:1.05rem;font-weight:700;}
        .sub-card .card-body h4 a{color:#0f172a;}
        .sub-card .card-body h4 a:hover{color:#2563eb;}
        .sub-card .card-body p{color:#64748b;font-size:.9rem;}
        .page-content{max-width:800px;margin:0 auto;padding:2rem 0;line-height:1.8;}
        .page-content h1{color:#2563eb;margin-bottom:1rem;}
        .page-content h2{color:#1e293b;margin-top:2rem;}
        .page-content p{color:#334155;margin-bottom:1rem;}
        .footer{background:#f8fafc;border-top:1px solid #e2e8f0;padding:2rem 0;color:#64748b;margin-top:3rem;}
    </style>
</head>
<body>
<nav class="navbar navbar-expand-lg navbar-light fixed-top bg-white" style="border-bottom:1px solid #e2e8f0;">
    <div class="container">
        <a class="navbar-brand fw-bold" href="/">{{ site_name }}</a>
    </div>
</nav>

<div class="hero">
    <div class="container">
        <h1>{{ category.category_name }}</h1>
        {% if category.description %}
        <p>{{ category.description }}</p>
        {% endif %}
    </div>
</div>

<section class="section">
    <div class="container">
        {% if page %}
        <div class="page-content">
            {{ page.pageContent | safe }}
        </div>
        {% else %}
        <div class="row">
            {% if children %}
            {% for child in children %}
            <div class="col-md-6 col-lg-4">
                <div class="sub-card">
                    <div class="card-body">
                        <h4><a href="/category/{{ child.short_url }}">{{ child.category_name }}</a></h4>
                        {% if child.description %}
                        <p>{{ child.description }}</p>
                        {% endif %}
                        <small class="text-muted">共 {{ child.count_topic }} 篇内容</small>
                    </div>
                </div>
            </div>
            {% endfor %}
            {% else %}
            <div class="col-12 text-center py-5">
                <p class="text-muted">暂无子栏目内容。</p>
            </div>
            {% endif %}
        </div>
        {% endif %}
    </div>
</section>

<footer class="footer text-center">
    <div class="container">
        <p>&copy; {{ site_name }} 版权所有 · {{ site_domain }}</p>
    </div>
</footer>
</body>
</html>' WHERE type_id = 6;

COMMIT;

SELECT 'type_id=6 template updated' AS result;
SELECT (temptext LIKE '%page.pageContent%') AS has_page_content_branch, (temptext LIKE '%if page%') AS has_page_conditional FROM mxx_template_user_data WHERE type_id = 6;
