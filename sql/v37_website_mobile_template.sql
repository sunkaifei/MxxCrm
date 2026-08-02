-- v37: G-2.4 移动端独立模板（PC/WAP 模板切换）
-- 为 mxx_website 添加 mobile_template_id 字段
-- 当字段为 NULL 或 0 时，移动端使用 template_id（与 PC 相同）
-- 当字段有值时，根据 User-Agent 检测移动设备，使用 mobile_template_id 指定的模板

ALTER TABLE mxx_website ADD COLUMN IF NOT EXISTS mobile_template_id BIGINT;

COMMENT ON COLUMN mxx_website.mobile_template_id IS 'G-2.4: 移动端模板ID（NULL/0 时与 template_id 相同）';
