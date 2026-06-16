-- ================================================
-- Mxx-CRM 数据库修复脚本
-- 修复表结构不一致的问题
-- ================================================

-- 1. 修复 mxx_system_post 表的 status 字段类型（bit(1) -> integer）
ALTER TABLE mxx_system_post ALTER COLUMN status TYPE integer USING CASE WHEN status = B'1' THEN 1 ELSE 0 END;

-- 2. 修复 mxx_product_category 表的 parent_id 外键约束（允许 NULL）
ALTER TABLE mxx_product_category DROP CONSTRAINT IF EXISTS mxx_product_category_parent_id_fkey;
ALTER TABLE mxx_product_category ALTER COLUMN parent_id SET DEFAULT 0;

-- 3. 修复 mxx_crm_followup 表的 lead_id 和 opportunity_id 外键约束（允许 NULL）
ALTER TABLE mxx_crm_followup DROP CONSTRAINT IF EXISTS mxx_crm_followup_lead_id_fkey;
ALTER TABLE mxx_crm_followup DROP CONSTRAINT IF EXISTS mxx_crm_followup_opportunity_id_fkey;

-- 4. 修复 mxx_industry_type 枚举类型，添加缺失值
ALTER TYPE mxx_industry_type ADD VALUE IF NOT EXISTS 'social';

-- 5. 重新插入岗位数据
INSERT INTO mxx_system_post (post_code, post_name, sort, status) VALUES
('SA', '系统管理员', 1, 0),
('SD', '销售总监', 2, 0),
('SM', '销售经理', 3, 0),
('SR', '业务员', 4, 0),
('PD', '采购经理', 5, 0),
('PS', '采购专员', 6, 0),
('FD', '财务经理', 7, 0),
('TD', '技术总监', 8, 0)
ON CONFLICT DO NOTHING;

-- 6. 重新插入产品分类数据（parent_id=0 现在允许）
INSERT INTO mxx_product_category (parent_id, name, sort_order) VALUES
(0, '电子产品', 1),
(0, '服装鞋帽', 2),
(0, '家居用品', 3)
ON CONFLICT DO NOTHING;

-- 等待主分类插入完成
SELECT pg_sleep(0.1);

INSERT INTO mxx_product_category (parent_id, name, sort_order) VALUES
((SELECT id FROM mxx_product_category WHERE name='电子产品'), '手机数码', 1),
((SELECT id FROM mxx_product_category WHERE name='电子产品'), '电脑配件', 2),
((SELECT id FROM mxx_product_category WHERE name='服装鞋帽'), '男装', 1),
((SELECT id FROM mxx_product_category WHERE name='服装鞋帽'), '女装', 2),
((SELECT id FROM mxx_product_category WHERE name='家居用品'), '厨房用品', 1),
((SELECT id FROM mxx_product_category WHERE name='家居用品'), '卧室用品', 2)
ON CONFLICT DO NOTHING;

-- 7. 修复用户角色关联表的重复数据问题
DELETE FROM mxx_system_admin_role_merge WHERE id IN (
    SELECT id FROM (
        SELECT id, ROW_NUMBER() OVER (PARTITION BY admin_id, role_id ORDER BY id) as rn
        FROM mxx_system_admin_role_merge
    ) t WHERE rn > 1
);

-- 8. 修复用户部门关联表的重复数据问题
DELETE FROM mxx_system_admin_dept_merge WHERE id IN (
    SELECT id FROM (
        SELECT id, ROW_NUMBER() OVER (PARTITION BY admin_id, dept_id ORDER BY id) as rn
        FROM mxx_system_admin_dept_merge
    ) t WHERE rn > 1
);

-- 9. 修复用户岗位关联表的重复数据问题
DELETE FROM mxx_system_admin_post_merge WHERE id IN (
    SELECT id FROM (
        SELECT id, ROW_NUMBER() OVER (PARTITION BY admin_id, post_id ORDER BY id) as rn
        FROM mxx_system_admin_post_merge
    ) t WHERE rn > 1
);

-- 10. 重新插入用户角色关联
INSERT INTO mxx_system_admin_role_merge (admin_id, role_id) VALUES
((SELECT id FROM mxx_system_admin WHERE user_name='admin'), (SELECT id FROM mxx_system_role WHERE role_key='super_admin')),
((SELECT id FROM mxx_system_admin WHERE user_name='system'), (SELECT id FROM mxx_system_role WHERE role_key='system_admin')),
((SELECT id FROM mxx_system_admin WHERE user_name='sales'), (SELECT id FROM mxx_system_role WHERE role_key='sales_director')),
((SELECT id FROM mxx_system_admin WHERE user_name='manager'), (SELECT id FROM mxx_system_role WHERE role_key='sales_manager')),
((SELECT id FROM mxx_system_admin WHERE user_name='rep'), (SELECT id FROM mxx_system_role WHERE role_key='sales_rep')),
((SELECT id FROM mxx_system_admin WHERE user_name='purchase'), (SELECT id FROM mxx_system_role WHERE role_key='purchase_staff'))
ON CONFLICT DO NOTHING;

-- 11. 重新插入用户部门关联
INSERT INTO mxx_system_admin_dept_merge (admin_id, dept_id) VALUES
((SELECT id FROM mxx_system_admin WHERE user_name='admin'), (SELECT id FROM mxx_system_dept WHERE code='ROOT')),
((SELECT id FROM mxx_system_admin WHERE user_name='system'), (SELECT id FROM mxx_system_dept WHERE code='TECH')),
((SELECT id FROM mxx_system_admin WHERE user_name='sales'), (SELECT id FROM mxx_system_dept WHERE code='SALES')),
((SELECT id FROM mxx_system_admin WHERE user_name='manager'), (SELECT id FROM mxx_system_dept WHERE code='SALES_01')),
((SELECT id FROM mxx_system_admin WHERE user_name='rep'), (SELECT id FROM mxx_system_dept WHERE code='SALES_01')),
((SELECT id FROM mxx_system_admin WHERE user_name='purchase'), (SELECT id FROM mxx_system_dept WHERE code='PURCHASE'))
ON CONFLICT DO NOTHING;

-- 12. 重新插入用户岗位关联
INSERT INTO mxx_system_admin_post_merge (admin_id, post_id) VALUES
((SELECT id FROM mxx_system_admin WHERE user_name='admin'), (SELECT id FROM mxx_system_post WHERE post_code='SA')),
((SELECT id FROM mxx_system_admin WHERE user_name='system'), (SELECT id FROM mxx_system_post WHERE post_code='SA')),
((SELECT id FROM mxx_system_admin WHERE user_name='sales'), (SELECT id FROM mxx_system_post WHERE post_code='SD')),
((SELECT id FROM mxx_system_admin WHERE user_name='manager'), (SELECT id FROM mxx_system_post WHERE post_code='SM')),
((SELECT id FROM mxx_system_admin WHERE user_name='rep'), (SELECT id FROM mxx_system_post WHERE post_code='SR')),
((SELECT id FROM mxx_system_admin WHERE user_name='purchase'), (SELECT id FROM mxx_system_post WHERE post_code='PS'))
ON CONFLICT DO NOTHING;

-- 13. 重新插入线索数据（使用正确的 industry 值）
INSERT INTO mxx_crm_lead (company_name, contact_name, title, email, phone, mobile, country, region, industry, source, status, assigned_to) VALUES
('美团', '王兴', 'CEO', 'wang@meituan.com', '010-12345678', '13800138006', '中国', '北京市', 'ecommerce', 'website', 'new', (SELECT id FROM mxx_system_admin WHERE user_name='rep')),
('京东集团', '刘强东', 'CEO', 'liu@jd.com', '010-87654321', '13800138007', '中国', '北京市', 'ecommerce', 'website', 'new', (SELECT id FROM mxx_system_admin WHERE user_name='rep')),
('拼多多', '黄峥', 'CEO', 'huang@pinduoduo.com', '021-11223344', '13800138008', '中国', '上海市', 'ecommerce', 'social', 'following', (SELECT id FROM mxx_system_admin WHERE user_name='manager')),
('网易', '丁磊', 'CEO', 'ding@netease.com', '0571-88888888', '13800138009', '中国', '浙江省杭州市', 'ecommerce', 'referral', 'new', (SELECT id FROM mxx_system_admin WHERE user_name='rep')),
('小红书', '毛文超', 'CEO', 'mao@xiaohongshu.com', '021-99999999', '13800138010', '中国', '上海市', 'social', 'social', 'following', (SELECT id FROM mxx_system_admin WHERE user_name='manager'))
ON CONFLICT DO NOTHING;

-- 14. 重新插入跟进记录（lead_id 现在可以正常引用）
INSERT INTO mxx_crm_followup (lead_id, customer_id, activity_type, subject, content, next_follow_date) VALUES
((SELECT id FROM mxx_crm_lead WHERE company_name='美团'), NULL, 'call', '初次联系', '电话联系客户，介绍公司产品和服务，客户表示有兴趣', '2024-04-20'),
((SELECT id FROM mxx_crm_lead WHERE company_name='京东集团'), NULL, 'email', '发送产品资料', '发送产品详细资料和报价单', '2024-04-25'),
((SELECT id FROM mxx_crm_lead WHERE company_name='拼多多'), NULL, 'meeting', '商务会谈', '与客户进行线下会谈，深入了解需求', '2024-04-30'),
(NULL, (SELECT id FROM mxx_crm_customer WHERE customer_no='CUST20240100001'), 'wechat', '日常跟进', '微信跟进项目进度，客户反馈良好', '2024-04-22'),
(NULL, (SELECT id FROM mxx_crm_customer WHERE customer_no='CUST20240100002'), 'email', '合同跟进', '跟进合同签署情况', '2024-04-28')
ON CONFLICT DO NOTHING;

-- 15. 更新菜单树路径
UPDATE mxx_system_menu SET tree_path = CASE 
    WHEN parent_id = 0 THEN id::TEXT
    ELSE (SELECT COALESCE(tree_path, '') FROM mxx_system_menu WHERE id = parent_id) || ',' || id::TEXT
END WHERE parent_id IS NOT NULL;

SELECT '修复完成' AS result;