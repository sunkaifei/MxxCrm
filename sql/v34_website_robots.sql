-- 网站表新增 robots.txt 内容字段
ALTER TABLE mxx_website ADD COLUMN IF NOT EXISTS robots_content TEXT;
COMMENT ON COLUMN mxx_website.robots_content IS 'robots.txt 自定义内容，为空则使用默认规则';
