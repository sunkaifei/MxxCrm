-- 全链路测试数据：工资配置、个税配置、社保配置、考勤记录
-- 为 admin(3), sales(5), manager(6), rep(7), finance(12) 创建测试数据

-- 上月年月
-- 2026-07 → 上月是 2026-06

-- ============ 1. 工资配置 ============
INSERT INTO mxx_finance_salary_config (employee_id, year, month, base_salary, position_allowance, performance_base, performance_coefficient, status, create_time, update_time, deleted)
VALUES
    (3, 2026, 6, 15000.00, 5000.00, 3000.00, 1.20, 1, NOW(), NOW(), 0),
    (5, 2026, 6, 12000.00, 3000.00, 2000.00, 1.10, 1, NOW(), NOW(), 0),
    (6, 2026, 6, 9000.00, 2000.00, 1500.00, 1.00, 1, NOW(), NOW(), 0),
    (7, 2026, 6, 6000.00, 1000.00, 1000.00, 0.90, 1, NOW(), NOW(), 0),
    (12, 2026, 6, 10000.00, 2000.00, 1500.00, 1.00, 1, NOW(), NOW(), 0)
ON CONFLICT DO NOTHING;

-- ============ 2. 员工个税配置 ============
INSERT INTO mxx_finance_employee_tax_config (employee_id, tax_year, special_deduction, cumulative_income, cumulative_tax_paid, status, create_time, update_time)
VALUES
    (3, 2026, 5000.00, 0, 0, 1, NOW(), NOW()),
    (5, 2026, 3000.00, 0, 0, 1, NOW(), NOW()),
    (6, 2026, 2000.00, 0, 0, 1, NOW(), NOW()),
    (7, 2026, 1000.00, 0, 0, 1, NOW(), NOW()),
    (12, 2026, 2000.00, 0, 0, 1, NOW(), NOW())
ON CONFLICT DO NOTHING;

-- ============ 3. 员工社保配置 ============
INSERT INTO mxx_finance_employee_insurance_config (employee_id, policy_id, base_salary, pension_personal, pension_company, medical_personal, medical_company, unemployment_personal, unemployment_company, work_injury_company, maternity_company, housing_fund_personal, housing_fund_company, effective_month, status, create_time, update_time)
SELECT
    e.emp_id,
    1,
    e.base,
    e.base * 0.08,
    e.base * 0.16,
    e.base * 0.02,
    e.base * 0.08,
    e.base * 0.005,
    e.base * 0.005,
    e.base * 0.002,
    e.base * 0.008,
    e.base * 0.12,
    e.base * 0.12,
    202606,
    1,
    NOW(),
    NOW()
FROM (VALUES
    (3, 15000.00),
    (5, 12000.00),
    (6, 9000.00),
    (7, 6000.00),
    (12, 10000.00)
) AS e(emp_id, base)
WHERE NOT EXISTS (
    SELECT 1 FROM mxx_finance_employee_insurance_config x
    WHERE x.employee_id = e.emp_id AND x.effective_month = 202606
);

-- ============ 4. 考勤记录(2026年6月) ============
INSERT INTO mxx_finance_attendance_record (employee_id, year, month, work_days, late_count, early_leave_count, absent_days, leave_days, overtime_hours, full_attendance_award, deduction_amount, status, create_time, update_time)
VALUES
    (3, 2026, 6, 22, 0, 0, 0, 0, 10, 500.00, 0, 1, NOW(), NOW()),
    (5, 2026, 6, 21, 1, 0, 0, 0, 15, 0, 50.00, 1, NOW(), NOW()),
    (6, 2026, 6, 22, 0, 0, 0, 1, 8, 500.00, 0, 1, NOW(), NOW()),
    (7, 2026, 6, 20, 2, 1, 0, 0, 5, 0, 150.00, 1, NOW(), NOW()),
    (12, 2026, 6, 22, 0, 0, 0, 0, 0, 500.00, 0, 1, NOW(), NOW())
ON CONFLICT DO NOTHING;

-- ============ 5. 验证数据 ============
SELECT '工资配置' AS type, COUNT(*) AS cnt FROM mxx_finance_salary_config WHERE deleted=0 AND year=2026 AND month=6
UNION ALL
SELECT '个税配置', COUNT(*) FROM mxx_finance_employee_tax_config WHERE status=1 AND tax_year=2026
UNION ALL
SELECT '社保配置', COUNT(*) FROM mxx_finance_employee_insurance_config WHERE status=1 AND effective_month=202606
UNION ALL
SELECT '考勤记录', COUNT(*) FROM mxx_finance_attendance_record WHERE year=2026 AND month=6;
