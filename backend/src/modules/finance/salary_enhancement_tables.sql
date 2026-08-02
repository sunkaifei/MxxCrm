-- 工资核算增强 - 底薪配置表 + 核算日志表
-- 执行环境：生产库 mxxcrm_data

-- 1. 员工底薪配置表
CREATE TABLE IF NOT EXISTS mxx_finance_salary_config (
    id BIGSERIAL PRIMARY KEY,
    employee_id BIGINT NOT NULL,
    year INT NOT NULL,
    month INT,
    base_salary NUMERIC(12,2) NOT NULL DEFAULT 0,
    position_allowance NUMERIC(12,2) DEFAULT 0,
    performance_base NUMERIC(12,2) DEFAULT 0,
    performance_coefficient NUMERIC(4,2),
    status INT NOT NULL DEFAULT 1,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted INT NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_salary_config_emp ON mxx_finance_salary_config (employee_id, year, month);
CREATE INDEX IF NOT EXISTS idx_salary_config_status ON mxx_finance_salary_config (status, deleted);

-- 2. 工资核算日志表
CREATE TABLE IF NOT EXISTS mxx_finance_salary_calc_log (
    id BIGSERIAL PRIMARY KEY,
    year INT NOT NULL,
    month INT NOT NULL,
    trigger_type INT NOT NULL DEFAULT 0,
    result INT,
    generated_count BIGINT DEFAULT 0,
    error_message TEXT,
    elapsed_ms BIGINT,
    operator_id BIGINT DEFAULT 0,
    operator_name VARCHAR(50),
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_salary_calc_log_ym ON mxx_finance_salary_calc_log (year, month);
CREATE INDEX IF NOT EXISTS idx_salary_calc_log_trigger ON mxx_finance_salary_calc_log (trigger_type, create_time);
