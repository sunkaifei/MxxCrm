-- ================================================
-- Mxx-CRM 数据库清理和重新初始化脚本
-- 清理旧数据，重新建立正确的数据关联
-- ================================================

-- ================================================
-- 1. 备份当前数据（可选，如需保留请取消注释）
-- ================================================

-- CREATE TABLE mxx_system_admin_backup AS SELECT * FROM mxx_system_admin;
-- CREATE TABLE mxx_system_role_backup AS SELECT * FROM mxx_system_role;
-- CREATE TABLE mxx_system_admin_role_merge_backup AS SELECT * FROM mxx_system_admin_role_merge;
-- CREATE TABLE mxx_system_menu_backup AS SELECT * FROM mxx_system_menu;

-- ================================================
-- 2. 清理关联表数据
-- ================================================

TRUNCATE TABLE mxx_system_admin_role_merge RESTART IDENTITY;
TRUNCATE TABLE mxx_system_admin_dept_merge RESTART IDENTITY;
TRUNCATE TABLE mxx_system_admin_post_merge RESTART IDENTITY;
TRUNCATE TABLE mxx_system_role_menu_merge RESTART IDENTITY;

-- ================================================
-- 3. 清理旧的用户数据（保留新创建的用户）
-- ================================================

DELETE FROM mxx_system_admin WHERE id IN (
    SELECT id FROM mxx_system_admin 
    WHERE user_name IN ('admin', 'admin1') AND id NOT IN (
        SELECT id FROM mxx_system_admin WHERE user_name='admin' ORDER BY id DESC LIMIT 1
    )
);

-- ================================================
-- 4. 更新保留的admin用户状态
-- ================================================

UPDATE mxx_system_admin SET status=0, user_type=1 WHERE user_name='admin';

-- ================================================
-- 5. 清理旧的角色数据（保留新创建的角色）
-- ================================================

DELETE FROM mxx_system_role WHERE role_key IN ('admin', 'common', 'erwer');

-- ================================================
-- 6. 清理旧的菜单数据，保留新创建的菜单
-- ================================================

DELETE FROM mxx_system_menu WHERE id NOT IN (
    SELECT id FROM mxx_system_menu WHERE parent_id = 0 AND name IN ('系统管理', 'CRM客户管理', '销售管理', '产品管理', '采购管理', '附件管理', '仪表盘')
    UNION ALL
    SELECT id FROM mxx_system_menu WHERE parent_id IN (
        SELECT id FROM mxx_system_menu WHERE parent_id = 0 AND name IN ('系统管理', 'CRM客户管理', '销售管理', '产品管理', '采购管理', '附件管理', '仪表盘')
    )
    UNION ALL
    SELECT id FROM mxx_system_menu WHERE parent_id IN (
        SELECT id FROM mxx_system_menu WHERE parent_id IN (
            SELECT id FROM mxx_system_menu WHERE parent_id = 0 AND name IN ('系统管理', 'CRM客户管理', '销售管理', '产品管理', '采购管理', '附件管理', '仪表盘')
        )
    )
);

-- ================================================
-- 7. 重新建立用户角色关联
-- ================================================

INSERT INTO mxx_system_admin_role_merge (admin_id, role_id) VALUES
((SELECT id FROM mxx_system_admin WHERE user_name='admin'), (SELECT id FROM mxx_system_role WHERE role_key='super_admin')),
((SELECT id FROM mxx_system_admin WHERE user_name='system'), (SELECT id FROM mxx_system_role WHERE role_key='system_admin')),
((SELECT id FROM mxx_system_admin WHERE user_name='sales'), (SELECT id FROM mxx_system_role WHERE role_key='sales_director')),
((SELECT id FROM mxx_system_admin WHERE user_name='manager'), (SELECT id FROM mxx_system_role WHERE role_key='sales_manager')),
((SELECT id FROM mxx_system_admin WHERE user_name='rep'), (SELECT id FROM mxx_system_role WHERE role_key='sales_rep')),
((SELECT id FROM mxx_system_admin WHERE user_name='purchase'), (SELECT id FROM mxx_system_role WHERE role_key='purchase_staff'));

-- ================================================
-- 8. 重新建立用户部门关联
-- ================================================

INSERT INTO mxx_system_admin_dept_merge (admin_id, dept_id) VALUES
((SELECT id FROM mxx_system_admin WHERE user_name='admin'), (SELECT id FROM mxx_system_dept WHERE code='ROOT')),
((SELECT id FROM mxx_system_admin WHERE user_name='system'), (SELECT id FROM mxx_system_dept WHERE code='TECH')),
((SELECT id FROM mxx_system_admin WHERE user_name='sales'), (SELECT id FROM mxx_system_dept WHERE code='SALES')),
((SELECT id FROM mxx_system_admin WHERE user_name='manager'), (SELECT id FROM mxx_system_dept WHERE code='SALES_01')),
((SELECT id FROM mxx_system_admin WHERE user_name='rep'), (SELECT id FROM mxx_system_dept WHERE code='SALES_01')),
((SELECT id FROM mxx_system_admin WHERE user_name='purchase'), (SELECT id FROM mxx_system_dept WHERE code='PURCHASE'));

-- ================================================
-- 9. 重新建立用户岗位关联
-- ================================================

INSERT INTO mxx_system_admin_post_merge (admin_id, post_id) VALUES
((SELECT id FROM mxx_system_admin WHERE user_name='admin'), (SELECT id FROM mxx_system_post WHERE post_code='SA')),
((SELECT id FROM mxx_system_admin WHERE user_name='system'), (SELECT id FROM mxx_system_post WHERE post_code='SA')),
((SELECT id FROM mxx_system_admin WHERE user_name='sales'), (SELECT id FROM mxx_system_post WHERE post_code='SD')),
((SELECT id FROM mxx_system_admin WHERE user_name='manager'), (SELECT id FROM mxx_system_post WHERE post_code='SM')),
((SELECT id FROM mxx_system_admin WHERE user_name='rep'), (SELECT id FROM mxx_system_post WHERE post_code='SR')),
((SELECT id FROM mxx_system_admin WHERE user_name='purchase'), (SELECT id FROM mxx_system_post WHERE post_code='PS'));

-- ================================================
-- 10. 重新建立角色菜单关联
-- ================================================

-- 超级管理员拥有所有权限
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='super_admin'), id 
FROM mxx_system_menu WHERE deleted=0;

-- 系统管理员拥有系统管理权限
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='system_admin'), id 
FROM mxx_system_menu WHERE perm LIKE 'system:%' AND deleted=0;

-- 销售总监拥有CRM和销售权限
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='sales_director'), id 
FROM mxx_system_menu WHERE (perm LIKE 'crm:%' OR perm LIKE 'sale:%' OR perm LIKE 'dashboard:%') AND deleted=0;

-- 销售经理拥有CRM和销售权限
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='sales_manager'), id 
FROM mxx_system_menu WHERE (perm LIKE 'crm:%' OR perm LIKE 'sale:%') AND deleted=0;

-- 业务员拥有线索、客户和跟进权限
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='sales_rep'), id 
FROM mxx_system_menu WHERE (perm LIKE 'crm:lead:%' OR perm LIKE 'crm:customer:%' OR perm LIKE 'crm:followup:%') AND deleted=0;

-- 采购专员拥有采购和产品权限
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='purchase_staff'), id 
FROM mxx_system_menu WHERE (perm LIKE 'purchase:%' OR perm LIKE 'product:%') AND deleted=0;

-- ================================================
-- 11. 更新菜单树路径
-- ================================================

UPDATE mxx_system_menu SET tree_path = CASE 
    WHEN parent_id = 0 THEN id::TEXT
    ELSE (SELECT COALESCE(tree_path, '') FROM mxx_system_menu WHERE id = parent_id) || ',' || id::TEXT
END WHERE parent_id IS NOT NULL;

-- ================================================
-- 12. 验证结果
-- ================================================

SELECT '清理和重新初始化完成' AS result;