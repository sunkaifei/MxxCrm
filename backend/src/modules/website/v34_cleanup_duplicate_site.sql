-- ============================================================
-- v34_cleanup_duplicate_site.sql
-- 清理单站模式下多余的站点记录，确保 mxx_website 表只有一条默认站点
-- 执行方式：psql -h 115.190.210.106 -U postgres -d mxxcrm_data -f v34_cleanup_duplicate_site.sql
-- ============================================================

BEGIN;

-- ============================================================
-- 1. 检查 mxx_website 表记录数（deleted IS NULL 也算未删除）
-- ============================================================
DO $$
DECLARE
    site_count INT;
    default_count INT;
    keep_id BIGINT;
BEGIN
    SELECT COUNT(*) INTO site_count FROM mxx_website WHERE (deleted IS NULL OR deleted = 0);
    SELECT COUNT(*) INTO default_count FROM mxx_website WHERE (deleted IS NULL OR deleted = 0) AND is_default = 1;

    RAISE NOTICE '当前站点总数: %, 默认站点数: %', site_count, default_count;

    -- ============================================================
    -- 2. 如果有多条记录，保留 is_default=1 的那条，删除其他
    --    如果没有 is_default=1 的记录，保留 id 最小的那条并设为默认
    -- ============================================================
    IF site_count > 1 THEN
        IF default_count >= 1 THEN
            -- 保留第一条 is_default=1 的记录（按 id 升序），删除其他
            SELECT id INTO keep_id FROM mxx_website WHERE (deleted IS NULL OR deleted = 0) AND is_default = 1 ORDER BY id LIMIT 1;
            RAISE NOTICE '保留默认站点 ID: %', keep_id;

            -- 删除非默认站点（逻辑删除，标记 deleted=1）
            UPDATE mxx_website
            SET deleted = 1, update_time = CURRENT_TIMESTAMP
            WHERE (deleted IS NULL OR deleted = 0) AND id != keep_id;
        ELSE
            -- 没有默认站点，保留 id 最小的
            SELECT id INTO keep_id FROM mxx_website WHERE (deleted IS NULL OR deleted = 0) ORDER BY id LIMIT 1;
            RAISE NOTICE '无默认站点，保留最小 ID: %', keep_id;

            -- 删除其他站点
            UPDATE mxx_website
            SET deleted = 1, update_time = CURRENT_TIMESTAMP
            WHERE (deleted IS NULL OR deleted = 0) AND id != keep_id;

            -- 设置保留站点为默认
            UPDATE mxx_website SET is_default = 1 WHERE id = keep_id;
        END IF;

        RAISE NOTICE '已清理多余站点，保留 ID: %', keep_id;
    ELSE
        RAISE NOTICE '站点记录数正常（<=1），无需清理';
    END IF;

    -- ============================================================
    -- 3. 确保唯一站点的 is_default = 1 且 deleted = 0
    -- ============================================================
    UPDATE mxx_website SET is_default = 1, deleted = 0 WHERE (deleted IS NULL OR deleted = 0) AND (is_default IS NULL OR is_default != 1);
    UPDATE mxx_website SET deleted = 0 WHERE deleted IS NULL;
END $$;

COMMIT;

-- ============================================================
-- 验证查询
-- ============================================================
SELECT id, site_name, is_default, status, deleted, create_time
FROM mxx_website
WHERE deleted = 0
ORDER BY id;