-- 为部门管理列表(id=229)添加4个按钮菜单：查看详细、编辑、添加、删除
-- 使用幂等写法 ON CONFLICT DO NOTHING，避免重复插入
-- perm 权限标识严格匹配后端 #[protect] 注解

-- 1. 添加按钮
INSERT INTO mxx_system_menu (parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted)
SELECT 229, NULL, 'page.system.dept.button.create', 'BUTTON', 'DeptCreate', '', 'system/dept/index', 'system:dept:save', 1, 0, 0, 0, 0, 0, 0, 1, '', NULL, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'system:dept:save' AND deleted = 0);

-- 2. 编辑按钮
INSERT INTO mxx_system_menu (parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted)
SELECT 229, NULL, 'page.system.dept.button.edit', 'BUTTON', 'DeptEdit', '', 'system/dept/index', 'system:dept:update', 1, 0, 0, 0, 0, 0, 0, 2, '', NULL, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'system:dept:update' AND deleted = 0);

-- 3. 删除按钮
INSERT INTO mxx_system_menu (parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted)
SELECT 229, NULL, 'page.system.dept.button.delete', 'BUTTON', 'DeptDelete', '', 'system/dept/index', 'system:dept:delete', 1, 0, 0, 0, 0, 0, 0, 3, '', NULL, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'system:dept:delete' AND deleted = 0);

-- 4. 查看详细按钮
INSERT INTO mxx_system_menu (parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted)
SELECT 229, NULL, 'page.system.dept.button.view', 'BUTTON', 'DeptView', '', 'system/dept/index', 'system:dept:view', 1, 0, 0, 0, 0, 0, 0, 4, '', NULL, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'system:dept:view' AND deleted = 0);

-- 验证插入结果
SELECT id, parent_id, name, type, route_name, path, component, perm, sort FROM mxx_system_menu WHERE parent_id = 229 AND deleted = 0 ORDER BY sort;
