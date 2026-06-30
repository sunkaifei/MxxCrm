-- =====================================================
-- 报价单模块V3迁移：明细表 + 审批记录表 + 主表字段增强
-- 执行环境：PostgreSQL mxxcrm_data
-- =====================================================

-- ==========================================
-- 第一部分：报价单明细表
-- ==========================================
CREATE TABLE IF NOT EXISTS mxx_sale_quotation_item (
    id BIGSERIAL PRIMARY KEY,
    quotation_id BIGINT NOT NULL,
    product_id BIGINT,
    product_name VARCHAR(255),
    product_code VARCHAR(100),
    spec VARCHAR(255),
    unit VARCHAR(50),
    quantity DECIMAL(18,4) DEFAULT 1,
    unit_price DECIMAL(18,4) DEFAULT 0,
    discount_rate DECIMAL(5,2) DEFAULT 0,
    discount_amount DECIMAL(18,4) DEFAULT 0,
    tax_rate DECIMAL(5,2) DEFAULT 0,
    tax_amount DECIMAL(18,4) DEFAULT 0,
    subtotal DECIMAL(18,4) DEFAULT 0,
    sort INT DEFAULT 0,
    remark VARCHAR(500),
    create_by VARCHAR(64),
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_by VARCHAR(64),
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted INT DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_quotation_item_quotation_id ON mxx_sale_quotation_item(quotation_id);
CREATE INDEX IF NOT EXISTS idx_quotation_item_product_id ON mxx_sale_quotation_item(product_id);

COMMENT ON TABLE mxx_sale_quotation_item IS '报价单明细表';
COMMENT ON COLUMN mxx_sale_quotation_item.quotation_id IS '报价单ID';
COMMENT ON COLUMN mxx_sale_quotation_item.product_id IS '产品ID';
COMMENT ON COLUMN mxx_sale_quotation_item.product_name IS '产品名称';
COMMENT ON COLUMN mxx_sale_quotation_item.product_code IS '产品编码';
COMMENT ON COLUMN mxx_sale_quotation_item.spec IS '规格';
COMMENT ON COLUMN mxx_sale_quotation_item.unit IS '单位';
COMMENT ON COLUMN mxx_sale_quotation_item.quantity IS '数量';
COMMENT ON COLUMN mxx_sale_quotation_item.unit_price IS '单价';
COMMENT ON COLUMN mxx_sale_quotation_item.discount_rate IS '折扣率(%)';
COMMENT ON COLUMN mxx_sale_quotation_item.discount_amount IS '折扣金额';
COMMENT ON COLUMN mxx_sale_quotation_item.tax_rate IS '税率(%)';
COMMENT ON COLUMN mxx_sale_quotation_item.tax_amount IS '税额';
COMMENT ON COLUMN mxx_sale_quotation_item.subtotal IS '小计(含税)';
COMMENT ON COLUMN mxx_sale_quotation_item.sort IS '排序';

-- ==========================================
-- 第二部分：报价单审批记录表
-- ==========================================
CREATE TABLE IF NOT EXISTS mxx_sale_quotation_approval (
    id BIGSERIAL PRIMARY KEY,
    quotation_id BIGINT NOT NULL,
    version INT DEFAULT 1,
    approval_type INT DEFAULT 1,
    approval_status INT DEFAULT 1,
    approver_id BIGINT,
    approver_name VARCHAR(100),
    original_amount DECIMAL(18,4),
    adjusted_amount DECIMAL(18,4),
    approval_remark VARCHAR(1000),
    create_by VARCHAR(64),
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_quotation_approval_quotation_id ON mxx_sale_quotation_approval(quotation_id);

COMMENT ON TABLE mxx_sale_quotation_approval IS '报价单审批记录表';
COMMENT ON COLUMN mxx_sale_quotation_approval.quotation_id IS '报价单ID';
COMMENT ON COLUMN mxx_sale_quotation_approval.version IS '版本号';
COMMENT ON COLUMN mxx_sale_quotation_approval.approval_type IS '审批类型: 1提交审批 2审批通过 3审批驳回 4修改重报';
COMMENT ON COLUMN mxx_sale_quotation_approval.approval_status IS '审批状态: 1待审批 2通过 3驳回';
COMMENT ON COLUMN mxx_sale_quotation_approval.approver_id IS '审批人ID';
COMMENT ON COLUMN mxx_sale_quotation_approval.approver_name IS '审批人姓名';
COMMENT ON COLUMN mxx_sale_quotation_approval.original_amount IS '原报价金额';
COMMENT ON COLUMN mxx_sale_quotation_approval.adjusted_amount IS '调整后金额';
COMMENT ON COLUMN mxx_sale_quotation_approval.approval_remark IS '审批意见';

-- ==========================================
-- 第三部分：报价单主表添加字段（幂等）
-- ==========================================

-- 审批状态: 1未提交 2待审批 3已通过 4已驳回 5需修改
ALTER TABLE mxx_sale_quotation ADD COLUMN IF NOT EXISTS approval_status INT DEFAULT 1;
-- 当前版本号
ALTER TABLE mxx_sale_quotation ADD COLUMN IF NOT EXISTS current_version INT DEFAULT 1;
-- 客户名称（冗余）
ALTER TABLE mxx_sale_quotation ADD COLUMN IF NOT EXISTS customer_name VARCHAR(255);
-- 联系人名称（冗余）
ALTER TABLE mxx_sale_quotation ADD COLUMN IF NOT EXISTS contact_name VARCHAR(100);
-- 商机标题（冗余）
ALTER TABLE mxx_sale_quotation ADD COLUMN IF NOT EXISTS opportunity_title VARCHAR(255);
-- 报价日期
ALTER TABLE mxx_sale_quotation ADD COLUMN IF NOT EXISTS quotation_date DATE;

COMMENT ON COLUMN mxx_sale_quotation.approval_status IS '审批状态: 1未提交 2待审批 3已通过 4已驳回 5需修改';
COMMENT ON COLUMN mxx_sale_quotation.current_version IS '当前版本号';
COMMENT ON COLUMN mxx_sale_quotation.customer_name IS '客户名称';
COMMENT ON COLUMN mxx_sale_quotation.contact_name IS '联系人名称';
COMMENT ON COLUMN mxx_sale_quotation.opportunity_title IS '商机标题';
COMMENT ON COLUMN mxx_sale_quotation.quotation_date IS '报价日期';

-- 更新状态含义说明：
-- status: 1=草稿 2=待审批 3=已审批 4=已发送 5=已接受 6=已拒绝 7=已过期 8=已转订单

-- ==========================================
-- 验证
-- ==========================================
SELECT 'mxx_sale_quotation_item' as table_name, count(*) as col_count FROM information_schema.columns WHERE table_name = 'mxx_sale_quotation_item'
UNION ALL
SELECT 'mxx_sale_quotation_approval', count(*) FROM information_schema.columns WHERE table_name = 'mxx_sale_quotation_approval'
UNION ALL
SELECT 'mxx_sale_quotation(new cols)', count(*) FROM information_schema.columns WHERE table_name = 'mxx_sale_quotation' AND column_name IN ('approval_status','current_version','customer_name','contact_name','opportunity_title','quotation_date');
