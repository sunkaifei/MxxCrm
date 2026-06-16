-- ================================================
-- Mxx-CRM 数据库结构验证脚本
-- 检查所有表是否符合开发文档要求
-- ================================================

-- 1. 检查枚举类型
SELECT '枚举类型' AS check_type, COUNT(*) AS count FROM pg_type WHERE typnamespace = (SELECT oid FROM pg_namespace WHERE nspname = 'public') AND typtype = 'e';

-- 2. 检查所有表
SELECT '所有表' AS check_type, COUNT(*) AS count FROM information_schema.tables WHERE table_schema='public' AND table_type='BASE TABLE';

-- 3. 检查系统模块表
SELECT '系统模块表' AS check_type, COUNT(*) AS count FROM information_schema.tables WHERE table_schema='public' AND table_name LIKE 'mxx_system_%' AND table_type='BASE TABLE';

-- 4. 检查CRM模块表
SELECT 'CRM模块表' AS check_type, COUNT(*) AS count FROM information_schema.tables WHERE table_schema='public' AND table_name LIKE 'mxx_crm_%' AND table_type='BASE TABLE';

-- 5. 检查销售模块表
SELECT '销售模块表' AS check_type, COUNT(*) AS count FROM information_schema.tables WHERE table_schema='public' AND table_name LIKE 'mxx_sale_%' AND table_type='BASE TABLE';

-- 6. 检查产品模块表
SELECT '产品模块表' AS check_type, COUNT(*) AS count FROM information_schema.tables WHERE table_schema='public' AND table_name LIKE 'mxx_product_%' AND table_type='BASE TABLE';

-- 7. 检查库存模块表
SELECT '库存模块表' AS check_type, COUNT(*) AS count FROM information_schema.tables WHERE table_schema='public' AND table_name LIKE 'mxx_inventory_%' AND table_type='BASE TABLE';

-- 8. 检查采购模块表
SELECT '采购模块表' AS check_type, COUNT(*) AS count FROM information_schema.tables WHERE table_schema='public' AND table_name LIKE 'mxx_purchase_%' AND table_type='BASE TABLE';

-- 9. 检查关联表
SELECT '关联表' AS check_type, COUNT(*) AS count FROM information_schema.tables WHERE table_schema='public' AND table_name LIKE '%_merge' AND table_type='BASE TABLE';

-- 10. 检查视图
SELECT '视图' AS check_type, COUNT(*) AS count FROM information_schema.views WHERE table_schema='public';

-- 11. 检查mxx_system_admin字段
SELECT column_name, data_type, character_maximum_length, is_nullable, column_default
FROM information_schema.columns WHERE table_schema='public' AND table_name='mxx_system_admin'
ORDER BY ordinal_position;

-- 12. 检查mxx_system_menu字段
SELECT column_name, data_type, character_maximum_length, is_nullable, column_default
FROM information_schema.columns WHERE table_schema='public' AND table_name='mxx_system_menu'
ORDER BY ordinal_position;

-- 13. 检查mxx_crm_customer字段
SELECT column_name, data_type, character_maximum_length, is_nullable, column_default
FROM information_schema.columns WHERE table_schema='public' AND table_name='mxx_crm_customer'
ORDER BY ordinal_position;

-- 14. 检查索引数量
SELECT '索引' AS check_type, COUNT(*) AS count FROM pg_indexes WHERE schemaname='public';

-- 15. 检查触发器数量
SELECT '触发器' AS check_type, COUNT(*) AS count FROM pg_triggers WHERE schemaname='public' AND NOT tgisconstraint;

-- 16. 检查字段注释
SELECT 'mxx_system_admin字段注释' AS check_type, COUNT(*) AS count FROM pg_description 
WHERE objoid = (SELECT oid FROM pg_class WHERE relname='mxx_system_admin') AND objsubid > 0;

-- 17. 检查缺失的表
WITH expected_tables AS (
    SELECT unnest(ARRAY[
        'mxx_system_admin', 'mxx_system_dept', 'mxx_system_post', 'mxx_system_role', 'mxx_system_menu',
        'mxx_system_admin_role_merge', 'mxx_system_admin_dept_merge', 'mxx_system_admin_post_merge',
        'mxx_system_role_menu_merge', 'mxx_system_dept_menu_merge', 'mxx_system_admin_notice_merge',
        'mxx_crm_lead', 'mxx_crm_customer', 'mxx_crm_contact', 'mxx_crm_followup',
        'mxx_crm_opportunity', 'mxx_crm_contract', 'mxx_crm_lead_pool',
        'mxx_sale_order', 'mxx_sale_order_item', 'mxx_sale_payment',
        'mxx_product_main', 'mxx_product_category',
        'mxx_inventory_warehouse', 'mxx_inventory_stock', 'mxx_inventory_transaction',
        'mxx_purchase_supplier', 'mxx_purchase_po', 'mxx_purchase_item',
        'mxx_attachment_file', 'mxx_seq_generator'
    ]) AS table_name
)
SELECT '缺失的表' AS check_type, et.table_name FROM expected_tables et
LEFT JOIN information_schema.tables it ON et.table_name = it.table_name AND it.table_schema='public'
WHERE it.table_name IS NULL;