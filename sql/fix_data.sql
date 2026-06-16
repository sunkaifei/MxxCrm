-- ================================================
-- Mxx-CRM 数据修复脚本
-- 安全清理重复数据，重新建立正确的关联
-- ================================================

-- 1. 将附件表中引用旧admin用户的记录更新为引用新admin用户
UPDATE mxx_attachment_file SET uploaded_by = (SELECT id FROM mxx_system_admin WHERE user_name='admin' ORDER BY id DESC LIMIT 1) 
WHERE uploaded_by = (SELECT id FROM mxx_system_admin WHERE user_name='admin' ORDER BY id ASC LIMIT 1);

-- 2. 删除重复的admin用户（保留最新创建的）
DELETE FROM mxx_system_admin 
WHERE user_name='admin' 
AND id != (SELECT id FROM mxx_system_admin WHERE user_name='admin' ORDER BY id DESC LIMIT 1);

-- 3. 删除旧的admin1用户
DELETE FROM mxx_system_admin WHERE user_name='admin1';

-- 4. 清理关联表中无效的关联
DELETE FROM mxx_system_admin_role_merge WHERE admin_id NOT IN (SELECT id FROM mxx_system_admin);
DELETE FROM mxx_system_admin_dept_merge WHERE admin_id NOT IN (SELECT id FROM mxx_system_admin);
DELETE FROM mxx_system_admin_post_merge WHERE admin_id NOT IN (SELECT id FROM mxx_system_admin);

-- 5. 更新admin用户状态
UPDATE mxx_system_admin SET status=0, user_type=1 WHERE user_name='admin';

-- 6. 重新建立用户角色关联
INSERT INTO mxx_system_admin_role_merge (admin_id, role_id) VALUES
((SELECT id FROM mxx_system_admin WHERE user_name='admin'), (SELECT id FROM mxx_system_role WHERE role_key='super_admin')),
((SELECT id FROM mxx_system_admin WHERE user_name='system'), (SELECT id FROM mxx_system_role WHERE role_key='system_admin')),
((SELECT id FROM mxx_system_admin WHERE user_name='sales'), (SELECT id FROM mxx_system_role WHERE role_key='sales_director')),
((SELECT id FROM mxx_system_admin WHERE user_name='manager'), (SELECT id FROM mxx_system_role WHERE role_key='sales_manager')),
((SELECT id FROM mxx_system_admin WHERE user_name='rep'), (SELECT id FROM mxx_system_role WHERE role_key='sales_rep')),
((SELECT id FROM mxx_system_admin WHERE user_name='purchase'), (SELECT id FROM mxx_system_role WHERE role_key='purchase_staff'));

-- 7. 重新建立用户部门关联
INSERT INTO mxx_system_admin_dept_merge (admin_id, dept_id) VALUES
((SELECT id FROM mxx_system_admin WHERE user_name='admin'), (SELECT id FROM mxx_system_dept WHERE code='ROOT')),
((SELECT id FROM mxx_system_admin WHERE user_name='system'), (SELECT id FROM mxx_system_dept WHERE code='TECH')),
((SELECT id FROM mxx_system_admin WHERE user_name='sales'), (SELECT id FROM mxx_system_dept WHERE code='SALES')),
((SELECT id FROM mxx_system_admin WHERE user_name='manager'), (SELECT id FROM mxx_system_dept WHERE code='SALES_01')),
((SELECT id FROM mxx_system_admin WHERE user_name='rep'), (SELECT id FROM mxx_system_dept WHERE code='SALES_01')),
((SELECT id FROM mxx_system_admin WHERE user_name='purchase'), (SELECT id FROM mxx_system_dept WHERE code='PURCHASE'));

-- 8. 重新建立用户岗位关联
INSERT INTO mxx_system_admin_post_merge (admin_id, post_id) VALUES
((SELECT id FROM mxx_system_admin WHERE user_name='admin'), (SELECT id FROM mxx_system_post WHERE post_code='SA')),
((SELECT id FROM mxx_system_admin WHERE user_name='system'), (SELECT id FROM mxx_system_post WHERE post_code='SA')),
((SELECT id FROM mxx_system_admin WHERE user_name='sales'), (SELECT id FROM mxx_system_post WHERE post_code='SD')),
((SELECT id FROM mxx_system_admin WHERE user_name='manager'), (SELECT id FROM mxx_system_post WHERE post_code='SM')),
((SELECT id FROM mxx_system_admin WHERE user_name='rep'), (SELECT id FROM mxx_system_post WHERE post_code='SR')),
((SELECT id FROM mxx_system_admin WHERE user_name='purchase'), (SELECT id FROM mxx_system_post WHERE post_code='PS'));

-- 9. 重新建立角色菜单关联
TRUNCATE TABLE mxx_system_role_menu_merge RESTART IDENTITY;

INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='super_admin'), id 
FROM mxx_system_menu WHERE deleted=0;

INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='system_admin'), id 
FROM mxx_system_menu WHERE perm LIKE 'system:%' AND deleted=0;

INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='sales_director'), id 
FROM mxx_system_menu WHERE (perm LIKE 'crm:%' OR perm LIKE 'sale:%' OR perm LIKE 'dashboard:%') AND deleted=0;

INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='sales_manager'), id 
FROM mxx_system_menu WHERE (perm LIKE 'crm:%' OR perm LIKE 'sale:%') AND deleted=0;

INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='sales_rep'), id 
FROM mxx_system_menu WHERE (perm LIKE 'crm:lead:%' OR perm LIKE 'crm:customer:%' OR perm LIKE 'crm:followup:%') AND deleted=0;

INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='purchase_staff'), id 
FROM mxx_system_menu WHERE (perm LIKE 'purchase:%' OR perm LIKE 'product:%') AND deleted=0;

-- 10. 更新菜单树路径
UPDATE mxx_system_menu SET tree_path = CASE 
    WHEN parent_id = 0 THEN id::TEXT
    ELSE (SELECT COALESCE(tree_path, '') FROM mxx_system_menu WHERE id = parent_id) || ',' || id::TEXT
END WHERE parent_id IS NOT NULL;

SELECT '数据修复完成' AS result;