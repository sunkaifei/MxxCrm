-- 修复远程数据库：创建回款计划 + 业绩计划 + 月度目标
-- 执行方式：psql -h 115.190.210.106 -U postgres -d mxxcrm_data -f v6_fix_remote_data.sql

-- ============================================================
-- 1. 创建合同回款计划（6月完全回款，用于工资核算提成）
-- ============================================================
INSERT INTO mxx_crm_contract_payment_plan (contract_id, stage_name, payment_type, plan_amount, received_amount, plan_date, actual_date, status, remark, create_time, update_time, deleted)
SELECT
    c.id,
    '首期款',
    1,
    COALESCE(c.amount, 0),
    COALESCE(c.amount, 0),
    DATE '2026-06-01',
    DATE '2026-06-15',
    2,
    'V6全链路测试回款数据',
    NOW(),
    NOW(),
    0
FROM mxx_crm_contract c
WHERE c.deleted = 0
  AND c.assigned_to IS NOT NULL
  AND c.amount IS NOT NULL
  AND c.amount > 0
  AND NOT EXISTS (
      SELECT 1 FROM mxx_crm_contract_payment_plan p
      WHERE p.contract_id = c.id AND p.deleted = 0
  );

-- ============================================================
-- 2. 创建业绩计划（已审批通过 status=2）
-- ============================================================
INSERT INTO mxx_statistics_performance_plan (employee_id, year, status, version, create_time, update_time, deleted, is_frozen)
SELECT a.id, 2026, 2, 1, NOW(), NOW(), 0, 0
FROM mxx_system_admin a
WHERE a.id IN (3, 5, 6, 7)
  AND a.deleted = 0
  AND NOT EXISTS (
      SELECT 1 FROM mxx_statistics_performance_plan p
      WHERE p.employee_id = a.id AND p.year = 2026 AND p.deleted = 0
  );

-- ============================================================
-- 3. 创建月度目标（6月目标，用于绩效系数计算）
-- ============================================================
INSERT INTO mxx_statistics_plan_monthly_target (plan_id, month, contract_target_amount, payment_target_amount, contract_target_count, create_time, update_time, deleted)
SELECT p.id, 6,
    CASE p.employee_id
        WHEN 3 THEN 5000000.00
        WHEN 5 THEN 2000000.00
        WHEN 6 THEN 3000000.00
        WHEN 7 THEN 1000000.00
    END,
    CASE p.employee_id
        WHEN 3 THEN 5000000.00
        WHEN 5 THEN 2000000.00
        WHEN 6 THEN 3000000.00
        WHEN 7 THEN 1000000.00
    END,
    5,
    NOW(), NOW(), 0
FROM mxx_statistics_performance_plan p
WHERE p.year = 2026 AND p.deleted = 0 AND p.status = 2
  AND NOT EXISTS (
      SELECT 1 FROM mxx_statistics_plan_monthly_target t
      WHERE t.plan_id = p.id AND t.month = 6 AND t.deleted = 0
  );

-- ============================================================
-- 4. 验证数据
-- ============================================================
SELECT '== 远程数据库修复后统计 ==' AS info;
SELECT
    (SELECT COUNT(*) FROM mxx_crm_contract_payment_plan WHERE deleted=0 AND actual_date IS NOT NULL) AS payment_plan_count,
    (SELECT COUNT(*) FROM mxx_statistics_performance_plan WHERE deleted=0 AND status=2) AS perf_plan_count,
    (SELECT COUNT(*) FROM mxx_statistics_plan_monthly_target WHERE deleted=0 AND month=6) AS monthly_target_count,
    (SELECT COUNT(*) FROM mxx_finance_salary_config WHERE deleted=0 AND year=2026) AS salary_config_count,
    (SELECT COUNT(*) FROM mxx_finance_employee_tax_config WHERE year=2026) AS tax_config_count,
    (SELECT COUNT(*) FROM mxx_finance_attendance_record WHERE year=2026 AND month=6) AS attendance_count;
