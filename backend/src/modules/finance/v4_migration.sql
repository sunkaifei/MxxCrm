-- V4 数据库迁移：工资确认/申诉 + 定时任务管理
-- 执行方式：psql -h 115.190.210.106 -p 5432 -U mxxcrm -d mxxcrm_data -f v4_migration.sql

-- ============================================================
-- 1. 工资确认/申诉记录表
-- ============================================================
CREATE TABLE IF NOT EXISTS mxx_finance_salary_confirm (
    id BIGSERIAL PRIMARY KEY,
    salary_record_id BIGINT NOT NULL,
    employee_id BIGINT NOT NULL,
    employee_name VARCHAR(50),
    year INT NOT NULL,
    month INT NOT NULL,
    action INT NOT NULL,
    reason TEXT,
    status INT NOT NULL DEFAULT 0,
    handler_id BIGINT,
    handler_name VARCHAR(50),
    handle_time TIMESTAMP,
    handle_remark TEXT,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_confirm_employee ON mxx_finance_salary_confirm (employee_id);
CREATE INDEX IF NOT EXISTS idx_confirm_status ON mxx_finance_salary_confirm (status);
CREATE INDEX IF NOT EXISTS idx_confirm_salary ON mxx_finance_salary_confirm (salary_record_id);

-- 工资记录表新增员工确认状态字段
ALTER TABLE mxx_finance_salary_record ADD COLUMN IF NOT EXISTS employee_confirmed INT NOT NULL DEFAULT 0;
ALTER TABLE mxx_finance_salary_record ADD COLUMN IF NOT EXISTS confirmed_time TIMESTAMP;

-- ============================================================
-- 2. 定时任务管理表
-- ============================================================
CREATE TABLE IF NOT EXISTS mxx_system_scheduler_job (
    id BIGSERIAL PRIMARY KEY,
    job_code VARCHAR(50) NOT NULL UNIQUE,
    job_name VARCHAR(100) NOT NULL,
    cron_expression VARCHAR(100) NOT NULL,
    handler VARCHAR(100) NOT NULL,
    handler_params JSONB,
    description TEXT,
    job_type INT NOT NULL DEFAULT 0,
    enabled INT NOT NULL DEFAULT 1,
    last_run_time TIMESTAMP,
    last_run_status INT,
    last_run_result TEXT,
    next_run_time TIMESTAMP,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted INT NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_job_enabled ON mxx_system_scheduler_job (enabled);

CREATE TABLE IF NOT EXISTS mxx_system_scheduler_log (
    id BIGSERIAL PRIMARY KEY,
    job_id BIGINT NOT NULL,
    job_code VARCHAR(50),
    trigger_type INT NOT NULL DEFAULT 0,
    status INT NOT NULL,
    result_message TEXT,
    error_message TEXT,
    elapsed_ms BIGINT,
    operator_id BIGINT,
    operator_name VARCHAR(50),
    start_time TIMESTAMP,
    end_time TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_log_job_id ON mxx_system_scheduler_log (job_id);
CREATE INDEX IF NOT EXISTS idx_log_start_time ON mxx_system_scheduler_log (start_time);

-- 初始化内置定时任务：月度工资自动核算（每月1号02:00）
INSERT INTO mxx_system_scheduler_job (job_code, job_name, cron_expression, handler, description, job_type, enabled)
VALUES ('salary_monthly_calculate', '月度工资自动核算', '0 0 2 1 * *', 'salary_calculate', '每月1号02:00自动核算上月全员工资', 0, 1)
ON CONFLICT (job_code) DO NOTHING;
