-- 提成规则表
CREATE TABLE mxx_finance_commission_rule (
    id BIGSERIAL PRIMARY KEY,
    rule_name VARCHAR(100) NOT NULL,
    department_id BIGINT,
    post_id BIGINT,
    trigger_condition INT NOT NULL DEFAULT 1,
    effective_date DATE NOT NULL,
    expiry_date DATE,
    enabled INT NOT NULL DEFAULT 1,
    description TEXT,
    created_by BIGINT,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_by BIGINT,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted INT NOT NULL DEFAULT 0
);
CREATE INDEX idx_rule_dept_post ON mxx_finance_commission_rule (department_id, post_id);
CREATE INDEX idx_rule_enabled ON mxx_finance_commission_rule (enabled);

-- 提成阶梯表
CREATE TABLE mxx_finance_commission_tier (
    id BIGSERIAL PRIMARY KEY,
    rule_id BIGINT NOT NULL,
    min_amount NUMERIC(14,2) NOT NULL DEFAULT 0,
    max_amount NUMERIC(14,2),
    commission_rate NUMERIC(6,4) NOT NULL,
    sort INT NOT NULL DEFAULT 0,
    CONSTRAINT fk_tier_rule FOREIGN KEY (rule_id) REFERENCES mxx_finance_commission_rule(id) ON DELETE CASCADE
);
CREATE INDEX idx_tier_rule_id ON mxx_finance_commission_tier (rule_id);

-- 月度工资记录表
CREATE TABLE mxx_finance_salary_record (
    id BIGSERIAL PRIMARY KEY,
    employee_id BIGINT NOT NULL,
    employee_name VARCHAR(50),
    department_name VARCHAR(50),
    year INT NOT NULL,
    month INT NOT NULL,
    base_salary NUMERIC(12,2) NOT NULL DEFAULT 0,
    commission_amount NUMERIC(14,2) NOT NULL DEFAULT 0,
    performance_bonus NUMERIC(12,2) NOT NULL DEFAULT 0,
    deduction_amount NUMERIC(12,2) NOT NULL DEFAULT 0,
    total_salary NUMERIC(14,2) NOT NULL DEFAULT 0,
    status INT NOT NULL DEFAULT 0,
    remark TEXT,
    created_by BIGINT,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_by BIGINT,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted INT NOT NULL DEFAULT 0
);
CREATE INDEX idx_salary_emp_month ON mxx_finance_salary_record (employee_id, year, month);
CREATE INDEX idx_salary_status ON mxx_finance_salary_record (status);
CREATE INDEX idx_salary_year_month ON mxx_finance_salary_record (year, month);

-- 提成明细表
CREATE TABLE mxx_finance_commission_detail (
    id BIGSERIAL PRIMARY KEY,
    salary_record_id BIGINT NOT NULL,
    contract_id BIGINT,
    contract_name VARCHAR(200),
    contract_amount NUMERIC(14,2),
    payment_amount NUMERIC(14,2),
    commission_base NUMERIC(14,2),
    commission_rate NUMERIC(6,4),
    commission_amount NUMERIC(14,2),
    rule_name VARCHAR(100),
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_detail_salary FOREIGN KEY (salary_record_id) REFERENCES mxx_finance_salary_record(id) ON DELETE CASCADE
);
CREATE INDEX idx_detail_salary ON mxx_finance_commission_detail (salary_record_id);
CREATE INDEX idx_detail_contract ON mxx_finance_commission_detail (contract_id);

-- 采购付款记录表
CREATE TABLE mxx_finance_payment (
    id BIGSERIAL PRIMARY KEY,
    payment_no VARCHAR(50),
    purchase_order_id BIGINT,
    purchase_order_no VARCHAR(50),
    supplier_name VARCHAR(100),
    payment_type INT,
    payment_amount NUMERIC(14,2) NOT NULL DEFAULT 0,
    payment_method INT,
    bank_account VARCHAR(50),
    status INT NOT NULL DEFAULT 0,
    applicant_id BIGINT,
    applicant_name VARCHAR(50),
    apply_time TIMESTAMP,
    approver_id BIGINT,
    approver_name VARCHAR(50),
    approve_time TIMESTAMP,
    approve_remark TEXT,
    payment_date DATE,
    remark TEXT,
    created_by BIGINT,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_by BIGINT,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted INT NOT NULL DEFAULT 0
);
CREATE INDEX idx_payment_po ON mxx_finance_payment (purchase_order_id);
CREATE INDEX idx_payment_status ON mxx_finance_payment (status);
CREATE INDEX idx_payment_no ON mxx_finance_payment (payment_no);
