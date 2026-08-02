-- ============================================================
-- V8-1 工资单审批流模板预置
-- 创建时间：2026-07-31
-- 说明：为 V8-1 工资单审批流对接提供 salary_approval 模板
-- 业务类型：salary
-- 审批节点：直属上级 → 财务负责人 → 结束
-- ============================================================

-- ============================================================
-- 新增：工资审批流（salary_approval）
-- ============================================================
INSERT INTO mxx_system_approval_flow (id, flow_code, flow_name, business_type, enabled, is_system)
SELECT 7, 'salary_approval', '工资审批流', 'salary', 1, 1
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow WHERE flow_code = 'salary_approval');

-- 审批节点：开始 → 直属上级审批 → 财务负责人审批 → 结束
-- approver_type: 6=直属上级, 1=指定角色
-- approve_mode: 1=或签
INSERT INTO mxx_system_approval_flow_node (flow_id, node_key, node_type, node_name, node_order, approver_type, approver_id, approve_mode, is_final, position_x, position_y)
SELECT 7, k.node_key, k.node_type, k.node_name, k.node_order, k.approver_type, k.approver_id, k.approve_mode, k.is_final, k.position_x, k.position_y
FROM (VALUES
  ('start',              1::int, '开始',           1::int, NULL::int, NULL::int, 1::int, 0::int, 250::int, 40::int),
  ('n_direct_manager',   2,      '直属上级审批',   2,      6,         NULL,      1,      0,      250,      160),
  ('n_finance_leader',   2,      '财务负责人审批', 3,      1,         10,        1,      0,      250,      280),
  ('end',                4,      '结束',           4,      NULL,      NULL,      1,      1,      250,      400)
) AS k(node_key, node_type, node_name, node_order, approver_type, approver_id, approve_mode, is_final, position_x, position_y)
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_node WHERE flow_id = 7);

-- 审批边
INSERT INTO mxx_system_approval_flow_edge (flow_id, source_node_key, target_node_key, condition_expr, label)
SELECT 7, src, tgt, cond, lbl
FROM (VALUES
  ('start',            'n_direct_manager', NULL::text, NULL::text),
  ('n_direct_manager', 'n_finance_leader', NULL,        NULL),
  ('n_finance_leader', 'end',              NULL,        NULL)
) AS t(src, tgt, cond, lbl)
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_edge WHERE flow_id = 7);

-- ============================================================
-- 补全 statistics_admin_controller 缺失的权限码
-- ============================================================
-- 财务统计查询权限
INSERT INTO mxx_system_menu (id, parent_id, name, path, type, perm, icon, sort_order, is_show, create_time)
SELECT 580, 321, '财务统计查询', '', 3, 'finance:statistics:list', '', 99, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'finance:statistics:list');

-- 财务统计管理权限（生成每日统计等）
INSERT INTO mxx_system_menu (id, parent_id, name, path, type, perm, icon, sort_order, is_show, create_time)
SELECT 581, 321, '财务统计管理', '', 3, 'finance:statistics:manage', '', 99, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'finance:statistics:manage');

-- 把统计权限授予 finance、super_admin、general_manager、boss 角色
DO $$
DECLARE
    r RECORD;
    role_ids BIGINT[] := ARRAY[10, 1, 5, 6]; -- finance=10, super_admin=1, general_manager=5, boss=6
BEGIN
    FOREACH r IN role_ids LOOP
        IF NOT EXISTS (SELECT 1 FROM mxx_system_role_menu_merge WHERE role_id = r AND menu_id = 580) THEN
            INSERT INTO mxx_system_role_menu_merge (role_id, menu_id) VALUES (r, 580);
        END IF;
        IF NOT EXISTS (SELECT 1 FROM mxx_system_role_menu_merge WHERE role_id = r AND menu_id = 581) THEN
            INSERT INTO mxx_system_role_menu_merge (role_id, menu_id) VALUES (r, 581);
        END IF;
    END LOOP;
END $$;

-- ============================================================
-- 补全 expense:save 权限码别名（修复前后端权限码不匹配）
-- 前端使用 finance:expense:save，后端使用 finance:expense:add
-- 方案：在 mxx_system_menu 中新增 finance:expense:save 作为别名权限
-- ============================================================
INSERT INTO mxx_system_menu (id, parent_id, name, path, type, perm, icon, sort_order, is_show, create_time)
SELECT 582, 321, '报销保存', '', 3, 'finance:expense:save', '', 99, 0, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'finance:expense:save');

-- 把 expense:save 别名权限授予与 expense:add 相同的角色
DO $$
DECLARE
    r RECORD;
BEGIN
    FOR r IN SELECT role_id FROM mxx_system_role_menu_merge rm
             JOIN mxx_system_menu m ON rm.menu_id = m.id
             WHERE m.perm = 'finance:expense:add'
    LOOP
        IF NOT EXISTS (SELECT 1 FROM mxx_system_role_menu_merge WHERE role_id = r.role_id AND menu_id = 582) THEN
            INSERT INTO mxx_system_role_menu_merge (role_id, menu_id) VALUES (r.role_id, 582);
        END IF;
    END LOOP;
END $$;

-- ============================================================
-- 创建消息通道配置表（V8-4 推送通道补齐基础）
-- 存储 sms/wecom/dingtalk/feishu 等外部通道的账号配置
-- ============================================================
CREATE TABLE IF NOT EXISTS mxx_system_notification_channel_config (
    id BIGSERIAL PRIMARY KEY,
    channel_code VARCHAR(32) NOT NULL UNIQUE,  -- sms/wecom/dingtalk/feishu/email
    channel_name VARCHAR(64) NOT NULL,
    enabled SMALLINT NOT NULL DEFAULT 0,
    config_json TEXT,                          -- 通道特定配置（app_id/app_secret/agent_id 等 JSON）
    remark VARCHAR(255),
    create_time TIMESTAMP NOT NULL DEFAULT NOW(),
    update_time TIMESTAMP
);

-- 预置默认通道配置（未启用，需管理员填入真实账号后启用）
INSERT INTO mxx_system_notification_channel_config (channel_code, channel_name, enabled, config_json, remark)
SELECT 'sms', '短信通道', 0, '{"provider":"","access_key":"","secret_key":"","sign_name":""}', '需配置短信服务商'
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_notification_channel_config WHERE channel_code = 'sms');

INSERT INTO mxx_system_notification_channel_config (channel_code, channel_name, enabled, config_json, remark)
SELECT 'wecom', '企业微信通道', 0, '{"corp_id":"","agent_id":"","secret":""}', '需配置企业微信应用'
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_notification_channel_config WHERE channel_code = 'wecom');

INSERT INTO mxx_system_notification_channel_config (channel_code, channel_name, enabled, config_json, remark)
SELECT 'dingtalk', '钉钉通道', 0, '{"app_key":"","app_secret":"","agent_id":""}', '需配置钉钉应用'
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_notification_channel_config WHERE channel_code = 'dingtalk');

INSERT INTO mxx_system_notification_channel_config (channel_code, channel_name, enabled, config_json, remark)
SELECT 'feishu', '飞书通道', 0, '{"app_id":"","app_secret":""}', '需配置飞书应用'
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_notification_channel_config WHERE channel_code = 'feishu');

-- ============================================================
-- 工资条表增加撤回相关字段（V8-4 撤回功能）
-- ============================================================
ALTER TABLE mxx_finance_payslip ADD COLUMN IF NOT EXISTS withdraw_time TIMESTAMP;
ALTER TABLE mxx_finance_payslip ADD COLUMN IF NOT EXISTS withdraw_reason VARCHAR(255);
ALTER TABLE mxx_finance_payslip ADD COLUMN IF NOT EXISTS withdrawn_by BIGINT;
-- 扩展 send_status：0=未发送 1=已发送 2=已读 3=已确认 4=已撤回

-- ============================================================
-- 完成标记
-- ============================================================
-- 迁移内容：
-- 1. 预置 salary_approval 审批流模板（V8-1）
-- 2. 补全 finance:statistics:list/manage 权限码
-- 3. 补全 finance:expense:save 权限码别名
-- 4. 创建消息通道配置表（V8-4 基础）
-- 5. 工资条表增加撤回字段（V8-4 撤回功能）
