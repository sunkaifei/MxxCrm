-- ============================================================
-- v23_cms_templates_and_testdata.sql
-- 单站 CMS：补充缺失模板(type_id 7,8,14,15) + 导航 + 文章测试数据
-- 数据库：mxxcrm_data
-- ============================================================

-- 1. 设置默认站点的 lead_owner_id（管理员 manager, id=6）
UPDATE mxx_website
SET lead_owner_id = 6,
    contact_email = 'contact@mxxshop.com'
WHERE id = 1888888888888888881 AND lead_owner_id IS NULL;

-- ============================================================
-- 2. 创建缺失的模板数据（type_id 7=产品列表, 8=产品详情, 14=页头, 15=页脚）
-- ============================================================

-- type_id=7 产品列表页
INSERT INTO mxx_template_user_data (id, template_id, type_id, name, temptext, sort, status)
VALUES (
  1888888888888888907,
  1,
  7,
  '产品列表页',
  '<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ site.site_name }} - 产品中心</title>
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
        .breadcrumb { padding: 15px 0; color: #999; font-size: 14px; }
        .breadcrumb a { color: #666; text-decoration: none; }
        .product-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 20px; padding: 20px 0; }
        .product-card { background: #fff; border-radius: 8px; overflow: hidden; box-shadow: 0 2px 8px rgba(0,0,0,0.08); transition: transform 0.2s; }
        .product-card:hover { transform: translateY(-4px); }
        .product-card img { width: 100%; height: 200px; object-fit: cover; }
        .product-info { padding: 15px; }
        .product-info h3 { font-size: 16px; margin-bottom: 8px; }
        .product-info h3 a { color: #333; text-decoration: none; }
        .product-price { color: #e53e3e; font-size: 18px; font-weight: bold; }
        .filters { background: #fff; padding: 15px 20px; border-radius: 8px; margin-bottom: 20px; display: flex; gap: 15px; align-items: center; }
        .filters a { color: #666; text-decoration: none; padding: 5px 15px; border-radius: 4px; }
        .filters a.active { background: #2563eb; color: #fff; }
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
            <a href="/">首页</a> &gt; 产品中心
        </div>

        <div class="filters">
            <a href="/product" {% if current_category_id is none %}class="active"{% endif %}>全部</a>
            {% for c in categories %}
            <a href="/product?category_id={{ c.id }}" {% if current_category_id == c.id %}class="active"{% endif %}>{{ c.category_name }}</a>
            {% endfor %}
        </div>

        <div class="product-grid">
            {% for p in products %}
            <div class="product-card">
                <a href="/product/{{ p.id }}">
                    <img src="{{ p.image_url || '' }}/static/images/no-image.png" alt="{{ p.name }}" onerror="this.src=''data:image/svg+xml,<svg xmlns=''http://www.w3.org/2000/svg'' width=''300'' height=''200''><rect fill=''%23eee'' width=''300'' height=''200''/><text fill=''%23999'' x=''50%'' y=''50%'' text-anchor=''middle'' dy=''.3em'' font-size=''16''>暂无图片</text></svg>''">
                </a>
                <div class="product-info">
                    <h3><a href="/product/{{ p.id }}">{{ p.name }}</a></h3>
                    {% if p.sale_price %}
                    <p class="product-price">¥{{ p.sale_price }}</p>
                    {% endif %}
                    {% if site_mode == 1 %}
                    <p><a href="/api/open/leave_msg/submit" style="color:#2563eb;font-size:13px;">立即咨询</a></p>
                    {% endif %}
                </div>
            </div>
            {% endfor %}
        </div>

        {% if products is empty %}
        <div style="text-align:center;padding:60px;color:#999;">
            <p>暂无产品数据</p>
        </div>
        {% endif %}

        <div class="pagination">
            {% set pg = pagination(current_page, total, 12) %}
            {% if pg.has_prev %}
            <a href="/product?page={{ pg.prev_page }}">上一页</a>
            {% endif %}
            {% for p in pg.pages %}
            {% if p == pg.current %}
            <span class="current">{{ p }}</span>
            {% else %}
            <a href="/product?page={{ p }}">{{ p }}</a>
            {% endif %}
            {% endfor %}
            {% if pg.has_next %}
            <a href="/product?page={{ pg.next_page }}">下一页</a>
            {% endif %}
            <span>共 {{ pg.total_pages }} 页 / {{ total }} 条</span>
        </div>
    </div>

    <div class="footer">
        <div class="container">
            <ul class="footer-links">
                <li><a href="/">首页</a></li>
                <li><a href="/product">产品中心</a></li>
                <li><a href="/category/news">新闻动态</a></li>
                <li><a href="/category/about">关于我们</a></li>
            </ul>
            <p class="footer-copyright">{{ site.copyright || site.site_name }} {% if site.icp %}| {{ site.icp }}{% endif %}</p>
        </div>
    </div>
</body>
</html>',
  0,
  1
)
ON CONFLICT (id) DO NOTHING;

-- type_id=8 产品详情页
INSERT INTO mxx_template_user_data (id, template_id, type_id, name, temptext, sort, status)
VALUES (
  1888888888888888908,
  1,
  8,
  '产品详情页',
  '<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ product.name }} - {{ site.site_name }}</title>
    <meta name="keywords" content="{{ product.keywords || product.name }}">
    <meta name="description" content="{{ product.description || product.name }}">
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
        .product-detail { background: #fff; border-radius: 8px; padding: 30px; margin: 20px 0; }
        .product-main { display: flex; gap: 40px; }
        .product-image { flex: 0 0 400px; }
        .product-image img { width: 100%; border-radius: 8px; border: 1px solid #eee; }
        .product-info { flex: 1; }
        .product-info h1 { font-size: 24px; margin-bottom: 15px; color: #2d3748; }
        .product-price-box { background: #fff5f5; padding: 20px; border-radius: 8px; margin-bottom: 20px; }
        .product-price { color: #e53e3e; font-size: 32px; font-weight: bold; }
        .product-meta { margin-bottom: 20px; }
        .product-meta dl { display: flex; margin-bottom: 10px; }
        .product-meta dt { width: 100px; color: #999; }
        .product-meta dd { flex: 1; }
        .btn-group { margin-top: 25px; display: flex; gap: 15px; }
        .btn { display: inline-block; padding: 12px 30px; border-radius: 6px; text-decoration: none; font-size: 16px; cursor: pointer; border: none; }
        .btn-primary { background: #2563eb; color: #fff; }
        .btn-outline { background: #fff; color: #2563eb; border: 1px solid #2563eb; }
        .product-content { margin-top: 30px; padding-top: 30px; border-top: 1px solid #eee; }
        .product-content h2 { font-size: 20px; margin-bottom: 15px; color: #2d3748; }
        .product-content p { line-height: 1.8; margin-bottom: 10px; color: #555; }
        .inquiry-form { background: #f7fafc; padding: 25px; border-radius: 8px; margin-top: 30px; }
        .inquiry-form h3 { margin-bottom: 15px; color: #2d3748; }
        .inquiry-form .form-group { margin-bottom: 12px; }
        .inquiry-form label { display: block; margin-bottom: 5px; color: #666; font-size: 14px; }
        .inquiry-form input, .inquiry-form textarea { width: 100%; padding: 8px 12px; border: 1px solid #ddd; border-radius: 4px; font-size: 14px; }
        .inquiry-form textarea { height: 80px; resize: vertical; }
        .footer { background: #2d3748; color: #cbd5e0; padding: 30px 0; margin-top: 40px; }
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
            <div class="product-main">
                <div class="product-image">
                    <img src="{{ product.image_url || '' }}/static/images/no-image.png" alt="{{ product.name }}" onerror="this.src=''data:image/svg+xml,<svg xmlns=''http://www.w3.org/2000/svg'' width=''400'' height=''300''><rect fill=''%23eee'' width=''400'' height=''300''/><text fill=''%23999'' x=''50%'' y=''50%'' text-anchor=''middle'' dy=''.3em'' font-size=''18''>暂无图片</text></svg>''">
                </div>
                <div class="product-info">
                    <h1>{{ product.name }}</h1>
                    {% if product.sale_price %}
                    <div class="product-price-box">
                        <span style="color:#999;font-size:14px;">售价：</span>
                        <span class="product-price">¥{{ product.sale_price }}</span>
                    </div>
                    {% endif %}
                    <div class="product-meta">
                        <dl><dt>产品编号：</dt><dd>{{ product.product_no || '-' }}</dd></dl>
                        <dl><dt>单位：</dt><dd>{{ product.unit || '件' }}</dd></dl>
                        <dl><dt>规格：</dt><dd>{{ product.spec_type || '标准' }}</dd></dl>
                        {% if product.weight %}
                        <dl><dt>重量：</dt><dd>{{ product.weight }} kg</dd></dl>
                        {% endif %}
                    </div>
                    <div class="btn-group">
                        {% if site_mode == 1 %}
                        <a href="#inquiry" class="btn btn-primary">立即咨询</a>
                        {% elif site_mode == 2 %}
                        <a href="#" class="btn btn-primary">加入购物车</a>
                        <a href="#" class="btn btn-outline">立即购买</a>
                        {% else %}
                        <a href="#inquiry" class="btn btn-primary">立即咨询</a>
                        <a href="#" class="btn btn-outline">加入购物车</a>
                        {% endif %}
                    </div>
                </div>
            </div>

            <div class="product-content">
                <h2>产品详情</h2>
                {% if product.detail %}
                {{ product.detail | safe }}
                {% else %}
                <p>{{ product.description || '暂无详细描述' }}</p>
                {% endif %}
            </div>

            {% if site_mode != 2 %}
            <div class="inquiry-form" id="inquiry">
                <h3>在线咨询</h3>
                <form id="leaveMsgForm">
                    <div class="form-group">
                        <label>您的姓名 *</label>
                        <input type="text" id="contactName" placeholder="请输入您的姓名">
                    </div>
                    <div class="form-group">
                        <label>联系电话 *</label>
                        <input type="text" id="contactPhone" placeholder="请输入您的电话">
                    </div>
                    <div class="form-group">
                        <label>邮箱</label>
                        <input type="text" id="contactEmail" placeholder="请输入您的邮箱">
                    </div>
                    <div class="form-group">
                        <label>留言内容</label>
                        <textarea id="content" placeholder="请输入您想咨询的内容">我对 {{ product.name }} 感兴趣，请与我联系。</textarea>
                    </div>
                    <button type="button" class="btn btn-primary" onclick="submitLeaveMsg({{ product.id }})">提交咨询</button>
                </form>
                <div id="msgResult" style="margin-top:15px;display:none;color:green;">咨询提交成功，我们会尽快与您联系！</div>
            </div>
            <script>
            function submitLeaveMsg(productId) {
                var data = {
                    contactName: document.getElementById("contactName").value,
                    contactPhone: document.getElementById("contactPhone").value,
                    contactEmail: document.getElementById("contactEmail").value,
                    content: document.getElementById("content").value,
                    productId: productId
                };
                fetch("/api/open/leave_msg/submit", {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify(data)
                }).then(function(r) {
                    document.getElementById("leaveMsgForm").style.display = "none";
                    document.getElementById("msgResult").style.display = "block";
                }).catch(function(e) {
                    alert("提交失败，请稍后重试");
                });
            }
            </script>
            {% endif %}
        </div>
    </div>

    <div class="footer">
        <div class="container">
            <p class="footer-copyright">{{ site.copyright || site.site_name }} {% if site.icp %}| {{ site.icp }}{% endif %}</p>
        </div>
    </div>
</body>
</html>',
  0,
  1
)
ON CONFLICT (id) DO NOTHING;

-- type_id=14 公共页头片段
INSERT INTO mxx_template_user_data (id, template_id, type_id, name, temptext, sort, status)
VALUES (
  1888888888888888914,
  1,
  14,
  '公共页头',
  '<header class="site-header">
    <div class="header-top">
        <div class="container">
            <span>{{ site.company_name || site.site_name }}</span>
            {% if site.company_phone %}
            <span style="float:right;">电话：{{ site.company_phone }}</span>
            {% endif %}
        </div>
    </div>
    <div class="header-main">
        <div class="container">
            <a href="/" class="logo">{{ site.site_name }}</a>
            <nav class="main-nav">
                <ul>
                    <li><a href="/">首页</a></li>
                    <li><a href="/product">产品中心</a></li>
                    {% for c in categories %}
                    <li><a href="/category/{{ c.short_url }}">{{ c.category_name }}</a></li>
                    {% endfor %}
                    <li><a href="/search">搜索</a></li>
                </ul>
            </nav>
        </div>
    </div>
</header>',
  0,
  1
)
ON CONFLICT (id) DO NOTHING;

-- type_id=15 公共页脚片段
INSERT INTO mxx_template_user_data (id, template_id, type_id, name, temptext, sort, status)
VALUES (
  1888888888888888915,
  1,
  15,
  '公共页脚',
  '<footer class="site-footer">
    <div class="container">
        <div class="footer-content">
            <div class="footer-info">
                {% if site.company_name %}<p><strong>{{ site.company_name }}</strong></p>{% endif %}
                {% if site.company_address %}<p>地址：{{ site.company_address }}</p>{% endif %}
                {% if site.company_phone %}<p>电话：{{ site.company_phone }}</p>{% endif %}
                {% if site.contact_email %}<p>邮箱：{{ site.contact_email }}</p>{% endif %}
            </div>
            <div class="footer-nav">
                <h4>快速导航</h4>
                <ul>
                    <li><a href="/">首页</a></li>
                    <li><a href="/product">产品中心</a></li>
                    {% for c in categories %}
                    <li><a href="/category/{{ c.short_url }}">{{ c.category_name }}</a></li>
                    {% endfor %}
                </ul>
            </div>
            <div class="footer-links-box">
                <h4>友情链接</h4>
                <ul>
                    {% for link in get_links(0) %}
                    <li><a href="{{ link.linkUrl }}" target="_blank">{{ link.linkName }}</a></li>
                    {% endfor %}
                    {% for link in get_links(1) %}
                    <li><a href="{{ link.linkUrl }}" target="_blank">{{ link.linkName }}</a></li>
                    {% endfor %}
                </ul>
            </div>
        </div>
        <div class="footer-bottom">
            <p>{{ site.copyright || site.site_name }} {% if site.icp %}| <a href="https://beian.miit.gov.cn/" target="_blank">{{ site.icp }}</a>{% endif %}</p>
        </div>
    </div>
</footer>',
  0,
  1
)
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 3. 创建导航菜单数据
-- ============================================================
INSERT INTO mxx_navigation (id, website_id, parent_id, name, web_url, data_type, nav_type, sort, is_show, is_new_window_open, target)
VALUES
(1888888888888889201, 1888888888888888881, 0, '首页', '/', 'custom', 'header', 1, 1, 0, '_self'),
(1888888888888889202, 1888888888888888881, 0, '产品中心', '/product', 'custom', 'header', 2, 1, 0, '_self'),
(1888888888888889203, 1888888888888888881, 0, '关于我们', '/category/about', 'custom', 'header', 3, 1, 0, '_self'),
(1888888888888889204, 1888888888888888881, 0, '新闻动态', '/category/news', 'custom', 'header', 4, 1, 0, '_self'),
(1888888888888889205, 1888888888888888881, 0, '服务支持', '/category/support', 'custom', 'header', 5, 1, 0, '_self'),
(1888888888888889206, 1888888888888888881, 0, '站点地图', '/sitemap', 'custom', 'header', 6, 1, 0, '_self'),
(1888888888888889207, 1888888888888888881, 0, '公司简介', '/category/company', 'custom', 'header', 7, 1, 0, '_self'),
(1888888888888889208, 1888888888888888881, 0, '团队介绍', '/category/team', 'custom', 'header', 8, 1, 0, '_self'),
(1888888888888889209, 1888888888888888881, 0, '行业新闻', '/category/industry', 'custom', 'header', 9, 1, 0, '_self')
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 4. 创建测试文章数据（status=2 已发布）
-- ============================================================
INSERT INTO mxx_article (id, website_id, category_id, short_url, user_id, title, short_title, author, description, content, status, sort, deleted, publish_time, create_time, update_time)
VALUES
(1888888888888889301, 1888888888888888881, 1888888888888888103, 'welcome-to-mxxcrm', 6, '欢迎使用 MxxCRM 客户关系管理系统', '欢迎使用MxxCRM', '管理员', 'MxxCRM 是一款开源免费的企业级客户关系管理系统，帮助企业高效管理客户、线索、商机和合同。', '<p>MxxCRM 是一款基于 Rust + Vue3 开发的企业级客户关系管理系统，具备以下核心功能：</p><h3>核心模块</h3><ul><li>客户管理：完整的客户生命周期管理，包括客户档案、联系人、跟进记录</li><li>线索管理：从线索录入到转客户的全流程跟踪</li><li>商机管理：可视化商机阶段推进，提升成交率</li><li>合同管理：合同审批、回款计划、佣金分配</li><li>产品中心：产品 SKU、库存、价格管理</li><li>采购管理：供应商、采购订单、入库管理</li><li>财务管理：会员费、支付流水、佣金统计</li></ul><p>欢迎联系我们了解更多详情。</p>', 2, 1, 0, NOW(), NOW(), NOW()),
(1888888888888889302, 1888888888888888881, 1888888888888888103, 'crm-best-practices', 'CRM最佳实践', '管理员', '本文分享 CRM 系统使用的最佳实践，帮助企业提升客户管理效率。', '<p>CRM 系统的成功在于持续使用和数据积累。以下是一些最佳实践建议：</p><h3>1. 及时录入线索</h3><p>所有潜在客户信息都应第一时间录入系统，避免遗漏。</p><h3>2. 定期跟进</h3><p>制定跟进计划，确保每个线索都得到及时跟进。</p><h3>3. 数据分析</h3><p>利用系统的统计报表功能，分析销售漏斗和转化率。</p><h3>4. 团队协作</h3><p>通过权限分配和公海机制，促进团队协作。</p>', 2, 2, 0, NOW(), NOW(), NOW()),
(1888888888888889303, 1888888888888888881, 1888888888888888107, 'industry-trends-2026', '2026年CRM行业趋势', '管理员', '分析2026年CRM行业的发展趋势和新技术应用。', '<p>2026年CRM行业呈现以下趋势：</p><h3>AI 赋能</h3><p>人工智能在客户画像、智能推荐、销售预测等方面发挥更大作用。</p><h3>移动化</h3><p>移动端 CRM 成为标配，销售人员随时随地处理业务。</p><h3>数据安全</h3><p>随着数据法规完善，CRM 系统的数据安全能力愈发重要。</p><h3>一体化</h3><p>CRM 与 ERP、OA 等系统的深度集成成为趋势。</p>', 2, 1, 0, NOW(), NOW(), NOW()),
(1888888888888889304, 1888888888888888881, 1888888888888888101, 'about-us', '关于我们', '管理员', '了解 MxxCRM 团队的故事和使命。', '<p>MxxCRM 团队致力于为企业提供高效、开源的客户关系管理解决方案。</p><h3>我们的使命</h3><p>让每一家企业都能用上专业的 CRM 系统，提升客户管理能力。</p><h3>我们的优势</h3><ul><li>开源免费，自主可控</li><li>Rust 后端，性能卓越</li><li>Vue3 前端，体验流畅</li><li>社区活跃，持续迭代</li></ul><p>联系我们：contact@mxxshop.com</p>', 2, 1, 0, NOW(), NOW(), NOW()),
(1888888888888889305, 1888888888888888881, 1888888888888888104, 'service-support', '服务支持', '管理员', 'MxxCRM 提供全方位的服务支持。', '<p>我们为客户提供以下服务支持：</p><h3>安装部署</h3><p>提供专业的系统安装和部署服务，确保系统稳定运行。</p><h3>二次开发</h3><p>根据客户需求进行功能定制和二次开发。</p><h3>技术培训</h3><p>提供系统使用培训，帮助团队快速上手。</p><h3>售后服务</h3><p>7x24小时技术支持，及时响应客户需求。</p>', 2, 1, 0, NOW(), NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 5. 创建测试留言数据
-- ============================================================
INSERT INTO mxx_website_leave_msg (id, website_id, contact_name, contact_phone, contact_email, content, status, source, converted_to_lead, deleted, create_time, update_time)
VALUES
(1888888888888889401, 1888888888888888881, '张先生', '13800138001', 'zhang@example.com', '我想了解 MxxCRM 的报价和部署方案，请与我联系。', 0, 'website', 0, 0, NOW(), NOW()),
(1888888888888889402, 1888888888888888881, '李女士', '13900139002', 'li@example.com', '请问 MxxCRM 支持定制开发吗？我们有一些特殊需求。', 0, 'website', 0, 0, NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 验证
-- ============================================================
SELECT '模板数据' AS item, count(*) AS cnt FROM mxx_template_user_data WHERE template_id = 1
UNION ALL
SELECT '导航数据', count(*) FROM mxx_navigation WHERE website_id = 1888888888888888881
UNION ALL
SELECT '文章数据', count(*) FROM mxx_article WHERE website_id = 1888888888888888881 AND deleted = 0
UNION ALL
SELECT '留言数据', count(*) FROM mxx_website_leave_msg WHERE website_id = 1888888888888888881 AND deleted = 0
UNION ALL
SELECT 'lead_owner_id', count(*) FROM mxx_website WHERE id = 1888888888888888881 AND lead_owner_id IS NOT NULL;
