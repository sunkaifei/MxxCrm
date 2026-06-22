-- ================================================
-- SKU规格管理菜单迁移脚本
-- 在产品管理下添加SKU规格管理菜单
-- 注意：此脚本适用于线上数据库已部署的表结构
-- ================================================

-- 1. SKU规格管理页面菜单（若尚未插入）
INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, icon, create_time, updated_time, deleted)
SELECT 125, 'page.product.sku.title', 'MENU', 'ProductSku', '/product/sku', 'product/sku/index', 'product:sku:list', 5, 0, 0, 0, 0, 0, 0, 1, 'Appstore', now(), now(), 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE route_name = 'ProductSku');

-- 2. 语言键需要在对应语言文件中添加（如 zh_CN、en）：
-- 在 zh_CN 语言文件（如 frontend/apps/web-antd/src/locales/zh_CN.ts）中添加：
--   'page.product.sku.title': 'SKU规格管理'
-- 在 en 语言文件（如 frontend/apps/web-antd/src/locales/en.ts）中添加：
--   'page.product.sku.title': 'SKU Management'

-- 3. 为新菜单分配角色权限（以产品管理角色为例）
-- INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
-- SELECT r.id, (SELECT id FROM mxx_system_menu WHERE route_name = 'ProductSku')
-- FROM mxx_system_role r
-- WHERE r.id IN (4, 5, 6, 7, 8)  -- 替换为实际角色ID
-- AND NOT EXISTS (SELECT 1 FROM mxx_system_role_menu_merge rm WHERE rm.role_id = r.id AND rm.menu_id = (SELECT id FROM mxx_system_menu WHERE route_name = 'ProductSku'));
