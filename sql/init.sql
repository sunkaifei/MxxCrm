-- ================================================
-- Mxx-CRM 数据库初始化脚本 (PostgreSQL)
-- 表名统一前缀: mxx_
-- 系统权限模块: mxx_system_*
-- 关联表: mxx_system_*_merge
-- ================================================

-- 切换到目标数据库
-- \c mxx_crm;

-- ================================================
-- 1. 枚举类型定义
-- ================================================

CREATE TYPE mxx_user_status        AS ENUM ('active', 'disabled', 'pending');
CREATE TYPE mxx_lead_status        AS ENUM ('new', 'following', 'converted', 'invalid', 'recycled');
CREATE TYPE mxx_lead_source        AS ENUM ('website', 'exhibition', 'social', 'referral', 'cold_call', 'customs', 'email', 'alibaba', 'amazon', 'tiktok', 'wechat', 'other');
CREATE TYPE mxx_customer_level     AS ENUM ('potential', 'normal', 'vip', 'svip', 'lost');
CREATE TYPE mxx_opportunity_stage  AS ENUM ('qualification', 'needs_analysis', 'proposal', 'negotiation', 'won', 'lost');
CREATE TYPE mxx_contract_status    AS ENUM ('draft', 'signed', 'executing', 'completed', 'terminated');
CREATE TYPE mxx_order_status       AS ENUM ('pending', 'confirmed', 'paid', 'shipping', 'delivered', 'completed', 'cancelled', 'refunded');
CREATE TYPE mxx_payment_status     AS ENUM ('unpaid', 'partial', 'paid', 'overdue');
CREATE TYPE mxx_payment_method     AS ENUM ('cash', 'bank_transfer', 'credit_card', 'paypal', 'wechat_pay', 'alipay', 'tt', 'lc', 'other');
CREATE TYPE mxx_purchase_status    AS ENUM ('draft', 'ordered', 'in_transit', 'received', 'completed', 'cancelled');
CREATE TYPE mxx_inventory_op_type  AS ENUM ('in', 'out', 'transfer', 'adjust');
CREATE TYPE mxx_currency_code      AS ENUM ('CNY', 'USD', 'EUR', 'GBP', 'JPY', 'HKD', 'AUD');
CREATE TYPE mxx_gender             AS ENUM ('male', 'female', 'other');
CREATE TYPE mxx_industry_type      AS ENUM ('retail', 'wholesale', 'manufacturer', 'trade_agent', 'ecommerce', 'wechat_business', 'other');
CREATE TYPE mxx_activity_type      AS ENUM ('call', 'email', 'meeting', 'visit', 'whatsapp', 'wechat', 'other');

-- ================================================
-- 2. 系统权限模块表 (mxx_system_*)
-- ================================================

-- 2.1 mxx_system_admin（管理员表）
CREATE TABLE mxx_system_admin (
    id               BIGSERIAL PRIMARY KEY,
    user_name        VARCHAR(64) NOT NULL UNIQUE,
    nick_name        VARCHAR(64),
    user_type        INT DEFAULT 0,           -- 0普通用户，1超级管理员
    email            VARCHAR(128),
    mobile           VARCHAR(32),
    gender           INT DEFAULT 2,           -- 0男 1女 2未知
    avatar           VARCHAR(255),
    password         VARCHAR(255) NOT NULL,
    status           INT DEFAULT 0,           -- 0正常 1停用
    deleted          INT DEFAULT 0,           -- 0未删除 1已删除
    login_ip         VARCHAR(64),
    login_date       TIMESTAMPTZ,
    create_by        VARCHAR(64),
    create_time      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    update_by        VARCHAR(64),
    update_time      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    remark           VARCHAR(255),
    sort             INT DEFAULT 0
);
CREATE INDEX idx_sys_admin_username ON mxx_system_admin(user_name);

-- 2.2 mxx_system_dept（部门表）
CREATE TABLE mxx_system_dept (
    id          BIGSERIAL PRIMARY KEY,
    parent_id   BIGINT REFERENCES mxx_system_dept(id),
    ancestors   VARCHAR(512),
    dept_name   VARCHAR(64) NOT NULL,
    code        VARCHAR(64),
    sort        INT DEFAULT 0,
    leader      VARCHAR(64),
    phone       VARCHAR(32),
    email       VARCHAR(128),
    status      INT DEFAULT 0,           -- 0正常 1停用
    del_flag    INT DEFAULT 0,           -- 0存在 2删除
    create_by   VARCHAR(64),
    create_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    update_by   VARCHAR(64),
    update_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_sys_dept_parent ON mxx_system_dept(parent_id);

-- 2.3 mxx_system_post（岗位表）
CREATE TABLE mxx_system_post (
    id          BIGSERIAL PRIMARY KEY,
    post_code   VARCHAR(64),
    post_name   VARCHAR(64) NOT NULL,
    status      INT DEFAULT 0,           -- 0正常 1停用
    sort        INT DEFAULT 0,
    remark      VARCHAR(255),
    create_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted     INT DEFAULT 0            -- 0未删除 1已删除
);
CREATE INDEX idx_sys_post_code ON mxx_system_post(post_code);

-- 2.4 mxx_system_role（角色表）
CREATE TABLE mxx_system_role (
    id          BIGSERIAL PRIMARY KEY,
    role_name   VARCHAR(64) NOT NULL,
    role_key    VARCHAR(64),
    sort        INT DEFAULT 0,
    data_scope  INT DEFAULT 1,           -- 1全部数据权限 2自定义 3本部门 4本部门及以下
    status      INT DEFAULT 0,           -- 0停用 1正常
    deleted     INT DEFAULT 0,           -- 0存在 2删除
    create_by   VARCHAR(64),
    create_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    update_by   VARCHAR(64),
    update_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    remark      VARCHAR(255)
);
CREATE INDEX idx_sys_role_key ON mxx_system_role(role_key);

-- 2.5 mxx_system_menu（菜单表）
CREATE TABLE mxx_system_menu (
    id                   BIGSERIAL PRIMARY KEY,
    parent_id            BIGINT DEFAULT 0,
    tree_path            VARCHAR(512),
    name                 VARCHAR(128) NOT NULL,
    type                 VARCHAR(16),      -- M目录 C菜单 F按钮
    route_name           VARCHAR(128),
    path                 VARCHAR(255),
    component            VARCHAR(255),
    perm                 VARCHAR(128),     -- 权限标识，用于控制器注解权限校验
    status               INT DEFAULT 0,     -- 0正常 1停用
    affix_tab            INT DEFAULT 0,     -- 是否固定标签页
    hide_children_in_menu INT DEFAULT 0,
    hide_in_breadcrumb   INT DEFAULT 0,
    hide_in_menu         INT DEFAULT 0,
    hide_in_tab          INT DEFAULT 0,
    keep_alive           INT DEFAULT 0,
    sort                 INT DEFAULT 0,
    icon                 VARCHAR(128),
    redirect             VARCHAR(255),
    params               JSONB,
    create_time          TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_time         TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted          INT DEFAULT 0         -- 0未删除 1已删除
);
CREATE INDEX idx_sys_menu_parent ON mxx_system_menu(parent_id);
CREATE INDEX idx_sys_menu_type   ON mxx_system_menu(type);

-- 2.6 mxx_system_config（系统配置表）
CREATE TABLE mxx_system_config (
    id          BIGSERIAL PRIMARY KEY,
    config_name VARCHAR(128),
    config_key  VARCHAR(128) UNIQUE NOT NULL,
    config_value VARCHAR(2048),
    config_type VARCHAR(16),       -- Y系统内置 N自定义
    remark      VARCHAR(255),
    sort        INT DEFAULT 0,
    create_by   VARCHAR(64),
    create_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    update_by   VARCHAR(64),
    update_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_sys_config_key ON mxx_system_config(config_key);

-- 2.7 mxx_system_dict（字典表）
CREATE TABLE mxx_system_dict (
    id          BIGSERIAL PRIMARY KEY,
    dict_name   VARCHAR(64) NOT NULL,
    dict_code   VARCHAR(64) UNIQUE NOT NULL,
    sort        INT DEFAULT 0,
    status      INT DEFAULT 0,           -- 0正常 1停用
    create_by   VARCHAR(64),
    create_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    update_by   VARCHAR(64),
    update_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    remark      VARCHAR(255)
);
CREATE INDEX idx_sys_dict_code ON mxx_system_dict(dict_code);

-- 2.8 mxx_system_dict_data（字典数据表）
CREATE TABLE mxx_system_dict_data (
    id          BIGSERIAL PRIMARY KEY,
    dict_code   VARCHAR(64),
    dict_label  VARCHAR(128),
    dict_value  VARCHAR(128),
    sort        INT DEFAULT 0,
    css_class   VARCHAR(128),
    list_class  VARCHAR(128),
    is_default  INT DEFAULT 0,
    status      INT DEFAULT 0,           -- 0正常 1停用
    create_by   VARCHAR(64),
    create_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    update_by   VARCHAR(64),
    update_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    remark      VARCHAR(255)
);
CREATE INDEX idx_sys_dict_data_code ON mxx_system_dict_data(dict_code);

-- 2.9 mxx_system_log（系统日志表）
CREATE TABLE mxx_system_log (
    id              BIGSERIAL PRIMARY KEY,
    title           VARCHAR(255),
    business_type   INT DEFAULT 0,       -- 0其它 1新增 2修改 3删除
    method          VARCHAR(255),
    request_method  VARCHAR(16),
    operator_type   INT DEFAULT 0,       -- 0其它 1后台用户 2手机端用户
    oper_name       VARCHAR(64),
    dept_name       VARCHAR(64),
    oper_url        VARCHAR(255),
    oper_ip         VARCHAR(64),
    oper_location   VARCHAR(255),
    oper_param      TEXT,
    json_result     TEXT,
    status          INT DEFAULT 0,       -- 0正常 1异常
    error_msg       TEXT,
    create_time     TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_sys_log_time   ON mxx_system_log(create_time DESC);
CREATE INDEX idx_sys_log_oper   ON mxx_system_log(oper_name);

-- 2.10 mxx_system_region（地区表）
CREATE TABLE mxx_system_region (
    id          BIGSERIAL PRIMARY KEY,
    parent_id   BIGINT DEFAULT 0,
    name        VARCHAR(128) NOT NULL,
    code        VARCHAR(64),
    zip_code    VARCHAR(32),
    level       INT DEFAULT 1,           -- 1省 2市 3区县
    sort        INT DEFAULT 0,
    create_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_sys_region_parent ON mxx_system_region(parent_id);
CREATE INDEX idx_sys_region_level ON mxx_system_region(level);

-- 2.11 mxx_system_ip_address（IP地址白名单表）
CREATE TABLE mxx_system_ip_address (
    id          BIGSERIAL PRIMARY KEY,
    ip_addr     VARCHAR(64) UNIQUE NOT NULL,
    ip_name     VARCHAR(128),
    status      INT DEFAULT 0,           -- 0禁用 1启用
    create_by   VARCHAR(64),
    create_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    update_by   VARCHAR(64),
    update_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    remark      VARCHAR(255)
);
CREATE INDEX idx_sys_ip_addr ON mxx_system_ip_address(ip_addr);

-- ================================================
-- 3. 关联表 (mxx_system_*_merge)
-- ================================================

-- 3.1 mxx_system_admin_role_merge（用户角色关联表）
CREATE TABLE mxx_system_admin_role_merge (
    id          BIGSERIAL PRIMARY KEY,
    admin_id    BIGINT REFERENCES mxx_system_admin(id) ON DELETE CASCADE,
    role_id     BIGINT REFERENCES mxx_system_role(id) ON DELETE CASCADE,
    create_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(admin_id, role_id)
);
CREATE INDEX idx_sys_admin_role_admin ON mxx_system_admin_role_merge(admin_id);
CREATE INDEX idx_sys_admin_role_role  ON mxx_system_admin_role_merge(role_id);

-- 3.2 mxx_system_admin_dept_merge（用户部门关联表）
CREATE TABLE mxx_system_admin_dept_merge (
    id          BIGSERIAL PRIMARY KEY,
    admin_id    BIGINT REFERENCES mxx_system_admin(id) ON DELETE CASCADE,
    dept_id     BIGINT REFERENCES mxx_system_dept(id) ON DELETE CASCADE,
    create_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(admin_id, dept_id)
);
CREATE INDEX idx_sys_admin_dept_admin ON mxx_system_admin_dept_merge(admin_id);
CREATE INDEX idx_sys_admin_dept_dept  ON mxx_system_admin_dept_merge(dept_id);

-- 3.3 mxx_system_admin_post_merge（用户岗位关联表）
CREATE TABLE mxx_system_admin_post_merge (
    id          BIGSERIAL PRIMARY KEY,
    admin_id    BIGINT REFERENCES mxx_system_admin(id) ON DELETE CASCADE,
    post_id     BIGINT REFERENCES mxx_system_post(id) ON DELETE CASCADE,
    create_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(admin_id, post_id)
);
CREATE INDEX idx_sys_admin_post_admin ON mxx_system_admin_post_merge(admin_id);
CREATE INDEX idx_sys_admin_post_post  ON mxx_system_admin_post_merge(post_id);

-- 3.4 mxx_system_role_menu_merge（角色菜单关联表）
CREATE TABLE mxx_system_role_menu_merge (
    id          BIGSERIAL PRIMARY KEY,
    role_id     BIGINT REFERENCES mxx_system_role(id) ON DELETE CASCADE,
    menu_id     BIGINT REFERENCES mxx_system_menu(id) ON DELETE CASCADE,
    status      INT DEFAULT 0,
    create_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(role_id, menu_id)
);
CREATE INDEX idx_sys_role_menu_role  ON mxx_system_role_menu_merge(role_id);
CREATE INDEX idx_sys_role_menu_menu  ON mxx_system_role_menu_merge(menu_id);

-- 3.5 mxx_system_dept_menu_merge（部门菜单关联表）
CREATE TABLE mxx_system_dept_menu_merge (
    id          BIGSERIAL PRIMARY KEY,
    dept_id     BIGINT REFERENCES mxx_system_dept(id) ON DELETE CASCADE,
    menu_id     BIGINT REFERENCES mxx_system_menu(id) ON DELETE CASCADE,
    create_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(dept_id, menu_id)
);
CREATE INDEX idx_sys_dept_menu_dept  ON mxx_system_dept_menu_merge(dept_id);
CREATE INDEX idx_sys_dept_menu_menu  ON mxx_system_dept_menu_merge(menu_id);

-- 3.6 mxx_system_admin_notice_merge（用户通知关联表）
CREATE TABLE mxx_system_admin_notice_merge (
    id          BIGSERIAL PRIMARY KEY,
    notice_id   BIGINT REFERENCES mxx_notice(id) ON DELETE CASCADE,
    user_id     BIGINT REFERENCES mxx_system_admin(id) ON DELETE CASCADE,
    is_read     INT DEFAULT 0,           -- 0未读 1已读
    read_time   TIMESTAMPTZ,
    create_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    is_deleted  INT DEFAULT 0,           -- 0未删除 1已删除
    UNIQUE(notice_id, user_id)
);
CREATE INDEX idx_sys_admin_notice_notice ON mxx_system_admin_notice_merge(notice_id);
CREATE INDEX idx_sys_admin_notice_user  ON mxx_system_admin_notice_merge(user_id);

-- ================================================
-- 4. 通知模块表 (mxx_notice_*)
-- ================================================

-- 4.1 mxx_notice（通知表）
CREATE TABLE mxx_notice (
    id              BIGSERIAL PRIMARY KEY,
    title           VARCHAR(255) NOT NULL,
    content         TEXT,
    type            INT,                    -- 通知类型（关联字典）
    level           VARCHAR(16),            -- 通知等级
    target_type     INT DEFAULT 1,          -- 1全体 2指定
    target_user_ids VARCHAR(1024),          -- 目标用户ID列表
    publisher_id    BIGINT REFERENCES mxx_system_admin(id),
    publish_status  INT DEFAULT 0,          -- 0未发布 1已发布 -1已撤回
    publish_time    TIMESTAMPTZ,
    revoke_time     TIMESTAMPTZ,
    create_by       BIGINT REFERENCES mxx_system_admin(id),
    create_time     TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    update_by       BIGINT REFERENCES mxx_system_admin(id),
    update_time     TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    is_deleted      INT DEFAULT 0           -- 0未删除 1已删除
);
CREATE INDEX idx_notice_publish_status ON mxx_notice(publish_status);

-- ================================================
-- 5. CRM客户与销售模块表 (mxx_crm_*)
-- ================================================

-- 5.1 mxx_crm_lead（线索表）
CREATE TABLE mxx_crm_lead (
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
    converted_to_customer_id BIGINT REFERENCES mxx_crm_customer(id),
    converted_at        TIMESTAMPTZ,
    description         TEXT,
    custom_fields       JSONB,
    created_by          BIGINT REFERENCES mxx_system_admin(id),
    created_at          TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_by          BIGINT REFERENCES mxx_system_admin(id),
    updated_at          TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_crm_lead_assigned ON mxx_crm_lead(assigned_to);
CREATE INDEX idx_crm_lead_status   ON mxx_crm_lead(status);
CREATE INDEX idx_crm_lead_source   ON mxx_crm_lead(source);

-- 5.2 mxx_crm_customer（客户表）
CREATE TABLE mxx_crm_customer (
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
    deleted_at          TIMESTAMPTZ
);
CREATE INDEX idx_crm_customer_no    ON mxx_crm_customer(customer_no);
CREATE INDEX idx_crm_customer_level ON mxx_crm_customer(level);

-- 5.3 mxx_crm_contact（联系人表）
CREATE TABLE mxx_crm_contact (
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
    deleted_at      TIMESTAMPTZ
);
CREATE INDEX idx_crm_contact_customer ON mxx_crm_contact(customer_id);

-- 5.4 mxx_crm_followup（跟进记录表）
CREATE TABLE mxx_crm_followup (
    id              BIGSERIAL PRIMARY KEY,
    lead_id         BIGINT REFERENCES mxx_crm_lead(id) ON DELETE CASCADE,
    customer_id     BIGINT REFERENCES mxx_crm_customer(id) ON DELETE CASCADE,
    opportunity_id  BIGINT REFERENCES mxx_crm_opportunity(id) ON DELETE CASCADE,
    activity_type   mxx_activity_type,
    subject         VARCHAR(255),
    content         TEXT,
    next_follow_date DATE,
    duration_minutes INT,
    outcome         VARCHAR(255),
    assigned_to     BIGINT REFERENCES mxx_system_admin(id),
    created_by      BIGINT REFERENCES mxx_system_admin(id),
    created_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_crm_followup_lead        ON mxx_crm_followup(lead_id);
CREATE INDEX idx_crm_followup_customer    ON mxx_crm_followup(customer_id);
CREATE INDEX idx_crm_followup_opportunity ON mxx_crm_followup(opportunity_id);

-- 5.5 mxx_crm_lead_pool（线索池表）
CREATE TABLE mxx_crm_lead_pool (
    id             BIGSERIAL PRIMARY KEY,
    name           VARCHAR(128) NOT NULL,
    description    TEXT,
    recycle_days   INT DEFAULT 30,
    member_ids     BIGINT[],
    created_at     TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 5.6 mxx_crm_opportunity（商机表）
CREATE TABLE mxx_crm_opportunity (
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
    deleted_at      TIMESTAMPTZ
);
CREATE INDEX idx_crm_opportunity_customer ON mxx_crm_opportunity(customer_id);
CREATE INDEX idx_crm_opportunity_stage    ON mxx_crm_opportunity(stage);

-- 5.7 mxx_crm_contract（合同表）
CREATE TABLE mxx_crm_contract (
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
    deleted_at      TIMESTAMPTZ
);
CREATE INDEX idx_crm_contract_customer    ON mxx_crm_contract(customer_id);
CREATE INDEX idx_crm_contract_status      ON mxx_crm_contract(status);

-- ================================================
-- 6. 订单与支付模块表 (mxx_sale_*)
-- ================================================

-- 6.1 mxx_sale_order（订单表）
CREATE TABLE mxx_sale_order (
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
    deleted_at      TIMESTAMPTZ
);
CREATE INDEX idx_sale_order_customer ON mxx_sale_order(customer_id);
CREATE INDEX idx_sale_order_status   ON mxx_sale_order(status);

-- 6.2 mxx_sale_order_item（订单明细表）
CREATE TABLE mxx_sale_order_item (
    id              BIGSERIAL PRIMARY KEY,
    order_id        BIGINT REFERENCES mxx_sale_order(id) ON DELETE CASCADE,
    product_id      BIGINT REFERENCES mxx_product_main(id),
    sku             VARCHAR(64),
    product_name    VARCHAR(255),
    quantity        NUMERIC(10,2) NOT NULL,
    unit_price      NUMERIC(15,2) NOT NULL,
    discount        NUMERIC(15,2) DEFAULT 0,
    tax_rate        NUMERIC(5,2) DEFAULT 0,
    total_amount    NUMERIC(18,2),
    notes           TEXT,
    created_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_sale_order_item_order   ON mxx_sale_order_item(order_id);
CREATE INDEX idx_sale_order_item_product ON mxx_sale_order_item(product_id);

-- 6.3 mxx_sale_payment（支付记录表）
CREATE TABLE mxx_sale_payment (
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
    updated_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_sale_payment_order    ON mxx_sale_payment(order_id);
CREATE INDEX idx_sale_payment_customer ON mxx_sale_payment(customer_id);

-- ================================================
-- 7. 产品与库存模块表 (mxx_product_* / mxx_inventory_*)
-- ================================================

-- 7.1 mxx_product_main（产品主表）
CREATE TABLE mxx_product_main (
    id              BIGSERIAL PRIMARY KEY,
    product_no      VARCHAR(32) UNIQUE NOT NULL,
    name            VARCHAR(255) NOT NULL,
    category_id     BIGINT REFERENCES mxx_product_category(id),
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
    deleted_at      TIMESTAMPTZ
);
CREATE INDEX idx_product_no       ON mxx_product_main(product_no);
CREATE INDEX idx_product_category ON mxx_product_main(category_id);

-- 7.2 mxx_product_category（产品分类表）
CREATE TABLE mxx_product_category (
    id              BIGSERIAL PRIMARY KEY,
    parent_id       BIGINT REFERENCES mxx_product_category(id),
    name            VARCHAR(128) NOT NULL,
    description     TEXT,
    sort_order      INT DEFAULT 0,
    created_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted_at      TIMESTAMPTZ
);
CREATE INDEX idx_product_category_parent ON mxx_product_category(parent_id);

-- 7.3 mxx_inventory_warehouse（仓库表）
CREATE TABLE mxx_inventory_warehouse (
    id              BIGSERIAL PRIMARY KEY,
    name            VARCHAR(128) NOT NULL,
    code            VARCHAR(32) UNIQUE,
    address         VARCHAR(255),
    contact_person  VARCHAR(64),
    contact_phone   VARCHAR(32),
    is_active       BOOLEAN DEFAULT true,
    created_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 7.4 mxx_inventory_stock（库存表）
CREATE TABLE mxx_inventory_stock (
    id                  BIGSERIAL PRIMARY KEY,
    product_id          BIGINT REFERENCES mxx_product_main(id),
    warehouse_id        BIGINT REFERENCES mxx_inventory_warehouse(id),
    quantity            NUMERIC(15,2) DEFAULT 0,
    reserved_quantity   NUMERIC(15,2) DEFAULT 0,
    available_quantity  NUMERIC(15,2) GENERATED ALWAYS AS (quantity - reserved_quantity) STORED,
    last_updated_at     TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(product_id, warehouse_id)
);
CREATE INDEX idx_inventory_product   ON mxx_inventory_stock(product_id);
CREATE INDEX idx_inventory_warehouse ON mxx_inventory_stock(warehouse_id);

-- 7.5 mxx_inventory_transaction（库存流水表）
CREATE TABLE mxx_inventory_transaction (
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
CREATE INDEX idx_inv_trans_product   ON mxx_inventory_transaction(product_id);
CREATE INDEX idx_inv_trans_date     ON mxx_inventory_transaction(created_at DESC);

-- ================================================
-- 8. 供应商与采购模块表 (mxx_purchase_*)
-- ================================================

-- 8.1 mxx_purchase_supplier（供应商表）
CREATE TABLE mxx_purchase_supplier (
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
    level           INT4,
    status          INT4,
    notes           TEXT,
    is_active       BOOLEAN DEFAULT true,
    created_by      BIGINT REFERENCES mxx_system_admin(id),
    created_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_by      BIGINT REFERENCES mxx_system_admin(id),
    updated_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted_at      TIMESTAMPTZ
);
CREATE INDEX idx_purchase_supplier_no ON mxx_purchase_supplier(supplier_no);

-- 8.2 mxx_purchase_po（采购单表）
CREATE TABLE mxx_purchase_po (
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
    updated_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_purchase_po_supplier ON mxx_purchase_po(supplier_id);
CREATE INDEX idx_purchase_po_status   ON mxx_purchase_po(status);

-- 8.3 mxx_purchase_item（采购明细表）
CREATE TABLE mxx_purchase_item (
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
    created_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_purchase_item_purchase ON mxx_purchase_item(purchase_id);
CREATE INDEX idx_purchase_item_product  ON mxx_purchase_item(product_id);

-- ================================================
-- 9. 附件模块表 (mxx_attachment_*)
-- ================================================

-- 9.1 mxx_attachment_file（附件表）
CREATE TABLE mxx_attachment_file (
    id              BIGSERIAL PRIMARY KEY,
    file_name       VARCHAR(255) NOT NULL,
    file_path       VARCHAR(512) NOT NULL,
    file_size       BIGINT,
    file_type       VARCHAR(128),
    mime_type       VARCHAR(128),
    related_type    VARCHAR(64),
    related_id      BIGINT,
    uploaded_by     BIGINT REFERENCES mxx_system_admin(id),
    created_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_attachment_related ON mxx_attachment_file(related_type, related_id);

-- ================================================
-- 10. 序列号生成器表 (mxx_seq_*)
-- ================================================

-- 10.1 mxx_seq_generator（序列号生成器）
CREATE TABLE mxx_seq_generator (
    id              BIGSERIAL PRIMARY KEY,
    name            VARCHAR(64) UNIQUE NOT NULL,
    prefix          VARCHAR(16),
    current_value   BIGINT DEFAULT 0,
    date_format     VARCHAR(32),
    padding         INT DEFAULT 4,
    description     VARCHAR(255),
    updated_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ================================================
-- 11. 权限控制初始化数据
-- ================================================

-- 11.1 初始化默认角色
INSERT INTO mxx_system_role (role_name, role_key, sort, data_scope, status, deleted, remark) VALUES
('超级管理员', 'admin', 1, 1, 1, 0, '超级管理员拥有系统所有权限'),
('系统管理员', 'system', 2, 1, 1, 0, '系统管理员负责系统配置管理'),
('销售经理', 'sales_manager', 3, 3, 1, 0, '销售经理管理销售团队'),
('销售人员', 'sales', 4, 3, 1, 0, '销售人员负责客户开发和维护'),
('采购专员', 'purchase', 5, 3, 1, 0, '采购专员负责采购业务');

-- 11.2 初始化默认部门
INSERT INTO mxx_system_dept (parent_id, ancestors, dept_name, code, sort, leader, status) VALUES
(0, '0', '总经办', '001', 1, '总经理', 0),
(1, '0,1', '销售部', '002', 2, '销售总监', 0),
(1, '0,1', '采购部', '003', 3, '采购经理', 0),
(1, '0,1', '财务部', '004', 4, '财务总监', 0),
(1, '0,1', '技术部', '005', 5, '技术总监', 0),
(2, '0,1,2', '销售一组', '00201', 1, '销售组长', 0),
(2, '0,1,2', '销售二组', '00202', 2, '销售组长', 0);

-- 11.3 初始化默认岗位
INSERT INTO mxx_system_post (post_code, post_name, status, sort) VALUES
('admin', '超级管理员', 0, 1),
('manager', '部门经理', 0, 2),
('supervisor', '主管', 0, 3),
('staff', '员工', 0, 4);

-- 11.4 初始化默认管理员（密码：admin123，已加密）
INSERT INTO mxx_system_admin (user_name, nick_name, user_type, email, status, deleted, password, create_time) VALUES
('admin', '超级管理员', 1, 'admin@mxxshop.com', 0, 0, '$2a$10$tQ5w5Gz7aQ7z7A7a8w8A8aQ5w5Gz7aQ7z7A7a8w8A8a', CURRENT_TIMESTAMP),
('system', '系统管理员', 0, 'system@mxxshop.com', 0, 0, '$2a$10$tQ5w5Gz7aQ7z7A7a8w8A8aQ5w5Gz7aQ7z7A7a8w8A8a', CURRENT_TIMESTAMP),
('sales_manager', '销售经理', 0, 'sales_manager@mxxshop.com', 0, 0, '$2a$10$tQ5w5Gz7aQ7z7A7a8w8A8aQ5w5Gz7aQ7z7A7a8w8A8a', CURRENT_TIMESTAMP),
('sales_staff', '销售人员', 0, 'sales_staff@mxxshop.com', 0, 0, '$2a$10$tQ5w5Gz7aQ7z7A7a8w8A8aQ5w5Gz7aQ7z7A7a8w8A8a', CURRENT_TIMESTAMP),
('purchase_staff', '采购专员', 0, 'purchase_staff@mxxshop.com', 0, 0, '$2a$10$tQ5w5Gz7aQ7z7A7a8w8A8aQ5w5Gz7aQ7z7A7a8w8A8a', CURRENT_TIMESTAMP),
('crm_manager', 'CRM管理员', 0, 'crm_manager@mxxshop.com', 0, 0, '$2a$10$tQ5w5Gz7aQ7z7A7a8w8A8aQ5w5Gz7aQ7z7A7a8w8A8a', CURRENT_TIMESTAMP),
('product_staff', '产品专员', 0, 'product_staff@mxxshop.com', 0, 0, '$2a$10$tQ5w5Gz7aQ7z7A7a8w8A8aQ5w5Gz7aQ7z7A7a8w8A8a', CURRENT_TIMESTAMP),
('notice_staff', '通知专员', 0, 'notice_staff@mxxshop.com', 0, 0, '$2a$10$tQ5w5Gz7aQ7z7A7a8w8A8aQ5w5Gz7aQ7z7A7a8w8A8a', CURRENT_TIMESTAMP);

-- 11.5 初始化系统菜单（与控制器注解权限对应）
INSERT INTO mxx_system_menu (parent_id, tree_path, name, type, route_name, path, component, perm, sort, icon) VALUES
-- 系统管理
(0, '0', '系统管理', 'M', 'system', '/system', '', '', 1, 'system'),
  -- 用户管理
  (1, '0,1', '用户管理', 'C', 'system-admin', '/system/admin', 'system/admin/index', '', 1, 'user'),
    (2, '0,1,2', '查询用户', 'F', '', '', '', 'system:admin:list', 1, ''),
    (2, '0,1,2', '新增用户', 'F', '', '', '', 'system:admin:create', 2, ''),
    (2, '0,1,2', '编辑用户', 'F', '', '', '', 'system:admin:update', 3, ''),
    (2, '0,1,2', '删除用户', 'F', '', '', '', 'system:admin:delete', 4, ''),
    (2, '0,1,2', '重置密码', 'F', '', '', '', 'system:admin:resetPwd', 5, ''),
  -- 角色管理
  (1, '0,1', '角色管理', 'C', 'system-role', '/system/role', 'system/role/index', '', 2, 'role'),
    (5, '0,1,5', '查询角色', 'F', '', '', '', 'system:role:list', 1, ''),
    (5, '0,1,5', '新增角色', 'F', '', '', '', 'system:role:create', 2, ''),
    (5, '0,1,5', '编辑角色', 'F', '', '', '', 'system:role:update', 3, ''),
    (5, '0,1,5', '删除角色', 'F', '', '', '', 'system:role:delete', 4, ''),
    (5, '0,1,5', '分配权限', 'F', '', '', '', 'system:role:assignPerm', 5, ''),
  -- 菜单管理
  (1, '0,1', '菜单管理', 'C', 'system-menu', '/system/menu', 'system/menu/index', '', 3, 'menu'),
    (8, '0,1,8', '查询菜单', 'F', '', '', '', 'system:menu:list', 1, ''),
    (8, '0,1,8', '新增菜单', 'F', '', '', '', 'system:menu:create', 2, ''),
    (8, '0,1,8', '编辑菜单', 'F', '', '', '', 'system:menu:update', 3, ''),
    (8, '0,1,8', '删除菜单', 'F', '', '', '', 'system:menu:delete', 4, ''),
  -- 部门管理
  (1, '0,1', '部门管理', 'C', 'system-dept', '/system/dept', 'system/dept/index', '', 4, 'dept'),
    (11, '0,1,11', '查询部门', 'F', '', '', '', 'system:dept:list', 1, ''),
    (11, '0,1,11', '新增部门', 'F', '', '', '', 'system:dept:create', 2, ''),
    (11, '0,1,11', '编辑部门', 'F', '', '', '', 'system:dept:update', 3, ''),
    (11, '0,1,11', '删除部门', 'F', '', '', '', 'system:dept:delete', 4, ''),
  -- 岗位管理
  (1, '0,1', '岗位管理', 'C', 'system-post', '/system/post', 'system/post/index', '', 5, 'post'),
    (14, '0,1,14', '查询岗位', 'F', '', '', '', 'system:post:list', 1, ''),
    (14, '0,1,14', '新增岗位', 'F', '', '', '', 'system:post:create', 2, ''),
    (14, '0,1,14', '编辑岗位', 'F', '', '', '', 'system:post:update', 3, ''),
    (14, '0,1,14', '删除岗位', 'F', '', '', '', 'system:post:delete', 4, ''),
  -- 字典管理
  (1, '0,1', '字典管理', 'C', 'system-dict', '/system/dict', 'system/dict/index', '', 6, 'dict'),
    (17, '0,1,17', '查询字典', 'F', '', '', '', 'system:dict:list', 1, ''),
    (17, '0,1,17', '新增字典', 'F', '', '', '', 'system:dict:create', 2, ''),
    (17, '0,1,17', '编辑字典', 'F', '', '', '', 'system:dict:update', 3, ''),
    (17, '0,1,17', '删除字典', 'F', '', '', '', 'system:dict:delete', 4, ''),
  -- 配置管理
  (1, '0,1', '配置管理', 'C', 'system-config', '/system/config', 'system/config/index', '', 7, 'config'),
    (20, '0,1,20', '查询配置', 'F', '', '', '', 'system:config:list', 1, ''),
    (20, '0,1,20', '新增配置', 'F', '', '', '', 'system:config:create', 2, ''),
    (20, '0,1,20', '编辑配置', 'F', '', '', '', 'system:config:update', 3, ''),
    (20, '0,1,20', '删除配置', 'F', '', '', '', 'system:config:delete', 4, ''),
  -- 系统日志
  (1, '0,1', '系统日志', 'C', 'system-log', '/system/log', 'system/log/index', '', 8, 'log'),
    (23, '0,1,23', '查询日志', 'F', '', '', '', 'system:log:list', 1, ''),

-- CRM客户管理
(0, '0', 'CRM客户管理', 'M', 'crm', '/crm', '', '', 2, 'customer'),
  -- 线索管理
  (25, '0,25', '线索管理', 'C', 'crm-lead', '/crm/lead', 'crm/lead/index', '', 1, 'lead'),
    (26, '0,25,26', '查询线索', 'F', '', '', '', 'crm:lead:list', 1, ''),
    (26, '0,25,26', '新增线索', 'F', '', '', '', 'crm:lead:create', 2, ''),
    (26, '0,25,26', '编辑线索', 'F', '', '', '', 'crm:lead:update', 3, ''),
    (26, '0,25,26', '删除线索', 'F', '', '', '', 'crm:lead:delete', 4, ''),
    (26, '0,25,26', '转化客户', 'F', '', '', '', 'crm:lead:convert', 5, ''),
  -- 客户管理
  (25, '0,25', '客户管理', 'C', 'crm-customer', '/crm/customer', 'crm/customer/index', '', 2, 'customers'),
    (31, '0,25,31', '查询客户', 'F', '', '', '', 'crm:customer:list', 1, ''),
    (31, '0,25,31', '新增客户', 'F', '', '', '', 'crm:customer:create', 2, ''),
    (31, '0,25,31', '编辑客户', 'F', '', '', '', 'crm:customer:update', 3, ''),
    (31, '0,25,31', '删除客户', 'F', '', '', '', 'crm:customer:delete', 4, ''),
  -- 联系人管理
  (25, '0,25', '联系人管理', 'C', 'crm-contact', '/crm/contact', 'crm/contact/index', '', 3, 'contact'),
    (36, '0,25,36', '查询联系人', 'F', '', '', '', 'crm:contact:list', 1, ''),
    (36, '0,25,36', '新增联系人', 'F', '', '', '', 'crm:contact:create', 2, ''),
    (36, '0,25,36', '编辑联系人', 'F', '', '', '', 'crm:contact:update', 3, ''),
    (36, '0,25,36', '删除联系人', 'F', '', '', '', 'crm:contact:delete', 4, ''),
  -- 商机管理
  (25, '0,25', '商机管理', 'C', 'crm-opportunity', '/crm/opportunity', 'crm/opportunity/index', '', 4, 'opportunity'),
    (41, '0,25,41', '查询商机', 'F', '', '', '', 'crm:opportunity:list', 1, ''),
    (41, '0,25,41', '新增商机', 'F', '', '', '', 'crm:opportunity:create', 2, ''),
    (41, '0,25,41', '编辑商机', 'F', '', '', '', 'crm:opportunity:update', 3, ''),
    (41, '0,25,41', '删除商机', 'F', '', '', '', 'crm:opportunity:delete', 4, ''),
  -- 合同管理
  (25, '0,25', '合同管理', 'C', 'crm-contract', '/crm/contract', 'crm/contract/index', '', 5, 'contract'),
    (46, '0,25,46', '查询合同', 'F', '', '', '', 'crm:contract:list', 1, ''),
    (46, '0,25,46', '新增合同', 'F', '', '', '', 'crm:contract:create', 2, ''),
    (46, '0,25,46', '编辑合同', 'F', '', '', '', 'crm:contract:update', 3, ''),
    (46, '0,25,46', '删除合同', 'F', '', '', '', 'crm:contract:delete', 4, ''),
  -- 跟进记录
  (25, '0,25', '跟进记录', 'C', 'crm-followup', '/crm/followup', 'crm/followup/index', '', 6, 'followup'),
    (51, '0,25,51', '查询记录', 'F', '', '', '', 'crm:followup:list', 1, ''),
    (51, '0,25,51', '新增记录', 'F', '', '', '', 'crm:followup:create', 2, ''),
    (51, '0,25,51', '编辑记录', 'F', '', '', '', 'crm:followup:update', 3, ''),
    (51, '0,25,51', '删除记录', 'F', '', '', '', 'crm:followup:delete', 4, ''),

-- 销售管理
(0, '0', '销售管理', 'M', 'sale', '/sale', '', '', 3, 'sale'),
  -- 订单管理
  (57, '0,57', '订单管理', 'C', 'sale-order', '/sale/order', 'sale/order/index', '', 1, 'order'),
    (58, '0,57,58', '查询订单', 'F', '', '', '', 'sale:order:list', 1, ''),
    (58, '0,57,58', '新增订单', 'F', '', '', '', 'sale:order:create', 2, ''),
    (58, '0,57,58', '编辑订单', 'F', '', '', '', 'sale:order:update', 3, ''),
    (58, '0,57,58', '删除订单', 'F', '', '', '', 'sale:order:delete', 4, ''),
    (58, '0,57,58', '订单详情', 'F', '', '', '', 'sale:order:detail', 5, ''),
  -- 支付管理
  (57, '0,57', '支付管理', 'C', 'sale-payment', '/sale/payment', 'sale/payment/index', '', 2, 'payment'),
    (63, '0,57,63', '查询支付', 'F', '', '', '', 'sale:payment:list', 1, ''),
    (63, '0,57,63', '新增支付', 'F', '', '', '', 'sale:payment:create', 2, ''),
    (63, '0,57,63', '编辑支付', 'F', '', '', '', 'sale:payment:update', 3, ''),

-- 产品管理
(0, '0', '产品管理', 'M', 'product', '/product', '', '', 4, 'product'),
  -- 产品列表
  (69, '0,69', '产品管理', 'C', 'product-main', '/product/main', 'product/main/index', '', 1, 'goods'),
    (70, '0,69,70', '查询产品', 'F', '', '', '', 'product:main:list', 1, ''),
    (70, '0,69,70', '新增产品', 'F', '', '', '', 'product:main:create', 2, ''),
    (70, '0,69,70', '编辑产品', 'F', '', '', '', 'product:main:update', 3, ''),
    (70, '0,69,70', '删除产品', 'F', '', '', '', 'product:main:delete', 4, ''),
  -- 产品分类
  (69, '0,69', '分类管理', 'C', 'product-category', '/product/category', 'product/category/index', '', 2, 'category'),
    (75, '0,69,75', '查询分类', 'F', '', '', '', 'product:category:list', 1, ''),
    (75, '0,69,75', '新增分类', 'F', '', '', '', 'product:category:create', 2, ''),
    (75, '0,69,75', '编辑分类', 'F', '', '', '', 'product:category:update', 3, ''),
    (75, '0,69,75', '删除分类', 'F', '', '', '', 'product:category:delete', 4, ''),
  -- 库存管理
  (69, '0,69', '库存管理', 'C', 'inventory-stock', '/inventory/stock', 'inventory/stock/index', '', 3, 'warehouse'),
    (80, '0,69,80', '查询库存', 'F', '', '', '', 'inventory:stock:list', 1, ''),
    (80, '0,69,80', '库存调整', 'F', '', '', '', 'inventory:stock:adjust', 2, ''),

-- 采购管理
(0, '0', '采购管理', 'M', 'purchase', '/purchase', '', '', 5, 'shopping-cart'),
  -- 供应商管理
  (86, '0,86', '供应商管理', 'C', 'purchase-supplier', '/purchase/supplier', 'purchase/supplier/index', '', 1, 'supplier'),
    (87, '0,86,87', '查询供应商', 'F', '', '', '', 'purchase:supplier:list', 1, ''),
    (87, '0,86,87', '新增供应商', 'F', '', '', '', 'purchase:supplier:create', 2, ''),
    (87, '0,86,87', '编辑供应商', 'F', '', '', '', 'purchase:supplier:update', 3, ''),
    (87, '0,86,87', '删除供应商', 'F', '', '', '', 'purchase:supplier:delete', 4, ''),
  -- 采购单管理
  (86, '0,86', '采购单管理', 'C', 'purchase-po', '/purchase/po', 'purchase/po/index', '', 2, 'purchase-order'),
    (92, '0,86,92', '查询采购单', 'F', '', '', '', 'purchase:po:list', 1, ''),
    (92, '0,86,92', '新增采购单', 'F', '', '', '', 'purchase:po:create', 2, ''),
    (92, '0,86,92', '编辑采购单', 'F', '', '', '', 'purchase:po:update', 3, ''),
    (92, '0,86,92', '删除采购单', 'F', '', '', '', 'purchase:po:delete', 4, ''),

-- 通知管理
(0, '0', '通知管理', 'M', 'notice', '/notice', '', '', 6, 'notification'),
  -- 通知列表
  (98, '0,98', '通知管理', 'C', 'notice-main', '/notice/main', 'notice/main/index', '', 1, 'bell'),
    (99, '0,98,99', '查询通知', 'F', '', '', '', 'notice:main:list', 1, ''),
    (99, '0,98,99', '新增通知', 'F', '', '', '', 'notice:main:create', 2, ''),
    (99, '0,98,99', '编辑通知', 'F', '', '', '', 'notice:main:update', 3, ''),
    (99, '0,98,99', '删除通知', 'F', '', '', '', 'notice:main:delete', 4, ''),
    (99, '0,98,99', '发布通知', 'F', '', '', '', 'notice:main:publish', 5, ''),
  -- 我的通知
  (98, '0,98', '我的通知', 'C', 'notice-my', '/notice/my', 'notice/my/index', '', 2, 'inbox'),
    (104, '0,98,104', '查询我的通知', 'F', '', '', '', 'notice:my:list', 1, ''),
    (104, '0,98,104', '标记已读', 'F', '', '', '', 'notice:my:read', 2, '');

-- 11.6 为超级管理员分配所有菜单权限
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id, status)
SELECT 1, id, 0 FROM mxx_system_menu;

-- 11.7 为系统管理员分配系统管理相关权限
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id, status)
SELECT 2, id, 0 FROM mxx_system_menu
WHERE path LIKE '/system/%';

-- 11.8 为管理员分配角色
INSERT INTO mxx_system_admin_role_merge (admin_id, role_id) VALUES (1, 1);
INSERT INTO mxx_system_admin_role_merge (admin_id, role_id) VALUES (2, 2);
INSERT INTO mxx_system_admin_role_merge (admin_id, role_id) VALUES (3, 3);
INSERT INTO mxx_system_admin_role_merge (admin_id, role_id) VALUES (4, 4);
INSERT INTO mxx_system_admin_role_merge (admin_id, role_id) VALUES (5, 5);

-- 11.9 为管理员分配部门
INSERT INTO mxx_system_admin_dept_merge (admin_id, dept_id) VALUES (1, 1);
INSERT INTO mxx_system_admin_dept_merge (admin_id, dept_id) VALUES (2, 5);
INSERT INTO mxx_system_admin_dept_merge (admin_id, dept_id) VALUES (3, 2);
INSERT INTO mxx_system_admin_dept_merge (admin_id, dept_id) VALUES (4, 6);
INSERT INTO mxx_system_admin_dept_merge (admin_id, dept_id) VALUES (5, 3);
INSERT INTO mxx_system_admin_dept_merge (admin_id, dept_id) VALUES (6, 2);
INSERT INTO mxx_system_admin_dept_merge (admin_id, dept_id) VALUES (7, 5);
INSERT INTO mxx_system_admin_dept_merge (admin_id, dept_id) VALUES (8, 1);

-- 11.10 为CRM管理员、产品专员、通知专员创建专用角色
INSERT INTO mxx_system_role (role_name, role_key, sort, data_scope, status, deleted, remark) VALUES
('CRM管理员', 'crm_manager', 6, 3, 1, 0, 'CRM管理员负责客户管理'),
('产品专员', 'product_staff', 7, 3, 1, 0, '产品专员负责产品管理'),
('通知专员', 'notice_staff', 8, 3, 1, 0, '通知专员负责通知管理');

-- 11.11 为专用角色分配对应菜单权限
-- CRM管理员：分配CRM客户管理权限
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id, status)
SELECT 6, id, 0 FROM mxx_system_menu
WHERE path LIKE '/crm/%';

-- 产品专员：分配产品管理权限
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id, status)
SELECT 7, id, 0 FROM mxx_system_menu
WHERE path LIKE '/product/%' OR path LIKE '/inventory/%';

-- 通知专员：分配通知管理权限
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id, status)
SELECT 8, id, 0 FROM mxx_system_menu
WHERE path LIKE '/notice/%';

-- 11.12 为专用角色分配给对应测试账户
INSERT INTO mxx_system_admin_role_merge (admin_id, role_id) VALUES (6, 6);
INSERT INTO mxx_system_admin_role_merge (admin_id, role_id) VALUES (7, 7);
INSERT INTO mxx_system_admin_role_merge (admin_id, role_id) VALUES (8, 8);

-- 11.10 初始化系统配置
INSERT INTO mxx_system_config (config_name, config_key, config_value, config_type, sort) VALUES
('系统名称', 'system_name', 'Mxx-CRM', 'Y', 1),
('系统版本', 'system_version', '1.0.0', 'Y', 2),
('版权信息', 'system_copyright', '© 2024 MxxShop', 'Y', 3),
('登录超时时间', 'login_timeout', '7200', 'Y', 4),
('密码过期天数', 'password_expire_days', '90', 'Y', 5),
('验证码开关', 'captcha_enabled', 'true', 'Y', 6),
('日志保留天数', 'log_retention_days', '90', 'Y', 7);

-- 11.14 初始化数据字典
INSERT INTO mxx_system_dict (dict_name, dict_code, sort, status) VALUES
('用户状态', 'user_status', 1, 0),
('部门状态', 'dept_status', 2, 0),
('岗位状态', 'post_status', 3, 0),
('角色状态', 'role_status', 4, 0),
('菜单类型', 'menu_type', 5, 0),
('通知类型', 'notice_type', 6, 0),
('通知等级', 'notice_level', 7, 0),
('数据范围', 'data_scope', 8, 0),
('业务类型', 'business_type', 9, 0),
('操作类型', 'operator_type', 10, 0);

-- 11.12 初始化字典数据
INSERT INTO mxx_system_dict_data (dict_code, dict_label, dict_value, sort) VALUES
-- 用户状态
('user_status', '正常', '0', 1),
('user_status', '停用', '1', 2),
-- 部门状态
('dept_status', '正常', '0', 1),
('dept_status', '停用', '1', 2),
-- 岗位状态
('post_status', '正常', '0', 1),
('post_status', '停用', '1', 2),
-- 角色状态
('role_status', '停用', '0', 1),
('role_status', '正常', '1', 2),
-- 菜单类型
('menu_type', '目录', 'M', 1),
('menu_type', '菜单', 'C', 2),
('menu_type', '按钮', 'F', 3),
-- 通知类型
('notice_type', '系统通知', '1', 1),
('notice_type', '业务通知', '2', 2),
('notice_type', '公告通知', '3', 3),
-- 通知等级
('notice_level', '信息', 'info', 1),
('notice_level', '成功', 'success', 2),
('notice_level', '警告', 'warning', 3),
('notice_level', '危险', 'danger', 4),
-- 数据范围
('data_scope', '全部数据权限', '1', 1),
('data_scope', '自定义数据权限', '2', 2),
('data_scope', '本部门数据权限', '3', 3),
('data_scope', '本部门及以下数据权限', '4', 4),
-- 业务类型
('business_type', '其它', '0', 1),
('business_type', '新增', '1', 2),
('business_type', '修改', '2', 3),
('business_type', '删除', '3', 4),
('business_type', '导出', '4', 5),
('business_type', '导入', '5', 6),
-- 操作类型
('operator_type', '其它', '0', 1),
('operator_type', '后台用户', '1', 2),
('operator_type', '手机端用户', '2', 3);

-- ================================================
-- 12. 创建触发器自动更新时间戳
-- ================================================

-- 更新时间戳触发器函数
CREATE OR REPLACE FUNCTION update_modified_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.update_time = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 为需要自动更新时间戳的表创建触发器
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

CREATE TRIGGER update_mxx_system_role_modtime
    BEFORE UPDATE ON mxx_system_role
    FOR EACH ROW EXECUTE FUNCTION update_modified_column();

-- ================================================
-- 13. 创建序列生成器初始化数据
-- ================================================

INSERT INTO mxx_seq_generator (name, prefix, date_format, padding, description) VALUES
('customer_no', 'CUST', 'YYYYMMDD', 6, '客户编号生成器'),
('order_no', 'ORD', 'YYYYMMDD', 6, '订单编号生成器'),
('contract_no', 'CON', 'YYYYMMDD', 6, '合同编号生成器'),
('purchase_no', 'PO', 'YYYYMMDD', 6, '采购单编号生成器'),
('payment_no', 'PAY', 'YYYYMMDD', 6, '支付编号生成器'),
('product_no', 'PRD', '', 6, '产品编号生成器'),
('supplier_no', 'SUP', '', 6, '供应商编号生成器'),
('lead_no', 'LD', 'YYYYMMDD', 6, '线索编号生成器');

-- ================================================
-- 14. 创建数据库视图
-- ================================================

-- 用户角色视图
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
WHERE a.del_flag = 0 AND (r.deleted = 0 OR r.deleted IS NULL);

-- 角色菜单权限视图
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

-- 系统管理员视图（包含权限信息）
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
-- 15. 创建索引优化查询性能
-- ================================================

-- 用户登录查询优化
CREATE INDEX idx_sys_admin_username_status_del ON mxx_system_admin(user_name, status, deleted);

-- 菜单权限查询优化
CREATE INDEX idx_sys_menu_status_type_parent ON mxx_system_menu(status, type, parent_id);

-- 角色菜单关联查询优化
CREATE INDEX idx_sys_role_menu_role_status ON mxx_system_role_menu_merge(role_id, status);

-- 客户查询优化（按负责人和等级）
CREATE INDEX idx_crm_customer_assigned_level ON mxx_crm_customer(assigned_to, level);

-- 线索查询优化（按状态和负责人）
CREATE INDEX idx_crm_lead_status_assigned ON mxx_crm_lead(status, assigned_to);

-- 订单查询优化（按客户和状态）
CREATE INDEX idx_sale_order_customer_status ON mxx_sale_order(customer_id, status);

-- 商机查询优化（按阶段和负责人）
CREATE INDEX idx_crm_opportunity_stage_assigned ON mxx_crm_opportunity(stage, assigned_to);

-- ================================================
-- 数据库初始化完成
-- ================================================

COMMIT;

-- ================================================
-- 权限控制说明
-- ================================================

-- 本系统采用基于角色的访问控制（RBAC）+ 菜单权限的双层权限控制机制
-- 权限校验流程：
-- 1. 登录认证：用户登录后获取 admin_id 和 role_ids
-- 2. 菜单权限：通过 mxx_system_role_menu_merge 获取用户有权限的菜单列表
-- 3. 按钮权限：通过菜单的 perm 字段进行细粒度控制
-- 4. 数据范围：根据角色的 data_scope 字段过滤数据
--    - 1: 全部数据权限
--    - 2: 自定义数据权限
--    - 3: 本部门数据权限
--    - 4: 本部门及以下数据权限

-- 控制器注解权限示例：
-- #[permission("system:admin:list")]
-- pub async fn list_admin() -> Result<Json<ResponseData>> {
--     // 业务逻辑
-- }

-- 默认管理员账号：
-- 用户名：admin
-- 密码：admin123

-- 默认系统管理员账号：
-- 用户名：system
-- 密码：admin123