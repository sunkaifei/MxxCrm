-- v29b: 修复产品详情模板 (type_id=8)
-- 问题1: line 59 有未闭合的字符串 ' 导致 syntax error
-- 问题2: ProductDetailVO 有 rename_all=camelCase，需用 camelCase 字段名
-- 问题3: product.model/brand 字段不存在于 ProductDetailVO
UPDATE mxx_template_user_data SET temptext = '<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ product.name }} - {{ site.siteName }}</title>
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
            <a href="/" class="logo">{{ site.siteName }}</a>
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
                {% if product.imageUrl %}
                <img src="{{ product.imageUrl }}" alt="{{ product.name }}">
                {% else %}
                <img src="/static/default/images/no-image.svg" alt="{{ product.name }}">
                {% endif %}
            </div>
            <div class="product-info">
                <h1>{{ product.name }}</h1>
                {% if product.salePrice %}
                <div class="product-price">¥{{ product.salePrice }}</div>
                {% endif %}
                <div class="product-desc">{{ product.description }}</div>
                <div class="product-attrs">
                    {% if product.sku %}<dl><dt>SKU:</dt><dd>{{ product.sku }}</dd></dl>{% endif %}
                    {% if product.unit %}<dl><dt>单位:</dt><dd>{{ product.unit }}</dd></dl>{% endif %}
                    {% if product.productNo %}<dl><dt>编号:</dt><dd>{{ product.productNo }}</dd></dl>{% endif %}
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
                <p>{{ site.siteName }}</p>
            </div>
        </div>
    </div>
</body>
</html>' WHERE type_id = 8 AND deleted=0;
