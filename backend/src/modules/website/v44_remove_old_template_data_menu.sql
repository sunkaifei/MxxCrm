-- ============================================================
-- v44_remove_old_template_data_menu.sql
-- 删除旧的"模板数据"菜单，只保留"模板管理"菜单
-- 模板数据（template-data）已合并到模板管理（template/index.vue）中
-- ============================================================

BEGIN;

-- 1. 删除角色-菜单关联（旧模板数据菜单及其子菜单）
DELETE FROM mxx_system_role_menu_merge
WHERE menu_id IN (
  SELECT id FROM mxx_system_menu WHERE parent_id = 400
  UNION ALL
  SELECT 400
);

-- 2. 删除旧模板数据菜单的子菜单（按钮权限）
DELETE FROM mxx_system_menu WHERE parent_id = 400;

-- 3. 删除旧模板数据菜单本身
DELETE FROM mxx_system_menu WHERE id = 400;

-- 4. 验证
SELECT id, parent_id, name, path, type, sort
FROM mxx_system_menu
WHERE parent_id = 345
ORDER BY sort, id;

COMMIT;