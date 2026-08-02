-- ============================================================
-- G-2.1: 文章自定义字段体系（对标帝国CMS自定义字段）
-- 包含：mxx_article_field（字段定义）+ mxx_article_field_value（字段值）
-- ============================================================

-- 字段定义表：每个栏目可定义不同的字段集
CREATE TABLE IF NOT EXISTS mxx_article_field (
  id BIGSERIAL PRIMARY KEY,
  category_id BIGINT NOT NULL,           -- 所属栏目（不同栏目不同字段集）
  field_name VARCHAR(64) NOT NULL,       -- 字段名（英文）
  field_label VARCHAR(128),              -- 字段标签（中文）
  field_type INT DEFAULT 1,              -- 1文本 2富文本 3图片 4数字 5日期 6下拉 7多选
  field_options VARCHAR(500),            -- 下拉/多选选项（JSON）
  default_value VARCHAR(255),
  is_required INT DEFAULT 0,
  sort INT DEFAULT 0,
  status INT DEFAULT 1,
  deleted INT DEFAULT 0,
  create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_field_category ON mxx_article_field(category_id, status, deleted);

COMMENT ON TABLE mxx_article_field IS 'G-2.1: 文章自定义字段定义表（按栏目分组）';
COMMENT ON COLUMN mxx_article_field.category_id IS '所属栏目ID';
COMMENT ON COLUMN mxx_article_field.field_name IS '字段名（英文标识）';
COMMENT ON COLUMN mxx_article_field.field_label IS '字段标签（中文显示名）';
COMMENT ON COLUMN mxx_article_field.field_type IS '字段类型：1文本 2富文本 3图片 4数字 5日期 6下拉 7多选';
COMMENT ON COLUMN mxx_article_field.field_options IS '下拉/多选选项（JSON 数组）';

-- 字段值表：存储每篇文章各自定义字段的值
CREATE TABLE IF NOT EXISTS mxx_article_field_value (
  id BIGSERIAL PRIMARY KEY,
  article_id BIGINT NOT NULL,
  field_id BIGINT NOT NULL,
  field_value TEXT,
  create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(article_id, field_id)
);
CREATE INDEX IF NOT EXISTS idx_field_value_article ON mxx_article_field_value(article_id);
CREATE INDEX IF NOT EXISTS idx_field_value_field ON mxx_article_field_value(field_id);

COMMENT ON TABLE mxx_article_field_value IS 'G-2.1: 文章自定义字段值表';
COMMENT ON COLUMN mxx_article_field_value.article_id IS '文章ID';
COMMENT ON COLUMN mxx_article_field_value.field_id IS '字段定义ID';
COMMENT ON COLUMN mxx_article_field_value.field_value IS '字段值（文本存储）';

-- ============================================================
-- 注册后台菜单：自定义字段管理
-- 使用 mxx_system_menu 正确列名：name/type/path/component/perm
-- type 类型：FOLDER=目录 / MENU=菜单 / BUTTON=按钮
-- ============================================================

-- 自定义字段管理（MENU），挂在文章管理(id=349)下
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, path, component, icon, sort, status)
VALUES (
  660,
  349,
  'page.website.articleFieldTitle',
  'MENU',
  'website:article:field',
  '/website/article-field',
  'views/website/article-field/index.vue',
  'mdi:form-textbox',
  30,
  1
)
ON CONFLICT (id) DO UPDATE SET
  parent_id = EXCLUDED.parent_id,
  name = EXCLUDED.name,
  type = EXCLUDED.type,
  perm = EXCLUDED.perm,
  path = EXCLUDED.path,
  component = EXCLUDED.component;

-- 按钮权限
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status) VALUES
  (661, 660, 'page.website.articleField.button.view', 'BUTTON', 'website:article:field:view', 1, 1),
  (662, 660, 'page.website.articleField.button.add', 'BUTTON', 'website:article:field:add', 2, 1),
  (663, 660, 'page.website.articleField.button.edit', 'BUTTON', 'website:article:field:update', 3, 1),
  (664, 660, 'page.website.articleField.button.delete', 'BUTTON', 'website:article:field:delete', 4, 1)
ON CONFLICT (id) DO UPDATE SET
  parent_id = EXCLUDED.parent_id,
  name = EXCLUDED.name,
  type = EXCLUDED.type,
  perm = EXCLUDED.perm;
