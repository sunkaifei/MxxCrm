-- 为销售角色添加工资查看权限
-- 执行方式：psql -h 115.190.210.106 -U postgres -d mxxcrm_data -f v6_add_salary_perms.sql

-- ============================================================
-- 1. 为销售总监(6)、销售经理(7)、业务员(8) 添加工资查看权限
-- ============================================================
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT r.id, m.id
FROM mxx_system_role r
CROSS JOIN mxx_system_menu m
WHERE r.id IN (6, 7, 8)
  AND m.id IN (321, 326)  -- finance:salary:list 权限
  AND NOT EXISTS (
      SELECT 1 FROM mxx_system_role_menu_merge rmm
      WHERE rmm.role_id = r.id AND rmm.menu_id = m.id
  );

-- ============================================================
-- 2. 创建工资确认权限菜单（如果不存在）
-- ============================================================
INSERT INTO mxx_system_menu (id, parent_id, name, perm, type, sort, status, create_time)
SELECT 568, 321, 'page.finance.salary.button.confirm', 'finance:salary:confirm', 3, 6, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 568);

-- 为所有销售角色添加确认权限
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT r.id, 568
FROM mxx_system_role r
WHERE r.id IN (4, 6, 7, 8, 10)  -- 超管、销售总监、销售经理、业务员、财务
  AND NOT EXISTS (
      SELECT 1 FROM mxx_system_role_menu_merge rmm
      WHERE rmm.role_id = r.id AND rmm.menu_id = 568
  );

-- ============================================================
-- 3. 验证权限
-- ============================================================
SELECT '== 权限验证 ==' AS info;
SELECT r.id AS role_id, r.role_name, m.perm
FROM mxx_system_role r
JOIN mxx_system_role_menu_merge rmm ON r.id = rmm.role_id
JOIN mxx_system_menu m ON rmm.menu_id = m.id
WHERE r.id IN (6, 7, 8) AND m.perm LIKE 'finance:salary%'
ORDER BY r.id, m.perm;
