-- ============================================================
-- v22_single_site_cms.sql
-- 单一网站 CMS 改造迁移脚本（site_mode 字段 + 统一 mxx_article 表 + 导航/留言补字段）
-- 对应文档：docs/single-website-cms-plan.md v1.7（第五章 数据模型设计）
-- 执行方式：psql -h 115.190.210.106 -U mxxcrm -d mxxcrm_data -f v22_single_site_cms.sql
-- 依赖：v21_cms_enhancement.sql 已执行
-- ============================================================

BEGIN;

-- ============================================================
-- 一、mxx_website 站点表改造（5.1.1 节）
-- ============================================================

-- 1. 新增 site_mode 字段：1展示型 2交易型 3混合型
ALTER TABLE mxx_website ADD COLUMN IF NOT EXISTS site_mode INT DEFAULT 1;
-- 2. 默认发货仓库（关联 mxx_inventory_warehouse）
ALTER TABLE mxx_website ADD COLUMN IF NOT EXISTS default_warehouse_id BIGINT;
-- 3. 联系邮箱（独立于 company_email 的客服邮箱）
ALTER TABLE mxx_website ADD COLUMN IF NOT EXISTS contact_email VARCHAR(100);
-- 4. 咨询转线索后的默认负责人
ALTER TABLE mxx_website ADD COLUMN IF NOT EXISTS lead_owner_id BIGINT;
-- 5. 默认站点设为展示型，并标记为默认站点
UPDATE mxx_website SET site_mode = 1, is_default = 1 WHERE is_default = 1 OR id = 1;

-- ============================================================
-- 二、mxx_article 统一文章表（5.1.2 节）—— 单站模式使用统一表
-- ============================================================

CREATE TABLE IF NOT EXISTS mxx_article (
  id BIGSERIAL PRIMARY KEY,
  website_id BIGINT NOT NULL DEFAULT 1,
  category_id BIGINT,
  short_url VARCHAR(255),
  user_id BIGINT,
  title VARCHAR(255) NOT NULL,
  short_title VARCHAR(255),
  title_image VARCHAR(500),
  author VARCHAR(100),
  original_link VARCHAR(500),
  description VARCHAR(1000),
  content TEXT,
  count_comment BIGINT DEFAULT 0,
  count_view BIGINT DEFAULT 0,
  count_love BIGINT DEFAULT 0,
  count_digg BIGINT DEFAULT 0,
  count_burys BIGINT DEFAULT 0,
  count_follow BIGINT DEFAULT 0,
  istop INT DEFAULT 0,
  isclose INT DEFAULT 0,
  iscomment INT DEFAULT 1,
  iscommentshow INT DEFAULT 0,
  isposts INT DEFAULT 0,
  isaudit INT DEFAULT 0,
  isrecommend INT DEFAULT 0,
  status INT DEFAULT 0,
  sort INT DEFAULT 0,
  deleted INT DEFAULT 0,
  seo_title VARCHAR(255),
  seo_keywords VARCHAR(255),
  seo_description VARCHAR(500),
  model_id BIGINT,
  publish_time TIMESTAMP,
  create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_article_category_id ON mxx_article(category_id);
CREATE INDEX IF NOT EXISTS idx_article_short_url ON mxx_article(short_url);
CREATE INDEX IF NOT EXISTS idx_article_status ON mxx_article(status, deleted);
CREATE INDEX IF NOT EXISTS idx_article_website ON mxx_article(website_id, deleted);
CREATE INDEX IF NOT EXISTS idx_article_model ON mxx_article(model_id);
CREATE INDEX IF NOT EXISTS idx_article_pubtime ON mxx_article(publish_time) WHERE deleted=0;

-- ============================================================
-- 三、mxx_article_category 栏目表补字段（5.1.2 节）
-- ============================================================

ALTER TABLE mxx_article_category ADD COLUMN IF NOT EXISTS model_id BIGINT;
ALTER TABLE mxx_article_category ADD COLUMN IF NOT EXISTS seo_title VARCHAR(255);
ALTER TABLE mxx_article_category ADD COLUMN IF NOT EXISTS seo_keywords VARCHAR(255);
ALTER TABLE mxx_article_category ADD COLUMN IF NOT EXISTS seo_description VARCHAR(500);

-- ============================================================
-- 四、mxx_website_leave_msg 留言表补字段（5.1.3 节）
-- ============================================================

ALTER TABLE mxx_website_leave_msg ADD COLUMN IF NOT EXISTS product_id BIGINT;
ALTER TABLE mxx_website_leave_msg ADD COLUMN IF NOT EXISTS source VARCHAR(32) DEFAULT 'website';
ALTER TABLE mxx_website_leave_msg ADD COLUMN IF NOT EXISTS source_url VARCHAR(500);
ALTER TABLE mxx_website_leave_msg ADD COLUMN IF NOT EXISTS ip_address VARCHAR(64);
ALTER TABLE mxx_website_leave_msg ADD COLUMN IF NOT EXISTS user_agent VARCHAR(500);
ALTER TABLE mxx_website_leave_msg ADD COLUMN IF NOT EXISTS lead_id BIGINT;
ALTER TABLE mxx_website_leave_msg ADD COLUMN IF NOT EXISTS converted_to_lead INT DEFAULT 0;
ALTER TABLE mxx_website_leave_msg ADD COLUMN IF NOT EXISTS remark VARCHAR(500);
ALTER TABLE mxx_website_leave_msg ADD COLUMN IF NOT EXISTS deleted INT DEFAULT 0;
-- 同步旧字段 convert_lead_id 到 lead_id
UPDATE mxx_website_leave_msg SET lead_id = convert_lead_id WHERE lead_id IS NULL AND convert_lead_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_leave_msg_status ON mxx_website_leave_msg(status, deleted);
CREATE INDEX IF NOT EXISTS idx_leave_msg_lead ON mxx_website_leave_msg(lead_id) WHERE lead_id IS NOT NULL;

-- ============================================================
-- 五、mxx_navigation 导航表补字段（7.2.4 节）
-- ============================================================

-- 导航表已存在，补充 SEO 友好的 target/is_show 字段（如不存在）
ALTER TABLE mxx_navigation ADD COLUMN IF NOT EXISTS target VARCHAR(16) DEFAULT '_self';
ALTER TABLE mxx_navigation ADD COLUMN IF NOT EXISTS icon VARCHAR(64);
CREATE INDEX IF NOT EXISTS idx_navigation_type ON mxx_navigation(nav_type, is_show, sort);

-- ============================================================
-- 六、初始化默认站点的 site_mode
-- ============================================================

-- 确保至少有一个默认站点
INSERT INTO mxx_website (id, site_name, site_mode, is_default, status, client, site_type, create_time, update_time)
SELECT 1, '默认站点', 1, 1, 1, 3, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
WHERE NOT EXISTS (SELECT 1 FROM mxx_website WHERE id = 1);

-- 确保默认站点 site_mode 不为 NULL
UPDATE mxx_website SET site_mode = 1 WHERE site_mode IS NULL;

COMMIT;

-- ============================================================
-- 验证查询
-- ============================================================
SELECT 'mxx_website.site_mode' AS item, COUNT(*) AS cnt FROM mxx_website WHERE site_mode IS NOT NULL
UNION ALL SELECT 'mxx_article 表', COUNT(*) FROM information_schema.tables WHERE table_name='mxx_article'
UNION ALL SELECT 'mxx_article_category.model_id', COUNT(*) FROM information_schema.columns WHERE table_name='mxx_article_category' AND column_name='model_id'
UNION ALL SELECT 'mxx_website_leave_msg.lead_id', COUNT(*) FROM information_schema.columns WHERE table_name='mxx_website_leave_msg' AND column_name='lead_id'
UNION ALL SELECT 'mxx_navigation.target', COUNT(*) FROM information_schema.columns WHERE table_name='mxx_navigation' AND column_name='target';
