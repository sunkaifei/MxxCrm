-- ================================================
-- Mxx-CRM 数据库迁移脚本 (PostgreSQL)
-- 将现有表结构迁移到符合开发文档的标准
-- 1. 统一软删除字段为 deleted INT
-- 2. 修正字段长度和类型
-- 3. 创建缺失的业务模块表
-- 4. 添加字段注释
-- ================================================

-- ================================================
-- 1. 创建枚举类型（如果不存在）
-- ================================================

DO $$ BEGIN
    CREATE TYPE mxx_user_status AS ENUM ('active', 'disabled', 'pending');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE mxx_lead_status AS ENUM ('new', 'following', 'converted', 'invalid', 'recycled');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE mxx_lead_source AS ENUM ('website', 'exhibition', 'social', 'referral', 'cold_call', 'customs', 'email', 'alibaba', 'amazon', 'tiktok', 'wechat', 'other');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE mxx_customer_level AS ENUM ('potential', 'normal', 'vip', 'svip', 'lost');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE mxx_opportunity_stage AS ENUM ('qualification', 'needs_analysis', 'proposal', 'negotiation', 'won', 'lost');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE mxx_contract_status AS ENUM ('draft', 'signed', 'executing', 'completed', 'terminated');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE mxx_order_status AS ENUM ('pending', 'confirmed', 'paid', 'shipping', 'delivered', 'completed', 'cancelled', 'refunded');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE mxx_payment_status AS ENUM ('unpaid', 'partial', 'paid', 'overdue');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE mxx_payment_method AS ENUM ('cash', 'bank_transfer', 'credit_card', 'paypal', 'wechat_pay', 'alipay', 'tt', 'lc', 'other');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE mxx_purchase_status AS ENUM ('draft', 'ordered', 'in_transit', 'received', 'completed', 'cancelled');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE mxx_inventory_op_type AS ENUM ('in', 'out', 'transfer', 'adjust');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE mxx_currency_code AS ENUM ('CNY', 'USD', 'EUR', 'GBP', 'JPY', 'HKD', 'AUD');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE mxx_gender AS ENUM ('male', 'female', 'other');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE mxx_industry_type AS ENUM ('retail', 'wholesale', 'manufacturer', 'trade_agent', 'ecommerce', 'wechat_business', 'other');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE mxx_activity_type AS ENUM ('call', 'email', 'meeting', 'visit', 'whatsapp', 'wechat', 'other');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

-- ================================================
-- 2. 修改现有系统表结构 - 字段长度
-- ================================================

ALTER TABLE mxx_system_admin
    ALTER COLUMN user_name TYPE VARCHAR(64),
    ALTER COLUMN nick_name TYPE VARCHAR(64),
    ALTER COLUMN email TYPE VARCHAR(128),
    ALTER COLUMN mobile TYPE VARCHAR(32),
    ALTER COLUMN avatar TYPE VARCHAR(255),
    ALTER COLUMN password TYPE VARCHAR(255),
    ALTER COLUMN login_ip TYPE VARCHAR(64),
    ALTER COLUMN remark TYPE VARCHAR(255);

ALTER TABLE mxx_system_dept
    ALTER COLUMN ancestors TYPE VARCHAR(512),
    ALTER COLUMN dept_name TYPE VARCHAR(64),
    ALTER COLUMN code TYPE VARCHAR(64),
    ALTER COLUMN leader TYPE VARCHAR(64),
    ALTER COLUMN phone TYPE VARCHAR(32),
    ALTER COLUMN email TYPE VARCHAR(128);

ALTER TABLE mxx_system_role
    ALTER COLUMN role_name TYPE VARCHAR(64),
    ALTER COLUMN role_key TYPE VARCHAR(64),
    ALTER COLUMN remark TYPE VARCHAR(255);

ALTER TABLE mxx_system_menu
    ALTER COLUMN tree_path TYPE VARCHAR(512),
    ALTER COLUMN name TYPE VARCHAR(128),
    ALTER COLUMN type TYPE VARCHAR(16),
    ALTER COLUMN route_name TYPE VARCHAR(128),
    ALTER COLUMN path TYPE VARCHAR(255),
    ALTER COLUMN component TYPE VARCHAR(255),
    ALTER COLUMN perm TYPE VARCHAR(128),
    ALTER COLUMN icon TYPE VARCHAR(128),
    ALTER COLUMN redirect TYPE VARCHAR(255);

ALTER TABLE mxx_system_post
    ALTER COLUMN post_code TYPE VARCHAR(64),
    ALTER COLUMN post_name TYPE VARCHAR(64),
    ALTER COLUMN remark TYPE VARCHAR(255);

-- ================================================
-- 3. 创建缺失的业务模块表
-- ================================================

-- 3.1 mxx_crm_lead（线索表）
CREATE TABLE IF NOT EXISTS mxx_crm_lead (
    id                  BIGSERIAL PRIMARY KEY,
    company_name        VARCHAR(255),
    contact_name        VARCHAR(128),
    title               VARCHAR(128),
    email               VARCHAR(128),
    phone               VARCHAR(32),
    mobile              VARCHAR(32),
    country             VARCHAR(64),
    region              VARCHAR(128),
    address             VARCHAR(512),
    website             VARCHAR(255),
    industry            mxx_industry_type,
    source              mxx_lead_source,
    source_detail       VARCHAR(128),
    status              mxx_lead_status DEFAULT 'new',
    level               mxx_customer_level,
    tags                VARCHAR(255)[],
    budget              NUMERIC(15,2),
    currency            mxx_currency_code DEFAULT 'USD',
    next_follow_at      TIMESTAMPTZ,
    assigned_to         BIGINT REFERENCES mxx_system_admin(id),
    converted_to_customer_id BIGINT,
    converted_at        TIMESTAMPTZ,
    description         TEXT,
    custom_fields       JSONB,
    created_by          BIGINT REFERENCES mxx_system_admin(id),
    created_at          TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_by          BIGINT REFERENCES mxx_system_admin(id),
    updated_at          TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted             INT DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_crm_lead_assigned ON mxx_crm_lead(assigned_to);
CREATE INDEX IF NOT EXISTS idx_crm_lead_status ON mxx_crm_lead(status);
CREATE INDEX IF NOT EXISTS idx_crm_lead_source ON mxx_crm_lead(source);

COMMENT ON TABLE mxx_crm_lead IS '线索表';
COMMENT ON COLUMN mxx_crm_lead.id IS '线索ID';
COMMENT ON COLUMN mxx_crm_lead.company_name IS '公司名称';
COMMENT ON COLUMN mxx_crm_lead.contact_name IS '联系人姓名';
COMMENT ON COLUMN mxx_crm_lead.title IS '职位';
COMMENT ON COLUMN mxx_crm_lead.email IS '邮箱';
COMMENT ON COLUMN mxx_crm_lead.phone IS '电话';
COMMENT ON COLUMN mxx_crm_lead.mobile IS '手机';
COMMENT ON COLUMN mxx_crm_lead.country IS '国家';
COMMENT ON COLUMN mxx_crm_lead.region IS '地区';
COMMENT ON COLUMN mxx_crm_lead.address IS '地址';
COMMENT ON COLUMN mxx_crm_lead.website IS '网站';
COMMENT ON COLUMN mxx_crm_lead.industry IS '行业类型';
COMMENT ON COLUMN mxx_crm_lead.source IS '线索来源';
COMMENT ON COLUMN mxx_crm_lead.source_detail IS '来源详情';
COMMENT ON COLUMN mxx_crm_lead.status IS '线索状态';
COMMENT ON COLUMN mxx_crm_lead.level IS '客户等级';
COMMENT ON COLUMN mxx_crm_lead.tags IS '标签';
COMMENT ON COLUMN mxx_crm_lead.budget IS '预算金额';
COMMENT ON COLUMN mxx_crm_lead.currency IS '货币类型';
COMMENT ON COLUMN mxx_crm_lead.next_follow_at IS '下次跟进时间';
COMMENT ON COLUMN mxx_crm_lead.assigned_to IS '负责人ID';
COMMENT ON COLUMN mxx_crm_lead.converted_to_customer_id IS '转化为客户ID';
COMMENT ON COLUMN mxx_crm_lead.converted_at IS '转化时间';
COMMENT ON COLUMN mxx_crm_lead.description IS '描述';
COMMENT ON COLUMN mxx_crm_lead.custom_fields IS '自定义字段';
COMMENT ON COLUMN mxx_crm_lead.created_by IS '创建者ID';
COMMENT ON COLUMN mxx_crm_lead.created_at IS '创建时间';
COMMENT ON COLUMN mxx_crm_lead.updated_by IS '更新者ID';
COMMENT ON COLUMN mxx_crm_lead.updated_at IS '更新时间';
COMMENT ON COLUMN mxx_crm_lead.deleted IS '0未删除 1已删除';

-- 3.2 mxx_crm_customer（客户表）
CREATE TABLE IF NOT EXISTS mxx_crm_customer (
    id                  BIGSERIAL PRIMARY KEY,
    customer_no         VARCHAR(32) UNIQUE,
    company_name        VARCHAR(255) NOT NULL,
    short_name          VARCHAR(64),
    country             VARCHAR(64),
    region              VARCHAR(128),
    address             VARCHAR(512),
    website             VARCHAR(255),
    industry            mxx_industry_type,
    level               mxx_customer_level DEFAULT 'normal',
    total_deal_amount   NUMERIC(18,2) DEFAULT 0,
    total_deal_count    INT DEFAULT 0,
    last_deal_at        TIMESTAMPTZ,
    source              mxx_lead_source,
    tags                VARCHAR(255)[],
    currency            mxx_currency_code DEFAULT 'USD',
    credit_limit        NUMERIC(18,2),
    credit_days         INT,
    assigned_to         BIGINT REFERENCES mxx_system_admin(id),
    cooperated_at       DATE,
    birthday_month      INT,
    next_follow_at      TIMESTAMPTZ,
    description         TEXT,
    custom_fields       JSONB,
    created_by          BIGINT REFERENCES mxx_system_admin(id),
    created_at          TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_by          BIGINT REFERENCES mxx_system_admin(id),
    updated_at          TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted             INT DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_crm_customer_no ON mxx_crm_customer(customer_no);
CREATE INDEX IF NOT EXISTS idx_crm_customer_level ON mxx_crm_customer(level);
CREATE INDEX IF NOT EXISTS idx_crm_customer_assigned ON mxx_crm_customer(assigned_to);

COMMENT ON TABLE mxx_crm_customer IS '客户表';
COMMENT ON COLUMN mxx_crm_customer.id IS '客户ID';
COMMENT ON COLUMN mxx_crm_customer.customer_no IS '客户编号';
COMMENT ON COLUMN mxx_crm_customer.company_name IS '公司名称';
COMMENT ON COLUMN mxx_crm_customer.short_name IS '简称';
COMMENT ON COLUMN mxx_crm_customer.country IS '国家';
COMMENT ON COLUMN mxx_crm_customer.region IS '地区';
COMMENT ON COLUMN mxx_crm_customer.address IS '地址';
COMMENT ON COLUMN mxx_crm_customer.website IS '网站';
COMMENT ON COLUMN mxx_crm_customer.industry IS '行业类型';
COMMENT ON COLUMN mxx_crm_customer.level IS '客户等级';
COMMENT ON COLUMN mxx_crm_customer.total_deal_amount IS '累计成交金额';
COMMENT ON COLUMN mxx_crm_customer.total_deal_count IS '累计成交次数';
COMMENT ON COLUMN mxx_crm_customer.last_deal_at IS '最后成交时间';
COMMENT ON COLUMN mxx_crm_customer.source IS '客户来源';
COMMENT ON COLUMN mxx_crm_customer.tags IS '标签';
COMMENT ON COLUMN mxx_crm_customer.currency IS '货币类型';
COMMENT ON COLUMN mxx_crm_customer.credit_limit IS '信用额度';
COMMENT ON COLUMN mxx_crm_customer.credit_days IS '信用天数';
COMMENT ON COLUMN mxx_crm_customer.assigned_to IS '负责人ID';
COMMENT ON COLUMN mxx_crm_customer.cooperated_at IS '合作日期';
COMMENT ON COLUMN mxx_crm_customer.birthday_month IS '生日月份';
COMMENT ON COLUMN mxx_crm_customer.next_follow_at IS '下次跟进时间';
COMMENT ON COLUMN mxx_crm_customer.description IS '描述';
COMMENT ON COLUMN mxx_crm_customer.custom_fields IS '自定义字段';
COMMENT ON COLUMN mxx_crm_customer.created_by IS '创建者ID';
COMMENT ON COLUMN mxx_crm_customer.created_at IS '创建时间';
COMMENT ON COLUMN mxx_crm_customer.updated_by IS '更新者ID';
COMMENT ON COLUMN mxx_crm_customer.updated_at IS '更新时间';
COMMENT ON COLUMN mxx_crm_customer.deleted IS '0未删除 1已删除';

-- 3.3 mxx_crm_contact（联系人表）
CREATE TABLE IF NOT EXISTS mxx_crm_contact (
    id              BIGSERIAL PRIMARY KEY,
    customer_id     BIGINT REFERENCES mxx_crm_customer(id) ON DELETE CASCADE,
    name            VARCHAR(128) NOT NULL,
    title           VARCHAR(128),
    email           VARCHAR(128),
    phone           VARCHAR(32),
    mobile          VARCHAR(32),
    whatsapp        VARCHAR(32),
    wechat          VARCHAR(64),
    is_primary      BOOLEAN DEFAULT false,
    is_billing      BOOLEAN DEFAULT false,
    is_shipping     BOOLEAN DEFAULT false,
    birthday        DATE,
    notes           TEXT,
    created_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted         INT DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_crm_contact_customer ON mxx_crm_contact(customer_id);

COMMENT ON TABLE mxx_crm_contact IS '联系人表';
COMMENT ON COLUMN mxx_crm_contact.id IS '联系人ID';
COMMENT ON COLUMN mxx_crm_contact.customer_id IS '客户ID';
COMMENT ON COLUMN mxx_crm_contact.name IS '姓名';
COMMENT ON COLUMN mxx_crm_contact.title IS '职位';
COMMENT ON COLUMN mxx_crm_contact.email IS '邮箱';
COMMENT ON COLUMN mxx_crm_contact.phone IS '电话';
COMMENT ON COLUMN mxx_crm_contact.mobile IS '手机';
COMMENT ON COLUMN mxx_crm_contact.whatsapp IS 'WhatsApp';
COMMENT ON COLUMN mxx_crm_contact.wechat IS '微信';
COMMENT ON COLUMN mxx_crm_contact.is_primary IS '是否主联系人';
COMMENT ON COLUMN mxx_crm_contact.is_billing IS '是否开票联系人';
COMMENT ON COLUMN mxx_crm_contact.is_shipping IS '是否收货联系人';
COMMENT ON COLUMN mxx_crm_contact.birthday IS '生日';
COMMENT ON COLUMN mxx_crm_contact.notes IS '备注';
COMMENT ON COLUMN mxx_crm_contact.created_at IS '创建时间';
COMMENT ON COLUMN mxx_crm_contact.updated_at IS '更新时间';
COMMENT ON COLUMN mxx_crm_contact.deleted IS '0未删除 1已删除';

-- 3.4 mxx_crm_followup（跟进记录表）
CREATE TABLE IF NOT EXISTS mxx_crm_followup (
    id              BIGSERIAL PRIMARY KEY,
    lead_id         BIGINT REFERENCES mxx_crm_lead(id) ON DELETE CASCADE,
    customer_id     BIGINT REFERENCES mxx_crm_customer(id) ON DELETE CASCADE,
    opportunity_id  BIGINT,
    activity_type   mxx_activity_type,
    subject         VARCHAR(255),
    content         TEXT,
    next_follow_date DATE,
    duration_minutes INT,
    result          VARCHAR(255),
    assigned_to     BIGINT REFERENCES mxx_system_admin(id),
    created_by      BIGINT REFERENCES mxx_system_admin(id),
    created_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted         INT DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_crm_followup_lead ON mxx_crm_followup(lead_id);
CREATE INDEX IF NOT EXISTS idx_crm_followup_customer ON mxx_crm_followup(customer_id);

COMMENT ON TABLE mxx_crm_followup IS '跟进记录表';
COMMENT ON COLUMN mxx_crm_followup.id IS '记录ID';
COMMENT ON COLUMN mxx_crm_followup.lead_id IS '线索ID';
COMMENT ON COLUMN mxx_crm_followup.customer_id IS '客户ID';
COMMENT ON COLUMN mxx_crm_followup.opportunity_id IS '商机ID';
COMMENT ON COLUMN mxx_crm_followup.activity_type IS '活动类型';
COMMENT ON COLUMN mxx_crm_followup.subject IS '主题';
COMMENT ON COLUMN mxx_crm_followup.content IS '内容';
COMMENT ON COLUMN mxx_crm_followup.next_follow_date IS '下次跟进日期';
COMMENT ON COLUMN mxx_crm_followup.duration_minutes IS '持续时长(分钟)';
COMMENT ON COLUMN mxx_crm_followup.result IS '跟进结果';
COMMENT ON COLUMN mxx_crm_followup.assigned_to IS '负责人ID';
COMMENT ON COLUMN mxx_crm_followup.created_by IS '创建者ID';
COMMENT ON COLUMN mxx_crm_followup.created_at IS '创建时间';
COMMENT ON COLUMN mxx_crm_followup.updated_at IS '更新时间';
COMMENT ON COLUMN mxx_crm_followup.deleted IS '0未删除 1已删除';

-- 3.5 mxx_crm_lead_pool（线索池表）
CREATE TABLE IF NOT EXISTS mxx_crm_lead_pool (
    id             BIGSERIAL PRIMARY KEY,
    name           VARCHAR(128) NOT NULL,
    description    TEXT,
    recycle_days   INT DEFAULT 30,
    member_ids     BIGINT[],
    created_at     TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at     TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted        INT DEFAULT 0
);

COMMENT ON TABLE mxx_crm_lead_pool IS '线索池表';
COMMENT ON COLUMN mxx_crm_lead_pool.id IS '线索池ID';
COMMENT ON COLUMN mxx_crm_lead_pool.name IS '线索池名称';
COMMENT ON COLUMN mxx_crm_lead_pool.description IS '描述';
COMMENT ON COLUMN mxx_crm_lead_pool.recycle_days IS '回收天数';
COMMENT ON COLUMN mxx_crm_lead_pool.member_ids IS '成员ID列表';
COMMENT ON COLUMN mxx_crm_lead_pool.created_at IS '创建时间';
COMMENT ON COLUMN mxx_crm_lead_pool.updated_at IS '更新时间';
COMMENT ON COLUMN mxx_crm_lead_pool.deleted IS '0未删除 1已删除';

-- 3.6 mxx_crm_opportunity（商机表）
CREATE TABLE IF NOT EXISTS mxx_crm_opportunity (
    id              BIGSERIAL PRIMARY KEY,
    customer_id     BIGINT REFERENCES mxx_crm_customer(id) ON DELETE CASCADE,
    name            VARCHAR(255) NOT NULL,
    amount          NUMERIC(18,2),
    currency        mxx_currency_code DEFAULT 'USD',
    stage           mxx_opportunity_stage DEFAULT 'qualification',
    probability     INT DEFAULT 0,
    expected_close_date DATE,
    actual_close_date  DATE,
    assigned_to     BIGINT REFERENCES mxx_system_admin(id),
    description     TEXT,
    competitor_info TEXT,
    loss_reason     VARCHAR(255),
    created_by      BIGINT REFERENCES mxx_system_admin(id),
    created_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_by      BIGINT REFERENCES mxx_system_admin(id),
    updated_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted         INT DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_crm_opportunity_customer ON mxx_crm_opportunity(customer_id);
CREATE INDEX IF NOT EXISTS idx_crm_opportunity_stage ON mxx_crm_opportunity(stage);

COMMENT ON TABLE mxx_crm_opportunity IS '商机表';
COMMENT ON COLUMN mxx_crm_opportunity.id IS '商机ID';
COMMENT ON COLUMN mxx_crm_opportunity.customer_id IS '客户ID';
COMMENT ON COLUMN mxx_crm_opportunity.name IS '商机名称';
COMMENT ON COLUMN mxx_crm_opportunity.amount IS '商机金额';
COMMENT ON COLUMN mxx_crm_opportunity.currency IS '货币类型';
COMMENT ON COLUMN mxx_crm_opportunity.stage IS '商机阶段';
COMMENT ON COLUMN mxx_crm_opportunity.probability IS '成功率(%)';
COMMENT ON COLUMN mxx_crm_opportunity.expected_close_date IS '预计成交日期';
COMMENT ON COLUMN mxx_crm_opportunity.actual_close_date IS '实际成交日期';
COMMENT ON COLUMN mxx_crm_opportunity.assigned_to IS '负责人ID';
COMMENT ON COLUMN mxx_crm_opportunity.description IS '描述';
COMMENT ON COLUMN mxx_crm_opportunity.competitor_info IS '竞争对手信息';
COMMENT ON COLUMN mxx_crm_opportunity.loss_reason IS '失败原因';
COMMENT ON COLUMN mxx_crm_opportunity.created_by IS '创建者ID';
COMMENT ON COLUMN mxx_crm_opportunity.created_at IS '创建时间';
COMMENT ON COLUMN mxx_crm_opportunity.updated_by IS '更新者ID';
COMMENT ON COLUMN mxx_crm_opportunity.updated_at IS '更新时间';
COMMENT ON COLUMN mxx_crm_opportunity.deleted IS '0未删除 1已删除';

-- 3.7 mxx_crm_contract（合同表）
CREATE TABLE IF NOT EXISTS mxx_crm_contract (
    id              BIGSERIAL PRIMARY KEY,
    contract_no     VARCHAR(32) UNIQUE NOT NULL,
    customer_id     BIGINT REFERENCES mxx_crm_customer(id),
    opportunity_id  BIGINT REFERENCES mxx_crm_opportunity(id),
    name            VARCHAR(255) NOT NULL,
    amount          NUMERIC(18,2),
    currency        mxx_currency_code DEFAULT 'USD',
    status          mxx_contract_status DEFAULT 'draft',
    sign_date       DATE,
    start_date      DATE,
    end_date        DATE,
    payment_terms   VARCHAR(255),
    delivery_terms  VARCHAR(255),
    assigned_to     BIGINT REFERENCES mxx_system_admin(id),
    description     TEXT,
    file_url        VARCHAR(255),
    created_by      BIGINT REFERENCES mxx_system_admin(id),
    created_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_by      BIGINT REFERENCES mxx_system_admin(id),
    updated_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted         INT DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_crm_contract_customer ON mxx_crm_contract(customer_id);
CREATE INDEX IF NOT EXISTS idx_crm_contract_status ON mxx_crm_contract(status);

COMMENT ON TABLE mxx_crm_contract IS '合同表';
COMMENT ON COLUMN mxx_crm_contract.id IS '合同ID';
COMMENT ON COLUMN mxx_crm_contract.contract_no IS '合同编号';
COMMENT ON COLUMN mxx_crm_contract.customer_id IS '客户ID';
COMMENT ON COLUMN mxx_crm_contract.opportunity_id IS '商机ID';
COMMENT ON COLUMN mxx_crm_contract.name IS '合同名称';
COMMENT ON COLUMN mxx_crm_contract.amount IS '合同金额';
COMMENT ON COLUMN mxx_crm_contract.currency IS '货币类型';
COMMENT ON COLUMN mxx_crm_contract.status IS '合同状态';
COMMENT ON COLUMN mxx_crm_contract.sign_date IS '签订日期';
COMMENT ON COLUMN mxx_crm_contract.start_date IS '开始日期';
COMMENT ON COLUMN mxx_crm_contract.end_date IS '结束日期';
COMMENT ON COLUMN mxx_crm_contract.payment_terms IS '付款条款';
COMMENT ON COLUMN mxx_crm_contract.delivery_terms IS '交货条款';
COMMENT ON COLUMN mxx_crm_contract.assigned_to IS '负责人ID';
COMMENT ON COLUMN mxx_crm_contract.description IS '描述';
COMMENT ON COLUMN mxx_crm_contract.file_url IS '合同文件URL';
COMMENT ON COLUMN mxx_crm_contract.created_by IS '创建者ID';
COMMENT ON COLUMN mxx_crm_contract.created_at IS '创建时间';
COMMENT ON COLUMN mxx_crm_contract.updated_by IS '更新者ID';
COMMENT ON COLUMN mxx_crm_contract.updated_at IS '更新时间';
COMMENT ON COLUMN mxx_crm_contract.deleted IS '0未删除 1已删除';

-- 3.8 mxx_sale_order（订单表）
CREATE TABLE IF NOT EXISTS mxx_sale_order (
    id              BIGSERIAL PRIMARY KEY,
    order_no        VARCHAR(32) UNIQUE NOT NULL,
    customer_id     BIGINT REFERENCES mxx_crm_customer(id),
    contract_id     BIGINT REFERENCES mxx_crm_contract(id),
    order_date      DATE NOT NULL,
    amount          NUMERIC(18,2),
    currency        mxx_currency_code DEFAULT 'USD',
    status          mxx_order_status DEFAULT 'pending',
    payment_status  mxx_payment_status DEFAULT 'unpaid',
    payment_due_date DATE,
    shipping_address TEXT,
    billing_address  TEXT,
    notes           TEXT,
    assigned_to     BIGINT REFERENCES mxx_system_admin(id),
    created_by      BIGINT REFERENCES mxx_system_admin(id),
    created_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_by      BIGINT REFERENCES mxx_system_admin(id),
    updated_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted         INT DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_sale_order_customer ON mxx_sale_order(customer_id);
CREATE INDEX IF NOT EXISTS idx_sale_order_status ON mxx_sale_order(status);

COMMENT ON TABLE mxx_sale_order IS '订单表';
COMMENT ON COLUMN mxx_sale_order.id IS '订单ID';
COMMENT ON COLUMN mxx_sale_order.order_no IS '订单编号';
COMMENT ON COLUMN mxx_sale_order.customer_id IS '客户ID';
COMMENT ON COLUMN mxx_sale_order.contract_id IS '合同ID';
COMMENT ON COLUMN mxx_sale_order.order_date IS '订单日期';
COMMENT ON COLUMN mxx_sale_order.amount IS '订单金额';
COMMENT ON COLUMN mxx_sale_order.currency IS '货币类型';
COMMENT ON COLUMN mxx_sale_order.status IS '订单状态';
COMMENT ON COLUMN mxx_sale_order.payment_status IS '支付状态';
COMMENT ON COLUMN mxx_sale_order.payment_due_date IS '付款截止日期';
COMMENT ON COLUMN mxx_sale_order.shipping_address IS '收货地址';
COMMENT ON COLUMN mxx_sale_order.billing_address IS '开票地址';
COMMENT ON COLUMN mxx_sale_order.notes IS '备注';
COMMENT ON COLUMN mxx_sale_order.assigned_to IS '负责人ID';
COMMENT ON COLUMN mxx_sale_order.created_by IS '创建者ID';
COMMENT ON COLUMN mxx_sale_order.created_at IS '创建时间';
COMMENT ON COLUMN mxx_sale_order.updated_by IS '更新者ID';
COMMENT ON COLUMN mxx_sale_order.updated_at IS '更新时间';
COMMENT ON COLUMN mxx_sale_order.deleted IS '0未删除 1已删除';

-- 3.9 mxx_sale_order_item（订单明细表）
CREATE TABLE IF NOT EXISTS mxx_sale_order_item (
    id              BIGSERIAL PRIMARY KEY,
    order_id        BIGINT REFERENCES mxx_sale_order(id) ON DELETE CASCADE,
    product_id      BIGINT,
    sku             VARCHAR(64),
    product_name    VARCHAR(255),
    quantity        NUMERIC(10,2) NOT NULL,
    unit_price      NUMERIC(15,2) NOT NULL,
    discount        NUMERIC(15,2) DEFAULT 0,
    tax_rate        NUMERIC(5,2) DEFAULT 0,
    total_amount    NUMERIC(18,2),
    notes           TEXT,
    created_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted         INT DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_sale_order_item_order ON mxx_sale_order_item(order_id);

COMMENT ON TABLE mxx_sale_order_item IS '订单明细表';
COMMENT ON COLUMN mxx_sale_order_item.id IS '明细ID';
COMMENT ON COLUMN mxx_sale_order_item.order_id IS '订单ID';
COMMENT ON COLUMN mxx_sale_order_item.product_id IS '产品ID';
COMMENT ON COLUMN mxx_sale_order_item.sku IS 'SKU';
COMMENT ON COLUMN mxx_sale_order_item.product_name IS '产品名称';
COMMENT ON COLUMN mxx_sale_order_item.quantity IS '数量';
COMMENT ON COLUMN mxx_sale_order_item.unit_price IS '单价';
COMMENT ON COLUMN mxx_sale_order_item.discount IS '折扣';
COMMENT ON COLUMN mxx_sale_order_item.tax_rate IS '税率';
COMMENT ON COLUMN mxx_sale_order_item.total_amount IS '总金额';
COMMENT ON COLUMN mxx_sale_order_item.notes IS '备注';
COMMENT ON COLUMN mxx_sale_order_item.created_at IS '创建时间';
COMMENT ON COLUMN mxx_sale_order_item.deleted IS '0未删除 1已删除';

-- 3.10 mxx_sale_payment（支付记录表）
CREATE TABLE IF NOT EXISTS mxx_sale_payment (
    id              BIGSERIAL PRIMARY KEY,
    payment_no      VARCHAR(32) UNIQUE NOT NULL,
    order_id        BIGINT REFERENCES mxx_sale_order(id),
    customer_id     BIGINT REFERENCES mxx_crm_customer(id),
    amount          NUMERIC(18,2) NOT NULL,
    currency        mxx_currency_code DEFAULT 'USD',
    payment_method  mxx_payment_method,
    payment_date    DATE,
    bank_account    VARCHAR(128),
    transaction_id  VARCHAR(128),
    notes           TEXT,
    created_by      BIGINT REFERENCES mxx_system_admin(id),
    created_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted         INT DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_sale_payment_order ON mxx_sale_payment(order_id);
CREATE INDEX IF NOT EXISTS idx_sale_payment_customer ON mxx_sale_payment(customer_id);

COMMENT ON TABLE mxx_sale_payment IS '支付记录表';
COMMENT ON COLUMN mxx_sale_payment.id IS '支付记录ID';
COMMENT ON COLUMN mxx_sale_payment.payment_no IS '支付编号';
COMMENT ON COLUMN mxx_sale_payment.order_id IS '订单ID';
COMMENT ON COLUMN mxx_sale_payment.customer_id IS '客户ID';
COMMENT ON COLUMN mxx_sale_payment.amount IS '支付金额';
COMMENT ON COLUMN mxx_sale_payment.currency IS '货币类型';
COMMENT ON COLUMN mxx_sale_payment.payment_method IS '支付方式';
COMMENT ON COLUMN mxx_sale_payment.payment_date IS '支付日期';
COMMENT ON COLUMN mxx_sale_payment.bank_account IS '银行账号';
COMMENT ON COLUMN mxx_sale_payment.transaction_id IS '交易ID';
COMMENT ON COLUMN mxx_sale_payment.notes IS '备注';
COMMENT ON COLUMN mxx_sale_payment.created_by IS '创建者ID';
COMMENT ON COLUMN mxx_sale_payment.created_at IS '创建时间';
COMMENT ON COLUMN mxx_sale_payment.updated_at IS '更新时间';
COMMENT ON COLUMN mxx_sale_payment.deleted IS '0未删除 1已删除';

-- 3.11 mxx_product_main（产品主表）
CREATE TABLE IF NOT EXISTS mxx_product_main (
    id              BIGSERIAL PRIMARY KEY,
    product_no      VARCHAR(32) UNIQUE NOT NULL,
    name            VARCHAR(255) NOT NULL,
    category_id     BIGINT,
    sku             VARCHAR(64),
    barcode         VARCHAR(64),
    unit            VARCHAR(32),
    cost_price      NUMERIC(15,2),
    sale_price      NUMERIC(15,2),
    currency        mxx_currency_code DEFAULT 'USD',
    weight          NUMERIC(10,3),
    dimensions      VARCHAR(64),
    description     TEXT,
    image_url       VARCHAR(255),
    is_active       BOOLEAN DEFAULT true,
    created_by      BIGINT REFERENCES mxx_system_admin(id),
    created_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_by      BIGINT REFERENCES mxx_system_admin(id),
    updated_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted         INT DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_product_no ON mxx_product_main(product_no);
CREATE INDEX IF NOT EXISTS idx_product_category ON mxx_product_main(category_id);

COMMENT ON TABLE mxx_product_main IS '产品主表';
COMMENT ON COLUMN mxx_product_main.id IS '产品ID';
COMMENT ON COLUMN mxx_product_main.product_no IS '产品编号';
COMMENT ON COLUMN mxx_product_main.name IS '产品名称';
COMMENT ON COLUMN mxx_product_main.category_id IS '分类ID';
COMMENT ON COLUMN mxx_product_main.sku IS 'SKU';
COMMENT ON COLUMN mxx_product_main.barcode IS '条形码';
COMMENT ON COLUMN mxx_product_main.unit IS '单位';
COMMENT ON COLUMN mxx_product_main.cost_price IS '成本价';
COMMENT ON COLUMN mxx_product_main.sale_price IS '销售价';
COMMENT ON COLUMN mxx_product_main.currency IS '货币类型';
COMMENT ON COLUMN mxx_product_main.weight IS '重量';
COMMENT ON COLUMN mxx_product_main.dimensions IS '尺寸';
COMMENT ON COLUMN mxx_product_main.description IS '描述';
COMMENT ON COLUMN mxx_product_main.image_url IS '图片URL';
COMMENT ON COLUMN mxx_product_main.is_active IS '是否启用';
COMMENT ON COLUMN mxx_product_main.created_by IS '创建者ID';
COMMENT ON COLUMN mxx_product_main.created_at IS '创建时间';
COMMENT ON COLUMN mxx_product_main.updated_by IS '更新者ID';
COMMENT ON COLUMN mxx_product_main.updated_at IS '更新时间';
COMMENT ON COLUMN mxx_product_main.deleted IS '0未删除 1已删除';

-- 3.12 mxx_product_category（产品分类表）
CREATE TABLE IF NOT EXISTS mxx_product_category (
    id              BIGSERIAL PRIMARY KEY,
    parent_id       BIGINT REFERENCES mxx_product_category(id),
    name            VARCHAR(128) NOT NULL,
    description     TEXT,
    sort_order      INT DEFAULT 0,
    created_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted         INT DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_product_category_parent ON mxx_product_category(parent_id);

COMMENT ON TABLE mxx_product_category IS '产品分类表';
COMMENT ON COLUMN mxx_product_category.id IS '分类ID';
COMMENT ON COLUMN mxx_product_category.parent_id IS '父分类ID';
COMMENT ON COLUMN mxx_product_category.name IS '分类名称';
COMMENT ON COLUMN mxx_product_category.description IS '描述';
COMMENT ON COLUMN mxx_product_category.sort_order IS '排序';
COMMENT ON COLUMN mxx_product_category.created_at IS '创建时间';
COMMENT ON COLUMN mxx_product_category.updated_at IS '更新时间';
COMMENT ON COLUMN mxx_product_category.deleted IS '0未删除 1已删除';

-- 3.13 mxx_inventory_warehouse（仓库表）
CREATE TABLE IF NOT EXISTS mxx_inventory_warehouse (
    id              BIGSERIAL PRIMARY KEY,
    name            VARCHAR(128) NOT NULL,
    code            VARCHAR(32) UNIQUE,
    address         VARCHAR(255),
    contact_person  VARCHAR(64),
    contact_phone   VARCHAR(32),
    is_active       BOOLEAN DEFAULT true,
    created_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted         INT DEFAULT 0
);

COMMENT ON TABLE mxx_inventory_warehouse IS '仓库表';
COMMENT ON COLUMN mxx_inventory_warehouse.id IS '仓库ID';
COMMENT ON COLUMN mxx_inventory_warehouse.name IS '仓库名称';
COMMENT ON COLUMN mxx_inventory_warehouse.code IS '仓库编码';
COMMENT ON COLUMN mxx_inventory_warehouse.address IS '仓库地址';
COMMENT ON COLUMN mxx_inventory_warehouse.contact_person IS '联系人';
COMMENT ON COLUMN mxx_inventory_warehouse.contact_phone IS '联系电话';
COMMENT ON COLUMN mxx_inventory_warehouse.is_active IS '是否启用';
COMMENT ON COLUMN mxx_inventory_warehouse.created_at IS '创建时间';
COMMENT ON COLUMN mxx_inventory_warehouse.updated_at IS '更新时间';
COMMENT ON COLUMN mxx_inventory_warehouse.deleted IS '0未删除 1已删除';

-- 3.14 mxx_inventory_stock（库存表）
CREATE TABLE IF NOT EXISTS mxx_inventory_stock (
    id                  BIGSERIAL PRIMARY KEY,
    product_id          BIGINT REFERENCES mxx_product_main(id),
    warehouse_id        BIGINT REFERENCES mxx_inventory_warehouse(id),
    quantity            NUMERIC(15,2) DEFAULT 0,
    reserved_quantity   NUMERIC(15,2) DEFAULT 0,
    available_quantity  NUMERIC(15,2) GENERATED ALWAYS AS (quantity - reserved_quantity) STORED,
    last_updated_at     TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted             INT DEFAULT 0,
    UNIQUE(product_id, warehouse_id)
);
CREATE INDEX IF NOT EXISTS idx_inventory_product ON mxx_inventory_stock(product_id);
CREATE INDEX IF NOT EXISTS idx_inventory_warehouse ON mxx_inventory_stock(warehouse_id);

COMMENT ON TABLE mxx_inventory_stock IS '库存表';
COMMENT ON COLUMN mxx_inventory_stock.id IS '库存ID';
COMMENT ON COLUMN mxx_inventory_stock.product_id IS '产品ID';
COMMENT ON COLUMN mxx_inventory_stock.warehouse_id IS '仓库ID';
COMMENT ON COLUMN mxx_inventory_stock.quantity IS '库存数量';
COMMENT ON COLUMN mxx_inventory_stock.reserved_quantity IS '预留数量';
COMMENT ON COLUMN mxx_inventory_stock.available_quantity IS '可用数量';
COMMENT ON COLUMN mxx_inventory_stock.last_updated_at IS '最后更新时间';
COMMENT ON COLUMN mxx_inventory_stock.deleted IS '0未删除 1已删除';

-- 3.15 mxx_inventory_transaction（库存流水表）
CREATE TABLE IF NOT EXISTS mxx_inventory_transaction (
    id              BIGSERIAL PRIMARY KEY,
    product_id      BIGINT REFERENCES mxx_product_main(id),
    warehouse_id    BIGINT REFERENCES mxx_inventory_warehouse(id),
    operation_type  mxx_inventory_op_type,
    quantity        NUMERIC(15,2) NOT NULL,
    reference_type  VARCHAR(64),
    reference_id    BIGINT,
    notes           TEXT,
    created_by      BIGINT REFERENCES mxx_system_admin(id),
    created_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_inv_trans_product ON mxx_inventory_transaction(product_id);
CREATE INDEX IF NOT EXISTS idx_inv_trans_date ON mxx_inventory_transaction(created_at DESC);

COMMENT ON TABLE mxx_inventory_transaction IS '库存流水表';
COMMENT ON COLUMN mxx_inventory_transaction.id IS '流水ID';
COMMENT ON COLUMN mxx_inventory_transaction.product_id IS '产品ID';
COMMENT ON COLUMN mxx_inventory_transaction.warehouse_id IS '仓库ID';
COMMENT ON COLUMN mxx_inventory_transaction.operation_type IS '操作类型';
COMMENT ON COLUMN mxx_inventory_transaction.quantity IS '数量';
COMMENT ON COLUMN mxx_inventory_transaction.reference_type IS '关联类型';
COMMENT ON COLUMN mxx_inventory_transaction.reference_id IS '关联ID';
COMMENT ON COLUMN mxx_inventory_transaction.notes IS '备注';
COMMENT ON COLUMN mxx_inventory_transaction.created_by IS '创建者ID';
COMMENT ON COLUMN mxx_inventory_transaction.created_at IS '创建时间';

-- 3.16 mxx_purchase_supplier（供应商表）
CREATE TABLE IF NOT EXISTS mxx_purchase_supplier (
    id              BIGSERIAL PRIMARY KEY,
    supplier_no     VARCHAR(32) UNIQUE NOT NULL,
    name            VARCHAR(255) NOT NULL,
    short_name      VARCHAR(64),
    contact_person  VARCHAR(64),
    phone           VARCHAR(32),
    email           VARCHAR(128),
    address         VARCHAR(512),
    country         VARCHAR(64),
    website         VARCHAR(255),
    industry        mxx_industry_type,
    payment_terms   VARCHAR(255),
    delivery_terms  VARCHAR(255),
    notes           TEXT,
    is_active       BOOLEAN DEFAULT true,
    created_by      BIGINT REFERENCES mxx_system_admin(id),
    created_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_by      BIGINT REFERENCES mxx_system_admin(id),
    updated_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted         INT DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_purchase_supplier_no ON mxx_purchase_supplier(supplier_no);

COMMENT ON TABLE mxx_purchase_supplier IS '供应商表';
COMMENT ON COLUMN mxx_purchase_supplier.id IS '供应商ID';
COMMENT ON COLUMN mxx_purchase_supplier.supplier_no IS '供应商编号';
COMMENT ON COLUMN mxx_purchase_supplier.name IS '供应商名称';
COMMENT ON COLUMN mxx_purchase_supplier.short_name IS '简称';
COMMENT ON COLUMN mxx_purchase_supplier.contact_person IS '联系人';
COMMENT ON COLUMN mxx_purchase_supplier.phone IS '电话';
COMMENT ON COLUMN mxx_purchase_supplier.email IS '邮箱';
COMMENT ON COLUMN mxx_purchase_supplier.address IS '地址';
COMMENT ON COLUMN mxx_purchase_supplier.country IS '国家';
COMMENT ON COLUMN mxx_purchase_supplier.website IS '网站';
COMMENT ON COLUMN mxx_purchase_supplier.industry IS '行业类型';
COMMENT ON COLUMN mxx_purchase_supplier.payment_terms IS '付款条款';
COMMENT ON COLUMN mxx_purchase_supplier.delivery_terms IS '交货条款';
COMMENT ON COLUMN mxx_purchase_supplier.notes IS '备注';
COMMENT ON COLUMN mxx_purchase_supplier.is_active IS '是否启用';
COMMENT ON COLUMN mxx_purchase_supplier.created_by IS '创建者ID';
COMMENT ON COLUMN mxx_purchase_supplier.created_at IS '创建时间';
COMMENT ON COLUMN mxx_purchase_supplier.updated_by IS '更新者ID';
COMMENT ON COLUMN mxx_purchase_supplier.updated_at IS '更新时间';
COMMENT ON COLUMN mxx_purchase_supplier.deleted IS '0未删除 1已删除';

-- 迁移：新增 level 和 status 列（INT4 类型）
ALTER TABLE mxx_purchase_supplier ADD COLUMN IF NOT EXISTS level INT4;
ALTER TABLE mxx_purchase_supplier ADD COLUMN IF NOT EXISTS status INT4;
COMMENT ON COLUMN mxx_purchase_supplier.level IS '供应商等级';
COMMENT ON COLUMN mxx_purchase_supplier.status IS '供应商状态';

-- 3.17 mxx_purchase_po（采购单表）
CREATE TABLE IF NOT EXISTS mxx_purchase_po (
    id              BIGSERIAL PRIMARY KEY,
    purchase_no     VARCHAR(32) UNIQUE NOT NULL,
    supplier_id     BIGINT REFERENCES mxx_purchase_supplier(id),
    purchase_date   DATE NOT NULL,
    expected_date   DATE,
    amount          NUMERIC(18,2),
    currency        mxx_currency_code DEFAULT 'CNY',
    status          mxx_purchase_status DEFAULT 'draft',
    payment_status  mxx_payment_status DEFAULT 'unpaid',
    notes           TEXT,
    created_by      BIGINT REFERENCES mxx_system_admin(id),
    created_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_by      BIGINT REFERENCES mxx_system_admin(id),
    updated_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted         INT DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_purchase_po_supplier ON mxx_purchase_po(supplier_id);
CREATE INDEX IF NOT EXISTS idx_purchase_po_status ON mxx_purchase_po(status);

COMMENT ON TABLE mxx_purchase_po IS '采购单表';
COMMENT ON COLUMN mxx_purchase_po.id IS '采购单ID';
COMMENT ON COLUMN mxx_purchase_po.purchase_no IS '采购单编号';
COMMENT ON COLUMN mxx_purchase_po.supplier_id IS '供应商ID';
COMMENT ON COLUMN mxx_purchase_po.purchase_date IS '采购日期';
COMMENT ON COLUMN mxx_purchase_po.expected_date IS '预计到货日期';
COMMENT ON COLUMN mxx_purchase_po.amount IS '采购金额';
COMMENT ON COLUMN mxx_purchase_po.currency IS '货币类型';
COMMENT ON COLUMN mxx_purchase_po.status IS '采购状态';
COMMENT ON COLUMN mxx_purchase_po.payment_status IS '支付状态';
COMMENT ON COLUMN mxx_purchase_po.notes IS '备注';
COMMENT ON COLUMN mxx_purchase_po.created_by IS '创建者ID';
COMMENT ON COLUMN mxx_purchase_po.created_at IS '创建时间';
COMMENT ON COLUMN mxx_purchase_po.updated_by IS '更新者ID';
COMMENT ON COLUMN mxx_purchase_po.updated_at IS '更新时间';
COMMENT ON COLUMN mxx_purchase_po.deleted IS '0未删除 1已删除';

-- 3.18 mxx_purchase_item（采购明细表）
CREATE TABLE IF NOT EXISTS mxx_purchase_item (
    id              BIGSERIAL PRIMARY KEY,
    purchase_id     BIGINT REFERENCES mxx_purchase_po(id) ON DELETE CASCADE,
    product_id      BIGINT REFERENCES mxx_product_main(id),
    sku             VARCHAR(64),
    product_name    VARCHAR(255),
    quantity        NUMERIC(10,2) NOT NULL,
    unit_price      NUMERIC(15,2) NOT NULL,
    discount        NUMERIC(15,2) DEFAULT 0,
    tax_rate        NUMERIC(5,2) DEFAULT 0,
    total_amount    NUMERIC(18,2),
    notes           TEXT,
    created_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted         INT DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_purchase_item_purchase ON mxx_purchase_item(purchase_id);
CREATE INDEX IF NOT EXISTS idx_purchase_item_product ON mxx_purchase_item(product_id);

COMMENT ON TABLE mxx_purchase_item IS '采购明细表';
COMMENT ON COLUMN mxx_purchase_item.id IS '明细ID';
COMMENT ON COLUMN mxx_purchase_item.purchase_id IS '采购单ID';
COMMENT ON COLUMN mxx_purchase_item.product_id IS '产品ID';
COMMENT ON COLUMN mxx_purchase_item.sku IS 'SKU';
COMMENT ON COLUMN mxx_purchase_item.product_name IS '产品名称';
COMMENT ON COLUMN mxx_purchase_item.quantity IS '数量';
COMMENT ON COLUMN mxx_purchase_item.unit_price IS '单价';
COMMENT ON COLUMN mxx_purchase_item.discount IS '折扣';
COMMENT ON COLUMN mxx_purchase_item.tax_rate IS '税率';
COMMENT ON COLUMN mxx_purchase_item.total_amount IS '总金额';
COMMENT ON COLUMN mxx_purchase_item.notes IS '备注';
COMMENT ON COLUMN mxx_purchase_item.created_at IS '创建时间';
COMMENT ON COLUMN mxx_purchase_item.deleted IS '0未删除 1已删除';

-- 3.19 mxx_attachment_file（附件表）
CREATE TABLE IF NOT EXISTS mxx_attachment_file (
    id              BIGSERIAL PRIMARY KEY,
    file_name       VARCHAR(255) NOT NULL,
    file_path       VARCHAR(512) NOT NULL,
    file_size       BIGINT,
    file_type       VARCHAR(128),
    mime_type       VARCHAR(128),
    related_type    VARCHAR(64),
    related_id      BIGINT,
    uploaded_by     BIGINT REFERENCES mxx_system_admin(id),
    created_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted         INT DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_attachment_related ON mxx_attachment_file(related_type, related_id);

COMMENT ON TABLE mxx_attachment_file IS '附件表';
COMMENT ON COLUMN mxx_attachment_file.id IS '附件ID';
COMMENT ON COLUMN mxx_attachment_file.file_name IS '文件名';
COMMENT ON COLUMN mxx_attachment_file.file_path IS '文件路径';
COMMENT ON COLUMN mxx_attachment_file.file_size IS '文件大小';
COMMENT ON COLUMN mxx_attachment_file.file_type IS '文件类型';
COMMENT ON COLUMN mxx_attachment_file.mime_type IS 'MIME类型';
COMMENT ON COLUMN mxx_attachment_file.related_type IS '关联类型';
COMMENT ON COLUMN mxx_attachment_file.related_id IS '关联ID';
COMMENT ON COLUMN mxx_attachment_file.uploaded_by IS '上传者ID';
COMMENT ON COLUMN mxx_attachment_file.created_at IS '创建时间';
COMMENT ON COLUMN mxx_attachment_file.deleted IS '0未删除 1已删除';

-- 3.20 mxx_seq_generator（序列号生成器）
CREATE TABLE IF NOT EXISTS mxx_seq_generator (
    id              BIGSERIAL PRIMARY KEY,
    name            VARCHAR(64) UNIQUE NOT NULL,
    prefix          VARCHAR(16),
    current_value   BIGINT DEFAULT 0,
    date_format     VARCHAR(32),
    padding         INT DEFAULT 4,
    description     VARCHAR(255),
    updated_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

COMMENT ON TABLE mxx_seq_generator IS '序列号生成器';
COMMENT ON COLUMN mxx_seq_generator.id IS 'ID';
COMMENT ON COLUMN mxx_seq_generator.name IS '生成器名称';
COMMENT ON COLUMN mxx_seq_generator.prefix IS '前缀';
COMMENT ON COLUMN mxx_seq_generator.current_value IS '当前值';
COMMENT ON COLUMN mxx_seq_generator.date_format IS '日期格式';
COMMENT ON COLUMN mxx_seq_generator.padding IS '补零位数';
COMMENT ON COLUMN mxx_seq_generator.description IS '描述';
COMMENT ON COLUMN mxx_seq_generator.updated_at IS '更新时间';

-- ================================================
-- 4. 更新视图（使用统一的 deleted 字段）
-- ================================================

DROP VIEW IF EXISTS v_mxx_admin_role;
DROP VIEW IF EXISTS v_mxx_role_menu;
DROP VIEW IF EXISTS v_mxx_system_admin;

CREATE VIEW v_mxx_admin_role AS
SELECT 
    a.id as admin_id,
    a.user_name,
    a.nick_name,
    r.id as role_id,
    r.role_name,
    r.role_key,
    r.data_scope
FROM mxx_system_admin a
LEFT JOIN mxx_system_admin_role_merge arm ON a.id = arm.admin_id
LEFT JOIN mxx_system_role r ON arm.role_id = r.id
WHERE a.deleted = 0 AND (r.deleted = 0 OR r.deleted IS NULL);

CREATE VIEW v_mxx_role_menu AS
SELECT 
    r.id as role_id,
    r.role_name,
    m.id as menu_id,
    m.name as menu_name,
    m.path,
    m.type,
    m.perm
FROM mxx_system_role r
LEFT JOIN mxx_system_role_menu_merge rmm ON r.id = rmm.role_id
LEFT JOIN mxx_system_menu m ON rmm.menu_id = m.id
WHERE r.deleted = 0 AND (m.status = 0 OR m.status IS NULL);

CREATE VIEW v_mxx_system_admin AS
SELECT 
    a.*,
    COALESCE(r.role_name, '') as role_name,
    COALESCE(d.dept_name, '') as dept_name
FROM mxx_system_admin a
LEFT JOIN mxx_system_admin_role_merge arm ON a.id = arm.admin_id
LEFT JOIN mxx_system_role r ON arm.role_id = r.id
LEFT JOIN mxx_system_admin_dept_merge adm ON a.id = adm.admin_id
LEFT JOIN mxx_system_dept d ON adm.dept_id = d.id
WHERE a.deleted = 0;

-- ================================================
-- 5. 添加索引优化查询性能
-- ================================================

CREATE INDEX IF NOT EXISTS idx_sys_admin_username_status_del ON mxx_system_admin(user_name, status, deleted);
CREATE INDEX IF NOT EXISTS idx_sys_menu_status_type_parent ON mxx_system_menu(status, type, parent_id);
CREATE INDEX IF NOT EXISTS idx_sys_role_menu_role_status ON mxx_system_role_menu_merge(role_id, status);
CREATE INDEX IF NOT EXISTS idx_crm_customer_assigned_level ON mxx_crm_customer(assigned_to, level);
CREATE INDEX IF NOT EXISTS idx_crm_lead_status_assigned ON mxx_crm_lead(status, assigned_to);
CREATE INDEX IF NOT EXISTS idx_sale_order_customer_status ON mxx_sale_order(customer_id, status);
CREATE INDEX IF NOT EXISTS idx_crm_opportunity_stage_assigned ON mxx_crm_opportunity(stage, assigned_to);

-- ================================================
-- 6. 创建触发器自动更新时间戳
-- ================================================

CREATE OR REPLACE FUNCTION update_modified_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.update_time = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_mxx_system_admin_modtime
    BEFORE UPDATE ON mxx_system_admin
    FOR EACH ROW EXECUTE FUNCTION update_modified_column();

CREATE TRIGGER update_mxx_system_dept_modtime
    BEFORE UPDATE ON mxx_system_dept
    FOR EACH ROW EXECUTE FUNCTION update_modified_column();

CREATE TRIGGER update_mxx_system_role_modtime
    BEFORE UPDATE ON mxx_system_role
    FOR EACH ROW EXECUTE FUNCTION update_modified_column();

CREATE TRIGGER update_mxx_system_menu_modtime
    BEFORE UPDATE ON mxx_system_menu
    FOR EACH ROW EXECUTE FUNCTION update_modified_column();

CREATE TRIGGER update_mxx_system_config_modtime
    BEFORE UPDATE ON mxx_system_config
    FOR EACH ROW EXECUTE FUNCTION update_modified_column();

CREATE TRIGGER update_mxx_system_dict_modtime
    BEFORE UPDATE ON mxx_system_dict
    FOR EACH ROW EXECUTE FUNCTION update_modified_column();

CREATE TRIGGER update_mxx_system_dict_data_modtime
    BEFORE UPDATE ON mxx_system_dict_data
    FOR EACH ROW EXECUTE FUNCTION update_modified_column();

-- ================================================
-- 7. 初始化序列号生成器数据
-- ================================================

INSERT INTO mxx_seq_generator (name, prefix, date_format, padding, description) VALUES
('customer_no', 'CUST', 'YYYYMMDD', 6, '客户编号生成器');
INSERT INTO mxx_seq_generator (name, prefix, date_format, padding, description) VALUES
('order_no', 'ORD', 'YYYYMMDD', 6, '订单编号生成器');
INSERT INTO mxx_seq_generator (name, prefix, date_format, padding, description) VALUES
('contract_no', 'CON', 'YYYYMMDD', 6, '合同编号生成器');
INSERT INTO mxx_seq_generator (name, prefix, date_format, padding, description) VALUES
('purchase_no', 'PO', 'YYYYMMDD', 6, '采购单编号生成器');
INSERT INTO mxx_seq_generator (name, prefix, date_format, padding, description) VALUES
('payment_no', 'PAY', 'YYYYMMDD', 6, '支付编号生成器');
INSERT INTO mxx_seq_generator (name, prefix, date_format, padding, description) VALUES
('product_no', 'PRD', '', 6, '产品编号生成器');
INSERT INTO mxx_seq_generator (name, prefix, date_format, padding, description) VALUES
('supplier_no', 'SUP', '', 6, '供应商编号生成器');
INSERT INTO mxx_seq_generator (name, prefix, date_format, padding, description) VALUES
('lead_no', 'LD', 'YYYYMMDD', 6, '线索编号生成器');