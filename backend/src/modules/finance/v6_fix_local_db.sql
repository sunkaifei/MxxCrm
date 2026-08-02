-- 修复本地数据库：创建缺失的统计表 + 清理重复财务账号 + 补全工资配置
-- 执行方式：psql -h 127.0.0.1 -U postgres -d mxxcrm_data -f v6_fix_local_db.sql

-- ============================================================
-- 1. 创建缺失的业绩计划相关表（本地数据库缺少这些表）
-- ============================================================

-- 业绩计划主表
CREATE TABLE IF NOT EXISTS mxx_statistics_performance_plan (
    id BIGSERIAL PRIMARY KEY,
    employee_id BIGINT NOT NULL,
    year INT NOT NULL,
    status INT DEFAULT 0,
    apply_reason VARCHAR(500),
    version INT DEFAULT 1,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted INT DEFAULT 0,
    current_approver_id BIGINT,
    current_approver_name VARCHAR(64),
    approval_level INT,
    total_levels INT,
    submit_time TIMESTAMP,
    is_frozen INT DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_perf_plan_employee ON mxx_statistics_performance_plan (employee_id, year);
CREATE INDEX IF NOT EXISTS idx_perf_plan_status ON mxx_statistics_performance_plan (status, deleted);

-- 月度目标明细表
CREATE TABLE IF NOT EXISTS mxx_statistics_plan_monthly_target (
    id BIGSERIAL PRIMARY KEY,
    plan_id BIGINT NOT NULL,
    month INT NOT NULL,
    contract_target_amount DECIMAL(18,2) DEFAULT 0,
    payment_target_amount DECIMAL(18,2) DEFAULT 0,
    contract_target_count INT DEFAULT 0,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted INT DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_plan_monthly_target_plan ON mxx_statistics_plan_monthly_target (plan_id, deleted);

-- 审批日志表
CREATE TABLE IF NOT EXISTS mxx_statistics_plan_approval_log (
    id BIGSERIAL PRIMARY KEY,
    plan_id BIGINT NOT NULL,
    action INT NOT NULL,
    operator_id BIGINT NOT NULL,
    operator_name VARCHAR(64),
    reason VARCHAR(500),
    previous_status INT,
    new_status INT,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted INT DEFAULT 0,
    snapshot TEXT,
    current_level INT
);
CREATE INDEX IF NOT EXISTS idx_plan_approval_log_plan ON mxx_statistics_plan_approval_log (plan_id, deleted);

-- 审批节点表
CREATE TABLE IF NOT EXISTS mxx_statistics_plan_approval_node (
    id BIGSERIAL PRIMARY KEY,
    plan_id BIGINT NOT NULL,
    level INT NOT NULL,
    approver_id BIGINT NOT NULL,
    approver_name VARCHAR(64),
    status INT DEFAULT 0,
    comment VARCHAR(500),
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted INT DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_plan_approval_node_plan ON mxx_statistics_plan_approval_node (plan_id, deleted);

-- ============================================================
-- 2. 清理重复的财务账号（保留 id=11，删除 id=13）
-- ============================================================
DELETE FROM mxx_system_admin_role_merge WHERE admin_id = 13;
DELETE FROM mxx_system_admin WHERE id = 13;

-- ============================================================
-- 3. 为 admin(id=3) 补充工资配置（admin 有合同回款，需要核算）
-- ============================================================
INSERT INTO mxx_finance_salary_config (employee_id, year, base_salary, position_allowance, performance_base, performance_coefficient, status, create_time, update_time, deleted)
SELECT 3, 2026, 15000.00, 5000.00, 3000.00, 1.20, 1, NOW(), NOW(), 0
WHERE NOT EXISTS (
    SELECT 1 FROM mxx_finance_salary_config WHERE employee_id = 3 AND year = 2026 AND deleted = 0
);

-- ============================================================
-- 4. 为 admin(id=3) 补充个税配置
-- ============================================================
INSERT INTO mxx_finance_employee_tax_config (employee_id, year, tax_threshold, children_education, continuing_education, housing_loan, housing_rent, supporting_elderly, infant_care, create_time, update_time)
SELECT 3, 2026, 5000, 1000, 0, 1000, 0, 2000, 0, NOW(), NOW()
WHERE NOT EXISTS (
    SELECT 1 FROM mxx_finance_employee_tax_config WHERE employee_id = 3 AND year = 2026
);

-- ============================================================
-- 5. 为 admin(id=3) 补充社保配置
-- ============================================================
INSERT INTO mxx_finance_employee_insurance_config (employee_id, city_code, base_amount, housing_fund_base, housing_fund_company_rate, housing_fund_personal_rate, participate_pension, participate_medical, participate_unemployment, participate_workinjury, participate_maternity, participate_housing_fund, effective_date, enabled, create_time, update_time)
SELECT 3, 'beijing', 15000.00, 15000.00, 0.12, 0.12, 1, 1, 1, 1, 1, 1, '2026-01-01', 1, NOW(), NOW()
WHERE NOT EXISTS (
    SELECT 1 FROM mxx_finance_employee_insurance_config WHERE employee_id = 3 AND enabled = 1
);

-- ============================================================
-- 6. 为 admin(id=3) 补充考勤记录
-- ============================================================
INSERT INTO mxx_finance_attendance_record (employee_id, year, month, work_days, actual_work_days, late_count, early_leave_count, absent_count, personal_leave_days, sick_leave_days, overtime_hours_weekday, overtime_hours_weekend, data_source, create_time)
SELECT 3, 2026, 6, 22, 22, 0, 0, 0, 0, 0, 10, 0, 1, NOW()
WHERE NOT EXISTS (
    SELECT 1 FROM mxx_finance_attendance_record WHERE employee_id = 3 AND year = 2026 AND month = 6
);

-- ============================================================
-- 7. 为 finance(id=11) 补充考勤记录
-- ============================================================
INSERT INTO mxx_finance_attendance_record (employee_id, year, month, work_days, actual_work_days, late_count, early_leave_count, absent_count, personal_leave_days, sick_leave_days, overtime_hours_weekday, overtime_hours_weekend, data_source, create_time)
SELECT 11, 2026, 6, 22, 22, 0, 0, 0, 0, 0, 0, 0, 1, NOW()
WHERE NOT EXISTS (
    SELECT 1 FROM mxx_finance_attendance_record WHERE employee_id = 11 AND year = 2026 AND month = 6
);

-- ============================================================
-- 8. 创建业绩计划数据（已审批通过状态 status=2）
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
-- 9. 创建月度目标（6月目标，用于绩效系数计算）
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
-- 10. 验证数据
-- ============================================================
SELECT '== 修复后数据统计 ==' AS info;
SELECT
    (SELECT COUNT(*) FROM mxx_statistics_performance_plan WHERE deleted=0) AS perf_plan_count,
    (SELECT COUNT(*) FROM mxx_statistics_plan_monthly_target WHERE deleted=0) AS monthly_target_count,
    (SELECT COUNT(*) FROM mxx_system_admin WHERE user_name='finance' AND deleted=0) AS finance_user_count,
    (SELECT COUNT(*) FROM mxx_finance_salary_config WHERE deleted=0) AS salary_config_count,
    (SELECT COUNT(*) FROM mxx_finance_employee_tax_config) AS tax_config_count,
    (SELECT COUNT(*) FROM mxx_finance_employee_insurance_config WHERE enabled=1) AS insurance_config_count,
    (SELECT COUNT(*) FROM mxx_finance_attendance_record WHERE year=2026 AND month=6) AS attendance_count;
