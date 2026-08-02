-- 创建财务用户(使用 admin 的密码哈希)
INSERT INTO mxx_system_admin (user_name, nick_name, email, mobile, password, status, deleted, create_time, update_time, create_by, update_by, sort, user_type, gender)
SELECT 'finance', '财务专员', 'finance@mxxcrm.com', '13800000010', a.password, 0, 0, NOW(), NOW(), 'admin', 'admin', 9, 0, 0
FROM mxx_system_admin a
WHERE a.user_name='admin' AND a.deleted=0
  AND NOT EXISTS (SELECT 1 FROM mxx_system_admin x WHERE x.user_name='finance' AND x.deleted=0);

-- 关联财务用户到财务角色(id=10)
INSERT INTO mxx_system_admin_role_merge (admin_id, role_id, create_time)
SELECT a.id, 10, NOW()
FROM mxx_system_admin a
WHERE a.user_name='finance' AND a.deleted=0
  AND NOT EXISTS (
    SELECT 1 FROM mxx_system_admin_role_merge m WHERE m.admin_id=a.id AND m.role_id=10
  );

-- 验证
SELECT a.id, a.user_name, a.nick_name, arm.role_id, r.role_name
FROM mxx_system_admin a
LEFT JOIN mxx_system_admin_role_merge arm ON arm.admin_id=a.id
LEFT JOIN mxx_system_role r ON r.id=arm.role_id
WHERE a.user_name='finance' AND a.deleted=0;
