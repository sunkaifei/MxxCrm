-- =====================================================
-- 提成方案系统 V2 数据库迁移脚本
-- 适用数据库: PostgreSQL
-- 执行时间: 2026-07-07
-- 说明: 不重命名旧表，仅新增字段和新表，保持向后兼容
-- =====================================================

-- =====================================================
-- 1. 提成规则表新增字段（升级为方案表）
-- =====================================================

ALTER TABLE mxx_finance_commission_rule
    ADD COLUMN IF NOT EXISTS apply_scope INT NOT NULL DEFAULT 1,
    ADD COLUMN IF NOT EXISTS commission_target_type INT,
    ADD COLUMN IF NOT EXISTS priority INT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS is_default INT NOT NULL DEFAULT 0;

-- 新增索引
CREATE INDEX IF NOT EXISTS idx_rule_is_default ON mxx_finance_commission_rule(is_default);

-- 注释
COMMENT ON COLUMN mxx_finance_commission_rule.apply_scope IS '适用范围: 1=指定部门 2=全公司 3=指定岗位 4=指定人员';
COMMENT ON COLUMN mxx_finance_commission_rule.rule_type IS '规则类型/方案类型: 1=个人提成 2=团队分成 3=部门经理 4=总监 5=团队长';
COMMENT ON COLUMN mxx_finance_commission_rule.commission_target_type IS '提成对象岗位类型（经理/总监/团队长的岗位标识）';
COMMENT ON COLUMN mxx_finance_commission_rule.priority IS '优先级（数字越小越先计算）';
COMMENT ON COLUMN mxx_finance_commission_rule.is_default IS '是否默认方案: 0=否 1=是';

-- =====================================================
-- 2. 提成规则成员表新增字段
-- =====================================================

ALTER TABLE mxx_finance_commission_rule_member
    ADD COLUMN IF NOT EXISTS distribution_type INT NOT NULL DEFAULT 1,
    ADD COLUMN IF NOT EXISTS role_name VARCHAR(50),
    ADD COLUMN IF NOT EXISTS default_ratio NUMERIC(6,4) NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS required INT NOT NULL DEFAULT 0;

-- 数据迁移：把原 fixed_rate 复制到 default_ratio（如果还没有值的话）
UPDATE mxx_finance_commission_rule_member SET default_ratio = fixed_rate WHERE default_ratio = 0;

-- 注释
COMMENT ON COLUMN mxx_finance_commission_rule_member.distribution_type IS '分配类型: 1=固定比例';
COMMENT ON COLUMN mxx_finance_commission_rule_member.role_name IS '角色名称';
COMMENT ON COLUMN mxx_finance_commission_rule_member.default_ratio IS '默认分成比例 0.6000=60%';
COMMENT ON COLUMN mxx_finance_commission_rule_member.required IS '是否必选: 0=否 1=是';

-- =====================================================
-- 3. 新增：合同提成人员表
-- =====================================================

CREATE TABLE IF NOT EXISTS mxx_crm_contract_commission_member (
    id BIGSERIAL PRIMARY KEY,
    contract_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    user_name VARCHAR(50),
    role_type INT NOT NULL,
    share_ratio NUMERIC(6,4) NOT NULL,
    sort INT NOT NULL DEFAULT 0,
    created_by BIGINT,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_by BIGINT,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_contract_member_contract
    ON mxx_crm_contract_commission_member(contract_id);

ALTER TABLE mxx_crm_contract_commission_member
    ADD CONSTRAINT fk_contract_member_contract FOREIGN KEY (contract_id)
        REFERENCES mxx_crm_contract(id) ON DELETE CASCADE;

COMMENT ON TABLE mxx_crm_contract_commission_member IS '合同提成人员表';
COMMENT ON COLUMN mxx_crm_contract_commission_member.role_type IS '角色类型: 1=主签人 2=参与人 3=技术支持 4=其他';
COMMENT ON COLUMN mxx_crm_contract_commission_member.share_ratio IS '分成比例 0.6000=60%';

-- =====================================================
-- 4. 新增：提成计算结果表
-- =====================================================

CREATE TABLE IF NOT EXISTS mxx_finance_commission_result (
    id BIGSERIAL PRIMARY KEY,
    salary_record_id BIGINT,
    contract_id BIGINT,
    contract_name VARCHAR(200),
    rule_id BIGINT NOT NULL,
    rule_name VARCHAR(100),
    rule_type INT NOT NULL,
    user_id BIGINT NOT NULL,
    user_name VARCHAR(50),
    user_post_id BIGINT,
    department_id BIGINT,
    calc_base_amount NUMERIC(14,2) NOT NULL DEFAULT 0,
    tier_min_amount NUMERIC(14,2),
    tier_max_amount NUMERIC(14,2),
    commission_rate NUMERIC(6,4) NOT NULL,
    share_ratio NUMERIC(6,4),
    commission_amount NUMERIC(14,2) NOT NULL DEFAULT 0,
    trigger_condition INT NOT NULL,
    trigger_source_id BIGINT,
    period_year INT NOT NULL,
    period_month INT NOT NULL,
    settled INT NOT NULL DEFAULT 0,
    remark VARCHAR(500),
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_result_contract ON mxx_finance_commission_result(contract_id);
CREATE INDEX IF NOT EXISTS idx_result_user_period ON mxx_finance_commission_result(user_id, period_year, period_month);
CREATE INDEX IF NOT EXISTS idx_result_rule ON mxx_finance_commission_result(rule_id);
CREATE INDEX IF NOT EXISTS idx_result_settled ON mxx_finance_commission_result(settled);
CREATE INDEX IF NOT EXISTS idx_result_salary ON mxx_finance_commission_result(salary_record_id);

COMMENT ON TABLE mxx_finance_commission_result IS '提成计算结果表';
COMMENT ON COLUMN mxx_finance_commission_result.rule_type IS '规则类型/方案类型: 1=个人提成 2=团队分成 3=部门经理 4=总监 5=团队长';
COMMENT ON COLUMN mxx_finance_commission_result.trigger_condition IS '触发条件: 1=完全回款 2=合同签订 3=部分回款 4=发货完成 5=客户验收';
COMMENT ON COLUMN mxx_finance_commission_result.settled IS '是否已结算到工资: 0=否 1=是';

-- =====================================================
-- 5. 合同表新增提成方案字段
-- =====================================================

ALTER TABLE mxx_crm_contract
    ADD COLUMN IF NOT EXISTS commission_rule_id BIGINT,
    ADD COLUMN IF NOT EXISTS commission_mode INT NOT NULL DEFAULT 1;

COMMENT ON COLUMN mxx_crm_contract.commission_rule_id IS '提成方案ID（为空则走默认方案）';
COMMENT ON COLUMN mxx_crm_contract.commission_mode IS '提成模式: 1=按方案自动计算 2=手动指定分成';

-- =====================================================
-- 迁移完成
-- =====================================================
