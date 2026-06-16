-- =============================================
-- Shop Module Database Migration Script
-- PostgreSQL Version (v3 - Fix default value issues)
-- Date: 2026-06-14
-- =============================================

-- 1. mxx_shop 表 - 修改字段类型为 SMALLINT
-- store_disable: character varying -> smallint (先删除默认值)
ALTER TABLE mxx_shop ALTER COLUMN store_disable DROP DEFAULT;
ALTER TABLE mxx_shop ALTER COLUMN store_disable TYPE SMALLINT USING CAST(store_disable AS INTEGER);
ALTER TABLE mxx_shop ALTER COLUMN store_disable SET DEFAULT 0;

-- page_show: bit -> smallint (先删除默认值)
ALTER TABLE mxx_shop ALTER COLUMN page_show DROP DEFAULT;
ALTER TABLE mxx_shop ALTER COLUMN page_show TYPE SMALLINT USING CAST(page_show AS INTEGER);
ALTER TABLE mxx_shop ALTER COLUMN page_show SET DEFAULT 0;

-- self_pick_flag: bit -> smallint (先删除默认值)
ALTER TABLE mxx_shop ALTER COLUMN self_pick_flag DROP DEFAULT;
ALTER TABLE mxx_shop ALTER COLUMN self_pick_flag TYPE SMALLINT USING CAST(self_pick_flag AS INTEGER);
ALTER TABLE mxx_shop ALTER COLUMN self_pick_flag SET DEFAULT 0;

-- delete_flag: bit -> smallint (先删除默认值)
ALTER TABLE mxx_shop ALTER COLUMN delete_flag DROP DEFAULT;
ALTER TABLE mxx_shop ALTER COLUMN delete_flag TYPE SMALLINT USING CAST(delete_flag AS INTEGER);
ALTER TABLE mxx_shop ALTER COLUMN delete_flag SET DEFAULT 0;

-- 2. mxx_shop_category 表
-- is_show: boolean -> smallint (先删除默认值)
ALTER TABLE mxx_shop_category ALTER COLUMN is_show DROP DEFAULT;
ALTER TABLE mxx_shop_category ALTER COLUMN is_show TYPE SMALLINT USING CASE WHEN is_show THEN 1 ELSE 0 END;
ALTER TABLE mxx_shop_category ALTER COLUMN is_show SET DEFAULT 1;

-- 3. mxx_shop_spu 表
-- is_commission: boolean -> smallint (先删除默认值)
ALTER TABLE mxx_shop_spu ALTER COLUMN is_commission DROP DEFAULT;
ALTER TABLE mxx_shop_spu ALTER COLUMN is_commission TYPE SMALLINT USING CASE WHEN is_commission THEN 1 ELSE 0 END;
ALTER TABLE mxx_shop_spu ALTER COLUMN is_commission SET DEFAULT 0;

-- 4. mxx_shop_cart 表
-- selected: boolean -> smallint (先删除默认值)
ALTER TABLE mxx_shop_cart ALTER COLUMN selected DROP DEFAULT;
ALTER TABLE mxx_shop_cart ALTER COLUMN selected TYPE SMALLINT USING CASE WHEN selected THEN 1 ELSE 0 END;
ALTER TABLE mxx_shop_cart ALTER COLUMN selected SET DEFAULT 1;

-- 5. mxx_shop_review 表
-- is_anonymous: boolean -> smallint (先删除默认值)
ALTER TABLE mxx_shop_review ALTER COLUMN is_anonymous DROP DEFAULT;
ALTER TABLE mxx_shop_review ALTER COLUMN is_anonymous TYPE SMALLINT USING CASE WHEN is_anonymous THEN 1 ELSE 0 END;
ALTER TABLE mxx_shop_review ALTER COLUMN is_anonymous SET DEFAULT 0;

-- 验证更新
SELECT 'mxx_shop' AS table_name, column_name, data_type 
FROM information_schema.columns 
WHERE table_name = 'mxx_shop' 
AND column_name IN ('self_operated', 'store_disable', 'page_show', 'self_pick_flag', 'delete_flag');

SELECT 'mxx_shop_category' AS table_name, column_name, data_type 
FROM information_schema.columns 
WHERE table_name = 'mxx_shop_category' 
AND column_name = 'is_show';

SELECT 'mxx_shop_spu' AS table_name, column_name, data_type 
FROM information_schema.columns 
WHERE table_name = 'mxx_shop_spu' 
AND column_name = 'is_commission';

SELECT 'mxx_shop_cart' AS table_name, column_name, data_type 
FROM information_schema.columns 
WHERE table_name = 'mxx_shop_cart' 
AND column_name = 'selected';

SELECT 'mxx_shop_review' AS table_name, column_name, data_type 
FROM information_schema.columns 
WHERE table_name = 'mxx_shop_review' 
AND column_name = 'is_anonymous';

SELECT '数据库字段类型迁移完成' AS result;