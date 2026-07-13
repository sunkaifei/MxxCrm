-- ============================================================
-- 更新所有已注册用户的状态为启用
-- ============================================================

-- 查看所有用户状态
SELECT id, user_name, user_type, status FROM mxx_system_admin WHERE deleted = 0;

-- 更新所有非超级管理员用户的状态为启用（status=1）
UPDATE mxx_system_admin SET status = 1 WHERE deleted = 0 AND user_type = 0;

-- 更新菜单数据的redirect（如果之前为空）
UPDATE mxx_system_menu SET 
    component = 'BasicLayout', 
    redirect = '/dashboard/analytics' 
WHERE id = 1;

UPDATE mxx_system_menu SET path = 'analytics' WHERE id = 2;
UPDATE mxx_system_menu SET path = 'workspace' WHERE id = 3;

-- 确保admin角色已授权dashboard菜单
INSERT INTO mxx_system_role_menu_merge (menu_id, role_id, status, create_time, update_time)
SELECT 1, 5, 1, NOW(), NOW() WHERE NOT EXISTS (SELECT 1 FROM mxx_system_role_menu_merge WHERE menu_id = 1 AND role_id = 5);
INSERT INTO mxx_system_role_menu_merge (menu_id, role_id, status, create_time, update_time)
SELECT 2, 5, 1, NOW(), NOW() WHERE NOT EXISTS (SELECT 1 FROM mxx_system_role_menu_merge WHERE menu_id = 2 AND role_id = 5);
INSERT INTO mxx_system_role_menu_merge (menu_id, role_id, status, create_time, update_time)
SELECT 3, 5, 1, NOW(), NOW() WHERE NOT EXISTS (SELECT 1 FROM mxx_system_role_menu_merge WHERE menu_id = 3 AND role_id = 5);

SELECT 'User status updated successfully' AS result;
