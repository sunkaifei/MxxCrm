-- 创建财务用户并补充财务角色缺失的权限
-- 密码: finance123 (bcrypt hash)

-- 1. 创建财务用户 (使用与 admin 相同的密码哈希)
INSERT INTO mxx_system_admin (user_name, nick_name, email, mobile, password, status, deleted, create_time, update_time, create_by, update_by, sort, user_type, gender)
SELECT 'finance', '财务专员', 'finance@mxxcrm.com', '13800000010', password, 0, 0, NOW(), NOW(), 'admin', 'admin', 9, 0, 0
FROM mxx_system_admin WHERE user_name='admin' AND deleted=0
ON CONFLICT (user_name) DO NOTHING;

-- 2. 关联财务用户到财务角色
INSERT INTO mxx_system_admin_role_merge (admin_id, role_id, create_time)
SELECT a.id, 10, NOW()
FROM mxx_system_admin a
WHERE a.user_name='finance' AND a.deleted=0
  AND NOT EXISTS (
    SELECT 1 FROM mxx_system_admin_role_merge m WHERE m.admin_id=a.id AND m.role_id=10
  );

-- 3. 给财务角色补充缺失的权限
-- 3.1 检查并添加 finance:salary:manage 权限菜单
INSERT INTO mxx_system_menu (parent_id, name, type, path, perm, status, sort, icon, create_time, update_time, deleted)
SELECT 321, 'page.finance.salary.button.manage', 'button', '', 'finance:salary:manage', 1, 30, '', NOW(), NOW(), 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm='finance:salary:manage' AND deleted=0);

-- 3.2 关联 finance:salary:manage 权限到财务角色
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id, create_time)
SELECT 10, m.id, NOW()
FROM mxx_system_menu m
WHERE m.perm='finance:salary:manage' AND m.deleted=0
  AND NOT EXISTS (
    SELECT 1 FROM mxx_system_role_menu_merge rmm WHERE rmm.role_id=10 AND rmm.menu_id=m.id
  );

-- 3.3 同时关联给超级管理员
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id, create_time)
SELECT 4, m.id, NOW()
FROM mxx_system_menu m
WHERE m.perm='finance:salary:manage' AND m.deleted=0
  AND NOT EXISTS (
    SELECT 1 FROM mxx_system_role_menu_merge rmm WHERE rmm.role_id=4 AND rmm.menu_id=m.id
  );

-- 4. 检查并添加团队提成权限菜单
INSERT INTO mxx_system_menu (parent_id, name, type, path, perm, status, sort, icon, create_time, update_time, deleted)
SELECT 315, 'page.finance.teamCommission.title', 'menu', 'team-commission', 'finance:team-commission:list', 1, 50, '', NOW(), NOW(), 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm='finance:team-commission:list' AND deleted=0);

INSERT INTO mxx_system_menu (parent_id, name, type, path, perm, status, sort, icon, create_time, update_time, deleted)
SELECT 315, 'page.finance.teamCommission.button.manage', 'button', '', 'finance:team-commission:manage', 1, 51, '', NOW(), NOW(), 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm='finance:team-commission:manage' AND deleted=0);

-- 4.1 关联团队提成权限到财务角色和超级管理员
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id, create_time)
SELECT r.role_id, m.id, NOW()
FROM mxx_system_menu m
CROSS JOIN (SELECT 4 AS role_id UNION SELECT 10 AS role_id) r
WHERE m.perm IN ('finance:team-commission:list', 'finance:team-commission:manage')
  AND m.deleted=0
  AND NOT EXISTS (
    SELECT 1 FROM mxx_system_role_menu_merge rmm WHERE rmm.role_id=r.role_id AND rmm.menu_id=m.id
  );

-- 5. 验证
SELECT '用户创建' AS step, a.id, a.user_name, a.nick_name
FROM mxx_system_admin a WHERE a.user_name='finance' AND a.deleted=0;

SELECT '角色关联' AS step, arm.admin_id, arm.role_id, r.role_name
FROM mxx_system_admin_role_merge arm
JOIN mxx_system_role r ON r.id=arm.role_id
WHERE arm.admin_id=(SELECT id FROM mxx_system_admin WHERE user_name='finance' AND deleted=0);
