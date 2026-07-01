-- =============================================================
-- 企业信息管理菜单初始化脚本
-- 顶级菜单"公司" + 子菜单"企业信息" + 按钮权限
-- 当前最大菜单ID=360，本脚本从 361 开始分配
-- 日期：2026-07-01
-- =============================================================

BEGIN;

-- 1. 顶级菜单"公司"（FOLDER 类型，parent_id=0，sort=35）
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted)
VALUES (361, 0, '', 'page.company.title', 'FOLDER', 'Company', '/company', '', '', 1, 0, 0, 0, 0, 0, 1, 35, 'lucide:building-2', '', NULL, NOW(), NOW(), 0);

-- 2. 子菜单"企业信息"（MENU 类型）
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted)
VALUES (362, 361, '', 'page.company.info.title', 'MENU', 'CompanyInfo', '/company/info', 'company/info/index', 'company:info:list', 1, 0, 0, 0, 0, 0, 1, 0, 'lucide:info', '', NULL, NOW(), NOW(), 0);

-- 3. 按钮权限
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted) VALUES
(363, 362, '', 'page.company.info.button.edit', 'BUTTON', 'CompanyInfoEdit', '', 'company/info/index', 'company:info:edit', 1, 0, 0, 0, 0, 0, 0, 1, '', '', NULL, NOW(), NOW(), 0),
(364, 362, '', 'page.company.info.button.accountSave', 'BUTTON', 'CompanyAccountSave', '', 'company/info/index', 'company:account:save', 1, 0, 0, 0, 0, 0, 0, 2, '', '', NULL, NOW(), NOW(), 0),
(365, 362, '', 'page.company.info.button.accountDelete', 'BUTTON', 'CompanyAccountDelete', '', 'company/info/index', 'company:account:delete', 1, 0, 0, 0, 0, 0, 0, 3, '', '', NULL, NOW(), NOW(), 0);

-- 4. 给超级管理员角色(role_id=1)分配新菜单权限
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id) VALUES
(1, 361), (1, 362), (1, 363), (1, 364), (1, 365);

COMMIT;

-- 校验：执行后可运行以下查询确认
-- SELECT id, parent_id, name, type, route_name, path, component, perm, sort, icon FROM mxx_system_menu WHERE id BETWEEN 361 AND 365 ORDER BY id;
