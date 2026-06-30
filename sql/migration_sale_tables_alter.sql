-- 销售模块表结构增量迁移脚本 v3（修复版）
-- 日期：2026-06-28

-- ============================================
-- 第一部分：订单表（安全添加字段+状态转换）
-- ============================================

BEGIN;

-- 添加缺失字段
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS title VARCHAR(200);
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS customer_name VARCHAR(100);
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS contact_id BIGINT;
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS contact_name VARCHAR(50);
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS quotation_id BIGINT;
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS delivery_date DATE;
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS exchange_rate DECIMAL(12,6) DEFAULT 1.000000;
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS product_amount DECIMAL(18,2) DEFAULT 0.00;
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS discount_amount DECIMAL(18,2) DEFAULT 0.00;
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS shipping_fee DECIMAL(18,2) DEFAULT 0.00;
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS other_fee DECIMAL(18,2) DEFAULT 0.00;
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS unpaid_amount DECIMAL(18,2) DEFAULT 0.00;
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS shipping_time TIMESTAMP;
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS complete_time TIMESTAMP;
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS receiver_name VARCHAR(50);
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS receiver_phone VARCHAR(30);
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS owner_user_id BIGINT;
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS dept_id BIGINT;
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS create_by BIGINT;
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS update_by BIGINT;

-- 数据回填
UPDATE mxx_sale_order SET owner_user_id = assigned_to WHERE owner_user_id IS NULL AND assigned_to IS NOT NULL;
UPDATE mxx_sale_order SET create_by = created_by WHERE create_by IS NULL;
UPDATE mxx_sale_order SET update_by = updated_by WHERE update_by IS NULL;
UPDATE mxx_sale_order SET product_amount = amount WHERE product_amount = 0 AND amount IS NOT NULL AND amount > 0;
UPDATE mxx_sale_order SET unpaid_amount = COALESCE(total_amount,0) - COALESCE(paid_amount,0) WHERE unpaid_amount = 0 AND total_amount IS NOT NULL;

-- 状态字段转换：添加integer新列
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS status_new INTEGER DEFAULT 0;
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS pay_status_new INTEGER DEFAULT 0;
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS order_type_new INTEGER DEFAULT 0;
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS pay_method_new INTEGER DEFAULT 0;
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS ship_method_new INTEGER DEFAULT 0;

-- 数据映射
UPDATE mxx_sale_order SET status_new = CASE
    WHEN status = 'pending' THEN 0
    WHEN status = 'shipping' THEN 4
    WHEN status = 'delivered' THEN 5
    WHEN status = 'completed' THEN 6
    ELSE 0
END;
UPDATE mxx_sale_order SET pay_status_new = CASE
    WHEN payment_status = 'unpaid' THEN 0
    WHEN payment_status = 'partial' THEN 1
    WHEN payment_status = 'paid' THEN 2
    ELSE 0
END;

COMMIT;

-- 注意：不DROP TYPE（会影响其他表），直接删除列即可
BEGIN;
ALTER TABLE mxx_sale_order DROP COLUMN IF EXISTS status;
ALTER TABLE mxx_sale_order DROP COLUMN IF EXISTS payment_status;
ALTER TABLE mxx_sale_order DROP COLUMN IF EXISTS order_type;
ALTER TABLE mxx_sale_order DROP COLUMN IF EXISTS payment_method;
ALTER TABLE mxx_sale_order DROP COLUMN IF EXISTS shipping_method;
ALTER TABLE mxx_sale_order DROP COLUMN IF EXISTS notes;
ALTER TABLE mxx_sale_order DROP COLUMN IF EXISTS assigned_to;
ALTER TABLE mxx_sale_order DROP COLUMN IF EXISTS created_by;
ALTER TABLE mxx_sale_order DROP COLUMN IF EXISTS updated_by;
ALTER TABLE mxx_sale_order DROP COLUMN IF EXISTS amount;

ALTER TABLE mxx_sale_order RENAME COLUMN status_new TO status;
ALTER TABLE mxx_sale_order RENAME COLUMN pay_status_new TO payment_status;
ALTER TABLE mxx_sale_order RENAME COLUMN order_type_new TO order_type;
ALTER TABLE mxx_sale_order RENAME COLUMN pay_method_new TO payment_method;
ALTER TABLE mxx_sale_order RENAME COLUMN ship_method_new TO shipping_method;

-- 设置默认值
ALTER TABLE mxx_sale_order ALTER COLUMN status SET DEFAULT 0;
ALTER TABLE mxx_sale_order ALTER COLUMN status SET NOT NULL;
ALTER TABLE mxx_sale_order ALTER COLUMN payment_status SET DEFAULT 0;
ALTER TABLE mxx_sale_order ALTER COLUMN order_type SET DEFAULT 0;
ALTER TABLE mxx_sale_order ALTER COLUMN currency SET DEFAULT 1;
ALTER TABLE mxx_sale_order ALTER COLUMN payment_method SET DEFAULT 0;
ALTER TABLE mxx_sale_order ALTER COLUMN shipping_method SET DEFAULT 0;
ALTER TABLE mxx_sale_order ALTER COLUMN exchange_rate SET DEFAULT 1.000000;
ALTER TABLE mxx_sale_order ALTER COLUMN product_amount SET DEFAULT 0.00;
ALTER TABLE mxx_sale_order ALTER COLUMN discount_amount SET DEFAULT 0.00;
ALTER TABLE mxx_sale_order ALTER COLUMN shipping_fee SET DEFAULT 0.00;
ALTER TABLE mxx_sale_order ALTER COLUMN tax_amount SET DEFAULT 0.00;
ALTER TABLE mxx_sale_order ALTER COLUMN other_fee SET DEFAULT 0.00;
ALTER TABLE mxx_sale_order ALTER COLUMN total_amount SET DEFAULT 0.00;
ALTER TABLE mxx_sale_order ALTER COLUMN paid_amount SET DEFAULT 0.00;
ALTER TABLE mxx_sale_order ALTER COLUMN unpaid_amount SET DEFAULT 0.00;
ALTER TABLE mxx_sale_order ALTER COLUMN deleted SET DEFAULT 0;
ALTER TABLE mxx_sale_order ALTER COLUMN create_time SET DEFAULT CURRENT_TIMESTAMP;

COMMIT;

-- ============================================
-- 第二部分：订单明细表
-- ============================================
BEGIN;
ALTER TABLE mxx_sale_order_item ADD COLUMN IF NOT EXISTS create_by BIGINT;
ALTER TABLE mxx_sale_order_item ADD COLUMN IF NOT EXISTS update_by BIGINT;
ALTER TABLE mxx_sale_order_item ADD COLUMN IF NOT EXISTS unit_id BIGINT;
ALTER TABLE mxx_sale_order_item ADD COLUMN IF NOT EXISTS discount_rate DECIMAL(8,4) DEFAULT 0.0000;
ALTER TABLE mxx_sale_order_item ADD COLUMN IF NOT EXISTS discount_amount DECIMAL(18,2) DEFAULT 0.00;
ALTER TABLE mxx_sale_order_item ADD COLUMN IF NOT EXISTS delivery_date DATE;
ALTER TABLE mxx_sale_order_item ADD COLUMN IF NOT EXISTS delivered_quantity DECIMAL(18,4) DEFAULT 0;
ALTER TABLE mxx_sale_order_item ADD COLUMN IF NOT EXISTS sort INTEGER DEFAULT 0;

UPDATE mxx_sale_order_item SET create_by = created_by WHERE create_by IS NULL;
UPDATE mxx_sale_order_item SET update_by = updated_by WHERE update_by IS NULL;
COMMIT;

-- ============================================
-- 第三部分：发票表（空表，添加缺失字段+类型转换）
-- ============================================
BEGIN;
ALTER TABLE mxx_sale_invoice ADD COLUMN IF NOT EXISTS title VARCHAR(200);
ALTER TABLE mxx_sale_invoice ADD COLUMN IF NOT EXISTS contract_id BIGINT;
ALTER TABLE mxx_sale_invoice ADD COLUMN IF NOT EXISTS customer_name VARCHAR(100);
ALTER TABLE mxx_sale_invoice ADD COLUMN IF NOT EXISTS tax_no VARCHAR(50);
ALTER TABLE mxx_sale_invoice ADD COLUMN IF NOT EXISTS invoice_date DATE;
ALTER TABLE mxx_sale_invoice ADD COLUMN IF NOT EXISTS tax_rate DECIMAL(5,4) DEFAULT 0.0000;
ALTER TABLE mxx_sale_invoice ADD COLUMN IF NOT EXISTS tax_amount DECIMAL(18,2) DEFAULT 0.00;
ALTER TABLE mxx_sale_invoice ADD COLUMN IF NOT EXISTS buyer_name VARCHAR(100);
ALTER TABLE mxx_sale_invoice ADD COLUMN IF NOT EXISTS buyer_tax_no VARCHAR(50);
ALTER TABLE mxx_sale_invoice ADD COLUMN IF NOT EXISTS buyer_address VARCHAR(255);
ALTER TABLE mxx_sale_invoice ADD COLUMN IF NOT EXISTS buyer_bank VARCHAR(100);
ALTER TABLE mxx_sale_invoice ADD COLUMN IF NOT EXISTS owner_user_id BIGINT;
ALTER TABLE mxx_sale_invoice ADD COLUMN IF NOT EXISTS dept_id BIGINT;
ALTER TABLE mxx_sale_invoice ADD COLUMN IF NOT EXISTS create_by VARCHAR(64);
ALTER TABLE mxx_sale_invoice ADD COLUMN IF NOT EXISTS update_by VARCHAR(64);
ALTER TABLE mxx_sale_invoice ADD COLUMN IF NOT EXISTS remark TEXT;

-- 类型转换（空表）
ALTER TABLE mxx_sale_invoice ADD COLUMN IF NOT EXISTS invoice_type_int INTEGER DEFAULT 0;
ALTER TABLE mxx_sale_invoice ADD COLUMN IF NOT EXISTS currency_int INTEGER DEFAULT 1;
ALTER TABLE mxx_sale_invoice ADD COLUMN IF NOT EXISTS status_int INTEGER DEFAULT 0;

ALTER TABLE mxx_sale_invoice DROP COLUMN IF EXISTS invoice_type;
ALTER TABLE mxx_sale_invoice DROP COLUMN IF EXISTS currency;
ALTER TABLE mxx_sale_invoice DROP COLUMN IF EXISTS status;
ALTER TABLE mxx_sale_invoice DROP COLUMN IF EXISTS issued_at;
ALTER TABLE mxx_sale_invoice DROP COLUMN IF EXISTS notes;
ALTER TABLE mxx_sale_invoice DROP COLUMN IF EXISTS created_by;
ALTER TABLE mxx_sale_invoice DROP COLUMN IF EXISTS updated_by;

ALTER TABLE mxx_sale_invoice RENAME COLUMN invoice_type_int TO invoice_type;
ALTER TABLE mxx_sale_invoice RENAME COLUMN currency_int TO currency;
ALTER TABLE mxx_sale_invoice RENAME COLUMN status_int TO status;

-- 设置默认值
ALTER TABLE mxx_sale_invoice ALTER COLUMN invoice_type SET DEFAULT 0;
ALTER TABLE mxx_sale_invoice ALTER COLUMN currency SET DEFAULT 1;
ALTER TABLE mxx_sale_invoice ALTER COLUMN status SET DEFAULT 0;
ALTER TABLE mxx_sale_invoice ALTER COLUMN tax_rate SET DEFAULT 0.0000;
ALTER TABLE mxx_sale_invoice ALTER COLUMN amount SET DEFAULT 0.00;
ALTER TABLE mxx_sale_invoice ALTER COLUMN tax_amount SET DEFAULT 0.00;
ALTER TABLE mxx_sale_invoice ALTER COLUMN deleted SET DEFAULT 0;
ALTER TABLE mxx_sale_invoice ALTER COLUMN create_time SET DEFAULT CURRENT_TIMESTAMP;
COMMIT;

-- ============================================
-- 第四部分：报价单表（空表，添加缺失字段+类型转换）
-- ============================================
BEGIN;
ALTER TABLE mxx_sale_quotation ADD COLUMN IF NOT EXISTS owner_user_id BIGINT;
ALTER TABLE mxx_sale_quotation ADD COLUMN IF NOT EXISTS dept_id BIGINT;
ALTER TABLE mxx_sale_quotation ADD COLUMN IF NOT EXISTS quotation_date DATE;
ALTER TABLE mxx_sale_quotation ADD COLUMN IF NOT EXISTS create_by VARCHAR(64);
ALTER TABLE mxx_sale_quotation ADD COLUMN IF NOT EXISTS update_by VARCHAR(64);
ALTER TABLE mxx_sale_quotation ADD COLUMN IF NOT EXISTS remark TEXT;

-- 类型转换（空表）
ALTER TABLE mxx_sale_quotation ADD COLUMN IF NOT EXISTS status_int INTEGER DEFAULT 0;
ALTER TABLE mxx_sale_quotation ADD COLUMN IF NOT EXISTS currency_int INTEGER DEFAULT 1;

UPDATE mxx_sale_quotation SET owner_user_id = assigned_to WHERE owner_user_id IS NULL AND assigned_to IS NOT NULL;
UPDATE mxx_sale_quotation SET create_by = CAST(created_by AS VARCHAR) WHERE create_by IS NULL;
UPDATE mxx_sale_quotation SET update_by = CAST(updated_by AS VARCHAR) WHERE update_by IS NULL;
UPDATE mxx_sale_quotation SET remark = notes WHERE remark IS NULL;

ALTER TABLE mxx_sale_quotation DROP COLUMN IF EXISTS status;
ALTER TABLE mxx_sale_quotation DROP COLUMN IF EXISTS currency;
ALTER TABLE mxx_sale_quotation DROP COLUMN IF EXISTS created_by;
ALTER TABLE mxx_sale_quotation DROP COLUMN IF EXISTS updated_by;
ALTER TABLE mxx_sale_quotation DROP COLUMN IF EXISTS assigned_to;
ALTER TABLE mxx_sale_quotation DROP COLUMN IF EXISTS notes;

ALTER TABLE mxx_sale_quotation RENAME COLUMN status_int TO status;
ALTER TABLE mxx_sale_quotation RENAME COLUMN currency_int TO currency;

-- 设置默认值
ALTER TABLE mxx_sale_quotation ALTER COLUMN status SET DEFAULT 0;
ALTER TABLE mxx_sale_quotation ALTER COLUMN currency SET DEFAULT 1;
ALTER TABLE mxx_sale_quotation ALTER COLUMN tax_amount SET DEFAULT 0.00;
ALTER TABLE mxx_sale_quotation ALTER COLUMN discount_amount SET DEFAULT 0.00;
ALTER TABLE mxx_sale_quotation ALTER COLUMN deleted SET DEFAULT 0;
ALTER TABLE mxx_sale_quotation ALTER COLUMN create_time SET DEFAULT CURRENT_TIMESTAMP;
COMMIT;
