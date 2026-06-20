-- ================================================
-- 报价单表（mxx_sale_quotation）
-- ================================================
CREATE TABLE IF NOT EXISTS mxx_sale_quotation (
    id BIGSERIAL PRIMARY KEY,
    quotation_no VARCHAR(32) NOT NULL UNIQUE,
    customer_id BIGINT,
    contact_id BIGINT,
    opportunity_id BIGINT,
    title VARCHAR(255),
    items JSONB,
    total_amount NUMERIC(12,2),
    currency VARCHAR(8) DEFAULT 'CNY',
    tax_amount NUMERIC(12,2) DEFAULT 0,
    discount_amount NUMERIC(12,2) DEFAULT 0,
    grand_total NUMERIC(12,2) NOT NULL DEFAULT 0,
    valid_until DATE,
    status VARCHAR(32) DEFAULT 'draft',
    payment_terms VARCHAR(255),
    delivery_terms VARCHAR(255),
    delivery_date DATE,
    port_of_loading VARCHAR(128),
    port_of_destination VARCHAR(128),
    bank_info VARCHAR(512),
    notes TEXT,
    assigned_to BIGINT,
    created_by BIGINT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_by BIGINT,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted INT DEFAULT 0
);

COMMENT ON TABLE mxx_sale_quotation IS '报价单表';
COMMENT ON COLUMN mxx_sale_quotation.quotation_no IS '报价单号（QUO-20240615-001）';
COMMENT ON COLUMN mxx_sale_quotation.status IS '状态（draft/sent/accepted/rejected/expired/revised）';
COMMENT ON COLUMN mxx_sale_quotation.items IS '产品明细JSON';
COMMENT ON COLUMN mxx_sale_quotation.grand_total IS '最终金额（含税/含折扣）';

-- ================================================
-- 发票表（mxx_sale_invoice）
-- ================================================
CREATE TABLE IF NOT EXISTS mxx_sale_invoice (
    id BIGSERIAL PRIMARY KEY,
    invoice_no VARCHAR(32) NOT NULL UNIQUE,
    order_id BIGINT,
    customer_id BIGINT,
    invoice_type VARCHAR(16) NOT NULL DEFAULT 'PI',
    amount NUMERIC(12,2) NOT NULL DEFAULT 0,
    currency VARCHAR(8) DEFAULT 'CNY',
    status VARCHAR(32) DEFAULT 'draft',
    issued_at DATE,
    due_date DATE,
    notes TEXT,
    created_by BIGINT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_by BIGINT,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted INT DEFAULT 0
);

COMMENT ON TABLE mxx_sale_invoice IS '发票表';
COMMENT ON COLUMN mxx_sale_invoice.invoice_type IS '发票类型（PI/CI/VAT）';
COMMENT ON COLUMN mxx_sale_invoice.status IS '状态（draft/issued/paid/cancelled）';