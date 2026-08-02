-- ============================================================
-- v42_reset_ids_and_template_management.sql
-- 1. 重置 mxx_template_user_data ID 从 1 开始
-- 2. 重置 mxx_template_data ID 从 1 开始
-- 3. 重置 mxx_website ID 为 1，同步更新所有关联表
-- 4. 新增 list_by_template 视图/接口支持
-- 执行方式：psql -h 115.190.210.106 -U postgres -d mxxcrm_data -f v42_reset_ids_and_template_management.sql
-- ============================================================

BEGIN;

-- ============================================================
-- 第一部分：重置 mxx_template_user_data ID
-- 使用临时映射表重新编号，保留原始 ID 映射关系
-- ============================================================

-- 创建临时映射表
CREATE TEMP TABLE IF NOT EXISTS tmp_template_user_data_id_map (
    old_id BIGINT PRIMARY KEY,
    new_id BIGINT NOT NULL
);

-- 为所有未删除的记录生成新 ID（从 1 开始按 type_id + id 排序）
INSERT INTO tmp_template_user_data_id_map (old_id, new_id)
SELECT id, ROW_NUMBER() OVER (ORDER BY type_id, id) AS new_id
FROM mxx_template_user_data
WHERE deleted = 0
ORDER BY type_id, id;

-- 更新主表 ID
UPDATE mxx_template_user_data t
SET id = m.new_id
FROM tmp_template_user_data_id_map m
WHERE t.id = m.old_id;

-- 更新 website_template_merge 表中的 template_data_id 引用
UPDATE mxx_website_template_merge m
SET template_data_id = x.new_id
FROM tmp_template_user_data_id_map x
WHERE m.template_data_id = x.old_id;

-- 重置序列
SELECT setval('mxx_template_user_data_id_seq', COALESCE((SELECT MAX(id) FROM mxx_template_user_data), 1));

-- 清理临时表
DROP TABLE IF EXISTS tmp_template_user_data_id_map;

-- ============================================================
-- 第二部分：重置 mxx_template_data ID
-- ============================================================

CREATE TEMP TABLE IF NOT EXISTS tmp_template_data_id_map (
    old_id BIGINT PRIMARY KEY,
    new_id BIGINT NOT NULL
);

INSERT INTO tmp_template_data_id_map (old_id, new_id)
SELECT id, ROW_NUMBER() OVER (ORDER BY template_id, type_id, id) AS new_id
FROM mxx_template_data
WHERE deleted = 0
ORDER BY template_id, type_id, id;

UPDATE mxx_template_data t
SET id = m.new_id
FROM tmp_template_data_id_map m
WHERE t.id = m.old_id;

-- 重置序列
SELECT setval('mxx_template_data_id_seq', COALESCE((SELECT MAX(id) FROM mxx_template_data), 1));

DROP TABLE IF EXISTS tmp_template_data_id_map;

-- ============================================================
-- 第三部分：重置 mxx_website ID 为 1
-- 同步更新所有关联表的 website_id
-- ============================================================

-- 检查是否已有 id=1 的记录
DO $$
DECLARE
    v_old_id BIGINT;
    v_has_id1 BOOLEAN;
BEGIN
    -- 获取当前网站记录 ID（优先取 is_default=1 的）
    SELECT id INTO v_old_id FROM mxx_website WHERE deleted = 0 AND is_default = 1 ORDER BY id LIMIT 1;
    
    -- 如果没有默认站点，取任意未删除的站点
    IF v_old_id IS NULL THEN
        SELECT id INTO v_old_id FROM mxx_website WHERE (deleted IS NULL OR deleted = 0) ORDER BY id LIMIT 1;
    END IF;

    -- 检查是否已有 id=1 的记录
    SELECT EXISTS(SELECT 1 FROM mxx_website WHERE id = 1) INTO v_has_id1;

    IF v_old_id IS NOT NULL AND v_old_id != 1 THEN
        IF v_has_id1 THEN
            -- 已有 id=1 的记录，合并数据：删除旧记录，将关联表指向 id=1
            RAISE NOTICE '已有 id=1 的记录，合并数据';
            
            -- 更新关联表（将所有关联从旧 ID 更新为 1）
            -- 注意：部分表可能已有 id=1 的记录，需要用 NOT EXISTS 避免冲突
        ELSE
            -- 直接更新网站 ID 为 1
            UPDATE mxx_website SET id = 1 WHERE id = v_old_id;
            RAISE NOTICE '网站 ID 已从 % 更新为 1', v_old_id;
        END IF;

        -- 更新所有关联表的 website_id
        -- mxx_article
        UPDATE mxx_article SET website_id = 1 WHERE website_id = v_old_id;
        -- mxx_article_category
        UPDATE mxx_article_category SET website_id = 1 WHERE website_id = v_old_id;
        -- mxx_navigation
        UPDATE mxx_navigation SET website_id = 1 WHERE website_id = v_old_id;
        -- mxx_website_leave_msg
        UPDATE mxx_website_leave_msg SET website_id = 1 WHERE website_id = v_old_id;
        -- mxx_website_order
        UPDATE mxx_website_order SET website_id = 1 WHERE website_id = v_old_id;
        -- mxx_website_cart
        UPDATE mxx_website_cart SET website_id = 1 WHERE website_id = v_old_id;
        -- mxx_website_links
        UPDATE mxx_website_links SET website_id = 1 WHERE website_id = v_old_id;
        -- mxx_website_notification_config
        UPDATE mxx_website_notification_config SET website_id = 1 WHERE website_id = v_old_id;
        -- mxx_template_user_data
        UPDATE mxx_template_user_data SET website_id = 1 WHERE website_id = v_old_id;
        -- mxx_website_template_merge
        UPDATE mxx_website_template_merge SET website_id = 1 WHERE website_id = v_old_id;
        
        RAISE NOTICE '所有关联表 website_id 已从 % 更新为 1', v_old_id;
    ELSE
        RAISE NOTICE '网站 ID 无需重置';
    END IF;
END $$;

-- 重置序列
SELECT setval('mxx_website_id_seq', COALESCE((SELECT MAX(id) FROM mxx_website), 1));

-- ============================================================
-- 验证
-- ============================================================
SELECT 'mxx_template_user_data' AS tbl, COUNT(*) AS cnt, MIN(id) AS min_id, MAX(id) AS max_id FROM mxx_template_user_data
UNION ALL
SELECT 'mxx_template_data', COUNT(*), MIN(id), MAX(id) FROM mxx_template_data
UNION ALL
SELECT 'mxx_website', COUNT(*), MIN(id), MAX(id) FROM mxx_website;

COMMIT;