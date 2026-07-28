-- ============================================================
-- 邮箱功能相关表结构 (2026-07-28)
-- 1. mxx_system_mail_config  邮箱账号配置表
-- 2. mxx_system_mail_template 邮件模板表
-- 3. mxx_crm_mail_log         邮件发送日志表
-- ============================================================

-- 邮箱账号配置表
CREATE TABLE IF NOT EXISTS mxx_system_mail_config (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(128) NOT NULL DEFAULT ''::character varying,
    host VARCHAR(255) NOT NULL DEFAULT ''::character varying,
    port INT NOT NULL DEFAULT 465,
    username VARCHAR(255) NOT NULL DEFAULT ''::character varying,
    password VARCHAR(255) NOT NULL DEFAULT ''::character varying,
    from_email VARCHAR(255) NOT NULL DEFAULT ''::character varying,
    from_name VARCHAR(128) NOT NULL DEFAULT ''::character varying,
    is_ssl INT NOT NULL DEFAULT 1,
    is_default INT NOT NULL DEFAULT 0,
    status INT NOT NULL DEFAULT 1,
    create_by BIGINT,
    create_time TIMESTAMP,
    update_by BIGINT,
    update_time TIMESTAMP,
    deleted INT NOT NULL DEFAULT 0
);
COMMENT ON TABLE mxx_system_mail_config IS '邮箱账号配置表';

-- 邮件模板表
CREATE TABLE IF NOT EXISTS mxx_system_mail_template (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(128) NOT NULL DEFAULT ''::character varying,
    subject VARCHAR(255) NOT NULL DEFAULT ''::character varying,
    body TEXT,
    create_by BIGINT,
    create_time TIMESTAMP,
    update_by BIGINT,
    update_time TIMESTAMP,
    deleted INT NOT NULL DEFAULT 0
);
COMMENT ON TABLE mxx_system_mail_template IS '邮件模板表';

-- 邮件发送日志表
CREATE TABLE IF NOT EXISTS mxx_crm_mail_log (
    id BIGSERIAL PRIMARY KEY,
    customer_id BIGINT,
    contact_ids VARCHAR(500),
    from_email VARCHAR(255),
    to_emails TEXT,
    cc_emails TEXT,
    subject VARCHAR(255),
    body TEXT,
    status INT NOT NULL DEFAULT 0,
    error_msg TEXT,
    smtp_message_id VARCHAR(255),
    sender_id BIGINT,
    sender_name VARCHAR(128),
    send_time TIMESTAMP,
    create_time TIMESTAMP
);
COMMENT ON TABLE mxx_crm_mail_log IS '邮件发送日志表';


-- ============================================================
-- 菜单数据插入（挂在"设置"目录 id=67 下）
-- ============================================================

-- 菜单：邮箱设置
INSERT INTO mxx_system_menu (id, parent_id, name, type, route_name, path, component, perm, status, sort, icon)
VALUES (
  530,
  67,
  'page.system.mail.config.title',
  'MENU',
  'MailConfig',
  '/system/mail-config',
  'system/mail-config/index',
  'system:mail:config',
  1,
  40,
  'lucide:mail'
)
ON CONFLICT (id) DO NOTHING;

-- 按钮权限：邮件模板管理
INSERT INTO mxx_system_menu (id, parent_id, name, type, route_name, path, component, perm, status, sort, icon)
VALUES (
  531,
  530,
  '邮件模板',
  'BUTTON',
  NULL,
  NULL,
  NULL,
  'system:mail:template',
  1,
  1,
  NULL
)
ON CONFLICT (id) DO NOTHING;

-- 按钮权限：发送邮件
INSERT INTO mxx_system_menu (id, parent_id, name, type, route_name, path, component, perm, status, sort, icon)
VALUES (
  532,
  530,
  '发送邮件',
  'BUTTON',
  NULL,
  NULL,
  NULL,
  'crm:mail:send',
  1,
  2,
  NULL
)
ON CONFLICT (id) DO NOTHING;

-- 按钮权限：邮件日志
INSERT INTO mxx_system_menu (id, parent_id, name, type, route_name, path, component, perm, status, sort, icon)
VALUES (
  533,
  530,
  '邮件日志',
  'BUTTON',
  NULL,
  NULL,
  NULL,
  'system:mail:log',
  1,
  3,
  NULL
)
ON CONFLICT (id) DO NOTHING;


-- ============================================================
-- 验证
-- ============================================================
-- SELECT id, parent_id, name, type, path, component, perm, sort FROM mxx_system_menu WHERE id IN (530, 531, 532, 533) ORDER BY sort;
