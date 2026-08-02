-- ============================================================
-- v28_add_articles_and_categories.sql
-- 添加文章分类和测试文章数据
-- ============================================================

BEGIN;

-- 文章分类（匹配导航链接）
INSERT INTO mxx_article_category (id, website_id, parent_id, short_url, category_name, sort, is_show, status, page_type, description, create_time, update_time)
SELECT 1888888888888888801, 1888888888888888881, 0, 'about', '关于我们', 1, 1, 1, 2, '关于我们', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
WHERE NOT EXISTS (SELECT 1 FROM mxx_article_category WHERE id = 1888888888888888801);

INSERT INTO mxx_article_category (id, website_id, parent_id, short_url, category_name, sort, is_show, status, page_type, description, create_time, update_time)
SELECT 1888888888888888802, 1888888888888888881, 0, 'news', '新闻动态', 2, 1, 1, 2, '新闻动态', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
WHERE NOT EXISTS (SELECT 1 FROM mxx_article_category WHERE id = 1888888888888888802);

INSERT INTO mxx_article_category (id, website_id, parent_id, short_url, category_name, sort, is_show, status, page_type, description, create_time, update_time)
SELECT 1888888888888888803, 1888888888888888881, 0, 'support', '服务支持', 3, 1, 1, 2, '服务支持', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
WHERE NOT EXISTS (SELECT 1 FROM mxx_article_category WHERE id = 1888888888888888803);

INSERT INTO mxx_article_category (id, website_id, parent_id, short_url, category_name, sort, is_show, status, page_type, description, create_time, update_time)
SELECT 1888888888888888804, 1888888888888888881, 0, 'company', '公司简介', 4, 1, 1, 2, '公司简介', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
WHERE NOT EXISTS (SELECT 1 FROM mxx_article_category WHERE id = 1888888888888888804);

INSERT INTO mxx_article_category (id, website_id, parent_id, short_url, category_name, sort, is_show, status, page_type, description, create_time, update_time)
SELECT 1888888888888888805, 1888888888888888881, 0, 'team', '团队介绍', 5, 1, 1, 2, '团队介绍', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
WHERE NOT EXISTS (SELECT 1 FROM mxx_article_category WHERE id = 1888888888888888805);

INSERT INTO mxx_article_category (id, website_id, parent_id, short_url, category_name, sort, is_show, status, page_type, description, create_time, update_time)
SELECT 1888888888888888806, 1888888888888888881, 0, 'industry', '行业新闻', 6, 1, 1, 2, '行业新闻', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
WHERE NOT EXISTS (SELECT 1 FROM mxx_article_category WHERE id = 1888888888888888806);

-- 文章数据（新闻动态分类下）
INSERT INTO mxx_article (id, website_id, category_id, short_url, title, short_title, author, description, content, count_view, istop, isrecommend, status, deleted, create_time, update_time)
SELECT 1888888888888889301, 1888888888888888881, 1888888888888888802, 'article-001', 'MxxCRM 系统正式发布', '系统发布', 'admin', 'MxxCRM 企业级客户关系管理系统正式发布', '<p>MxxCRM 是一款基于 Rust + Vue3 开发的企业级客户关系管理系统，提供完整的客户管理、销售管理、合同管理等功能。</p><p>系统采用前后端分离架构，后端使用 Actix-Web 框架，前端使用 Vben Admin v5，数据库支持 PostgreSQL。</p>', 128, 1, 1, 2, 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
WHERE NOT EXISTS (SELECT 1 FROM mxx_article WHERE id = 1888888888888889301);

INSERT INTO mxx_article (id, website_id, category_id, short_url, title, short_title, author, description, content, count_view, istop, isrecommend, status, deleted, create_time, update_time)
SELECT 1888888888888889302, 1888888888888888881, 1888888888888888802, 'article-002', 'CRM 系统选型指南', '选型指南', 'admin', '如何选择适合企业的 CRM 系统', '<p>选择 CRM 系统需要考虑以下因素：</p><ul><li>功能是否满足业务需求</li><li>系统是否易于使用</li><li>是否支持定制开发</li><li>数据安全性</li><li>售后服务质量</li></ul>', 86, 0, 1, 2, 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
WHERE NOT EXISTS (SELECT 1 FROM mxx_article WHERE id = 1888888888888889302);

INSERT INTO mxx_article (id, website_id, category_id, short_url, title, short_title, author, description, content, count_view, istop, isrecommend, status, deleted, create_time, update_time)
SELECT 1888888888888889303, 1888888888888888881, 1888888888888888806, 'article-003', '2024 年 SaaS 行业发展趋势', 'SaaS 趋势', 'admin', '分析 SaaS 行业最新发展趋势', '<p>2024 年 SaaS 行业呈现以下趋势：</p><ol><li>AI 赋能成为标配</li><li>低代码平台兴起</li><li>垂直领域 SaaS 增长迅速</li><li>数据安全要求提升</li></ol>', 56, 0, 0, 2, 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
WHERE NOT EXISTS (SELECT 1 FROM mxx_article WHERE id = 1888888888888889303);

INSERT INTO mxx_article (id, website_id, category_id, short_url, title, short_title, author, description, content, count_view, istop, isrecommend, status, deleted, create_time, update_time)
SELECT 1888888888888889304, 1888888888888888881, 1888888888888888801, 'article-004', '关于我们', '关于我们', 'admin', '了解 MxxCRM 团队', '<p>我们是一家专注于企业级软件开发的技术团队，致力于为客户提供高质量的 CRM 解决方案。</p><p>团队拥有多年企业软件开发经验，技术栈涵盖 Rust、Vue3、PostgreSQL 等现代技术。</p>', 42, 0, 0, 2, 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
WHERE NOT EXISTS (SELECT 1 FROM mxx_article WHERE id = 1888888888888889304);

INSERT INTO mxx_article (id, website_id, category_id, short_url, title, short_title, author, description, content, count_view, istop, isrecommend, status, deleted, create_time, update_time)
SELECT 1888888888888889305, 1888888888888888881, 1888888888888888803, 'article-005', '系统使用常见问题', '常见问题', 'admin', 'MxxCRM 系统使用常见问题解答', '<h3>1. 如何创建客户？</h3><p>在客户管理模块中点击"新增"按钮即可创建客户。</p><h3>2. 如何分配客户给销售？</h3><p>在客户详情页面可以分配负责人。</p><h3>3. 如何查看销售报表？</h3><p>在统计报表模块中可以查看各类销售数据。</p>', 35, 0, 0, 2, 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
WHERE NOT EXISTS (SELECT 1 FROM mxx_article WHERE id = 1888888888888889305);

COMMIT;

-- 验证
SELECT '分类数' AS item, count(*) AS cnt FROM mxx_article_category WHERE website_id = 1888888888888888881
UNION ALL SELECT '文章数', count(*) FROM mxx_article WHERE website_id = 1888888888888888881 AND deleted = 0;
