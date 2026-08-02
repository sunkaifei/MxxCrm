-- v30: 交易型系统数据库表（阶段5-7）
-- 包含：前台用户、购物车、订单、订单项、发货表
-- 遵循项目规则：TIMESTAMP（非TIMESTAMPTZ）、create_time/update_time、deleted INT、status INT

-- =====================================================
-- 阶段5：前台用户表
-- =====================================================
CREATE TABLE IF NOT EXISTS mxx_website_user (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR(64) NOT NULL,
    password VARCHAR(128) NOT NULL,
    real_name VARCHAR(64),
    phone VARCHAR(20),
    email VARCHAR(128),
    avatar VARCHAR(255),
    gender SMALLINT DEFAULT 0,
    status INT DEFAULT 0,
    member_level INT DEFAULT 0,
    total_points INT DEFAULT 0,
    total_spent DECIMAL(12,2) DEFAULT 0,
    order_count INT DEFAULT 0,
    last_login_time TIMESTAMP,
    last_login_ip VARCHAR(64),
    register_ip VARCHAR(64),
    register_source VARCHAR(32) DEFAULT 'website',
    open_id VARCHAR(64),
    union_id VARCHAR(64),
    remark VARCHAR(255),
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted INT DEFAULT 0
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_website_user_username ON mxx_website_user (username) WHERE deleted = 0;
CREATE INDEX IF NOT EXISTS idx_website_user_phone ON mxx_website_user (phone) WHERE deleted = 0;
CREATE INDEX IF NOT EXISTS idx_website_user_email ON mxx_website_user (email) WHERE deleted = 0;
CREATE INDEX IF NOT EXISTS idx_website_user_open_id ON mxx_website_user (open_id) WHERE deleted = 0;

-- =====================================================
-- 阶段6：购物车表
-- =====================================================
CREATE TABLE IF NOT EXISTS mxx_website_cart (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    product_id BIGINT NOT NULL,
    sku_id BIGINT,
    product_name VARCHAR(255),
    product_image VARCHAR(255),
    sku_code VARCHAR(64),
    sku_specs VARCHAR(512),
    price DECIMAL(12,2) NOT NULL DEFAULT 0,
    quantity INT NOT NULL DEFAULT 1,
    selected INT DEFAULT 1,
    website_id BIGINT,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_cart_user_id ON mxx_website_cart (user_id);

-- =====================================================
-- 阶段6：订单表
-- =====================================================
CREATE TABLE IF NOT EXISTS mxx_website_order (
    id BIGSERIAL PRIMARY KEY,
    order_no VARCHAR(64) NOT NULL,
    user_id BIGINT NOT NULL,
    website_id BIGINT,
    total_amount DECIMAL(12,2) NOT NULL DEFAULT 0,
    discount_amount DECIMAL(12,2) DEFAULT 0,
    shipping_fee DECIMAL(12,2) DEFAULT 0,
    pay_amount DECIMAL(12,2) NOT NULL DEFAULT 0,
    status INT DEFAULT 0,
    pay_status INT DEFAULT 0,
    ship_status INT DEFAULT 0,
    pay_type INT,
    pay_time TIMESTAMP,
    ship_time TIMESTAMP,
    finish_time TIMESTAMP,
    cancel_time TIMESTAMP,
    cancel_reason VARCHAR(255),
    consignee_name VARCHAR(64),
    consignee_phone VARCHAR(20),
    consignee_address VARCHAR(500),
    consignee_province VARCHAR(32),
    consignee_city VARCHAR(32),
    consignee_district VARCHAR(32),
    consignee_zipcode VARCHAR(20),
    buyer_remark VARCHAR(500),
    seller_remark VARCHAR(500),
    transaction_id VARCHAR(64),
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted INT DEFAULT 0
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_order_no ON mxx_website_order (order_no) WHERE deleted = 0;
CREATE INDEX IF NOT EXISTS idx_order_user_id ON mxx_website_order (user_id) WHERE deleted = 0;
CREATE INDEX IF NOT EXISTS idx_order_status ON mxx_website_order (status) WHERE deleted = 0;

-- =====================================================
-- 阶段6：订单项表
-- =====================================================
CREATE TABLE IF NOT EXISTS mxx_website_order_item (
    id BIGSERIAL PRIMARY KEY,
    order_id BIGINT NOT NULL,
    product_id BIGINT NOT NULL,
    sku_id BIGINT,
    product_name VARCHAR(255),
    product_image VARCHAR(255),
    sku_code VARCHAR(64),
    sku_specs VARCHAR(512),
    price DECIMAL(12,2) NOT NULL DEFAULT 0,
    quantity INT NOT NULL DEFAULT 1,
    total_amount DECIMAL(12,2) NOT NULL DEFAULT 0,
    refund_status INT DEFAULT 0,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_order_item_order_id ON mxx_website_order_item (order_id);

-- =====================================================
-- 阶段7：发货表
-- =====================================================
CREATE TABLE IF NOT EXISTS mxx_website_delivery (
    id BIGSERIAL PRIMARY KEY,
    order_id BIGINT NOT NULL,
    order_no VARCHAR(64),
    delivery_no VARCHAR(64),
    delivery_company VARCHAR(64),
    delivery_type INT DEFAULT 1,
    status INT DEFAULT 0,
    shipper_id BIGINT,
    shipper_name VARCHAR(64),
    consignee_name VARCHAR(64),
    consignee_phone VARCHAR(20),
    consignee_address VARCHAR(500),
    item_count INT DEFAULT 0,
    remark VARCHAR(500),
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted INT DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_delivery_order_id ON mxx_website_delivery (order_id) WHERE deleted = 0;
CREATE INDEX IF NOT EXISTS idx_delivery_no ON mxx_website_delivery (delivery_no) WHERE deleted = 0;

-- =====================================================
-- 阶段7：退款表
-- =====================================================
CREATE TABLE IF NOT EXISTS mxx_website_refund (
    id BIGSERIAL PRIMARY KEY,
    refund_no VARCHAR(64) NOT NULL,
    order_id BIGINT NOT NULL,
    order_no VARCHAR(64),
    user_id BIGINT NOT NULL,
    order_item_id BIGINT,
    refund_type INT DEFAULT 1,
    refund_reason VARCHAR(255),
    refund_amount DECIMAL(12,2) NOT NULL DEFAULT 0,
    status INT DEFAULT 0,
    refund_way INT,
    transaction_id VARCHAR(64),
    handle_remark VARCHAR(500),
    handle_by BIGINT,
    handle_time TIMESTAMP,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted INT DEFAULT 0
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_refund_no ON mxx_website_refund (refund_no) WHERE deleted = 0;
CREATE INDEX IF NOT EXISTS idx_refund_order_id ON mxx_website_refund (order_id) WHERE deleted = 0;
CREATE INDEX IF NOT EXISTS idx_refund_user_id ON mxx_website_refund (user_id) WHERE deleted = 0;

-- =====================================================
-- 验证
-- =====================================================
SELECT 'mxx_website_user' as tbl, count(*) as cnt FROM mxx_website_user
UNION ALL SELECT 'mxx_website_cart', count(*) FROM mxx_website_cart
UNION ALL SELECT 'mxx_website_order', count(*) FROM mxx_website_order
UNION ALL SELECT 'mxx_website_order_item', count(*) FROM mxx_website_order_item
UNION ALL SELECT 'mxx_website_delivery', count(*) FROM mxx_website_delivery
UNION ALL SELECT 'mxx_website_refund', count(*) FROM mxx_website_refund;
