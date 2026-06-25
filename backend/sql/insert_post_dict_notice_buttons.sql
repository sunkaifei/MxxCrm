-- 为岗位管理(id=230)、字典管理(id=232)、通知公告(id=233)添加4个按钮菜单：查看详细、编辑、添加、删除
-- 使用幂等写法 WHERE NOT EXISTS，避免重复插入

-- ==================== 岗位管理 (parent_id=230) ====================
-- 1. 添加按钮
INSERT INTO mxx_system_menu (parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted)
SELECT 230, NULL, 'page.system.post.button.create', 'BUTTON', 'PostCreate', '', 'system/post/index', 'system:post:save', 1, 0, 0, 0, 0, 0, 0, 1, '', NULL, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'system:post:save' AND deleted = 0);

-- 2. 编辑按钮
INSERT INTO mxx_system_menu (parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted)
SELECT 230, NULL, 'page.system.post.button.edit', 'BUTTON', 'PostEdit', '', 'system/post/index', 'system:post:update', 1, 0, 0, 0, 0, 0, 0, 2, '', NULL, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'system:post:update' AND deleted = 0);

-- 3. 删除按钮
INSERT INTO mxx_system_menu (parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted)
SELECT 230, NULL, 'page.system.post.button.delete', 'BUTTON', 'PostDelete', '', 'system/post/index', 'system:post:delete', 1, 0, 0, 0, 0, 0, 0, 3, '', NULL, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'system:post:delete' AND deleted = 0);

-- 4. 查看详细按钮
INSERT INTO mxx_system_menu (parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted)
SELECT 230, NULL, 'page.system.post.button.view', 'BUTTON', 'PostView', '', 'system/post/index', 'system:post:view', 1, 0, 0, 0, 0, 0, 0, 4, '', NULL, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'system:post:view' AND deleted = 0);

-- ==================== 字典管理 (parent_id=232) ====================
-- 1. 添加按钮
INSERT INTO mxx_system_menu (parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted)
SELECT 232, NULL, 'page.system.dict.button.create', 'BUTTON', 'DictCreate', '', 'system/dict/index', 'system:dict:save', 1, 0, 0, 0, 0, 0, 0, 1, '', NULL, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'system:dict:save' AND deleted = 0);

-- 2. 编辑按钮
INSERT INTO mxx_system_menu (parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted)
SELECT 232, NULL, 'page.system.dict.button.edit', 'BUTTON', 'DictEdit', '', 'system/dict/index', 'system:dict:update', 1, 0, 0, 0, 0, 0, 0, 2, '', NULL, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'system:dict:update' AND deleted = 0);

-- 3. 删除按钮
INSERT INTO mxx_system_menu (parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted)
SELECT 232, NULL, 'page.system.dict.button.delete', 'BUTTON', 'DictDelete', '', 'system/dict/index', 'system:dict:delete', 1, 0, 0, 0, 0, 0, 0, 3, '', NULL, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'system:dict:delete' AND deleted = 0);

-- 4. 查看详细按钮
INSERT INTO mxx_system_menu (parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted)
SELECT 232, NULL, 'page.system.dict.button.view', 'BUTTON', 'DictView', '', 'system/dict/index', 'system:dict:view', 1, 0, 0, 0, 0, 0, 0, 4, '', NULL, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'system:dict:view' AND deleted = 0);

-- ==================== 通知公告 (parent_id=233) ====================
-- 1. 添加按钮
INSERT INTO mxx_system_menu (parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted)
SELECT 233, NULL, 'page.system.notice.button.create', 'BUTTON', 'NoticeCreate', '', 'system/notice/index', 'system:notice:add', 1, 0, 0, 0, 0, 0, 0, 1, '', NULL, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'system:notice:add' AND deleted = 0);

-- 2. 编辑按钮
INSERT INTO mxx_system_menu (parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted)
SELECT 233, NULL, 'page.system.notice.button.edit', 'BUTTON', 'NoticeEdit', '', 'system/notice/index', 'system:notice:update', 1, 0, 0, 0, 0, 0, 0, 2, '', NULL, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'system:notice:update' AND deleted = 0);

-- 3. 删除按钮
INSERT INTO mxx_system_menu (parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted)
SELECT 233, NULL, 'page.system.notice.button.delete', 'BUTTON', 'NoticeDelete', '', 'system/notice/index', 'system:notice:delete', 1, 0, 0, 0, 0, 0, 0, 3, '', NULL, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'system:notice:delete' AND deleted = 0);

-- 4. 查看详细按钮
INSERT INTO mxx_system_menu (parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted)
SELECT 233, NULL, 'page.system.notice.button.view', 'BUTTON', 'NoticeView', '', 'system/notice/index', 'system:notice:view', 1, 0, 0, 0, 0, 0, 0, 4, '', NULL, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'system:notice:view' AND deleted = 0);

-- 验证结果
SELECT id, parent_id, name, type, perm, sort FROM mxx_system_menu WHERE parent_id IN (230, 232, 233) AND deleted = 0 AND type = 'BUTTON' ORDER BY parent_id, sort;
