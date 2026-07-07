-- =============================================================
-- 客户列表 Tab 权限菜单初始化脚本
-- 为不同 Tab 添加按钮级权限码，用于控制 Tab 显示
-- 当前最大菜单ID=370（从 backup 和 company_menu 推算）
-- =============================================================

BEGIN;

-- 1. 添加Tab按钮权限（parent_id=161 客户管理菜单）
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted) VALUES
(371, 161, '', 'page.crm.customer.tab.all', 'BUTTON', '', '', '', 'crm:customer:tab:all', 1, 0, 0, 0, 0, 0, 0, 5, '', '', NULL, NOW(), NOW(), 0),
(372, 161, '', 'page.crm.customer.tab.my', 'BUTTON', '', '', '', 'crm:customer:tab:my', 1, 0, 0, 0, 0, 0, 0, 6, '', '', NULL, NOW(), NOW(), 0),
(373, 161, '', 'page.crm.customer.tab.subordinate', 'BUTTON', '', '', '', 'crm:customer:tab:subordinate', 1, 0, 0, 0, 0, 0, 0, 7, '', '', NULL, NOW(), NOW(), 0),
(374, 161, '', 'page.crm.customer.tab.todayFollow', 'BUTTON', '', '', '', 'crm:customer:tab:todayFollow', 1, 0, 0, 0, 0, 0, 0, 8, '', '', NULL, NOW(), NOW(), 0);

-- 2. 给超级管理员(role_id=4)、系统管理员(role_id=5)、销售总监(role_id=6)分配所有Tab权限
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id) VALUES
(4, 371), (4, 372), (4, 373), (4, 374),
(5, 371), (5, 372), (5, 373), (5, 374),
(6, 371), (6, 372), (6, 373), (6, 374);

COMMIT;

-- 校验：查看新增的菜单记录
-- SELECT id, parent_id, name, type, perm, sort FROM mxx_system_menu WHERE id BETWEEN 371 AND 374 ORDER BY id;
