-- v16 修复财务子菜单 path 格式不一致
-- 问题：accessMode=backend 模式下，子菜单 path 必须带父菜单前缀
-- 13 个菜单缺少 /finance/ 前缀，导致路由匹配失败

BEGIN;

UPDATE mxx_system_menu SET path = '/finance/expense'        WHERE id = 538 AND path = 'expense';
UPDATE mxx_system_menu SET path = '/finance/expense-type'   WHERE id = 545 AND path = 'expense-type';
UPDATE mxx_system_menu SET path = '/finance/tax'            WHERE id = 547 AND path = 'tax';
UPDATE mxx_system_menu SET path = '/finance/social-insurance' WHERE id = 550 AND path = 'social-insurance';
UPDATE mxx_system_menu SET path = '/finance/payslip'       WHERE id = 553 AND path = 'payslip';
UPDATE mxx_system_menu SET path = '/finance/bank-export'   WHERE id = 556 AND path = 'bank-export';
UPDATE mxx_system_menu SET path = '/finance/attendance'    WHERE id = 559 AND path = 'attendance';
UPDATE mxx_system_menu SET path = '/finance/salary-adjustment' WHERE id = 562 AND path = 'salary-adjustment';
UPDATE mxx_system_menu SET path = '/finance/salary-item'   WHERE id = 565 AND path = 'salary-item';
UPDATE mxx_system_menu SET path = '/finance/statistics'    WHERE id = 600 AND path = 'statistics';
UPDATE mxx_system_menu SET path = '/finance/refund-record' WHERE id = 601 AND path = 'refund-record';
UPDATE mxx_system_menu SET path = '/finance/payment-record' WHERE id = 602 AND path = 'payment-record';
UPDATE mxx_system_menu SET path = '/finance/member-fee'    WHERE id = 603 AND path = 'member-fee';

-- 工资详情菜单的 parent_id 确认（确保挂在 finance 父菜单下）
UPDATE mxx_system_menu SET parent_id = 315, sort = 99 WHERE id = 326 AND deleted = 0;

COMMIT;

-- 验证：所有财务子菜单 path 应统一带 /finance/ 前缀
SELECT id, path, route_name FROM mxx_system_menu
WHERE parent_id = 315 AND deleted = 0
ORDER BY sort;
