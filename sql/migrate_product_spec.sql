-- ========================================
-- 产品动态规格系统迁移脚本
-- 1. 规格定义表
-- 2. 规格值表
-- 3. SKU表改造（添加specs JSONB字段）
-- ========================================
-- 注意：时间字段使用 create_time/update_time + TIMESTAMP 风格
-- 与 system 模块保持一致，避免 NaiveDateTime ↔ TIMESTAMPTZ 不兼容
-- ========================================

-- 1. 产品规格定义表
CREATE TABLE IF NOT EXISTS mxx_product_spec (
    id              BIGSERIAL PRIMARY KEY,
    product_id      BIGINT NOT NULL REFERENCES mxx_product_main(id) ON DELETE CASCADE,
    name            VARCHAR(64) NOT NULL,   -- 规格名：颜色、尺寸、CPU型号、内存
    sort_order      INT DEFAULT 0,
    create_time     TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time     TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 产品ID索引
CREATE INDEX IF NOT EXISTS idx_product_spec_product ON mxx_product_spec(product_id);

-- 2. 产品规格值表
CREATE TABLE IF NOT EXISTS mxx_product_spec_value (
    id              BIGSERIAL PRIMARY KEY,
    spec_id         BIGINT NOT NULL REFERENCES mxx_product_spec(id) ON DELETE CASCADE,
    value           VARCHAR(128) NOT NULL,   -- 规格值：红色、S、i7、16GB
    sort_order      INT DEFAULT 0,
    create_time     TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time     TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 规格ID索引
CREATE INDEX IF NOT EXISTS idx_product_spec_value_spec ON mxx_product_spec_value(spec_id);

-- 3. SKU表改造：添加 specs JSONB 字段
ALTER TABLE mxx_product_sku ADD COLUMN IF NOT EXISTS specs JSONB DEFAULT '{}'::jsonb;

-- 说明：
-- specs 字段存储规格键值对，如：
-- 服装: {"颜色": "红色", "尺寸": "S"}
-- 电脑: {"CPU型号": "i7-13700", "内存": "16GB", "硬盘": "512GB SSD"}
