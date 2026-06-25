-- 业绩目标计划表（计划头）
CREATE TABLE IF NOT EXISTS mxx_statistics_performance_plan (
    id BIGSERIAL PRIMARY KEY,
    employee_id BIGINT NOT NULL,
    year INT NOT NULL,
    status INT DEFAULT 0,           -- 0=草稿 1=待审批 2=已通过 3=已驳回
    apply_reason TEXT,              -- 申请/修改理由
    version INT DEFAULT 1,          -- 版本号（每次修改递增）
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted INT DEFAULT 0
);

-- 月度目标明细表
CREATE TABLE IF NOT EXISTS mxx_statistics_plan_monthly_target (
    id BIGSERIAL PRIMARY KEY,
    plan_id BIGINT NOT NULL REFERENCES mxx_statistics_performance_plan(id),
    month INT NOT NULL,                          -- 1-12月
    contract_target_amount DECIMAL(18,2) DEFAULT 0,
    payment_target_amount DECIMAL(18,2) DEFAULT 0,
    contract_target_count INT DEFAULT 0,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted INT DEFAULT 0
);

-- 审批记录表（可追溯）
CREATE TABLE IF NOT EXISTS mxx_statistics_plan_approval_log (
    id BIGSERIAL PRIMARY KEY,
    plan_id BIGINT NOT NULL REFERENCES mxx_statistics_performance_plan(id),
    action INT NOT NULL,            -- 1=提交 2=通过 3=驳回 4=修改申请
    operator_id BIGINT NOT NULL,    -- 操作人ID
    operator_name VARCHAR(100),     -- 操作人姓名
    reason TEXT,                    -- 理由/批复原因
    previous_status INT,            -- 操作前状态
    new_status INT,                 -- 操作后状态
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted INT DEFAULT 0
);