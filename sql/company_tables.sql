-- 企业基本信息表（单条记录，id固定为1）
CREATE TABLE IF NOT EXISTS mxx_company_info (
    id BIGSERIAL PRIMARY KEY,
    company_name VARCHAR(128),           -- 企业名称
    credit_code VARCHAR(64),             -- 统一社会信用代码
    legal_person VARCHAR(64),            -- 法定代表人
    legal_phone VARCHAR(32),             -- 法人联系电话
    register_address VARCHAR(256),       -- 注册地址
    business_scope TEXT,                 -- 经营范围
    contact_phone VARCHAR(32),           -- 联系电话
    contact_email VARCHAR(128),          -- 联系邮箱
    logo_url VARCHAR(512),               -- 企业Logo URL
    tax_number VARCHAR(64),              -- 税号
    invoice_title VARCHAR(128),          -- 发票抬头
    remark TEXT,                         -- 备注
    deleted INT DEFAULT 0,
    create_by BIGINT,
    update_by BIGINT,
    create_time TIMESTAMP,
    update_time TIMESTAMP
);

-- 企业银行账户表（多条）
CREATE TABLE IF NOT EXISTS mxx_company_account (
    id BIGSERIAL PRIMARY KEY,
    bank_name VARCHAR(128),              -- 开户银行
    account_name VARCHAR(128),           -- 账户名称
    account_number VARCHAR(64),          -- 银行账号
    account_type INT DEFAULT 1,          -- 账户类型(1=基本户 2=一般户 3=其他)
    is_default INT DEFAULT 0,            -- 是否默认(0=否 1=是)
    sort_order INT DEFAULT 0,            -- 排序
    remark VARCHAR(256),                 -- 备注
    deleted INT DEFAULT 0,
    create_by BIGINT,
    update_by BIGINT,
    create_time TIMESTAMP,
    update_time TIMESTAMP
);

-- 插入默认企业信息记录
INSERT INTO mxx_company_info (id, company_name, deleted, create_time, update_time)
VALUES (1, '我的企业', 0, NOW(), NOW())
ON CONFLICT (id) DO NOTHING;
