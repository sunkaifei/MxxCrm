-- ============================================================
-- V6 数据库迁移：个税 + 社保公积金 + 工资条 + 银行代发 + 团队提成 + 明细项 + 考勤 + 调薪
-- 执行方式：psql -h 115.190.210.106 -p 5432 -U mxxcrm -d mxxcrm_data -f v6_migration.sql
-- ============================================================

-- 1. 个税税率表
CREATE TABLE IF NOT EXISTS mxx_finance_tax_rate (
    id BIGSERIAL PRIMARY KEY,
    level INT NOT NULL,
    min_amount DECIMAL(12,2) NOT NULL,
    max_amount DECIMAL(12,2),
    rate DECIMAL(5,4) NOT NULL,
    quick_deduction DECIMAL(12,2) NOT NULL DEFAULT 0,
    tax_type INT NOT NULL,
    effective_date DATE NOT NULL,
    expiry_date DATE,
    enabled INT NOT NULL DEFAULT 1,
    create_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 2. 员工个税配置表
CREATE TABLE IF NOT EXISTS mxx_finance_employee_tax_config (
    id BIGSERIAL PRIMARY KEY,
    employee_id BIGINT NOT NULL,
    year INT NOT NULL,
    tax_threshold DECIMAL(12,2) NOT NULL DEFAULT 5000,
    children_education DECIMAL(12,2) NOT NULL DEFAULT 0,
    continuing_education DECIMAL(12,2) NOT NULL DEFAULT 0,
    housing_loan DECIMAL(12,2) NOT NULL DEFAULT 0,
    housing_rent DECIMAL(12,2) NOT NULL DEFAULT 0,
    supporting_elderly DECIMAL(12,2) NOT NULL DEFAULT 0,
    infant_care DECIMAL(12,2) NOT NULL DEFAULT 0,
    serious_illness DECIMAL(12,2) NOT NULL DEFAULT 0,
    other_deduction DECIMAL(12,2) NOT NULL DEFAULT 0,
    foreigner_allowance DECIMAL(12,2) NOT NULL DEFAULT 0,
    cumulative_income DECIMAL(12,2) NOT NULL DEFAULT 0,
    cumulative_threshold_deduction DECIMAL(12,2) NOT NULL DEFAULT 0,
    cumulative_special_deduction DECIMAL(12,2) NOT NULL DEFAULT 0,
    cumulative_other_deduction DECIMAL(12,2) NOT NULL DEFAULT 0,
    cumulative_tax_paid DECIMAL(12,2) NOT NULL DEFAULT 0,
    create_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(employee_id, year)
);

-- 3. 工资个税明细表
CREATE TABLE IF NOT EXISTS mxx_finance_salary_tax_detail (
    id BIGSERIAL PRIMARY KEY,
    salary_record_id BIGINT NOT NULL,
    employee_id BIGINT NOT NULL,
    year INT NOT NULL,
    month INT NOT NULL,
    monthly_income DECIMAL(12,2),
    monthly_threshold DECIMAL(12,2),
    monthly_special_deduction DECIMAL(12,2),
    monthly_other_deduction DECIMAL(12,2),
    cumulative_income DECIMAL(12,2),
    cumulative_taxable DECIMAL(12,2),
    applicable_rate DECIMAL(5,4),
    quick_deduction DECIMAL(12,2),
    cumulative_tax_should DECIMAL(12,2),
    cumulative_tax_paid DECIMAL(12,2),
    monthly_tax DECIMAL(12,2),
    create_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_tax_detail_employee ON mxx_finance_salary_tax_detail (employee_id, year, month);
CREATE INDEX IF NOT EXISTS idx_tax_detail_record ON mxx_finance_salary_tax_detail (salary_record_id);

-- 4. 城市社保政策库
CREATE TABLE IF NOT EXISTS mxx_finance_social_insurance_policy (
    id BIGSERIAL PRIMARY KEY,
    city_code VARCHAR(20) NOT NULL,
    city_name VARCHAR(50) NOT NULL,
    year INT NOT NULL,
    base_lower DECIMAL(12,2) NOT NULL,
    base_upper DECIMAL(12,2) NOT NULL,
    pension_company_rate DECIMAL(5,4) NOT NULL DEFAULT 0.16,
    pension_personal_rate DECIMAL(5,4) NOT NULL DEFAULT 0.08,
    medical_company_rate DECIMAL(5,4) NOT NULL DEFAULT 0.09,
    medical_personal_rate DECIMAL(5,4) NOT NULL DEFAULT 0.02,
    unemployment_company_rate DECIMAL(5,4) NOT NULL DEFAULT 0.005,
    unemployment_personal_rate DECIMAL(5,4) NOT NULL DEFAULT 0.005,
    workinjury_company_rate DECIMAL(5,4) NOT NULL DEFAULT 0.002,
    maternity_company_rate DECIMAL(5,4) NOT NULL DEFAULT 0.008,
    housing_fund_company_rate DECIMAL(5,4) NOT NULL DEFAULT 0.12,
    housing_fund_personal_rate DECIMAL(5,4) NOT NULL DEFAULT 0.12,
    effective_month INT NOT NULL DEFAULT 7,
    enabled INT NOT NULL DEFAULT 1,
    create_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(city_code, year)
);

-- 5. 员工社保配置表
CREATE TABLE IF NOT EXISTS mxx_finance_employee_insurance_config (
    id BIGSERIAL PRIMARY KEY,
    employee_id BIGINT NOT NULL,
    city_code VARCHAR(20) NOT NULL,
    base_amount DECIMAL(12,2) NOT NULL,
    housing_fund_base DECIMAL(12,2),
    housing_fund_company_rate DECIMAL(5,4),
    housing_fund_personal_rate DECIMAL(5,4),
    participate_pension INT NOT NULL DEFAULT 1,
    participate_medical INT NOT NULL DEFAULT 1,
    participate_unemployment INT NOT NULL DEFAULT 1,
    participate_workinjury INT NOT NULL DEFAULT 1,
    participate_maternity INT NOT NULL DEFAULT 1,
    participate_housing_fund INT NOT NULL DEFAULT 1,
    effective_date DATE,
    expiry_date DATE,
    enabled INT NOT NULL DEFAULT 1,
    create_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 6. 工资条表
CREATE TABLE IF NOT EXISTS mxx_finance_payslip (
    id BIGSERIAL PRIMARY KEY,
    salary_record_id BIGINT NOT NULL,
    employee_id BIGINT NOT NULL,
    year INT NOT NULL,
    month INT NOT NULL,
    total_salary DECIMAL(12,2),
    social_insurance_personal DECIMAL(12,2),
    tax_amount DECIMAL(12,2),
    net_salary DECIMAL(12,2),
    detail_json JSONB,
    send_status INT NOT NULL DEFAULT 0,
    send_channels VARCHAR(100),
    send_time TIMESTAMPTZ,
    read_time TIMESTAMPTZ,
    confirm_time TIMESTAMPTZ,
    password_protected INT NOT NULL DEFAULT 0,
    password_hash VARCHAR(100),
    create_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_payslip_employee ON mxx_finance_payslip (employee_id, year, month);

-- 7. 银行代发文件记录表
CREATE TABLE IF NOT EXISTS mxx_finance_bank_payment_file (
    id BIGSERIAL PRIMARY KEY,
    year INT NOT NULL,
    month INT NOT NULL,
    bank_type VARCHAR(20) NOT NULL,
    file_name VARCHAR(200),
    file_path VARCHAR(500),
    file_format VARCHAR(10),
    total_count INT,
    total_amount DECIMAL(14,2),
    status INT NOT NULL DEFAULT 0,
    creator_id BIGINT,
    creator_name VARCHAR(50),
    create_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 8. 工资项目定义表
CREATE TABLE IF NOT EXISTS mxx_finance_salary_item (
    id BIGSERIAL PRIMARY KEY,
    item_code VARCHAR(50) NOT NULL UNIQUE,
    item_name VARCHAR(100) NOT NULL,
    item_type INT NOT NULL,
    calc_mode INT NOT NULL,
    formula TEXT,
    default_value DECIMAL(12,2) NOT NULL DEFAULT 0,
    is_taxable INT NOT NULL DEFAULT 1,
    is_pretax INT NOT NULL DEFAULT 0,
    sort INT NOT NULL DEFAULT 0,
    enabled INT NOT NULL DEFAULT 1,
    create_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 9. 工资项目明细值表
CREATE TABLE IF NOT EXISTS mxx_finance_salary_item_value (
    id BIGSERIAL PRIMARY KEY,
    salary_record_id BIGINT NOT NULL,
    item_id BIGINT NOT NULL,
    item_code VARCHAR(50),
    item_name VARCHAR(100),
    amount DECIMAL(12,2) NOT NULL DEFAULT 0,
    is_taxable INT NOT NULL DEFAULT 1
);
CREATE INDEX IF NOT EXISTS idx_item_value_record ON mxx_finance_salary_item_value (salary_record_id);

-- 10. 月度考勤汇总表
CREATE TABLE IF NOT EXISTS mxx_finance_attendance_record (
    id BIGSERIAL PRIMARY KEY,
    employee_id BIGINT NOT NULL,
    year INT NOT NULL,
    month INT NOT NULL,
    work_days DECIMAL(5,1),
    actual_work_days DECIMAL(5,1),
    late_count INT NOT NULL DEFAULT 0,
    early_leave_count INT NOT NULL DEFAULT 0,
    absent_count INT NOT NULL DEFAULT 0,
    personal_leave_days DECIMAL(5,1) NOT NULL DEFAULT 0,
    sick_leave_days DECIMAL(5,1) NOT NULL DEFAULT 0,
    annual_leave_days DECIMAL(5,1) NOT NULL DEFAULT 0,
    overtime_hours_weekday DECIMAL(6,1) NOT NULL DEFAULT 0,
    overtime_hours_weekend DECIMAL(6,1) NOT NULL DEFAULT 0,
    overtime_hours_holiday DECIMAL(6,1) NOT NULL DEFAULT 0,
    data_source INT NOT NULL DEFAULT 1,
    create_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(employee_id, year, month)
);

-- 11. 调薪记录表
CREATE TABLE IF NOT EXISTS mxx_finance_salary_adjustment (
    id BIGSERIAL PRIMARY KEY,
    employee_id BIGINT NOT NULL,
    adjustment_date DATE NOT NULL,
    adjustment_type INT NOT NULL,
    old_base_salary DECIMAL(12,2),
    new_base_salary DECIMAL(12,2),
    old_position_allowance DECIMAL(12,2),
    new_position_allowance DECIMAL(12,2),
    old_performance_base DECIMAL(12,2),
    new_performance_base DECIMAL(12,2),
    adjustment_reason TEXT,
    approver_id BIGINT,
    approver_name VARCHAR(50),
    approve_time TIMESTAMPTZ,
    status INT NOT NULL DEFAULT 0,
    create_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_adjustment_employee ON mxx_finance_salary_adjustment (employee_id);

-- 12. 工资记录表扩展字段
ALTER TABLE mxx_finance_salary_record ADD COLUMN IF NOT EXISTS social_insurance_personal DECIMAL(12,2) NOT NULL DEFAULT 0;
ALTER TABLE mxx_finance_salary_record ADD COLUMN IF NOT EXISTS housing_fund_personal DECIMAL(12,2) NOT NULL DEFAULT 0;
ALTER TABLE mxx_finance_salary_record ADD COLUMN IF NOT EXISTS social_insurance_company DECIMAL(12,2) NOT NULL DEFAULT 0;
ALTER TABLE mxx_finance_salary_record ADD COLUMN IF NOT EXISTS housing_fund_company DECIMAL(12,2) NOT NULL DEFAULT 0;
ALTER TABLE mxx_finance_salary_record ADD COLUMN IF NOT EXISTS tax_amount DECIMAL(12,2) NOT NULL DEFAULT 0;
ALTER TABLE mxx_finance_salary_record ADD COLUMN IF NOT EXISTS net_salary DECIMAL(12,2) NOT NULL DEFAULT 0;
ALTER TABLE mxx_finance_salary_record ADD COLUMN IF NOT EXISTS team_commission_amount DECIMAL(12,2) NOT NULL DEFAULT 0;

-- 13. 提成规则表扩展字段
ALTER TABLE mxx_finance_commission_rule ADD COLUMN IF NOT EXISTS calc_base_field VARCHAR(50) NOT NULL DEFAULT 'payment_amount';
ALTER TABLE mxx_finance_commission_rule ADD COLUMN IF NOT EXISTS tier_mode INT NOT NULL DEFAULT 1;

-- 14. 员工表扩展银行账户字段
ALTER TABLE mxx_system_admin ADD COLUMN IF NOT EXISTS bank_card_no VARCHAR(50);
ALTER TABLE mxx_system_admin ADD COLUMN IF NOT EXISTS bank_name VARCHAR(50);
ALTER TABLE mxx_system_admin ADD COLUMN IF NOT EXISTS bank_account_name VARCHAR(50);

-- 15. 初始化个税税率表（2026 综合所得累计预扣率表）
INSERT INTO mxx_finance_tax_rate (level, min_amount, max_amount, rate, quick_deduction, tax_type, effective_date) VALUES
(1, 0,     36000,    0.03, 0,       1, '2026-01-01'),
(2, 36000, 144000,   0.10, 2520,    1, '2026-01-01'),
(3, 144000, 300000,  0.20, 16920,   1, '2026-01-01'),
(4, 300000, 420000,  0.25, 31920,   1, '2026-01-01'),
(5, 420000, 660000,  0.30, 52920,   1, '2026-01-01'),
(6, 660000, 960000,  0.35, 85920,   1, '2026-01-01'),
(7, 960000, NULL,    0.45, 181920,  1, '2026-01-01')
ON CONFLICT DO NOTHING;

-- 月度税率表（年终奖用，tax_type=2）
INSERT INTO mxx_finance_tax_rate (level, min_amount, max_amount, rate, quick_deduction, tax_type, effective_date) VALUES
(1, 0,     3000,    0.03, 0,       2, '2026-01-01'),
(2, 3000,  12000,   0.10, 210,     2, '2026-01-01'),
(3, 12000, 25000,   0.20, 1410,    2, '2026-01-01'),
(4, 25000, 35000,   0.25, 2660,    2, '2026-01-01'),
(5, 35000, 55000,   0.30, 4410,    2, '2026-01-01'),
(6, 55000, 80000,   0.35, 7160,    2, '2026-01-01'),
(7, 80000, NULL,    0.45, 15160,   2, '2026-01-01')
ON CONFLICT DO NOTHING;

-- 16. 预置工资项目
INSERT INTO mxx_finance_salary_item (item_code, item_name, item_type, calc_mode, default_value, is_taxable, sort) VALUES
('meal_allowance', '餐补', 1, 1, 500, 1, 10),
('transport_allowance', '交通补贴', 1, 1, 300, 1, 11),
('communication_allowance', '通讯补贴', 1, 1, 200, 1, 12),
('overtime_pay', '加班费', 1, 2, 0, 1, 13),
('full_attendance_bonus', '全勤奖', 1, 1, 200, 1, 14),
('late_deduction', '迟到扣款', 2, 2, 0, 0, 20),
('leave_deduction', '请假扣款', 2, 3, 0, 0, 21),
('other_bonus', '其他奖金', 1, 3, 0, 1, 15),
('other_deduction', '其他扣款', 2, 3, 0, 0, 22)
ON CONFLICT (item_code) DO NOTHING;

-- 17. 预置北京 2025 社保政策
INSERT INTO mxx_finance_social_insurance_policy (city_code, city_name, year, base_lower, base_upper, effective_month) VALUES
('beijing', '北京', 2025, 7162, 35811, 7)
ON CONFLICT (city_code, year) DO NOTHING;

-- 18. 新增菜单与权限码（先查父菜单ID，财务父菜单通常 id=500 或通过 menu_name='财务' 查询）
-- 此处使用固定 ID，若冲突请手动调整
INSERT INTO mxx_system_menu (id, parent_id, menu_name, route_name, route_path, component, menu_type, perms, icon, sort, is_show, created_at)
VALUES
(530, 500, '个税管理', 'FinanceTax', 'tax', 'finance/tax/index', 2, 'finance:tax:list', 'mdi:calculator', 60, 1, NOW()),
(531, 530, '查看', NULL, NULL, NULL, 3, 'finance:tax:list', NULL, 1, 1, NOW()),
(532, 530, '管理', NULL, NULL, NULL, 3, 'finance:tax:manage', NULL, 2, 1, NOW()),
(533, 500, '社保公积金', 'FinanceSocialInsurance', 'social-insurance', 'finance/social-insurance/index', 2, 'finance:insurance:list', 'mdi:shield-account', 61, 1, NOW()),
(534, 533, '查看', NULL, NULL, NULL, 3, 'finance:insurance:list', NULL, 1, 1, NOW()),
(535, 533, '管理', NULL, NULL, NULL, 3, 'finance:insurance:manage', NULL, 2, 1, NOW()),
(536, 500, '工资条下发', 'FinancePayslip', 'payslip', 'finance/payslip/index', 2, 'finance:payslip:list', 'mdi:email-send', 62, 1, NOW()),
(537, 536, '查看', NULL, NULL, NULL, 3, 'finance:payslip:list', NULL, 1, 1, NOW()),
(538, 536, '管理', NULL, NULL, NULL, 3, 'finance:payslip:manage', NULL, 2, 1, NOW()),
(539, 500, '银行代发', 'FinanceBankExport', 'bank-export', 'finance/bank-export/index', 2, 'finance:bank-export:list', 'mdi:bank', 63, 1, NOW()),
(540, 539, '查看', NULL, NULL, NULL, 3, 'finance:bank-export:list', NULL, 1, 1, NOW()),
(541, 539, '管理', NULL, NULL, NULL, 3, 'finance:bank-export:manage', NULL, 2, 1, NOW()),
(542, 500, '考勤扣款', 'FinanceAttendance', 'attendance', 'finance/attendance/index', 2, 'finance:attendance:list', 'mdi:clock-check', 64, 1, NOW()),
(543, 542, '查看', NULL, NULL, NULL, 3, 'finance:attendance:list', NULL, 1, 1, NOW()),
(544, 542, '管理', NULL, NULL, NULL, 3, 'finance:attendance:manage', NULL, 2, 1, NOW())
ON CONFLICT (id) DO NOTHING;

-- 19. 增加财务角色（若不存在）
INSERT INTO mxx_system_role (role_name, role_key, status, sort, data_scope, created_at)
SELECT '财务', 'finance', 1, 50, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_role WHERE role_key = 'finance');

-- 20. 为财务角色分配财务相关菜单权限（含工资核算所有权限）
-- 此处需要根据实际 role_id 和 menu_id 调整，先提供一个示例
-- 假设财务角色 id 通过查询获取
DO $$
DECLARE
    finance_role_id BIGINT;
BEGIN
    SELECT id INTO finance_role_id FROM mxx_system_role WHERE role_key = 'finance' LIMIT 1;
    IF finance_role_id IS NOT NULL THEN
        -- 分配财务相关菜单权限
        INSERT INTO mxx_system_role_menu_merge (role_id, menu_id, created_at)
        SELECT finance_role_id, id, NOW() FROM mxx_system_menu
        WHERE perms LIKE 'finance:%' OR perms LIKE 'system:scheduler:%'
        ON CONFLICT DO NOTHING;
    END IF;
END $$;
