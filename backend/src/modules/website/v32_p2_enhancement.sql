-- v32: P2 增强体验 - 数据库迁移
-- 包含：文章版本历史(G-2.2)、URL伪静态规则(G-2.6)、邮件通知配置(G-2.8)
-- 遵循项目规则：TIMESTAMP、create_time/update_time、deleted INT、status INT

-- =====================================================
-- G-2.2: 文章版本历史表
-- =====================================================
CREATE TABLE IF NOT EXISTS mxx_article_revision (
    id BIGSERIAL PRIMARY KEY,
    article_id BIGINT NOT NULL,
    revision_no INT NOT NULL,
    title VARCHAR(255),
    short_title VARCHAR(255),
    title_image VARCHAR(500),
    author VARCHAR(64),
    description VARCHAR(1000),
    content TEXT,
    -- 快照字段（JSON 存储编辑时的完整字段快照）
    snapshot TEXT,
    -- 编辑者
    editor_id BIGINT,
    editor_name VARCHAR(64),
    -- 编辑备注
    edit_remark VARCHAR(255),
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_article_revision_article_id ON mxx_article_revision (article_id);
CREATE INDEX IF NOT EXISTS idx_article_revision_create_time ON mxx_article_revision (create_time);

-- =====================================================
-- G-2.6: URL伪静态规则 - mxx_website 新增字段
-- =====================================================
-- url_rule: URL伪静态规则
-- 0=默认动态URL（/category?id=1, /article?id=1）
-- 1=短URL模式（/category/1, /article/short_url）
-- 2=目录模式（/category/1/, /article/short_url.html）
-- 3=自定义模式（按 url_rule_pattern 字段配置）
ALTER TABLE mxx_website ADD COLUMN IF NOT EXISTS url_rule INT DEFAULT 0;
ALTER TABLE mxx_website ADD COLUMN IF NOT EXISTS url_rule_pattern VARCHAR(255);

-- =====================================================
-- G-2.8: 邮件通知配置表
-- =====================================================
CREATE TABLE IF NOT EXISTS mxx_website_notification_config (
    id BIGSERIAL PRIMARY KEY,
    website_id BIGINT,
    -- 通知场景：leave_msg=新留言 order_created=新订单 order_paid=订单已支付
    -- refund_apply=退款申请 refund_handled=退款已处理 article_comment=新评论
    scene_code VARCHAR(32) NOT NULL,
    scene_name VARCHAR(64),
    -- 通知渠道（逗号分隔）：email,sms,wecom,system
    channels VARCHAR(128) DEFAULT 'email',
    -- 收件人邮箱（逗号分隔，留空则用站点联系邮箱）
    recipient_emails VARCHAR(500),
    -- 邮件模板（minijinja 模板字符串）
    email_subject VARCHAR(255),
    email_body TEXT,
    -- 是否启用
    enabled INT DEFAULT 0,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted INT DEFAULT 0,
    UNIQUE(website_id, scene_code)
);
CREATE INDEX IF NOT EXISTS idx_notification_config_website ON mxx_website_notification_config (website_id, enabled, deleted);

-- 预置默认通知配置（site_id=1）
INSERT INTO mxx_website_notification_config (website_id, scene_code, scene_name, channels, email_subject, email_body, enabled)
SELECT 1, 'leave_msg', '新留言通知', 'email', '【站点通知】收到新留言：{{ contact_name }}', '您收到一条新留言：\n联系人：{{ contact_name }}\n电话：{{ contact_phone }}\n内容：{{ content }}\n请及时处理。', 1
WHERE NOT EXISTS (SELECT 1 FROM mxx_website_notification_config WHERE website_id = 1 AND scene_code = 'leave_msg');

INSERT INTO mxx_website_notification_config (website_id, scene_code, scene_name, channels, email_subject, email_body, enabled)
SELECT 1, 'order_created', '新订单通知', 'email', '【站点通知】新订单：{{ order_no }}', '您收到一个新订单：\n订单号：{{ order_no }}\n金额：{{ pay_amount }}\n请及时处理。', 1
WHERE NOT EXISTS (SELECT 1 FROM mxx_website_notification_config WHERE website_id = 1 AND scene_code = 'order_created');

INSERT INTO mxx_website_notification_config (website_id, scene_code, scene_name, channels, email_subject, email_body, enabled)
SELECT 1, 'order_paid', '订单已付款', 'email', '【站点通知】订单已付款：{{ order_no }}', '订单已付款：\n订单号：{{ order_no }}\n金额：{{ pay_amount }}\n支付时间：{{ pay_time }}\n请尽快发货。', 1
WHERE NOT EXISTS (SELECT 1 FROM mxx_website_notification_config WHERE website_id = 1 AND scene_code = 'order_paid');

INSERT INTO mxx_website_notification_config (website_id, scene_code, scene_name, channels, email_subject, email_body, enabled)
SELECT 1, 'refund_apply', '退款申请通知', 'email', '【站点通知】收到退款申请：{{ refund_no }}', '收到退款申请：\n退款单号：{{ refund_no }}\n订单号：{{ order_no }}\n退款金额：{{ refund_amount }}\n退款原因：{{ refund_reason }}\n请及时审核。', 1
WHERE NOT EXISTS (SELECT 1 FROM mxx_website_notification_config WHERE website_id = 1 AND scene_code = 'refund_apply');

INSERT INTO mxx_website_notification_config (website_id, scene_code, scene_name, channels, email_subject, email_body, enabled)
SELECT 1, 'refund_handled', '退款处理结果', 'email', '【站点通知】退款处理结果：{{ refund_no }}', '退款处理结果：\n退款单号：{{ refund_no }}\n处理结果：{{ result }}\n处理备注：{{ handle_remark }}', 1
WHERE NOT EXISTS (SELECT 1 FROM mxx_website_notification_config WHERE website_id = 1 AND scene_code = 'refund_handled');

-- =====================================================
-- G-2.2 菜单：文章版本历史（按钮权限，挂在文章管理下）
-- =====================================================
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 650, 349, 'page.website.article.button.revision', 'BUTTON', 'website:article:revision', 5, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 650);

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 651, 349, 'page.website.article.button.revisionRestore', 'BUTTON', 'website:article:revision_restore', 6, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 651);

-- =====================================================
-- G-2.8 菜单：通知配置管理
-- =====================================================
INSERT INTO mxx_system_menu (id, parent_id, name, path, type, route_name, component, perm, icon, sort, status, create_time)
SELECT 652, 345, 'page.website.notificationTitle', '/website/notification', 'MENU', 'WebsiteNotification', 'views/website/notification/index.vue', 'website:notification:list', 'lucide-bell', 13, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 652);

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 653, 652, 'page.website.notification.button.view', 'BUTTON', 'website:notification:view', 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 653);

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 654, 652, 'page.website.notification.button.edit', 'BUTTON', 'website:notification:update', 2, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 654);

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 655, 652, 'page.website.notification.button.toggle', 'BUTTON', 'website:notification:toggle', 3, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 655);

-- =====================================================
-- 授权：将 P2 菜单授予 super_admin(1) 角色
-- =====================================================
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT 1, m.id FROM mxx_system_menu m
WHERE m.id BETWEEN 650 AND 655
AND NOT EXISTS (SELECT 1 FROM mxx_system_role_menu_merge rm WHERE rm.role_id = 1 AND rm.menu_id = m.id);

-- =====================================================
-- 验证
-- =====================================================
SELECT 'mxx_article_revision' as tbl, count(*) as cnt FROM mxx_article_revision
UNION ALL SELECT 'mxx_website_notification_config', count(*) FROM mxx_website_notification_config
UNION ALL SELECT 'url_rule_column', count(*) FROM information_schema.columns WHERE table_name = 'mxx_website' AND column_name = 'url_rule';
