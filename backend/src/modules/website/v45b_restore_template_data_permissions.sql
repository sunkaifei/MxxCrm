-- v45b_restore_template_data_permissions.sql
-- 恢复 template:data:* 权限到模板管理菜单(413)下（使用不冲突的ID）

BEGIN;

-- 新增 template:data:* 按钮权限到模板管理菜单(413)下
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 794, 413, 'page.website.templateData.button.list', 'BUTTON', 'template:data:list', 10, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 794);

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 795, 413, 'page.website.templateData.button.add', 'BUTTON', 'template:data:add', 11, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 795);

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 796, 413, 'page.website.templateData.button.edit', 'BUTTON', 'template:data:update', 12, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 796);

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 797, 413, 'page.website.templateData.button.delete', 'BUTTON', 'template:data:delete', 13, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 797);

-- template:data:view already exists as id=424

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 798, 413, 'page.website.templateData.button.revision', 'BUTTON', 'template:revision:list', 15, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 798);

-- 授权给 super_admin(1)
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT 1, id FROM mxx_system_menu WHERE id IN (794, 795, 796, 797, 424, 798)
AND NOT EXISTS (SELECT 1 FROM mxx_system_role_menu_merge rm WHERE rm.role_id = 1 AND rm.menu_id = mxx_system_menu.id);

-- 验证
SELECT id, parent_id, name, perm, type, sort FROM mxx_system_menu WHERE parent_id = 413 ORDER BY sort, id;

COMMIT;
