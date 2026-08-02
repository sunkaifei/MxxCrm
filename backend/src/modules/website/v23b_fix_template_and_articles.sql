-- v23b_fix_template_and_articles.sql
-- 修复 type_id=8 产品详情页模板 + 文章测试数据

-- 1. 插入 type_id=8 产品详情页模板（使用 dollar-quoting 避免单引号冲突）
INSERT INTO mxx_template_user_data (id, template_id, type_id, name, temptext, sort, status)
VALUES (
  1888888888888888908,
  1,
  8,
  '产品详情页',
  $template$<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ product.name }} - {{ site.site_name }}</title>
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
                    <img src="{{ product.image_url }}" alt="{{ product.name }}" style="width:100%;border-radius:8px;border:1px solid #eee;">
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
                        <dl><dt>产品编号：</dt><dd>{{ product.product_no }}</dd></dl>
                        <dl><dt>单位：</dt><dd>{{ product.unit }}</dd></dl>
                        <dl><dt>规格：</dt><dd>{{ product.spec_type }}</dd></dl>
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
                <p>{{ product.description }}</p>
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
            function submitLeaveMsg(productId){
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
</html>$template$,
  0,
  1
)
ON CONFLICT (id) DO NOTHING;

-- 2. 插入文章测试数据（每行 16 列，与 INSERT 列定义一致）
INSERT INTO mxx_article (id, website_id, category_id, short_url, user_id, title, short_title, author, description, content, status, sort, deleted, publish_time, create_time, update_time)
VALUES
(1888888888888889301, 1888888888888888881, 1888888888888888103, 'welcome-to-mxxcrm', 6, '欢迎使用 MxxCRM 客户关系管理系统', '欢迎使用MxxCRM', '管理员', 'MxxCRM 是一款开源免费的企业级客户关系管理系统，帮助企业高效管理客户、线索、商机和合同。', '<p>MxxCRM 是一款基于 Rust + Vue3 开发的企业级客户关系管理系统，具备以下核心功能：</p><h3>核心模块</h3><ul><li>客户管理：完整的客户生命周期管理，包括客户档案、联系人、跟进记录</li><li>线索管理：从线索录入到转客户的全流程跟踪</li><li>商机管理：可视化商机阶段推进，提升成交率</li><li>合同管理：合同审批、回款计划、佣金分配</li><li>产品中心：产品 SKU、库存、价格管理</li><li>采购管理：供应商、采购订单、入库管理</li><li>财务管理：会员费、支付流水、佣金统计</li></ul><p>欢迎联系我们了解更多详情。</p>', 2, 1, 0, NOW(), NOW(), NOW()),
(1888888888888889302, 1888888888888888881, 1888888888888888103, 'crm-best-practices', 6, 'CRM最佳实践指南', 'CRM最佳实践', '管理员', '本文分享 CRM 系统使用的最佳实践，帮助企业提升客户管理效率。', '<p>CRM 系统的成功在于持续使用和数据积累。以下是一些最佳实践建议：</p><h3>1. 及时录入线索</h3><p>所有潜在客户信息都应第一时间录入系统，避免遗漏。</p><h3>2. 定期跟进</h3><p>制定跟进计划，确保每个线索都得到及时跟进。</p><h3>3. 数据分析</h3><p>利用系统的统计报表功能，分析销售漏斗和转化率。</p><h3>4. 团队协作</h3><p>通过权限分配和公海机制，促进团队协作。</p>', 2, 2, 0, NOW(), NOW(), NOW()),
(1888888888888889303, 1888888888888888881, 1888888888888888107, 'industry-trends-2026', 6, '2026年CRM行业趋势', '行业趋势', '管理员', '分析2026年CRM行业的发展趋势和新技术应用。', '<p>2026年CRM行业呈现以下趋势：</p><h3>AI 赋能</h3><p>人工智能在客户画像、智能推荐、销售预测等方面发挥更大作用。</p><h3>移动化</h3><p>移动端 CRM 成为标配，销售人员随时随地处理业务。</p><h3>数据安全</h3><p>随着数据法规完善，CRM 系统的数据安全能力愈发重要。</p><h3>一体化</h3><p>CRM 与 ERP、OA 等系统的深度集成成为趋势。</p>', 2, 1, 0, NOW(), NOW(), NOW()),
(1888888888888889304, 1888888888888888881, 1888888888888888101, 'about-us', 6, '关于我们', '关于我们', '管理员', '了解 MxxCRM 团队的故事和使命。', '<p>MxxCRM 团队致力于为企业提供高效、开源的客户关系管理解决方案。</p><h3>我们的使命</h3><p>让每一家企业都能用上专业的 CRM 系统，提升客户管理能力。</p><h3>我们的优势</h3><ul><li>开源免费，自主可控</li><li>Rust 后端，性能卓越</li><li>Vue3 前端，体验流畅</li><li>社区活跃，持续迭代</li></ul><p>联系我们：contact@mxxshop.com</p>', 2, 1, 0, NOW(), NOW(), NOW()),
(1888888888888889305, 1888888888888888881, 1888888888888888104, 'service-support', 6, '服务支持', '服务支持', '管理员', 'MxxCRM 提供全方位的服务支持。', '<p>我们为客户提供以下服务支持：</p><h3>安装部署</h3><p>提供专业的系统安装和部署服务，确保系统稳定运行。</p><h3>二次开发</h3><p>根据客户需求进行功能定制和二次开发。</p><h3>技术培训</h3><p>提供系统使用培训，帮助团队快速上手。</p><h3>售后服务</h3><p>7x24小时技术支持，及时响应客户需求。</p>', 2, 1, 0, NOW(), NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

-- 3. 验证
SELECT '模板数据' AS item, count(*) AS cnt FROM mxx_template_user_data WHERE template_id = 1
UNION ALL
SELECT '导航数据', count(*) FROM mxx_navigation WHERE website_id = 1888888888888888881
UNION ALL
SELECT '文章数据', count(*) FROM mxx_article WHERE website_id = 1888888888888888881 AND deleted = 0
UNION ALL
SELECT '留言数据', count(*) FROM mxx_website_leave_msg WHERE website_id = 1888888888888888881 AND deleted = 0
UNION ALL
SELECT 'lead_owner_id', count(*) FROM mxx_website WHERE id = 1888888888888888881 AND lead_owner_id IS NOT NULL;
