-- ================================================
-- 统一时间字段命名迁移脚本
-- 将 product 模块的 created_at/updated_at 重命名为
-- create_time/update_time，与 system 模块保持一致
-- ================================================

-- 产品主表
ALTER TABLE mxx_product_main RENAME COLUMN created_at TO create_time;
ALTER TABLE mxx_product_main RENAME COLUMN updated_at TO update_time;

-- 产品SKU表
ALTER TABLE mxx_product_sku RENAME COLUMN created_at TO create_time;
ALTER TABLE mxx_product_sku RENAME COLUMN updated_at TO update_time;

-- 产品分类表
ALTER TABLE mxx_product_category RENAME COLUMN created_at TO create_time;
ALTER TABLE mxx_product_category RENAME COLUMN updated_at TO update_time;
