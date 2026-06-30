-- ============================================================
-- 回款模块数据库迁移（合同回款重构）
-- 包含：回款计划表 + 回款记录表字段改造
-- ============================================================

-- 1. 创建回款计划表 mxx_crm_contract_payment_plan
CREATE TABLE IF NOT EXISTS mxx_crm_contract_payment_plan (
    id BIGSERIAL PRIMARY KEY,
    contract_id BIGINT NOT NULL,
    stage_name VARCHAR(64) NOT NULL,
    payment_type INTEGER NOT NULL DEFAULT 1,
    plan_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    received_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    plan_date DATE,
    actual_date DATE,
    status INTEGER NOT NULL DEFAULT 1,
    sort INTEGER NOT NULL DEFAULT 0,
    remark VARCHAR(500),
    owner_user_id BIGINT,
    dept_id BIGINT,
    create_by VARCHAR(64),
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_by VARCHAR(64),
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted INTEGER DEFAULT 0
);

COMMENT ON TABLE mxx_crm_contract_payment_plan IS '合同回款计划表';
COMMENT ON COLUMN mxx_crm_contract_payment_plan.payment_type IS '款项类型:1=预付款 2=进度款 3=到货款 4=验收款 5=质保金 6=尾款';
COMMENT ON COLUMN mxx_crm_contract_payment_plan.status IS '状态:1=未到期 2=待回款 3=部分回款 4=已回款 5=逾期';

CREATE INDEX IF NOT EXISTS idx_payment_plan_contract ON mxx_crm_contract_payment_plan(contract_id);
CREATE INDEX IF NOT EXISTS idx_payment_plan_status ON mxx_crm_contract_payment_plan(status);
CREATE INDEX IF NOT EXISTS idx_payment_plan_deleted ON mxx_crm_contract_payment_plan(deleted);

-- 2. 改造回款记录表 mxx_sale_payment
-- 2.1 添加新字段（幂等）
ALTER TABLE mxx_sale_payment ADD COLUMN IF NOT EXISTS plan_id BIGINT;
ALTER TABLE mxx_sale_payment ADD COLUMN IF NOT EXISTS customer_name VARCHAR(128);
ALTER TABLE mxx_sale_payment ADD COLUMN IF NOT EXISTS payer VARCHAR(128);
ALTER TABLE mxx_sale_payment ADD COLUMN IF NOT EXISTS payer_account VARCHAR(128);
ALTER TABLE mxx_sale_payment ADD COLUMN IF NOT EXISTS bank_flow_no VARCHAR(128);
ALTER TABLE mxx_sale_payment ADD COLUMN IF NOT EXISTS attachment VARCHAR(500);
ALTER TABLE mxx_sale_payment ADD COLUMN IF NOT EXISTS owner_user_id BIGINT;
ALTER TABLE mxx_sale_payment ADD COLUMN IF NOT EXISTS dept_id BIGINT;
ALTER TABLE mxx_sale_payment ADD COLUMN IF NOT EXISTS create_by VARCHAR(64);
ALTER TABLE mxx_sale_payment ADD COLUMN IF NOT EXISTS update_by VARCHAR(64);
ALTER TABLE mxx_sale_payment ADD COLUMN IF NOT EXISTS confirm_time TIMESTAMP;
ALTER TABLE mxx_sale_payment ADD COLUMN IF NOT EXISTS confirm_by VARCHAR(64);

-- 2.2 将payment_method枚举类型转换为integer
DO $$
BEGIN
    -- 创建integer类型的新列
    ALTER TABLE mxx_sale_payment ADD COLUMN IF NOT EXISTS payment_method_int INTEGER;

    -- 迁移数据（枚举名称对应数字值，按mxx_payment_method枚举顺序：alipay=1,wechat=2,bank=3,cash=4,other=5）
    -- 先检查原列是否是USER-DEFINED类型
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name='mxx_sale_payment' AND column_name='payment_method' AND data_type='USER-DEFINED'
    ) THEN
        UPDATE mxx_sale_payment SET payment_method_int =
            CASE payment_method::text
                WHEN 'alipay' THEN 2
                WHEN 'wechat' THEN 3
                WHEN 'bank' THEN 1
                WHEN 'cash' THEN 4
                ELSE 6
            END
        WHERE payment_method IS NOT NULL;

        -- 删除旧列
        ALTER TABLE mxx_sale_payment DROP COLUMN payment_method;
    ELSIF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name='mxx_sale_payment' AND column_name='payment_method' AND data_type='integer'
    ) THEN
        -- 已经是integer，直接使用
        UPDATE mxx_sale_payment SET payment_method_int = payment_method WHERE payment_method_int IS NULL;
        ALTER TABLE mxx_sale_payment DROP COLUMN payment_method;
    ELSE
        -- 既不是枚举也不是int，可能已经迁移过
    END IF;
END $$;

-- 如果存在临时列就重命名
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_payment' AND column_name='payment_method_int') THEN
        ALTER TABLE mxx_sale_payment RENAME COLUMN payment_method_int TO payment_method;
    END IF;
END $$;

-- 设置payment_method默认值
ALTER TABLE mxx_sale_payment ALTER COLUMN payment_method SET DEFAULT 1;

-- 2.3 将status从varchar转换为integer
DO $$
BEGIN
    -- 创建integer类型的新列
    ALTER TABLE mxx_sale_payment ADD COLUMN IF NOT EXISTS status_int INTEGER;

    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name='mxx_sale_payment' AND column_name='status' AND data_type='character varying'
    ) THEN
        -- 迁移数据：1=待确认 2=已确认 3=已驳回
        UPDATE mxx_sale_payment SET status_int =
            CASE status
                WHEN 'pending' THEN 1
                WHEN 'confirmed' THEN 2
                WHEN 'rejected' THEN 3
                WHEN 'cancelled' THEN 4
                ELSE 1
            END
        WHERE status IS NOT NULL AND status != '';

        -- 默认未确认的设为1
        UPDATE mxx_sale_payment SET status_int = 1 WHERE status_int IS NULL;

        -- 删除旧列
        ALTER TABLE mxx_sale_payment DROP COLUMN status;
    ELSIF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name='mxx_sale_payment' AND column_name='status' AND data_type='integer'
    ) THEN
        UPDATE mxx_sale_payment SET status_int = status WHERE status_int IS NULL;
        ALTER TABLE mxx_sale_payment DROP COLUMN status;
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_payment' AND column_name='status_int') THEN
        ALTER TABLE mxx_sale_payment RENAME COLUMN status_int TO status;
    END IF;
END $$;

ALTER TABLE mxx_sale_payment ALTER COLUMN status SET DEFAULT 1;

-- 2.4 将currency默认值从2改为1(CNY)
ALTER TABLE mxx_sale_payment ALTER COLUMN currency SET DEFAULT 1;

-- 2.5 迁移created_by(bigint)到create_by(varchar)，updated_by(bigint)到update_by(varchar)
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_payment' AND column_name='created_by') THEN
        UPDATE mxx_sale_payment SET create_by = created_by::VARCHAR
        WHERE created_by IS NOT NULL AND create_by IS NULL;
        ALTER TABLE mxx_sale_payment DROP COLUMN created_by;
    END IF;

    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_payment' AND column_name='updated_by') THEN
        UPDATE mxx_sale_payment SET update_by = updated_by::VARCHAR
        WHERE updated_by IS NOT NULL AND update_by IS NULL;
        ALTER TABLE mxx_sale_payment DROP COLUMN updated_by;
    END IF;
END $$;

-- 2.6 备份旧字段数据到bank_flow_no后删除冗余字段（transaction_no重命名为bank_flow_no更直观）
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_payment' AND column_name='transaction_no') THEN
        UPDATE mxx_sale_payment SET bank_flow_no = COALESCE(bank_flow_no, transaction_no)
        WHERE transaction_no IS NOT NULL AND bank_flow_no IS NULL;
        ALTER TABLE mxx_sale_payment DROP COLUMN transaction_no;
    END IF;

    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_payment' AND column_name='notes') THEN
        UPDATE mxx_sale_payment SET remark = COALESCE(remark, notes)
        WHERE notes IS NOT NULL AND remark IS NULL;
        ALTER TABLE mxx_sale_payment DROP COLUMN notes;
    END IF;

    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_payment' AND column_name='bank_account') THEN
        UPDATE mxx_sale_payment SET payer_account = COALESCE(payer_account, bank_account)
        WHERE bank_account IS NOT NULL AND payer_account IS NULL;
        ALTER TABLE mxx_sale_payment DROP COLUMN bank_account;
    END IF;

    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_payment' AND column_name='transaction_id') THEN
        ALTER TABLE mxx_sale_payment DROP COLUMN transaction_id;
    END IF;
END $$;

-- 2.7 添加索引
CREATE INDEX IF NOT EXISTS idx_sale_payment_contract ON mxx_sale_payment(contract_id);
CREATE INDEX IF NOT EXISTS idx_sale_payment_order ON mxx_sale_payment(order_id);
CREATE INDEX IF NOT EXISTS idx_sale_payment_plan ON mxx_sale_payment(plan_id);
CREATE INDEX IF NOT EXISTS idx_sale_payment_customer ON mxx_sale_payment(customer_id);
CREATE INDEX IF NOT EXISTS idx_sale_payment_status ON mxx_sale_payment(status);
CREATE INDEX IF NOT EXISTS idx_sale_payment_date ON mxx_sale_payment(payment_date);
CREATE INDEX IF NOT EXISTS idx_sale_payment_deleted ON mxx_sale_payment(deleted);

-- 3. 更新超级管理员对回款计划相关按钮权限（后续通过菜单系统配置）
-- 4. 删除孤立枚举类型（如果其他表不再使用）
DO $$
BEGIN
    -- 检查是否还有其他表使用 mxx_payment_method 枚举
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns c
        JOIN information_schema.tables t ON c.table_name = t.table_name
        WHERE c.udt_name = 'mxx_payment_method' AND c.table_name != 'mxx_sale_payment'
    ) THEN
        -- 确保已没有列使用该类型后删除
        IF NOT EXISTS (
            SELECT 1 FROM information_schema.columns WHERE udt_name = 'mxx_payment_method'
        ) THEN
            DROP TYPE IF EXISTS mxx_payment_method;
        END IF;
    END IF;
EXCEPTION WHEN OTHERS THEN
    RAISE NOTICE 'Could not drop enum mxx_payment_method: %', SQLERRM;
END $$;

COMMENT ON TABLE mxx_sale_payment IS '销售回款记录表';
COMMENT ON COLUMN mxx_sale_payment.plan_id IS '关联回款计划期次ID';
COMMENT ON COLUMN mxx_sale_payment.payment_method IS '支付方式:1=银行转账 2=支付宝 3=微信支付 4=现金 5=支票 6=其他';
COMMENT ON COLUMN mxx_sale_payment.status IS '状态:1=待确认 2=已确认 3=已驳回 4=已取消';
COMMENT ON COLUMN mxx_sale_payment.bank_flow_no IS '银行流水号';
