-- Mxx-CRM 产品模块表结构迁移
-- 适用数据库：PostgreSQL
-- 说明：补齐 product/sku 表新增字段

-- 产品主表新增字段
ALTER TABLE mxx_product_main
    ADD COLUMN IF NOT EXISTS detail TEXT,
    ADD COLUMN IF NOT EXISTS carousel_images JSONB,
    ADD COLUMN IF NOT EXISTS spec_type VARCHAR(16),
    ADD COLUMN IF NOT EXISTS keywords VARCHAR(255),
    ADD COLUMN IF NOT EXISTS market_price NUMERIC(12, 2),
    ADD COLUMN IF NOT EXISTS stock INT;

COMMENT ON COLUMN mxx_product_main.detail IS '产品详情（富文本/HTML）';
COMMENT ON COLUMN mxx_product_main.carousel_images IS '轮播图URL数组';
COMMENT ON COLUMN mxx_product_main.spec_type IS '规格类型：single/multiple';
COMMENT ON COLUMN mxx_product_main.keywords IS '关键字';
COMMENT ON COLUMN mxx_product_main.market_price IS '市场价/原价';
COMMENT ON COLUMN mxx_product_main.stock IS '库存数量（单规格模式）';

-- 产品SKU表新增字段
ALTER TABLE mxx_product_sku
    ADD COLUMN IF NOT EXISTS specs JSONB,
    ADD COLUMN IF NOT EXISTS original_price NUMERIC(12, 2),
    ADD COLUMN IF NOT EXISTS weight NUMERIC(12, 3),
    ADD COLUMN IF NOT EXISTS volume NUMERIC(12, 6),
    ADD COLUMN IF NOT EXISTS is_default BOOLEAN DEFAULT FALSE;

COMMENT ON COLUMN mxx_product_sku.specs IS '动态规格键值对，如 {"颜色": "红色", "尺寸": "S"}';
COMMENT ON COLUMN mxx_product_sku.original_price IS '市场价/原价';
COMMENT ON COLUMN mxx_product_sku.weight IS '重量（kg）';
COMMENT ON COLUMN mxx_product_sku.volume IS '体积（m³）';
COMMENT ON COLUMN mxx_product_sku.is_default IS '是否默认选中';
