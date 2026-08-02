-- v45_restore_template_data_permissions.sql
-- 恢复 template:data:* 权限到模板管理菜单(413)下

BEGIN;

-- 删除重复的按钮权限（771-774 与 414-417 重复）
DELETE FROM mxx_system_role_menu_merge WHERE menu_id IN (771, 772, 773, 774);
DELETE FROM mxx_system_menu WHERE id IN (771, 772, 773, 774);

-- 新增 template:data:* 按钮权限到模板管理菜单(413)下
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 420, 413, 'page.website.templateData.button.list', 'BUTTON', 'template:data:list', 10, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 420);

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 421, 413, 'page.website.templateData.button.add', 'BUTTON', 'template:data:add', 11, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 421);

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 422, 413, 'page.website.templateData.button.edit', 'BUTTON', 'template:data:update', 12, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 422);

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 423, 413, 'page.website.templateData.button.delete', 'BUTTON', 'template:data:delete', 13, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 423);

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 424, 413, 'page.website.templateData.button.view', 'BUTTON', 'template:data:view', 14, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 424);

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 425, 413, 'page.website.templateData.button.revision', 'BUTTON', 'template:revision:list', 15, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 425);

-- 授权给 super_admin(1)
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT 1, id FROM mxx_system_menu WHERE id IN (420, 421, 422, 423, 424, 425)
AND NOT EXISTS (SELECT 1 FROM mxx_system_role_menu_merge rm WHERE rm.role_id = 1 AND rm.menu_id = mxx_system_menu.id);

-- 验证
SELECT id, parent_id, name, perm, type, sort FROM mxx_system_menu WHERE parent_id = 413 ORDER BY sort, id;

COMMIT;
