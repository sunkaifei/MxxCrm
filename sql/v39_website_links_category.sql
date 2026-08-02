-- ============================================================
-- G-1.16: 友情链接分类字段（link_category）
-- 用于按分类筛选友情链接（如：partner/friend/media）
-- ============================================================

ALTER TABLE mxx_website_links ADD COLUMN IF NOT EXISTS link_category VARCHAR(64);
COMMENT ON COLUMN mxx_website_links.link_category IS 'G-1.16: 链接分类（如：partner/friend/media，用于前端分类筛选）';
CREATE INDEX IF NOT EXISTS idx_links_category ON mxx_website_links(link_category);
