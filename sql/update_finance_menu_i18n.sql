-- ============================================================
-- 财务模块菜单国际化修改
-- 将所有财务模块菜单名称从中文修改为国际化key
-- ============================================================

-- 1. 财务管理（顶级目录）
UPDATE mxx_system_menu 
SET name = 'page.finance.title' 
WHERE id = 315 AND name = '财务管理';

-- 2. 提成规则
UPDATE mxx_system_menu 
SET name = 'page.finance.commissionRule.title' 
WHERE id = 316 AND name = '提成规则';

UPDATE mxx_system_menu 
SET name = 'page.finance.commissionRule.button.create' 
WHERE id = 317 AND name = '新增规则';

UPDATE mxx_system_menu 
SET name = 'page.finance.commissionRule.button.edit' 
WHERE id = 318 AND name = '编辑规则';

UPDATE mxx_system_menu 
SET name = 'page.finance.commissionRule.button.delete' 
WHERE id = 319 AND name = '删除规则';

UPDATE mxx_system_menu 
SET name = 'page.finance.commissionRule.button.toggle' 
WHERE id = 320 AND name = '启用禁用';

-- 3. 工资核算
UPDATE mxx_system_menu 
SET name = 'page.finance.salary.title' 
WHERE id = 321 AND name = '工资核算';

UPDATE mxx_system_menu 
SET name = 'page.finance.salary.button.calculate' 
WHERE id = 322 AND name = '执行核算';

UPDATE mxx_system_menu 
SET name = 'page.finance.salary.button.adjust' 
WHERE id = 323 AND name = '手动调整';

UPDATE mxx_system_menu 
SET name = 'page.finance.salary.button.approve' 
WHERE id = 324 AND name = '工资审核';

UPDATE mxx_system_menu 
SET name = 'page.finance.salary.button.pay' 
WHERE id = 325 AND name = '工资发放';

UPDATE mxx_system_menu 
SET name = 'page.finance.salary.detail' 
WHERE id = 326 AND name = '工资详情';

-- 4. 采购付款
UPDATE mxx_system_menu 
SET name = 'page.finance.payment.title' 
WHERE id = 332 AND name = '采购付款';

UPDATE mxx_system_menu 
SET name = 'page.finance.payment.button.apply' 
WHERE id = 333 AND name = '申请付款';

UPDATE mxx_system_menu 
SET name = 'page.finance.payment.button.approve' 
WHERE id = 334 AND name = '付款审批';

UPDATE mxx_system_menu 
SET name = 'page.finance.payment.button.confirm' 
WHERE id = 335 AND name = '确认付款';

UPDATE mxx_system_menu 
SET name = 'page.finance.payment.button.cancel' 
WHERE id = 336 AND name = '取消付款';

-- 输出修改结果
SELECT id, parent_id, name, path, type 
FROM mxx_system_menu 
WHERE id IN (315, 316, 317, 318, 319, 320, 321, 322, 323, 324, 325, 326, 332, 333, 334, 335, 336)
ORDER BY id;
