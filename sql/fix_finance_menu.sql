-- 财务菜单国际化修复脚本
-- 将 mxx_system_menu.name 字段从中文更新为国际化键（page.finance.*）
-- 日期：2026-06-29

BEGIN;

-- 1. 财务管理（顶级目录）
UPDATE mxx_system_menu SET name = 'page.finance.title' WHERE id = 315;

-- 2. 提成规则（菜单）及其按钮
UPDATE mxx_system_menu SET name = 'page.finance.commissionRule.title' WHERE id = 316;
UPDATE mxx_system_menu SET name = 'page.finance.commissionRule.button.create' WHERE id = 317;
UPDATE mxx_system_menu SET name = 'page.finance.commissionRule.button.edit' WHERE id = 318;
UPDATE mxx_system_menu SET name = 'page.finance.commissionRule.button.delete' WHERE id = 319;
UPDATE mxx_system_menu SET name = 'page.finance.commissionRule.button.toggle' WHERE id = 320;

-- 3. 工资核算（菜单）及其按钮
UPDATE mxx_system_menu SET name = 'page.finance.salary.title' WHERE id = 321;
UPDATE mxx_system_menu SET name = 'page.finance.salary.button.calculate' WHERE id = 322;
UPDATE mxx_system_menu SET name = 'page.finance.salary.button.adjust' WHERE id = 323;
UPDATE mxx_system_menu SET name = 'page.finance.salary.button.approve' WHERE id = 324;
UPDATE mxx_system_menu SET name = 'page.finance.salary.button.pay' WHERE id = 325;

-- 4. 工资详情（隐藏菜单）
UPDATE mxx_system_menu SET name = 'page.finance.salary.detail' WHERE id = 326;

-- 5. 采购付款（菜单）及其按钮
UPDATE mxx_system_menu SET name = 'page.finance.payment.title' WHERE id = 332;
UPDATE mxx_system_menu SET name = 'page.finance.payment.button.apply' WHERE id = 333;
UPDATE mxx_system_menu SET name = 'page.finance.payment.button.approve' WHERE id = 334;
UPDATE mxx_system_menu SET name = 'page.finance.payment.button.confirm' WHERE id = 335;
UPDATE mxx_system_menu SET name = 'page.finance.payment.button.cancel' WHERE id = 336;

COMMIT;

-- 校验：执行后可运行以下查询确认 name 已更新为国际化键
-- SELECT id, name, route_name FROM mxx_system_menu WHERE id BETWEEN 315 AND 336 ORDER BY id;
