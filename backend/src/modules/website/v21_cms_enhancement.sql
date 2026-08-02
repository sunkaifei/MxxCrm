-- ============================================================
-- v21_cms_enhancement.sql
-- CMS 增强功能数据库迁移脚本（模板开发 + 媒体管理 + 内容模型）
-- 对应文档：docs/single-website-cms-plan.md v1.7（7.3 + 7.4 + 7.5）
-- 执行方式：psql -h 115.190.210.106 -U postgres -d mxxcrm_data -f v21_cms_enhancement.sql
-- ============================================================

BEGIN;

-- ============================================================
-- 一、内容模型系统（7.3 节）
-- ============================================================

-- 1. 内容模型表
CREATE TABLE IF NOT EXISTS mxx_content_model (
  id BIGSERIAL PRIMARY KEY,
  model_code VARCHAR(32) NOT NULL UNIQUE,
  model_name VARCHAR(64) NOT NULL,
  model_icon VARCHAR(64),
  description VARCHAR(255),
  has_title INT DEFAULT 1,
  has_content INT DEFAULT 1,
  has_cover INT DEFAULT 1,
  has_author INT DEFAULT 1,
  has_summary INT DEFAULT 1,
  has_seo INT DEFAULT 1,
  has_images INT DEFAULT 0,
  has_attachment INT DEFAULT 0,
  list_template_id BIGINT,
  detail_template_id BIGINT,
  sort INT DEFAULT 0,
  status INT DEFAULT 1,
  is_system INT DEFAULT 0,
  deleted INT DEFAULT 0,
  create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_model_code ON mxx_content_model(model_code, status, deleted);

-- 2. 模型字段定义表
CREATE TABLE IF NOT EXISTS mxx_content_model_field (
  id BIGSERIAL PRIMARY KEY,
  model_id BIGINT NOT NULL,
  field_name VARCHAR(64) NOT NULL,
  field_label VARCHAR(128),
  field_type INT DEFAULT 1,
  field_options TEXT,
  default_value VARCHAR(500),
  placeholder VARCHAR(255),
  is_required INT DEFAULT 0,
  is_searchable INT DEFAULT 0,
  is_list_show INT DEFAULT 1,
  is_detail_show INT DEFAULT 1,
  sort INT DEFAULT 0,
  status INT DEFAULT 1,
  deleted INT DEFAULT 0,
  create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(model_id, field_name)
);
CREATE INDEX IF NOT EXISTS idx_field_model ON mxx_content_model_field(model_id, status, deleted);

-- 3. 文章表新增 SEO 字段 + model_id + publish_time
-- 注意：文章表按网站动态创建，表名为 mxx_article_{website_id}，无统一 mxx_article 表
-- 使用 PL/pgSQL 动态遍历所有 mxx_article_* 表添加字段
DO $$
DECLARE
    tbl RECORD;
BEGIN
    FOR tbl IN
        SELECT table_name FROM information_schema.tables
        WHERE table_schema = 'public'
          AND table_name LIKE 'mxx_article_%'
          AND table_name NOT LIKE 'mxx_article_category%'
          AND table_name NOT LIKE 'mxx_article_comment%'
          AND table_name NOT LIKE 'mxx_article_label%'
          AND table_name NOT LIKE 'mxx_article_revision%'
    LOOP
        EXECUTE format('ALTER TABLE %I ADD COLUMN IF NOT EXISTS seo_title VARCHAR(255)', tbl.table_name);
        EXECUTE format('ALTER TABLE %I ADD COLUMN IF NOT EXISTS seo_keywords VARCHAR(255)', tbl.table_name);
        EXECUTE format('ALTER TABLE %I ADD COLUMN IF NOT EXISTS seo_description VARCHAR(500)', tbl.table_name);
        EXECUTE format('ALTER TABLE %I ADD COLUMN IF NOT EXISTS model_id BIGINT', tbl.table_name);
        EXECUTE format('ALTER TABLE %I ADD COLUMN IF NOT EXISTS publish_time TIMESTAMP', tbl.table_name);
        EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%s_model ON %I(model_id)', tbl.table_name, tbl.table_name);
        EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%s_pubtime ON %I(publish_time) WHERE deleted=0', tbl.table_name, tbl.table_name);
        RAISE NOTICE '已为表 % 添加 SEO/模型字段', tbl.table_name;
    END LOOP;
END $$;

-- 4. 栏目表新增 model_id + SEO 字段
ALTER TABLE mxx_article_category
  ADD COLUMN IF NOT EXISTS model_id BIGINT,
  ADD COLUMN IF NOT EXISTS seo_title VARCHAR(255),
  ADD COLUMN IF NOT EXISTS seo_keywords VARCHAR(255),
  ADD COLUMN IF NOT EXISTS seo_description VARCHAR(500);

-- 5. 文章标签关联表
CREATE TABLE IF NOT EXISTS mxx_article_label_merge (
  id BIGSERIAL PRIMARY KEY,
  article_id BIGINT NOT NULL,
  label_id BIGINT NOT NULL,
  create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(article_id, label_id)
);
CREATE INDEX IF NOT EXISTS idx_article_label_aid ON mxx_article_label_merge(article_id);
CREATE INDEX IF NOT EXISTS idx_article_label_lid ON mxx_article_label_merge(label_id);

-- 6. 文章版本历史表
CREATE TABLE IF NOT EXISTS mxx_article_revision (
  id BIGSERIAL PRIMARY KEY,
  article_id BIGINT NOT NULL,
  content TEXT NOT NULL,
  revision_note VARCHAR(255),
  create_by BIGINT,
  create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_article_rev_aid ON mxx_article_revision(article_id, create_time DESC);

-- ============================================================
-- 二、模板开发加强（7.4 节）
-- ============================================================

-- 7. 模板变量表
CREATE TABLE IF NOT EXISTS mxx_template_var (
  id BIGSERIAL PRIMARY KEY,
  var_key VARCHAR(64) NOT NULL UNIQUE,
  var_label VARCHAR(128),
  var_value TEXT,
  var_type INT DEFAULT 1,
  var_group VARCHAR(32) DEFAULT 'default',
  sort INT DEFAULT 0,
  status INT DEFAULT 1,
  deleted INT DEFAULT 0,
  create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_tpl_var_key ON mxx_template_var(var_key, status, deleted);

-- 8. 模板版本历史表
CREATE TABLE IF NOT EXISTS mxx_template_revision (
  id BIGSERIAL PRIMARY KEY,
  template_data_id BIGINT NOT NULL,
  temptext TEXT NOT NULL,
  revision_note VARCHAR(255),
  create_by BIGINT,
  create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_tpl_rev_data ON mxx_template_revision(template_data_id, create_time DESC);

-- 9. 留言表（P0 任务 G-0.1）— 兼容已有表结构
CREATE TABLE IF NOT EXISTS mxx_website_leave_msg (
  id BIGSERIAL PRIMARY KEY,
  website_id BIGINT,
  category_id BIGINT,
  contact_name VARCHAR(100),
  contact_phone VARCHAR(50),
  contact_email VARCHAR(200),
  content TEXT,
  status INT DEFAULT 0,
  convert_lead_id BIGINT,
  create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
-- 补充新字段（兼容已有表）
ALTER TABLE mxx_website_leave_msg ADD COLUMN IF NOT EXISTS product_id BIGINT;
ALTER TABLE mxx_website_leave_msg ADD COLUMN IF NOT EXISTS source_url VARCHAR(500);
ALTER TABLE mxx_website_leave_msg ADD COLUMN IF NOT EXISTS ip_address VARCHAR(64);
ALTER TABLE mxx_website_leave_msg ADD COLUMN IF NOT EXISTS user_agent VARCHAR(500);
ALTER TABLE mxx_website_leave_msg ADD COLUMN IF NOT EXISTS lead_id BIGINT;
ALTER TABLE mxx_website_leave_msg ADD COLUMN IF NOT EXISTS converted_to_lead INT DEFAULT 0;
ALTER TABLE mxx_website_leave_msg ADD COLUMN IF NOT EXISTS remark VARCHAR(500);
ALTER TABLE mxx_website_leave_msg ADD COLUMN IF NOT EXISTS deleted INT DEFAULT 0;
-- 如果有 convert_lead_id 但没有 lead_id，同步数据
UPDATE mxx_website_leave_msg SET lead_id = convert_lead_id WHERE lead_id IS NULL AND convert_lead_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_leave_msg_status ON mxx_website_leave_msg(status, deleted);
CREATE INDEX IF NOT EXISTS idx_leave_msg_lead ON mxx_website_leave_msg(lead_id) WHERE lead_id IS NOT NULL;

-- 10. Banner 表
CREATE TABLE IF NOT EXISTS mxx_website_banner (
  id BIGSERIAL PRIMARY KEY,
  title VARCHAR(128) NOT NULL,
  image_url VARCHAR(500) NOT NULL,
  link_url VARCHAR(500),
  alt_text VARCHAR(255),
  position VARCHAR(32) DEFAULT 'home_top',
  target VARCHAR(16) DEFAULT '_self',
  sort INT DEFAULT 0,
  start_time TIMESTAMP,
  end_time TIMESTAMP,
  status INT DEFAULT 1,
  deleted INT DEFAULT 0,
  create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_banner_position ON mxx_website_banner(position, status, deleted, sort);

-- 11. 区块表
CREATE TABLE IF NOT EXISTS mxx_website_block (
  id BIGSERIAL PRIMARY KEY,
  block_code VARCHAR(64) NOT NULL,
  block_name VARCHAR(128) NOT NULL,
  block_type INT DEFAULT 1,
  content TEXT,
  image_url VARCHAR(500),
  link_url VARCHAR(500),
  sort INT DEFAULT 0,
  status INT DEFAULT 1,
  deleted INT DEFAULT 0,
  create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_block_code ON mxx_website_block(block_code, status, deleted);

-- 12. 自定义页面表
CREATE TABLE IF NOT EXISTS mxx_website_page (
  id BIGSERIAL PRIMARY KEY,
  page_code VARCHAR(64) NOT NULL UNIQUE,
  page_name VARCHAR(128) NOT NULL,
  page_title VARCHAR(255),
  page_content TEXT,
  seo_keywords VARCHAR(255),
  seo_description VARCHAR(500),
  template_id BIGINT,
  sort INT DEFAULT 0,
  status INT DEFAULT 1,
  deleted INT DEFAULT 0,
  create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_page_code ON mxx_website_page(page_code, status, deleted);

-- ============================================================
-- 三、媒体管理加强（7.5 节）
-- ============================================================

-- 13. 媒体库表
CREATE TABLE IF NOT EXISTS mxx_website_media (
  id BIGSERIAL PRIMARY KEY,
  original_name VARCHAR(255) NOT NULL,
  storage_name VARCHAR(255) NOT NULL,
  file_path VARCHAR(500) NOT NULL,
  file_url VARCHAR(500) NOT NULL,
  file_ext VARCHAR(16),
  file_size BIGINT,
  file_type INT DEFAULT 1,
  mime_type VARCHAR(64),
  width INT,
  height INT,
  thumb_small VARCHAR(500),
  thumb_medium VARCHAR(500),
  thumb_large VARCHAR(500),
  alt_text VARCHAR(255),
  title VARCHAR(255),
  caption VARCHAR(500),
  description TEXT,
  category_id BIGINT,
  tags TEXT[],
  ref_count INT DEFAULT 0,
  has_watermark INT DEFAULT 0,
  sort INT DEFAULT 0,
  status INT DEFAULT 1,
  deleted INT DEFAULT 0,
  create_by BIGINT,
  create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_media_type ON mxx_website_media(file_type, deleted);
CREATE INDEX IF NOT EXISTS idx_media_category ON mxx_website_media(category_id, deleted);
CREATE INDEX IF NOT EXISTS idx_media_tags ON mxx_website_media USING GIN(tags);
CREATE INDEX IF NOT EXISTS idx_media_create ON mxx_website_media(create_time DESC) WHERE deleted=0;

-- 14. 媒体分类表
CREATE TABLE IF NOT EXISTS mxx_website_media_category (
  id BIGSERIAL PRIMARY KEY,
  category_name VARCHAR(64) NOT NULL,
  parent_id BIGINT DEFAULT 0,
  sort INT DEFAULT 0,
  deleted INT DEFAULT 0,
  create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_media_cat_parent ON mxx_website_media_category(parent_id, deleted);

-- ============================================================
-- 四、初始化数据
-- ============================================================

-- 15. 模板变量初始化
INSERT INTO mxx_template_var (var_key, var_label, var_value, var_type, var_group) VALUES
  ('phone', '客服电话', '400-888-8888', 1, 'contact'),
  ('email', '企业邮箱', 'service@example.com', 1, 'contact'),
  ('qq', '客服QQ', '88888888', 1, 'contact'),
  ('wechat', '微信号', 'mxxshop', 1, 'contact'),
  ('address', '公司地址', '北京市朝阳区xx路xx号', 1, 'contact'),
  ('work_time', '工作时间', '周一至周五 9:00-18:00', 1, 'contact'),
  ('icp', '备案号', '京ICP备12345678号', 1, 'stats'),
  ('stats_code', '统计代码', '', 4, 'stats'),
  ('logo_text', 'LOGO文字', 'MxxCRM', 1, 'brand')
ON CONFLICT (var_key) DO NOTHING;

-- 16. 系统内置内容模型初始化
INSERT INTO mxx_content_model (model_code, model_name, model_icon, description, has_title, has_content, has_cover, has_author, has_summary, has_seo, has_images, has_attachment, is_system, sort) VALUES
  ('article', '文章', 'file-text', '标准文章模型', 1, 1, 1, 1, 1, 1, 0, 0, 1, 1),
  ('product', '产品', 'package', '产品展示模型', 1, 1, 1, 0, 1, 1, 1, 0, 1, 2),
  ('download', '下载', 'download', '软件下载模型', 1, 1, 1, 1, 1, 1, 1, 1, 1, 3)
ON CONFLICT (model_code) DO NOTHING;

-- 17. 媒体分类初始化
INSERT INTO mxx_website_media_category (category_name, parent_id, sort) VALUES
  ('图片', 0, 1),
  ('视频', 0, 2),
  ('文档', 0, 3),
  ('产品图', 1, 1),
  ('文章配图', 1, 2),
  ('Banner图', 1, 3)
ON CONFLICT DO NOTHING;

COMMIT;

-- ============================================================
-- 验证查询
-- ============================================================
SELECT '内容模型' AS item, COUNT(*) AS cnt FROM mxx_content_model WHERE deleted=0
UNION ALL SELECT '模型字段', COUNT(*) FROM mxx_content_model_field WHERE deleted=0
UNION ALL SELECT '模板变量', COUNT(*) FROM mxx_template_var WHERE deleted=0
UNION ALL SELECT '模板版本历史表', COUNT(*) FROM information_schema.tables WHERE table_name='mxx_template_revision'
UNION ALL SELECT '留言表', COUNT(*) FROM information_schema.tables WHERE table_name='mxx_website_leave_msg'
UNION ALL SELECT 'Banner表', COUNT(*) FROM information_schema.tables WHERE table_name='mxx_website_banner'
UNION ALL SELECT '区块表', COUNT(*) FROM information_schema.tables WHERE table_name='mxx_website_block'
UNION ALL SELECT '自定义页面表', COUNT(*) FROM information_schema.tables WHERE table_name='mxx_website_page'
UNION ALL SELECT '媒体库表', COUNT(*) FROM information_schema.tables WHERE table_name='mxx_website_media'
UNION ALL SELECT '媒体分类表', COUNT(*) FROM information_schema.tables WHERE table_name='mxx_website_media_category'
UNION ALL SELECT '文章标签关联表', COUNT(*) FROM information_schema.tables WHERE table_name='mxx_article_label_merge'
UNION ALL SELECT '文章版本历史表', COUNT(*) FROM information_schema.tables WHERE table_name='mxx_article_revision';
