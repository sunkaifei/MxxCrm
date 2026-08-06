-- 采购模块数据库表迁移
-- 2026-08-05

-- ============ 品牌表 ============
CREATE TABLE IF NOT EXISTS mxx_product_brand (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255),
    name_en VARCHAR(255),
    logo VARCHAR(500),
    description TEXT,
    country VARCHAR(128),
    website VARCHAR(255),
    status INT DEFAULT 1,
    sort_order INT DEFAULT 0,
    remark TEXT,
    deleted INT DEFAULT 0,
    created_by BIGINT,
    updated_by BIGINT,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_brand_name ON mxx_product_brand(name);
CREATE INDEX IF NOT EXISTS idx_brand_status ON mxx_product_brand(status) WHERE deleted = 0;

-- ============ 采购申请主表 ============
CREATE TABLE IF NOT EXISTS mxx_purchase_requisition (
    id BIGSERIAL PRIMARY KEY,
    pr_no VARCHAR(64) NOT NULL,
    pr_type VARCHAR(32) NOT NULL DEFAULT 'replenish',
    title VARCHAR(255),
    department_id BIGINT,
    requester_id BIGINT NOT NULL,
    expected_date DATE,
    urgency INT DEFAULT 0,
    total_amount DECIMAL(18,2) DEFAULT 0,
    currency VARCHAR(3) DEFAULT 'CNY',
    status INT DEFAULT 0,
    source_type VARCHAR(32),
    source_id BIGINT,
    source_no VARCHAR(64),
    reason TEXT,
    remark TEXT,
    deleted INT DEFAULT 0,
    created_by BIGINT,
    updated_by BIGINT,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_requisition_no ON mxx_purchase_requisition(pr_no);
CREATE INDEX IF NOT EXISTS idx_requisition_status ON mxx_purchase_requisition(status) WHERE deleted = 0;
CREATE INDEX IF NOT EXISTS idx_requisition_requester ON mxx_purchase_requisition(requester_id);
CREATE INDEX IF NOT EXISTS idx_requisition_type ON mxx_purchase_requisition(pr_type);

-- ============ 采购申请明细表 ============
CREATE TABLE IF NOT EXISTS mxx_purchase_requisition_item (
    id BIGSERIAL PRIMARY KEY,
    pr_id BIGINT NOT NULL,
    product_id BIGINT,
    product_name VARCHAR(255),
    product_sku VARCHAR(64),
    spec VARCHAR(255),
    unit VARCHAR(32),
    quantity DECIMAL(18,2) NOT NULL,
    estimated_price DECIMAL(18,2),
    estimated_amount DECIMAL(18,2),
    remark TEXT,
    deleted INT DEFAULT 0,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_requisition_item_pr ON mxx_purchase_requisition_item(pr_id);
CREATE INDEX IF NOT EXISTS idx_requisition_item_product ON mxx_purchase_requisition_item(product_id);

-- ============ 采购订单明细表 ============
CREATE TABLE IF NOT EXISTS mxx_purchase_po_item (
    id BIGSERIAL PRIMARY KEY,
    po_id BIGINT NOT NULL,
    pr_item_id BIGINT,
    product_id BIGINT,
    product_name VARCHAR(255),
    product_sku VARCHAR(64),
    spec VARCHAR(255),
    unit VARCHAR(32),
    quantity DECIMAL(18,2) NOT NULL,
    received_quantity DECIMAL(18,2) DEFAULT 0,
    unit_price DECIMAL(18,6),
    amount DECIMAL(18,2),
    tax_rate DECIMAL(5,2) DEFAULT 0,
    tax_amount DECIMAL(18,2) DEFAULT 0,
    expected_date DATE,
    remark TEXT,
    deleted INT DEFAULT 0,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_po_item_po ON mxx_purchase_po_item(po_id);
CREATE INDEX IF NOT EXISTS idx_po_item_pr ON mxx_purchase_po_item(pr_item_id);
CREATE INDEX IF NOT EXISTS idx_po_item_product ON mxx_purchase_po_item(product_id);

-- ============ 审批记录表 ============
CREATE TABLE IF NOT EXISTS mxx_purchase_approval_record (
    id BIGSERIAL PRIMARY KEY,
    biz_type VARCHAR(32) NOT NULL,
    biz_id BIGINT NOT NULL,
    approval_level INT NOT NULL DEFAULT 1,
    approver_id BIGINT NOT NULL,
    action VARCHAR(16) NOT NULL,
    comment TEXT,
    deleted INT DEFAULT 0,
    created_by BIGINT,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_approval_biz ON mxx_purchase_approval_record(biz_type, biz_id);
CREATE INDEX IF NOT EXISTS idx_approval_approver ON mxx_purchase_approval_record(approver_id);

-- ============ 收货单主表 ============
CREATE TABLE IF NOT EXISTS mxx_purchase_receipt (
    id BIGSERIAL PRIMARY KEY,
    receipt_no VARCHAR(64) NOT NULL,
    po_id BIGINT NOT NULL,
    po_no VARCHAR(64),
    supplier_id BIGINT,
    warehouse_id BIGINT,
    status INT DEFAULT 0,
    total_quantity DECIMAL(18,2) DEFAULT 0,
    remark TEXT,
    inbound_id BIGINT,
    deleted INT DEFAULT 0,
    created_by BIGINT,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_receipt_no ON mxx_purchase_receipt(receipt_no);
CREATE INDEX IF NOT EXISTS idx_receipt_po ON mxx_purchase_receipt(po_id);

-- ============ 收货单明细表 ============
CREATE TABLE IF NOT EXISTS mxx_purchase_receipt_item (
    id BIGSERIAL PRIMARY KEY,
    receipt_id BIGINT NOT NULL,
    po_item_id BIGINT,
    product_id BIGINT,
    quantity DECIMAL(18,2) NOT NULL,
    remark TEXT,
    deleted INT DEFAULT 0,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_receipt_item_receipt ON mxx_purchase_receipt_item(receipt_id);
CREATE INDEX IF NOT EXISTS idx_receipt_item_po_item ON mxx_purchase_receipt_item(po_item_id);

-- ============ 采购退货主表 ============
CREATE TABLE IF NOT EXISTS mxx_purchase_return (
    id BIGSERIAL PRIMARY KEY,
    return_no VARCHAR(64),
    receipt_id BIGINT,
    po_id BIGINT,
    supplier_id BIGINT,
    return_date DATE,
    total_amount DECIMAL(18,2),
    reason TEXT,
    status INT DEFAULT 0,
    remark TEXT,
    deleted INT DEFAULT 0,
    created_by BIGINT,
    updated_by BIGINT,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_return_no ON mxx_purchase_return(return_no);
CREATE INDEX IF NOT EXISTS idx_return_po ON mxx_purchase_return(po_id);

-- ============ 采购退货明细表 ============
CREATE TABLE IF NOT EXISTS mxx_purchase_return_item (
    id BIGSERIAL PRIMARY KEY,
    return_id BIGINT,
    po_item_id BIGINT,
    product_id BIGINT,
    product_name VARCHAR(255),
    product_sku VARCHAR(64),
    unit VARCHAR(32),
    return_quantity DECIMAL(18,2),
    unit_price DECIMAL(18,6),
    amount DECIMAL(18,2),
    reason TEXT,
    deleted INT DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_return_item_return ON mxx_purchase_return_item(return_id);

-- ============ 备货计划主表 ============
CREATE TABLE IF NOT EXISTS mxx_purchase_stock_plan (
    id BIGSERIAL PRIMARY KEY,
    plan_no VARCHAR(64) NOT NULL,
    product_id BIGINT NOT NULL,
    plan_date DATE NOT NULL,
    demand_quantity DECIMAL(18,2) NOT NULL,
    demand_source VARCHAR(32) DEFAULT 'manual',
    available_quantity DECIMAL(18,2) DEFAULT 0,
    net_demand DECIMAL(18,2) DEFAULT 0,
    safety_stock DECIMAL(18,2) DEFAULT 0,
    suggested_order_date DATE,
    suggested_quantity DECIMAL(18,2),
    supplier_id BIGINT,
    lead_time_days INT DEFAULT 0,
    status INT DEFAULT 0,
    actual_pr_id BIGINT,
    remark TEXT,
    deleted INT DEFAULT 0,
    created_by BIGINT,
    updated_by BIGINT,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_stock_plan_no ON mxx_purchase_stock_plan(plan_no);
CREATE INDEX IF NOT EXISTS idx_stock_plan_product ON mxx_purchase_stock_plan(product_id);
CREATE INDEX IF NOT EXISTS idx_stock_plan_status ON mxx_purchase_stock_plan(status) WHERE deleted = 0;
CREATE INDEX IF NOT EXISTS idx_stock_plan_suggested_date ON mxx_purchase_stock_plan(suggested_order_date) WHERE deleted = 0 AND status = 0;

-- ============ 供应商-产品关联表 ============
CREATE TABLE IF NOT EXISTS mxx_purchase_supplier_product (
    id BIGSERIAL PRIMARY KEY,
    supplier_id BIGINT NOT NULL,
    product_id BIGINT NOT NULL,
    lead_time_days INT DEFAULT 0,
    moq DECIMAL(18,2) DEFAULT 0,
    supplier_sku VARCHAR(64),
    agreement_price DECIMAL(18,6),
    is_preferred INT DEFAULT 0,
    remark TEXT,
    deleted INT DEFAULT 0,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (supplier_id, product_id)
);
CREATE INDEX IF NOT EXISTS idx_supplier_product_supplier ON mxx_purchase_supplier_product(supplier_id);
CREATE INDEX IF NOT EXISTS idx_supplier_product_product ON mxx_purchase_supplier_product(product_id);
CREATE INDEX IF NOT EXISTS idx_supplier_product_preferred ON mxx_purchase_supplier_product(product_id, is_preferred) WHERE deleted = 0 AND is_preferred = 1;

-- ============ 供应商-品牌关联表 ============
CREATE TABLE IF NOT EXISTS mxx_purchase_supplier_brand (
    id BIGSERIAL PRIMARY KEY,
    supplier_id BIGINT NOT NULL,
    brand_id BIGINT NOT NULL,
    is_authorized INT DEFAULT 0,
    authorization_no VARCHAR(128),
    authorization_start DATE,
    authorization_end DATE,
    authorization_file VARCHAR(255),
    remark TEXT,
    deleted INT DEFAULT 0,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (supplier_id, brand_id)
);
CREATE INDEX IF NOT EXISTS idx_supplier_brand_supplier ON mxx_purchase_supplier_brand(supplier_id);
CREATE INDEX IF NOT EXISTS idx_supplier_brand_brand ON mxx_purchase_supplier_brand(brand_id);

-- ============ mxx_purchase_po 采购订单表新增字段 ============
ALTER TABLE mxx_purchase_po 
    ADD COLUMN IF NOT EXISTS pr_id BIGINT,
    ADD COLUMN IF NOT EXISTS pr_no VARCHAR(64),
    ADD COLUMN IF NOT EXISTS department_id BIGINT,
    ADD COLUMN IF NOT EXISTS buyer_id BIGINT,
    ADD COLUMN IF NOT EXISTS total_quantity DECIMAL(18,2) DEFAULT 0,
    ADD COLUMN IF NOT EXISTS tax_total DECIMAL(18,2) DEFAULT 0,
    ADD COLUMN IF NOT EXISTS discount_amount DECIMAL(18,2) DEFAULT 0,
    ADD COLUMN IF NOT EXISTS freight_amount DECIMAL(18,2) DEFAULT 0,
    ADD COLUMN IF NOT EXISTS delivery_address TEXT,
    ADD COLUMN IF NOT EXISTS delivery_terms VARCHAR(255),
    ADD COLUMN IF NOT EXISTS payment_terms VARCHAR(255),
    ADD COLUMN IF NOT EXISTS audit_by BIGINT,
    ADD COLUMN IF NOT EXISTS audit_time TIMESTAMP;

-- ============ mxx_product 增加品牌字段和库存策略字段 ============
ALTER TABLE mxx_product
    ADD COLUMN IF NOT EXISTS brand_id BIGINT,
    ADD COLUMN IF NOT EXISTS safety_stock DECIMAL(18,2) DEFAULT 0,
    ADD COLUMN IF NOT EXISTS max_stock DECIMAL(18,2) DEFAULT 0,
    ADD COLUMN IF NOT EXISTS stock_warning_days INT DEFAULT 7;

-- ============ mxx_product 增加生产管理字段 ============
ALTER TABLE mxx_product
    ADD COLUMN IF NOT EXISTS is_self_produced INT DEFAULT 0,
    ADD COLUMN IF NOT EXISTS production_lead_time INT DEFAULT 0,
    ADD COLUMN IF NOT EXISTS production_safety_stock DECIMAL(18,2) DEFAULT 0;

-- ============ mxx_purchase_stock_plan 补充 source_type/source_id ============
ALTER TABLE mxx_purchase_stock_plan
    ADD COLUMN IF NOT EXISTS source_type VARCHAR(32),
    ADD COLUMN IF NOT EXISTS source_id BIGINT;

CREATE INDEX IF NOT EXISTS idx_product_brand ON mxx_product(brand_id);