-- 销售订单模块数据库表结构
-- 版本: v2.0
-- 日期: 2026-04-28
-- 说明: 按专业CRM标准重新设计销售订单模块，包含订单主表和订单明细表

-- ==================== 订单主表 ====================
CREATE TABLE IF NOT EXISTS mxx_sale_order (
    id BIGSERIAL PRIMARY KEY,
    order_no VARCHAR(32) NOT NULL DEFAULT '',
    title VARCHAR(255) NOT NULL DEFAULT '',
    order_type INTEGER NOT NULL DEFAULT 1,
    order_status INTEGER NOT NULL DEFAULT 1,
    customer_id BIGINT NOT NULL DEFAULT 0,
    customer_name VARCHAR(100) NOT NULL DEFAULT '',
    contact_id BIGINT NOT NULL DEFAULT 0,
    contact_name VARCHAR(50) NOT NULL DEFAULT '',
    opportunity_id BIGINT NOT NULL DEFAULT 0,
    quotation_id BIGINT NOT NULL DEFAULT 0,
    contract_id BIGINT NOT NULL DEFAULT 0,
    order_date DATE,
    delivery_date DATE,
    currency INTEGER NOT NULL DEFAULT 1,
    exchange_rate DECIMAL(12,6) NOT NULL DEFAULT 1.000000,
    product_amount DECIMAL(14,2) NOT NULL DEFAULT 0.00,
    discount_amount DECIMAL(14,2) NOT NULL DEFAULT 0.00,
    shipping_fee DECIMAL(14,2) NOT NULL DEFAULT 0.00,
    tax_amount DECIMAL(14,2) NOT NULL DEFAULT 0.00,
    other_fee DECIMAL(14,2) NOT NULL DEFAULT 0.00,
    total_amount DECIMAL(14,2) NOT NULL DEFAULT 0.00,
    paid_amount DECIMAL(14,2) NOT NULL DEFAULT 0.00,
    unpaid_amount DECIMAL(14,2) NOT NULL DEFAULT 0.00,
    payment_status INTEGER NOT NULL DEFAULT 1,
    payment_method INTEGER NOT NULL DEFAULT 0,
    payment_due_date DATE,
    shipping_method INTEGER NOT NULL DEFAULT 0,
    tracking_no VARCHAR(100) NOT NULL DEFAULT '',
    shipping_time TIMESTAMP,
    complete_time TIMESTAMP,
    receiver_name VARCHAR(50) NOT NULL DEFAULT '',
    receiver_phone VARCHAR(20) NOT NULL DEFAULT '',
    shipping_address VARCHAR(500) NOT NULL DEFAULT '',
    billing_address VARCHAR(500) NOT NULL DEFAULT '',
    remark TEXT,
    owner_user_id BIGINT NOT NULL DEFAULT 0,
    dept_id BIGINT NOT NULL DEFAULT 0,
    create_by VARCHAR(64) NOT NULL DEFAULT '',
    create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_by VARCHAR(64) NOT NULL DEFAULT '',
    update_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted INTEGER NOT NULL DEFAULT 0
);

COMMENT ON TABLE mxx_sale_order IS '销售订单主表';
COMMENT ON COLUMN mxx_sale_order.id IS '主键ID';
COMMENT ON COLUMN mxx_sale_order.order_no IS '订单编号 SO+yyyyMMdd+4位流水号';
COMMENT ON COLUMN mxx_sale_order.title IS '订单标题';
COMMENT ON COLUMN mxx_sale_order.order_type IS '订单类型 1=销售订单 2=退货订单';
COMMENT ON COLUMN mxx_sale_order.order_status IS '订单状态 1=草稿 2=已确认 3=待发货 4=部分发货 5=已发货 6=已完成 7=已取消 8=已关闭';
COMMENT ON COLUMN mxx_sale_order.customer_id IS '客户ID';
COMMENT ON COLUMN mxx_sale_order.customer_name IS '客户名称(冗余)';
COMMENT ON COLUMN mxx_sale_order.contact_id IS '联系人ID';
COMMENT ON COLUMN mxx_sale_order.contact_name IS '联系人姓名(冗余)';
COMMENT ON COLUMN mxx_sale_order.opportunity_id IS '商机ID';
COMMENT ON COLUMN mxx_sale_order.quotation_id IS '报价单ID';
COMMENT ON COLUMN mxx_sale_order.contract_id IS '合同ID';
COMMENT ON COLUMN mxx_sale_order.order_date IS '下单日期';
COMMENT ON COLUMN mxx_sale_order.delivery_date IS '预计交付日期';
COMMENT ON COLUMN mxx_sale_order.currency IS '币种 1=CNY 2=USD 3=EUR 4=GBP 5=JPY 6=HKD';
COMMENT ON COLUMN mxx_sale_order.exchange_rate IS '汇率';
COMMENT ON COLUMN mxx_sale_order.product_amount IS '商品金额合计';
COMMENT ON COLUMN mxx_sale_order.discount_amount IS '整单折扣金额';
COMMENT ON COLUMN mxx_sale_order.shipping_fee IS '运费';
COMMENT ON COLUMN mxx_sale_order.tax_amount IS '税额';
COMMENT ON COLUMN mxx_sale_order.other_fee IS '其他费用';
COMMENT ON COLUMN mxx_sale_order.total_amount IS '订单总金额';
COMMENT ON COLUMN mxx_sale_order.paid_amount IS '已付金额';
COMMENT ON COLUMN mxx_sale_order.unpaid_amount IS '未付金额';
COMMENT ON COLUMN mxx_sale_order.payment_status IS '支付状态 1=未支付 2=部分支付 3=已支付 4=已退款';
COMMENT ON COLUMN mxx_sale_order.payment_method IS '支付方式 1=银行转账 2=支付宝 3=微信支付 4=现金 5=支票 6=其他';
COMMENT ON COLUMN mxx_sale_order.payment_due_date IS '付款截止日期';
COMMENT ON COLUMN mxx_sale_order.shipping_method IS '配送方式 1=快递 2=物流 3=自提 4=送货上门';
COMMENT ON COLUMN mxx_sale_order.tracking_no IS '物流单号';
COMMENT ON COLUMN mxx_sale_order.shipping_time IS '发货时间';
COMMENT ON COLUMN mxx_sale_order.complete_time IS '完成时间';
COMMENT ON COLUMN mxx_sale_order.receiver_name IS '收货人';
COMMENT ON COLUMN mxx_sale_order.receiver_phone IS '收货人电话';
COMMENT ON COLUMN mxx_sale_order.shipping_address IS '收货地址';
COMMENT ON COLUMN mxx_sale_order.billing_address IS '账单地址';
COMMENT ON COLUMN mxx_sale_order.remark IS '备注';
COMMENT ON COLUMN mxx_sale_order.owner_user_id IS '负责人ID';
COMMENT ON COLUMN mxx_sale_order.dept_id IS '所属部门ID';
COMMENT ON COLUMN mxx_sale_order.create_by IS '创建人';
COMMENT ON COLUMN mxx_sale_order.create_time IS '创建时间';
COMMENT ON COLUMN mxx_sale_order.update_by IS '更新人';
COMMENT ON COLUMN mxx_sale_order.update_time IS '更新时间';
COMMENT ON COLUMN mxx_sale_order.deleted IS '删除标志 0=未删除 1=已删除';

CREATE INDEX IF NOT EXISTS idx_sale_order_order_no ON mxx_sale_order(order_no);
CREATE INDEX IF NOT EXISTS idx_sale_order_customer_id ON mxx_sale_order(customer_id);
CREATE INDEX IF NOT EXISTS idx_sale_order_owner ON mxx_sale_order(owner_user_id);
CREATE INDEX IF NOT EXISTS idx_sale_order_status ON mxx_sale_order(order_status, payment_status);
CREATE INDEX IF NOT EXISTS idx_sale_order_date ON mxx_sale_order(order_date);
CREATE INDEX IF NOT EXISTS idx_sale_order_deleted ON mxx_sale_order(deleted);

-- ==================== 订单明细表 ====================
CREATE TABLE IF NOT EXISTS mxx_sale_order_item (
    id BIGSERIAL PRIMARY KEY,
    order_id BIGINT NOT NULL DEFAULT 0,
    product_id BIGINT NOT NULL DEFAULT 0,
    product_name VARCHAR(200) NOT NULL DEFAULT '',
    product_code VARCHAR(100) NOT NULL DEFAULT '',
    sku VARCHAR(100) NOT NULL DEFAULT '',
    spec VARCHAR(200) NOT NULL DEFAULT '',
    unit VARCHAR(20) NOT NULL DEFAULT '',
    quantity INTEGER NOT NULL DEFAULT 1,
    unit_price DECIMAL(14,2) NOT NULL DEFAULT 0.00,
    discount_rate DECIMAL(5,2) NOT NULL DEFAULT 100.00,
    discount_amount DECIMAL(14,2) NOT NULL DEFAULT 0.00,
    tax_rate DECIMAL(5,2) NOT NULL DEFAULT 0.00,
    tax_amount DECIMAL(14,2) NOT NULL DEFAULT 0.00,
    amount DECIMAL(14,2) NOT NULL DEFAULT 0.00,
    remark VARCHAR(500) NOT NULL DEFAULT '',
    sort INTEGER NOT NULL DEFAULT 0,
    create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted INTEGER NOT NULL DEFAULT 0
);

COMMENT ON TABLE mxx_sale_order_item IS '销售订单明细表';
COMMENT ON COLUMN mxx_sale_order_item.id IS '主键ID';
COMMENT ON COLUMN mxx_sale_order_item.order_id IS '订单ID';
COMMENT ON COLUMN mxx_sale_order_item.product_id IS '产品ID';
COMMENT ON COLUMN mxx_sale_order_item.product_name IS '产品名称';
COMMENT ON COLUMN mxx_sale_order_item.product_code IS '产品编码';
COMMENT ON COLUMN mxx_sale_order_item.sku IS 'SKU';
COMMENT ON COLUMN mxx_sale_order_item.spec IS '规格型号';
COMMENT ON COLUMN mxx_sale_order_item.unit IS '单位';
COMMENT ON COLUMN mxx_sale_order_item.quantity IS '数量';
COMMENT ON COLUMN mxx_sale_order_item.unit_price IS '单价';
COMMENT ON COLUMN mxx_sale_order_item.discount_rate IS '折扣率(%) 100=无折扣';
COMMENT ON COLUMN mxx_sale_order_item.discount_amount IS '折扣金额';
COMMENT ON COLUMN mxx_sale_order_item.tax_rate IS '税率(%)';
COMMENT ON COLUMN mxx_sale_order_item.tax_amount IS '税额';
COMMENT ON COLUMN mxx_sale_order_item.amount IS '行金额(含折扣和税)';
COMMENT ON COLUMN mxx_sale_order_item.remark IS '备注';
COMMENT ON COLUMN mxx_sale_order_item.sort IS '排序';
COMMENT ON COLUMN mxx_sale_order_item.create_time IS '创建时间';
COMMENT ON COLUMN mxx_sale_order_item.update_time IS '更新时间';
COMMENT ON COLUMN mxx_sale_order_item.deleted IS '删除标志 0=未删除 1=已删除';

CREATE INDEX IF NOT EXISTS idx_sale_order_item_order_id ON mxx_sale_order_item(order_id);
CREATE INDEX IF NOT EXISTS idx_sale_order_item_product ON mxx_sale_order_item(product_id);
CREATE INDEX IF NOT EXISTS idx_sale_order_item_deleted ON mxx_sale_order_item(deleted);

-- ==================== 如果表已存在，进行字段补充（兼容旧表） ====================
-- 给已有表添加可能缺失的字段（幂等操作）

-- mxx_sale_order 补充字段
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='order_type') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN order_type INTEGER NOT NULL DEFAULT 1;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='title') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN title VARCHAR(255) NOT NULL DEFAULT '';
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='contact_id') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN contact_id BIGINT NOT NULL DEFAULT 0;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='contact_name') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN contact_name VARCHAR(50) NOT NULL DEFAULT '';
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='opportunity_id') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN opportunity_id BIGINT NOT NULL DEFAULT 0;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='quotation_id') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN quotation_id BIGINT NOT NULL DEFAULT 0;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='contract_id') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN contract_id BIGINT NOT NULL DEFAULT 0;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='order_date') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN order_date DATE;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='delivery_date') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN delivery_date DATE;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='currency') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN currency INTEGER NOT NULL DEFAULT 1;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='exchange_rate') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN exchange_rate DECIMAL(12,6) NOT NULL DEFAULT 1.000000;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='product_amount') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN product_amount DECIMAL(14,2) NOT NULL DEFAULT 0.00;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='discount_amount') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN discount_amount DECIMAL(14,2) NOT NULL DEFAULT 0.00;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='shipping_fee') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN shipping_fee DECIMAL(14,2) NOT NULL DEFAULT 0.00;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='tax_amount') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN tax_amount DECIMAL(14,2) NOT NULL DEFAULT 0.00;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='other_fee') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN other_fee DECIMAL(14,2) NOT NULL DEFAULT 0.00;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='total_amount') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN total_amount DECIMAL(14,2) NOT NULL DEFAULT 0.00;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='paid_amount') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN paid_amount DECIMAL(14,2) NOT NULL DEFAULT 0.00;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='unpaid_amount') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN unpaid_amount DECIMAL(14,2) NOT NULL DEFAULT 0.00;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='payment_status') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN payment_status INTEGER NOT NULL DEFAULT 1;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='payment_method') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN payment_method INTEGER NOT NULL DEFAULT 0;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='payment_due_date') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN payment_due_date DATE;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='shipping_method') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN shipping_method INTEGER NOT NULL DEFAULT 0;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='tracking_no') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN tracking_no VARCHAR(100) NOT NULL DEFAULT '';
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='shipping_time') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN shipping_time TIMESTAMP;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='complete_time') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN complete_time TIMESTAMP;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='receiver_name') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN receiver_name VARCHAR(50) NOT NULL DEFAULT '';
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='receiver_phone') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN receiver_phone VARCHAR(20) NOT NULL DEFAULT '';
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='shipping_address') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN shipping_address VARCHAR(500) NOT NULL DEFAULT '';
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='billing_address') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN billing_address VARCHAR(500) NOT NULL DEFAULT '';
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='dept_id') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN dept_id BIGINT NOT NULL DEFAULT 0;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order' AND column_name='deleted') THEN
        ALTER TABLE mxx_sale_order ADD COLUMN deleted INTEGER NOT NULL DEFAULT 0;
    END IF;
END $$;

-- mxx_sale_order_item 补充字段（如果明细表不存在则创建，存在则补字段）
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name='mxx_sale_order_item') THEN
        CREATE TABLE mxx_sale_order_item (
            id BIGSERIAL PRIMARY KEY,
            order_id BIGINT NOT NULL DEFAULT 0,
            product_id BIGINT NOT NULL DEFAULT 0,
            product_name VARCHAR(200) NOT NULL DEFAULT '',
            product_code VARCHAR(100) NOT NULL DEFAULT '',
            sku VARCHAR(100) NOT NULL DEFAULT '',
            spec VARCHAR(200) NOT NULL DEFAULT '',
            unit VARCHAR(20) NOT NULL DEFAULT '',
            quantity INTEGER NOT NULL DEFAULT 1,
            unit_price DECIMAL(14,2) NOT NULL DEFAULT 0.00,
            discount_rate DECIMAL(5,2) NOT NULL DEFAULT 100.00,
            discount_amount DECIMAL(14,2) NOT NULL DEFAULT 0.00,
            tax_rate DECIMAL(5,2) NOT NULL DEFAULT 0.00,
            tax_amount DECIMAL(14,2) NOT NULL DEFAULT 0.00,
            amount DECIMAL(14,2) NOT NULL DEFAULT 0.00,
            remark VARCHAR(500) NOT NULL DEFAULT '',
            sort INTEGER NOT NULL DEFAULT 0,
            create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            update_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            deleted INTEGER NOT NULL DEFAULT 0
        );
    ELSE
        IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order_item' AND column_name='product_code') THEN
            ALTER TABLE mxx_sale_order_item ADD COLUMN product_code VARCHAR(100) NOT NULL DEFAULT '';
        END IF;
        IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order_item' AND column_name='sku') THEN
            ALTER TABLE mxx_sale_order_item ADD COLUMN sku VARCHAR(100) NOT NULL DEFAULT '';
        END IF;
        IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order_item' AND column_name='discount_rate') THEN
            ALTER TABLE mxx_sale_order_item ADD COLUMN discount_rate DECIMAL(5,2) NOT NULL DEFAULT 100.00;
        END IF;
        IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order_item' AND column_name='discount_amount') THEN
            ALTER TABLE mxx_sale_order_item ADD COLUMN discount_amount DECIMAL(14,2) NOT NULL DEFAULT 0.00;
        END IF;
        IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order_item' AND column_name='tax_rate') THEN
            ALTER TABLE mxx_sale_order_item ADD COLUMN tax_rate DECIMAL(5,2) NOT NULL DEFAULT 0.00;
        END IF;
        IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order_item' AND column_name='tax_amount') THEN
            ALTER TABLE mxx_sale_order_item ADD COLUMN tax_amount DECIMAL(14,2) NOT NULL DEFAULT 0.00;
        END IF;
        IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='mxx_sale_order_item' AND column_name='sort') THEN
            ALTER TABLE mxx_sale_order_item ADD COLUMN sort INTEGER NOT NULL DEFAULT 0;
        END IF;
    END IF;
END $$;
