-- ============================================================
-- v14 支付记录/会员费用/退款记录/财务统计表迁移（PostgreSQL）
-- 修复 finance_tables.sql 使用 MySQL 语法导致表未创建的问题
-- 数据库：PostgreSQL
-- 执行方式：psql -h 127.0.0.1 -p 5432 -U postgres -d mxxcrm_data -f v14_payment_tables.sql
-- ============================================================

-- ============ 1. 支付记录表 ============
CREATE TABLE IF NOT EXISTS mxx_payment_record (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL DEFAULT 0,
    member_product_id BIGINT,
    order_id VARCHAR(64),
    payment_type INT DEFAULT 1,
    amount DECIMAL(10,2) NOT NULL DEFAULT 0.00,
    pay_method INT DEFAULT 1,
    status INT DEFAULT 0,
    transaction_id VARCHAR(128),
    pay_time TIMESTAMP,
    remark VARCHAR(255),
    created_by BIGINT,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_by BIGINT,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted INT DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_payment_record_user_id ON mxx_payment_record (user_id);
CREATE INDEX IF NOT EXISTS idx_payment_record_order_id ON mxx_payment_record (order_id);
CREATE INDEX IF NOT EXISTS idx_payment_record_status ON mxx_payment_record (status);
CREATE INDEX IF NOT EXISTS idx_payment_record_payment_type ON mxx_payment_record (payment_type);
CREATE INDEX IF NOT EXISTS idx_payment_record_create_time ON mxx_payment_record (create_time);

-- ============ 2. 会员费用表 ============
CREATE TABLE IF NOT EXISTS mxx_member_fee (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL DEFAULT 0,
    member_type INT DEFAULT 1,
    amount DECIMAL(10,2) NOT NULL DEFAULT 0.00,
    valid_start_time TIMESTAMP,
    valid_end_time TIMESTAMP,
    status INT DEFAULT 0,
    payment_record_id BIGINT,
    remark VARCHAR(255),
    created_by BIGINT,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_by BIGINT,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted INT DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_member_fee_user_id ON mxx_member_fee (user_id);
CREATE INDEX IF NOT EXISTS idx_member_fee_status ON mxx_member_fee (status);
CREATE INDEX IF NOT EXISTS idx_member_fee_member_type ON mxx_member_fee (member_type);
CREATE INDEX IF NOT EXISTS idx_member_fee_payment_record_id ON mxx_member_fee (payment_record_id);
CREATE INDEX IF NOT EXISTS idx_member_fee_create_time ON mxx_member_fee (create_time);

-- ============ 3. 退款记录表 ============
CREATE TABLE IF NOT EXISTS mxx_refund_record (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL DEFAULT 0,
    payment_record_id BIGINT NOT NULL DEFAULT 0,
    amount DECIMAL(10,2) NOT NULL DEFAULT 0.00,
    status INT DEFAULT 0,
    transaction_id VARCHAR(128),
    refund_time TIMESTAMP,
    reason VARCHAR(255),
    remark VARCHAR(255),
    created_by BIGINT,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_by BIGINT,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted INT DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_refund_record_user_id ON mxx_refund_record (user_id);
CREATE INDEX IF NOT EXISTS idx_refund_record_payment_record_id ON mxx_refund_record (payment_record_id);
CREATE INDEX IF NOT EXISTS idx_refund_record_status ON mxx_refund_record (status);
CREATE INDEX IF NOT EXISTS idx_refund_record_create_time ON mxx_refund_record (create_time);

-- ============ 4. 财务统计表 ============
CREATE TABLE IF NOT EXISTS mxx_finance_statistics (
    id BIGSERIAL PRIMARY KEY,
    stat_date TIMESTAMP,
    stat_type INT DEFAULT 1,
    total_income DECIMAL(15,2) NOT NULL DEFAULT 0.00,
    success_amount DECIMAL(15,2) NOT NULL DEFAULT 0.00,
    refund_amount DECIMAL(15,2) NOT NULL DEFAULT 0.00,
    member_fee_amount DECIMAL(15,2) NOT NULL DEFAULT 0.00,
    order_count BIGINT DEFAULT 0,
    success_count BIGINT DEFAULT 0,
    refund_count BIGINT DEFAULT 0,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX IF NOT EXISTS uk_finance_statistics_date_type ON mxx_finance_statistics (stat_date, stat_type);
CREATE INDEX IF NOT EXISTS idx_finance_statistics_stat_type ON mxx_finance_statistics (stat_type);
CREATE INDEX IF NOT EXISTS idx_finance_statistics_stat_date ON mxx_finance_statistics (stat_date);

-- ============ 验证 SQL ============
SELECT
    (SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'mxx_payment_record') AS payment_record_exists,
    (SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'mxx_member_fee') AS member_fee_exists,
    (SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'mxx_refund_record') AS refund_record_exists,
    (SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'mxx_finance_statistics') AS finance_statistics_exists;
