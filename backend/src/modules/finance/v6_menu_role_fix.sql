-- ============================================================
-- V6 菜单与角色修复脚本
-- 修正 v6_migration.sql 中菜单/角色插入的列名与ID冲突问题
-- 真实表结构：mxx_system_menu 使用 name/perm/type/hide_in_menu/create_time
--            mxx_system_role 使用 create_time（无 created_at）
-- ============================================================

-- 1. 新增菜单（ID 从 547 开始，避免与现有 530-546 冲突）
-- 父菜单：财务(id=315, type=FOLDER)
INSERT INTO mxx_system_menu (id, parent_id, name, type, route_name, path, component, perm, icon, sort, hide_in_menu, status, create_time, update_time, deleted)
VALUES
-- 个税管理
(547, 315, 'page.finance.tax.title', 'MENU', 'FinanceTax', 'tax', 'finance/tax/index', 'finance:tax:list', 'mdi:calculator', 60, 0, 1, NOW(), NOW(), 0),
(548, 547, 'page.finance.tax.button.list', 'BUTTON', NULL, NULL, NULL, 'finance:tax:list', NULL, 1, 0, 1, NOW(), NOW(), 0),
(549, 547, 'page.finance.tax.button.manage', 'BUTTON', NULL, NULL, NULL, 'finance:tax:manage', NULL, 2, 0, 1, NOW(), NOW(), 0),
-- 社保公积金
(550, 315, 'page.finance.insurance.title', 'MENU', 'FinanceSocialInsurance', 'social-insurance', 'finance/social-insurance/index', 'finance:insurance:list', 'mdi:shield-account', 61, 0, 1, NOW(), NOW(), 0),
(551, 550, 'page.finance.insurance.button.list', 'BUTTON', NULL, NULL, NULL, 'finance:insurance:list', NULL, 1, 0, 1, NOW(), NOW(), 0),
(552, 550, 'page.finance.insurance.button.manage', 'BUTTON', NULL, NULL, NULL, 'finance:insurance:manage', NULL, 2, 0, 1, NOW(), NOW(), 0),
-- 工资条下发
(553, 315, 'page.finance.payslip.title', 'MENU', 'FinancePayslip', 'payslip', 'finance/payslip/index', 'finance:payslip:list', 'mdi:email-send', 62, 0, 1, NOW(), NOW(), 0),
(554, 553, 'page.finance.payslip.button.list', 'BUTTON', NULL, NULL, NULL, 'finance:payslip:list', NULL, 1, 0, 1, NOW(), NOW(), 0),
(555, 553, 'page.finance.payslip.button.manage', 'BUTTON', NULL, NULL, NULL, 'finance:payslip:manage', NULL, 2, 0, 1, NOW(), NOW(), 0),
-- 银行代发
(556, 315, 'page.finance.bankExport.title', 'MENU', 'FinanceBankExport', 'bank-export', 'finance/bank-export/index', 'finance:bank-export:list', 'mdi:bank', 63, 0, 1, NOW(), NOW(), 0),
(557, 556, 'page.finance.bankExport.button.list', 'BUTTON', NULL, NULL, NULL, 'finance:bank-export:list', NULL, 1, 0, 1, NOW(), NOW(), 0),
(558, 556, 'page.finance.bankExport.button.manage', 'BUTTON', NULL, NULL, NULL, 'finance:bank-export:manage', NULL, 2, 0, 1, NOW(), NOW(), 0),
-- 考勤扣款
(559, 315, 'page.finance.attendance.title', 'MENU', 'FinanceAttendance', 'attendance', 'finance/attendance/index', 'finance:attendance:list', 'mdi:clock-check', 64, 0, 1, NOW(), NOW(), 0),
(560, 559, 'page.finance.attendance.button.list', 'BUTTON', NULL, NULL, NULL, 'finance:attendance:list', NULL, 1, 0, 1, NOW(), NOW(), 0),
(561, 559, 'page.finance.attendance.button.manage', 'BUTTON', NULL, NULL, NULL, 'finance:attendance:manage', NULL, 2, 0, 1, NOW(), NOW(), 0),
-- 调薪记录
(562, 315, 'page.finance.adjustment.title', 'MENU', 'FinanceSalaryAdjustment', 'salary-adjustment', 'finance/salary-adjustment/index', 'finance:adjustment:list', 'mdi:cash-plus', 65, 0, 1, NOW(), NOW(), 0),
(563, 562, 'page.finance.adjustment.button.list', 'BUTTON', NULL, NULL, NULL, 'finance:adjustment:list', NULL, 1, 0, 1, NOW(), NOW(), 0),
(564, 562, 'page.finance.adjustment.button.manage', 'BUTTON', NULL, NULL, NULL, 'finance:adjustment:manage', NULL, 2, 0, 1, NOW(), NOW(), 0),
-- 工资项目（自定义明细项引擎）
(565, 315, 'page.finance.salaryItem.title', 'MENU', 'FinanceSalaryItem', 'salary-item', 'finance/salary-item/index', 'finance:salary-item:list', 'mdi:format-list-bulleted', 66, 0, 1, NOW(), NOW(), 0),
(566, 565, 'page.finance.salaryItem.button.list', 'BUTTON', NULL, NULL, NULL, 'finance:salary-item:list', NULL, 1, 0, 1, NOW(), NOW(), 0),
(567, 565, 'page.finance.salaryItem.button.manage', 'BUTTON', NULL, NULL, NULL, 'finance:salary-item:manage', NULL, 2, 0, 1, NOW(), NOW(), 0)
ON CONFLICT (id) DO NOTHING;

-- 2. 新增财务角色（若不存在）
INSERT INTO mxx_system_role (role_name, role_key, sort, data_scope, status, deleted, create_time, update_time)
SELECT '财务', 'finance', 50, 1, 1, 0, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_role WHERE role_key = 'finance' AND deleted = 0);

-- 3. 为财务角色分配财务相关菜单权限（含工资核算所有权限 + 定时任务）
DO $$
DECLARE
    finance_role_id BIGINT;
BEGIN
    SELECT id INTO finance_role_id FROM mxx_system_role WHERE role_key = 'finance' AND deleted = 0 LIMIT 1;
    IF finance_role_id IS NOT NULL THEN
        INSERT INTO mxx_system_role_menu_merge (role_id, menu_id, create_time, update_time, status)
        SELECT finance_role_id, m.id, NOW(), NOW(), 1
        FROM mxx_system_menu m
        WHERE (m.perm LIKE 'finance:%' OR m.perm = 'system:scheduler:list' OR m.perm = 'system:scheduler:manage')
          AND m.deleted = 0
        ON CONFLICT (role_id, menu_id) DO NOTHING;
    END IF;
END $$;

-- 4. 同时把财务菜单权限分配给 super_admin、system_admin、general_manager（若存在）
DO $$
DECLARE
    r RECORD;
BEGIN
    FOR r IN SELECT id FROM mxx_system_role WHERE role_key IN ('super_admin','system_admin','general_manager','boss') AND deleted = 0 LOOP
        INSERT INTO mxx_system_role_menu_merge (role_id, menu_id, create_time, update_time, status)
        SELECT r.id, m.id, NOW(), NOW(), 1
        FROM mxx_system_menu m
        WHERE m.perm LIKE 'finance:%'
          AND m.deleted = 0
        ON CONFLICT (role_id, menu_id) DO NOTHING;
    END LOOP;
END $$;
