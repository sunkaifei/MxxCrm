-- 全链路测试数据(修正版)
-- ============ 1. 员工个税配置 ============
INSERT INTO mxx_finance_employee_tax_config (
    employee_id, year, tax_threshold,
    children_education, continuing_education, housing_loan, housing_rent,
    supporting_elderly, infant_care, serious_illness, other_deduction, foreigner_allowance,
    cumulative_income, cumulative_threshold_deduction, cumulative_special_deduction, cumulative_other_deduction
)
VALUES
    (3, 2026, 5000, 1000, 0, 1000, 0, 2000, 0, 0, 0, 0, 0, 0, 0, 0),
    (5, 2026, 5000, 1000, 400, 0, 1500, 0, 0, 0, 0, 0, 0, 0, 0, 0),
    (6, 2026, 5000, 0, 0, 0, 2000, 0, 0, 0, 0, 0, 0, 0, 0, 0),
    (7, 2026, 5000, 0, 0, 0, 1000, 0, 0, 0, 0, 0, 0, 0, 0, 0),
    (12, 2026, 5000, 0, 0, 1000, 1000, 0, 0, 0, 0, 0, 0, 0, 0, 0)
ON CONFLICT DO NOTHING;

-- ============ 2. 员工社保配置 ============
INSERT INTO mxx_finance_employee_insurance_config (
    employee_id, city_code, base_amount, housing_fund_base,
    housing_fund_company_rate, housing_fund_personal_rate,
    participate_pension, participate_medical, participate_unemployment,
    participate_workinjury, participate_maternity, participate_housing_fund,
    effective_date, enabled
)
VALUES
    (3, 'beijing', 15000.00, 15000.00, 0.12, 0.12, 1, 1, 1, 1, 1, 1, '2026-01-01', 1),
    (5, 'beijing', 12000.00, 12000.00, 0.12, 0.12, 1, 1, 1, 1, 1, 1, '2026-01-01', 1),
    (6, 'beijing', 9000.00, 9000.00, 0.12, 0.12, 1, 1, 1, 1, 1, 1, '2026-01-01', 1),
    (7, 'beijing', 7162.00, 7162.00, 0.12, 0.12, 1, 1, 1, 1, 1, 1, '2026-01-01', 1),
    (12, 'beijing', 10000.00, 10000.00, 0.12, 0.12, 1, 1, 1, 1, 1, 1, '2026-01-01', 1)
ON CONFLICT DO NOTHING;

-- ============ 3. 考勤记录(2026年6月) ============
INSERT INTO mxx_finance_attendance_record (
    employee_id, year, month, work_days, actual_work_days,
    late_count, early_leave_count, absent_count,
    personal_leave_days, sick_leave_days, annual_leave_days,
    overtime_hours_weekday, overtime_hours_weekend, overtime_hours_holiday,
    data_source
)
VALUES
    (3, 2026, 6, 22, 22, 0, 0, 0, 0, 0, 0, 10, 0, 0, 1),
    (5, 2026, 6, 22, 21, 1, 0, 0, 0, 0, 0, 10, 5, 0, 1),
    (6, 2026, 6, 22, 21, 0, 0, 0, 1, 0, 0, 8, 0, 0, 1),
    (7, 2026, 6, 22, 20, 2, 1, 0, 0, 0, 0, 5, 0, 0, 1),
    (12, 2026, 6, 22, 22, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1)
ON CONFLICT (employee_id, year, month) DO NOTHING;

-- ============ 4. 验证数据 ============
SELECT 'salary_config' AS type, COUNT(*) AS cnt FROM mxx_finance_salary_config WHERE deleted=0 AND year=2026 AND month=6
UNION ALL
SELECT 'tax_config', COUNT(*) FROM mxx_finance_employee_tax_config WHERE year=2026
UNION ALL
SELECT 'insurance_config', COUNT(*) FROM mxx_finance_employee_insurance_config WHERE enabled=1
UNION ALL
SELECT 'attendance', COUNT(*) FROM mxx_finance_attendance_record WHERE year=2026 AND month=6;
