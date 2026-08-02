-- v25_fix_template_vars_and_testdata.sql
-- 修复 type_id=6 栏目封面模板中的错误字段引用，并补充测试数据（Banner、自定义页面）
-- 执行方式：psql -h 115.190.210.106 -U postgres -d mxxcrm_data -f v25_fix_template_vars_and_testdata.sql

BEGIN;

-- ============================================================
-- 1. 修复 type_id=6 栏目封面模板：错误的字段引用
--    category.name  → category.category_name
--    child.name     → child.category_name
--    child.count    → child.count_topic
-- ============================================================
UPDATE mxx_template_user_data
SET temptext = REPLACE(temptext,
    '{{ category.name }}',
    '{{ category.category_name }}')
WHERE type_id = 6;

UPDATE mxx_template_user_data
SET temptext = REPLACE(temptext,
    '{{ child.name }}',
    '{{ child.category_name }}')
WHERE type_id = 6;

UPDATE mxx_template_user_data
SET temptext = REPLACE(temptext,
    '{{ child.count }}',
    '{{ child.count_topic }}')
WHERE type_id = 6;

-- ============================================================
-- 2. 补充 Banner 测试数据（首页轮播图）
--    mxx_website_banner 表无 website_id 列
-- ============================================================
INSERT INTO mxx_website_banner (id, title, image_url, link_url, alt_text, position, target, sort, status, deleted, create_time, update_time)
SELECT 1888888888888888401, 'MxxCRM 企业级客户管理', '/static/default/images/banner1.svg', '/', 'MxxCRM Banner', 'home_top', '_self', 1, 1, 0, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_website_banner WHERE id = 1888888888888888401);

INSERT INTO mxx_website_banner (id, title, image_url, link_url, alt_text, position, target, sort, status, deleted, create_time, update_time)
SELECT 1888888888888888402, '全流程销售管理', '/static/default/images/banner2.svg', '/category/products', '销售管理 Banner', 'home_top', '_self', 2, 1, 0, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_website_banner WHERE id = 1888888888888888402);

INSERT INTO mxx_website_banner (id, title, image_url, link_url, alt_text, position, target, sort, status, deleted, create_time, update_time)
SELECT 1888888888888888403, '数据分析与报表', '/static/default/images/banner3.svg', '/category/news', '数据分析 Banner', 'home_top', '_self', 3, 1, 0, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_website_banner WHERE id = 1888888888888888403);

-- ============================================================
-- 3. 补充自定义页面测试数据
--    mxx_website_page 表无 website_id 列
-- ============================================================
INSERT INTO mxx_website_page (id, page_code, page_name, page_title, page_content, seo_keywords, seo_description, sort, status, deleted, create_time, update_time)
SELECT 1888888888888888501, 'contact', '联系我们', '联系我们 - MxxCRM',
'<div style="max-width:800px;margin:0 auto;padding:40px 20px;">
<h1 style="color:#2563eb;">联系我们</h1>
<p>如有任何问题或合作意向，欢迎通过以下方式联系我们：</p>
<div style="margin-top:30px;">
    <p><strong>公司名称：</strong>北京心月狐科技有限公司</p>
    <p><strong>联系电话：</strong>400-888-8888</p>
    <p><strong>电子邮箱：</strong>contact@mxxshop.com</p>
    <p><strong>公司地址：</strong>北京市朝阳区科技园区</p>
    <p><strong>工作时间：</strong>周一至周五 9:00-18:00</p>
</div>
<div style="margin-top:30px;padding:20px;background:#f8fafc;border-radius:8px;">
    <h3>在线留言</h3>
    <p>您也可以通过 <a href="/api/open/leave_msg/submit">在线留言</a> 提交您的需求，我们会尽快与您联系。</p>
</div>
</div>',
'联系我们,联系方式,MxxCRM', '联系北京心月狐科技有限公司，获取 MxxCRM 产品咨询与技术支持',
1, 1, 0, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_website_page WHERE id = 1888888888888888501);

INSERT INTO mxx_website_page (id, page_code, page_name, page_title, page_content, seo_keywords, seo_description, sort, status, deleted, create_time, update_time)
SELECT 1888888888888888502, 'privacy', '隐私政策', '隐私政策 - MxxCRM',
'<div style="max-width:800px;margin:0 auto;padding:40px 20px;">
<h1 style="color:#2563eb;">隐私政策</h1>
<p>MxxCRM 尊重并保护用户的个人隐私。本隐私政策说明我们如何收集、使用和保护您的个人信息。</p>
<h2>1. 信息收集</h2>
<p>我们仅收集为您提供服务所必需的信息，包括姓名、联系方式和留言内容。</p>
<h2>2. 信息使用</h2>
<p>收集的信息仅用于客户服务和产品改进，不会出售给第三方。</p>
<h2>3. 信息安全</h2>
<p>我们采取行业标准的安全措施保护您的个人信息。</p>
</div>',
'隐私政策,数据保护,MxxCRM', 'MxxCRM 隐私政策与数据保护说明',
2, 1, 0, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_website_page WHERE id = 1888888888888888502);

-- ============================================================
-- 4. 补充导航数据
--    mxx_navigation.value 是 bigint 类型，传 0
-- ============================================================
INSERT INTO mxx_navigation (id, website_id, parent_id, name, web_url, value, data_type, nav_type, sort, is_show, is_new_window_open, create_time, update_time)
SELECT 1888888888888888601, 1888888888888888881, 0, '首页', '/', 0, 'custom', 'header', 1, 1, 0, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_navigation WHERE id = 1888888888888888601);

INSERT INTO mxx_navigation (id, website_id, parent_id, name, web_url, value, data_type, nav_type, sort, is_show, is_new_window_open, create_time, update_time)
SELECT 1888888888888888602, 1888888888888888881, 0, '关于我们', '/category/about', 0, 'custom', 'header', 2, 1, 0, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_navigation WHERE id = 1888888888888888602);

INSERT INTO mxx_navigation (id, website_id, parent_id, name, web_url, value, data_type, nav_type, sort, is_show, is_new_window_open, create_time, update_time)
SELECT 1888888888888888603, 1888888888888888881, 0, '产品中心', '/product', 0, 'custom', 'header', 3, 1, 0, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_navigation WHERE id = 1888888888888888603);

INSERT INTO mxx_navigation (id, website_id, parent_id, name, web_url, value, data_type, nav_type, sort, is_show, is_new_window_open, create_time, update_time)
SELECT 1888888888888888604, 1888888888888888881, 0, '新闻动态', '/category/news', 0, 'custom', 'header', 4, 1, 0, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_navigation WHERE id = 1888888888888888604);

INSERT INTO mxx_navigation (id, website_id, parent_id, name, web_url, value, data_type, nav_type, sort, is_show, is_new_window_open, create_time, update_time)
SELECT 1888888888888888605, 1888888888888888881, 0, '服务支持', '/category/support', 0, 'custom', 'header', 5, 1, 0, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_navigation WHERE id = 1888888888888888605);

INSERT INTO mxx_navigation (id, website_id, parent_id, name, web_url, value, data_type, nav_type, sort, is_show, is_new_window_open, create_time, update_time)
SELECT 1888888888888888606, 1888888888888888881, 0, '联系我们', '/page/contact', 0, 'custom', 'header', 6, 1, 0, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_navigation WHERE id = 1888888888888888606);

-- 页脚导航
INSERT INTO mxx_navigation (id, website_id, parent_id, name, web_url, value, data_type, nav_type, sort, is_show, is_new_window_open, create_time, update_time)
SELECT 1888888888888888607, 1888888888888888881, 0, '关于我们', '/category/about', 0, 'custom', 'footer', 1, 1, 0, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_navigation WHERE id = 1888888888888888607);

INSERT INTO mxx_navigation (id, website_id, parent_id, name, web_url, value, data_type, nav_type, sort, is_show, is_new_window_open, create_time, update_time)
SELECT 1888888888888888608, 1888888888888888881, 0, '隐私政策', '/page/privacy', 0, 'custom', 'footer', 2, 1, 0, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_navigation WHERE id = 1888888888888888608);

INSERT INTO mxx_navigation (id, website_id, parent_id, name, web_url, value, data_type, nav_type, sort, is_show, is_new_window_open, create_time, update_time)
SELECT 1888888888888888609, 1888888888888888881, 0, '站点地图', '/sitemap.html', 0, 'custom', 'footer', 3, 1, 0, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_navigation WHERE id = 1888888888888888609);

-- ============================================================
-- 5. 验证数据
-- ============================================================
SELECT '--- Banner count ---' AS section;
SELECT count(*) AS banner_count FROM mxx_website_banner WHERE deleted=0 AND status=1;

SELECT '--- Page count ---' AS section;
SELECT count(*) AS page_count FROM mxx_website_page WHERE deleted=0 AND status=1;

SELECT '--- Navigation count ---' AS section;
SELECT count(*) AS nav_count FROM mxx_navigation;

SELECT '--- Template type_id=6 fix verify ---' AS section;
SELECT
    (temptext LIKE '%category.category_name%') AS category_name_fixed,
    (temptext LIKE '%child.category_name%') AS child_name_fixed,
    (temptext LIKE '%child.count_topic%') AS child_count_fixed
FROM mxx_template_user_data WHERE type_id = 6;

COMMIT;
