-- V6 测试数据：财务账号 + 回款计划 + 工资配置 + 员工个税/社保配置 + 考勤记录
-- 执行方式：psql -h 127.0.0.1 -U postgres -d mxxcrm_data -f v6_test_data.sql

-- ============================================================
-- 0. 创建缺失的工资确认表
-- ============================================================
CREATE TABLE IF NOT EXISTS mxx_finance_salary_confirm (
    id BIGSERIAL PRIMARY KEY,
    salary_record_id BIGINT NOT NULL,
    employee_id BIGINT NOT NULL,
    employee_name VARCHAR(64),
    year INT NOT NULL,
    month INT NOT NULL,
    action INT NOT NULL,
    reason TEXT,
    status INT DEFAULT 0,
    handler_id BIGINT,
    handler_name VARCHAR(64),
    handle_time TIMESTAMP,
    handle_remark TEXT,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_salary_confirm_record ON mxx_finance_salary_confirm (salary_record_id);
CREATE INDEX IF NOT EXISTS idx_salary_confirm_employee ON mxx_finance_salary_confirm (employee_id, year, month);

-- ============================================================
-- 1. 创建财务账号（复制 admin 的密码哈希，密码 = 123456）
-- ============================================================
INSERT INTO mxx_system_admin (user_name, nick_name, user_type, email, mobile, gender, password, status, deleted, create_time, update_time)
SELECT 'finance', '财务专员', 0, 'finance@mxxcrm.com', '13800138002', 0, password, 0, 0, NOW(), NOW()
FROM mxx_system_admin WHERE id = 3
RETURNING id;

-- 分配财务角色（role_id=10）
INSERT INTO mxx_system_admin_role_merge (admin_id, role_id, create_time)
SELECT a.id, 10, NOW()
FROM mxx_system_admin a
WHERE a.user_name = 'finance' AND a.deleted = 0
  AND NOT EXISTS (SELECT 1 FROM mxx_system_admin_role_merge arm WHERE arm.admin_id = a.id AND arm.role_id = 10);

-- ============================================================
-- 2. 为现有员工配置银行账户（用于银行代发文件导出）
-- ============================================================
UPDATE mxx_system_admin SET
    bank_card_no = '6222021234567890001',
    bank_name = '工商银行',
    bank_account_name = nick_name
WHERE id = 5 AND bank_card_no IS NULL;

UPDATE mxx_system_admin SET
    bank_card_no = '6222021234567890002',
    bank_name = '工商银行',
    bank_account_name = nick_name
WHERE id = 6 AND bank_card_no IS NULL;

UPDATE mxx_system_admin SET
    bank_card_no = '6222021234567890003',
    bank_name = '工商银行',
    bank_account_name = nick_name
WHERE id = 7 AND bank_card_no IS NULL;

UPDATE mxx_system_admin SET
    bank_card_no = '6222021234567890004',
    bank_name = '建设银行',
    bank_account_name = nick_name
WHERE id = 9 AND bank_card_no IS NULL;

UPDATE mxx_system_admin SET
    bank_card_no = '6222021234567890005',
    bank_name = '建设银行',
    bank_account_name = nick_name
WHERE id = 10 AND bank_card_no IS NULL;

-- ============================================================
-- 3. 创建回款计划（基于现有合同）
-- 上月 = 2026-06，actual_date 设为 2026-06-15，received = plan_amount
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
    'V6测试回款数据',
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
-- 4. 创建工资配置（底薪 + 岗位津贴 + 绩效基数）
-- ============================================================
INSERT INTO mxx_finance_salary_config (employee_id, year, base_salary, position_allowance, performance_base, performance_coefficient, create_time, update_time, deleted)
SELECT
    a.id,
    2026,
    8000,
    2000,
    5000,
    1.0,
    NOW(),
    NOW(),
    0
FROM mxx_system_admin a
WHERE a.id IN (5, 6, 7, 9, 10)
  AND a.deleted = 0
  AND NOT EXISTS (
      SELECT 1 FROM mxx_finance_salary_config sc
      WHERE sc.employee_id = a.id AND sc.year = 2026 AND sc.deleted = 0
  );

-- ============================================================
-- 5. 创建员工个税配置（2026年）
-- ============================================================
INSERT INTO mxx_finance_employee_tax_config (employee_id, year, tax_threshold, children_education, continuing_education, housing_loan, housing_rent, supporting_elderly, infant_care, create_time, update_time)
SELECT
    a.id,
    2026,
    5000,
    0,
    0,
    1000,
    0,
    2000,
    0,
    NOW(),
    NOW()
FROM mxx_system_admin a
WHERE a.id IN (5, 6, 7, 9, 10)
  AND a.deleted = 0
  AND NOT EXISTS (
      SELECT 1 FROM mxx_finance_employee_tax_config tc
      WHERE tc.employee_id = a.id AND tc.year = 2026
  );

-- ============================================================
-- 6. 创建员工社保配置（北京 2026）
-- ============================================================
INSERT INTO mxx_finance_employee_insurance_config (employee_id, city_code, base_amount, participate_pension, participate_medical, participate_unemployment, participate_workinjury, participate_maternity, participate_housing_fund, enabled, create_time, update_time)
SELECT
    a.id,
    'beijing',
    10000,
    1, 1, 1, 1, 1, 1,
    1,
    NOW(),
    NOW()
FROM mxx_system_admin a
WHERE a.id IN (5, 6, 7, 9, 10)
  AND a.deleted = 0
  AND NOT EXISTS (
      SELECT 1 FROM mxx_finance_employee_insurance_config ic
      WHERE ic.employee_id = a.id AND ic.enabled = 1
  );

-- 添加北京 2026 社保政策（基于 2025 数据）
INSERT INTO mxx_finance_social_insurance_policy (city_code, city_name, year, base_lower, base_upper, effective_month, enabled)
SELECT 'beijing', '北京', 2026, 7162, 35811, 7, 1
WHERE NOT EXISTS (
    SELECT 1 FROM mxx_finance_social_insurance_policy p
    WHERE p.city_code = 'beijing' AND p.year = 2026
);

-- ============================================================
-- 7. 创建考勤记录（2026-06）
-- ============================================================
INSERT INTO mxx_finance_attendance_record (employee_id, year, month, work_days, actual_work_days, late_count, early_leave_count, absent_count, personal_leave_days, sick_leave_days, overtime_hours_weekday, overtime_hours_weekend, create_time)
SELECT
    a.id,
    2026,
    6,
    22,
    22,
    0,
    0,
    0,
    0,
    0,
    10,
    8,
    NOW()
FROM mxx_system_admin a
WHERE a.id IN (5, 6, 7, 9, 10)
  AND a.deleted = 0
  AND NOT EXISTS (
      SELECT 1 FROM mxx_finance_attendance_record ar
      WHERE ar.employee_id = a.id AND ar.year = 2026 AND ar.month = 6
  );

-- ============================================================
-- 8. 验证数据
-- ============================================================
SELECT '== 数据统计 ==';
SELECT
    (SELECT COUNT(*) FROM mxx_system_admin WHERE user_name = 'finance' AND deleted = 0) AS finance_user_count,
    (SELECT COUNT(*) FROM mxx_system_admin WHERE bank_card_no IS NOT NULL) AS employees_with_bank,
    (SELECT COUNT(*) FROM mxx_crm_contract_payment_plan WHERE deleted = 0 AND actual_date IS NOT NULL) AS payment_plan_count,
    (SELECT COUNT(*) FROM mxx_finance_salary_config WHERE year = 2026 AND deleted = 0) AS salary_config_count,
    (SELECT COUNT(*) FROM mxx_finance_employee_tax_config WHERE year = 2026) AS tax_config_count,
    (SELECT COUNT(*) FROM mxx_finance_employee_insurance_config WHERE enabled = 1) AS insurance_config_count,
    (SELECT COUNT(*) FROM mxx_finance_attendance_record WHERE year = 2026 AND month = 6) AS attendance_count;
