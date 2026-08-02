-- 修复所有 TIMESTAMPTZ 列为 TIMESTAMP
ALTER TABLE mxx_finance_bank_payment_file ALTER COLUMN create_time TYPE timestamp(6) without time zone USING create_time AT TIME ZONE 'Asia/Shanghai';
ALTER TABLE mxx_finance_employee_insurance_config ALTER COLUMN create_time TYPE timestamp(6) without time zone USING create_time AT TIME ZONE 'Asia/Shanghai';
ALTER TABLE mxx_finance_employee_insurance_config ALTER COLUMN update_time TYPE timestamp(6) without time zone USING update_time AT TIME ZONE 'Asia/Shanghai';
ALTER TABLE mxx_finance_payslip ALTER COLUMN confirm_time TYPE timestamp(6) without time zone USING confirm_time AT TIME ZONE 'Asia/Shanghai';
ALTER TABLE mxx_finance_payslip ALTER COLUMN create_time TYPE timestamp(6) without time zone USING create_time AT TIME ZONE 'Asia/Shanghai';
ALTER TABLE mxx_finance_payslip ALTER COLUMN read_time TYPE timestamp(6) without time zone USING read_time AT TIME ZONE 'Asia/Shanghai';
ALTER TABLE mxx_finance_payslip ALTER COLUMN send_time TYPE timestamp(6) without time zone USING send_time AT TIME ZONE 'Asia/Shanghai';
ALTER TABLE mxx_finance_salary_tax_detail ALTER COLUMN create_time TYPE timestamp(6) without time zone USING create_time AT TIME ZONE 'Asia/Shanghai';
ALTER TABLE mxx_finance_social_insurance_policy ALTER COLUMN create_time TYPE timestamp(6) without time zone USING create_time AT TIME ZONE 'Asia/Shanghai';

-- 添加定时任务管理权限菜单
INSERT INTO mxx_system_menu (parent_id, name, type, path, component, perm, status, sort, icon, create_time, update_time, deleted)
SELECT 0, 'page.system.scheduler.title', 'menu', 'system-ext/scheduler', 'system-ext/scheduler/index', 'system:scheduler:list', 1, 100, 'ant-design:clock-circle-outlined', NOW(), NOW(), 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm='system:scheduler:list' AND deleted=0);

INSERT INTO mxx_system_menu (parent_id, name, type, path, perm, status, sort, icon, create_time, update_time, deleted)
SELECT (SELECT id FROM mxx_system_menu WHERE perm='system:scheduler:list' AND deleted=0 LIMIT 1),
       'page.system.scheduler.button.manage', 'button', '', 'system:scheduler:manage', 1, 1, '', NOW(), NOW(), 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm='system:scheduler:manage' AND deleted=0);

-- 关联定时任务权限到超级管理员(4)和财务(10)
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id, create_time)
SELECT r.role_id, m.id, NOW()
FROM mxx_system_menu m
CROSS JOIN (SELECT 4 AS role_id UNION SELECT 5 AS role_id UNION SELECT 10 AS role_id) r
WHERE m.perm IN ('system:scheduler:list', 'system:scheduler:manage')
  AND m.deleted=0
  AND NOT EXISTS (
    SELECT 1 FROM mxx_system_role_menu_merge rmm WHERE rmm.role_id=r.role_id AND rmm.menu_id=m.id
  );

SELECT 'Done' AS result;
