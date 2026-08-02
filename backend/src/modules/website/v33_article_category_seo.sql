-- v33: 文章 SEO + 栏目 SEO + 文章定时发布字段
-- 对应任务：G-0.10 文章 SEO 字段、G-1.5 栏目 SEO 字段、G-1.10 文章定时发布
-- 遵循项目规则：TIMESTAMP、create_time/update_time、status INT

-- =====================================================
-- G-0.10: 文章 SEO 字段 - mxx_article 新增字段
-- =====================================================
ALTER TABLE mxx_article ADD COLUMN IF NOT EXISTS seo_title VARCHAR(255);
ALTER TABLE mxx_article ADD COLUMN IF NOT EXISTS seo_keywords VARCHAR(500);
ALTER TABLE mxx_article ADD COLUMN IF NOT EXISTS seo_description VARCHAR(1000);

-- =====================================================
-- G-1.10: 文章定时发布 - mxx_article 新增 publish_time 字段
-- 到达该时间后，文章自动从草稿(status=0)变为已发布(status=1)
-- =====================================================
ALTER TABLE mxx_article ADD COLUMN IF NOT EXISTS publish_time TIMESTAMP;
CREATE INDEX IF NOT EXISTS idx_article_publish_time ON mxx_article (publish_time);

-- =====================================================
-- G-1.5: 栏目 SEO 字段 - mxx_article_category 新增字段
-- =====================================================
ALTER TABLE mxx_article_category ADD COLUMN IF NOT EXISTS seo_title VARCHAR(255);
ALTER TABLE mxx_article_category ADD COLUMN IF NOT EXISTS seo_keywords VARCHAR(500);
ALTER TABLE mxx_article_category ADD COLUMN IF NOT EXISTS seo_description VARCHAR(1000);

-- =====================================================
-- 校验 SQL：列出新增字段是否成功添加
-- =====================================================
SELECT 'article_seo_title' AS check_key, count(*) AS exists_count
FROM information_schema.columns
WHERE table_name = 'mxx_article' AND column_name = 'seo_title'
UNION ALL SELECT 'article_seo_keywords', count(*) FROM information_schema.columns
WHERE table_name = 'mxx_article' AND column_name = 'seo_keywords'
UNION ALL SELECT 'article_seo_description', count(*) FROM information_schema.columns
WHERE table_name = 'mxx_article' AND column_name = 'seo_description'
UNION ALL SELECT 'article_publish_time', count(*) FROM information_schema.columns
WHERE table_name = 'mxx_article' AND column_name = 'publish_time'
UNION ALL SELECT 'category_seo_title', count(*) FROM information_schema.columns
WHERE table_name = 'mxx_article_category' AND column_name = 'seo_title'
UNION ALL SELECT 'category_seo_keywords', count(*) FROM information_schema.columns
WHERE table_name = 'mxx_article_category' AND column_name = 'seo_keywords'
UNION ALL SELECT 'category_seo_description', count(*) FROM information_schema.columns
WHERE table_name = 'mxx_article_category' AND column_name = 'seo_description';
