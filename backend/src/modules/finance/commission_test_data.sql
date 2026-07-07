-- ============================================================
-- 提成系统完整测试数据
-- 包含：提成规则（含阶梯+成员）、客户、合同、合同提成成员、回款计划
-- 用于端到端验证提成计算和合同提成配置功能
-- ============================================================

-- ============================================================
-- 1. 部门测试数据（如已存在请跳过）
-- ============================================================
INSERT INTO mxx_system_dept (id, parent_id, dept_name, leader, phone, email, sort, status, create_time, deleted)
VALUES (1, 0, '总公司', NULL, NULL, NULL, 1, 1, NOW(), 0)
ON CONFLICT (id) DO NOTHING;

INSERT INTO mxx_system_dept (id, parent_id, dept_name, leader, phone, email, sort, status, create_time, deleted)
VALUES (2, 1, '销售部', NULL, NULL, NULL, 1, 1, NOW(), 0)
ON CONFLICT (id) DO NOTHING;

INSERT INTO mxx_system_dept (id, parent_id, dept_name, leader, phone, email, sort, status, create_time, deleted)
VALUES (3, 1, '技术部', NULL, NULL, NULL, 2, 1, NOW(), 0)
ON CONFLICT (id) DO NOTHING;

INSERT INTO mxx_system_dept (id, parent_id, dept_name, leader, phone, email, sort, status, create_time, deleted)
VALUES (4, 2, '销售一组', NULL, NULL, NULL, 1, 1, NOW(), 0)
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 2. 管理员用户测试数据（如已存在请跳过）
-- ============================================================
INSERT INTO mxx_system_admin (id, user_name, nick_name, dept_id, email, phone, sex, status, create_time, deleted)
VALUES (1, 'admin', '管理员', 2, 'admin@example.com', '13800000001', 1, 1, NOW(), 0)
ON CONFLICT (id) DO NOTHING;

INSERT INTO mxx_system_admin (id, user_name, nick_name, dept_id, email, phone, sex, status, create_time, deleted)
VALUES (2, 'zhangsan', '张三', 4, 'zhangsan@example.com', '13800000002', 1, 1, NOW(), 0)
ON CONFLICT (id) DO NOTHING;

INSERT INTO mxx_system_admin (id, user_name, nick_name, dept_id, email, phone, sex, status, create_time, deleted)
VALUES (3, 'lisi', '李四', 4, 'lisi@example.com', '13800000003', 2, 1, NOW(), 0)
ON CONFLICT (id) DO NOTHING;

INSERT INTO mxx_system_admin (id, user_name, nick_name, dept_id, email, phone, sex, status, create_time, deleted)
VALUES (4, 'wangwu', '王五', 3, 'wangwu@example.com', '13800000004', 1, 1, NOW(), 0)
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 3. 提成规则测试数据（含阶梯配置和成员配置）
-- ============================================================

-- 规则1：销售部2026提成方案 - 个人业绩类型
INSERT INTO mxx_finance_commission_rule (
    id, rule_name, rule_type, apply_scope, department_id, post_id,
    commission_target_type, priority, is_default, calc_base_type, trigger_condition,
    effective_date, expiry_date, enabled, description,
    created_by, create_time, updated_by, update_time, deleted
) VALUES (
    1, '销售部2026提成方案', 1, 1, 2, NULL,
    NULL, 1, 1, 3, 2,
    '2026-01-01', '2026-12-31', 1, '销售部个人业绩提成，按合同金额阶梯计算',
    1, NOW(), 1, NOW(), 0
) ON CONFLICT (id) DO NOTHING;

-- 规则1的阶梯配置
INSERT INTO mxx_finance_commission_tier (id, rule_id, min_amount, max_amount, commission_rate, sort) VALUES
(1, 1, 0.00, 500000.00, 0.0150, 1),
(2, 1, 500000.00, 1000000.00, 0.0200, 2),
(3, 1, 1000000.00, NULL, 0.0300, 3)
ON CONFLICT (id) DO NOTHING;

-- 规则2：全公司通用提成方案 - 个人业绩类型
INSERT INTO mxx_finance_commission_rule (
    id, rule_name, rule_type, apply_scope, department_id, post_id,
    commission_target_type, priority, is_default, calc_base_type, trigger_condition,
    effective_date, expiry_date, enabled, description,
    created_by, create_time, updated_by, update_time, deleted
) VALUES (
    2, '全公司通用提成方案', 1, 2, NULL, NULL,
    NULL, 10, 0, 3, 1,
    '2026-01-01', NULL, 1, '全公司通用的基础提成方案',
    1, NOW(), 1, NOW(), 0
) ON CONFLICT (id) DO NOTHING;

-- 规则2的阶梯配置
INSERT INTO mxx_finance_commission_tier (id, rule_id, min_amount, max_amount, commission_rate, sort) VALUES
(4, 2, 0.00, 300000.00, 0.0100, 1),
(5, 2, 300000.00, NULL, 0.0150, 2)
ON CONFLICT (id) DO NOTHING;

-- 规则3：销售经理专项提成方案 - 团队分成类型（有成员配置）
INSERT INTO mxx_finance_commission_rule (
    id, rule_name, rule_type, apply_scope, department_id, post_id,
    commission_target_type, priority, is_default, calc_base_type, trigger_condition,
    effective_date, expiry_date, enabled, description,
    created_by, create_time, updated_by, update_time, deleted
) VALUES (
    3, '销售经理专项提成方案', 2, 1, 2, NULL,
    NULL, 2, 0, 3, 2,
    '2026-01-01', '2026-12-31', 1, '团队分成模式，主签+参与+技术支持按比例分成',
    1, NOW(), 1, NOW(), 0
) ON CONFLICT (id) DO NOTHING;

-- 规则3的阶梯配置
INSERT INTO mxx_finance_commission_tier (id, rule_id, min_amount, max_amount, commission_rate, sort) VALUES
(6, 3, 0.00, 800000.00, 0.0200, 1),
(7, 3, 800000.00, 2000000.00, 0.0250, 2),
(8, 3, 2000000.00, NULL, 0.0350, 3)
ON CONFLICT (id) DO NOTHING;

-- 规则3的默认成员配置（团队分成模式）
INSERT INTO mxx_finance_commission_rule_member (
    id, rule_id, member_type, role_name, member_name, distribution_type,
    fixed_rate, default_ratio, required, sort, create_time, update_time
) VALUES
(1, 3, 1, '主签人', '', 1, 0.0000, 0.6000, 1, 1, NOW(), NOW()),
(2, 3, 2, '参与人', '', 1, 0.0000, 0.2500, 0, 2, NOW(), NOW()),
(3, 3, 3, '技术支持', '', 1, 0.0000, 0.1500, 0, 3, NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

-- 规则4：部门经理提成方案 - 部门经理类型
INSERT INTO mxx_finance_commission_rule (
    id, rule_name, rule_type, apply_scope, department_id, post_id,
    commission_target_type, priority, is_default, calc_base_type, trigger_condition,
    effective_date, expiry_date, enabled, description,
    created_by, create_time, updated_by, update_time, deleted
) VALUES (
    4, '部门经理提成方案', 3, 1, 2, NULL,
    1, 3, 0, 2, 2,
    '2026-01-01', '2026-12-31', 1, '部门经理按部门月累计业绩计提',
    1, NOW(), 1, NOW(), 0
) ON CONFLICT (id) DO NOTHING;

-- 规则4的阶梯配置
INSERT INTO mxx_finance_commission_tier (id, rule_id, min_amount, max_amount, commission_rate, sort) VALUES
(9, 4, 0.00, 2000000.00, 0.0050, 1),
(10, 4, 2000000.00, 5000000.00, 0.0080, 2),
(11, 4, 5000000.00, NULL, 0.0100, 3)
ON CONFLICT (id) DO NOTHING;

-- 规则5：总监提成方案 - 总监类型
INSERT INTO mxx_finance_commission_rule (
    id, rule_name, rule_type, apply_scope, department_id, post_id,
    commission_target_type, priority, is_default, calc_base_type, trigger_condition,
    effective_date, expiry_date, enabled, description,
    created_by, create_time, updated_by, update_time, deleted
) VALUES (
    5, '销售总监提成方案', 4, 2, NULL, NULL,
    2, 5, 0, 2, 2,
    '2026-01-01', '2026-12-31', 1, '总监按全公司月累计业绩计提',
    1, NOW(), 1, NOW(), 0
) ON CONFLICT (id) DO NOTHING;

-- 规则5的阶梯配置
INSERT INTO mxx_finance_commission_tier (id, rule_id, min_amount, max_amount, commission_rate, sort) VALUES
(12, 5, 0.00, 5000000.00, 0.0030, 1),
(13, 5, 5000000.00, NULL, 0.0050, 2)
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 4. 客户测试数据
-- ============================================================
INSERT INTO mxx_crm_customer (id, company_name, short_name, customer_source, industry, level, status, assigned_to, created_by, create_time, update_time, deleted)
VALUES (1, '北京科技有限公司', '北京科技', 1, 1, 1, 1, 1, 1, NOW(), NOW(), 0)
ON CONFLICT (id) DO NOTHING;

INSERT INTO mxx_crm_customer (id, company_name, short_name, customer_source, industry, level, status, assigned_to, created_by, create_time, update_time, deleted)
VALUES (2, '上海贸易有限公司', '上海贸易', 2, 2, 2, 1, 2, 1, NOW(), NOW(), 0)
ON CONFLICT (id) DO NOTHING;

INSERT INTO mxx_crm_customer (id, company_name, short_name, customer_source, industry, level, status, assigned_to, created_by, create_time, update_time, deleted)
VALUES (3, '深圳电子科技有限公司', '深圳电子', 3, 3, 1, 1, 1, 1, NOW(), NOW(), 0)
ON CONFLICT (id) DO NOTHING;

INSERT INTO mxx_crm_customer (id, company_name, short_name, customer_source, industry, level, status, assigned_to, created_by, create_time, update_time, deleted)
VALUES (4, '广州制造有限公司', '广州制造', 4, 4, 2, 1, 2, 1, NOW(), NOW(), 0)
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 5. 合同测试数据 - 覆盖不同提成模式
-- ============================================================

-- 合同1：销售部2026方案（个人业绩）| 一次性收款 | 已审批 | ¥904,000
-- 预期：按方案1第2档(50万-100万) 2% 计算，提成 = 904,000 * 2% = 18,080，归属assigned_to=1(管理员)
INSERT INTO mxx_crm_contract (
    id, contract_no, customer_id, title, contract_type, amount, currency, tax_amount, total_amount,
    status, start_date, end_date, sign_date, payment_method_type, assigned_to,
    approval_status, commission_rule_id, commission_mode,
    remark, created_by, create_time, update_time, deleted
) VALUES (
    1, 'CON-20260701-0001', 1, '北京科技2026年度采购合同', 1, 800000.00, 1, 104000.00, 904000.00,
    2, '2026-07-01', '2026-12-31', '2026-07-01', 1, 1,
    3, 1, 1,
    '年度框架采购合同，使用销售部2026提成方案（个人业绩）', 1, NOW(), NOW(), 0
) ON CONFLICT (id) DO NOTHING;

-- 合同2：全公司通用方案（个人业绩）| 分期收款 | 已审批 | ¥265,000
-- 预期：按方案2第1档(0-30万) 1% 计算，提成 = 265,000 * 1% = 2,650，归属assigned_to=1(管理员)
INSERT INTO mxx_crm_contract (
    id, contract_no, customer_id, title, contract_type, amount, currency, tax_amount, total_amount,
    status, start_date, end_date, sign_date, payment_method_type, assigned_to,
    approval_status, commission_rule_id, commission_mode,
    remark, created_by, create_time, update_time, deleted
) VALUES (
    2, 'CON-20260702-0001', 2, '上海贸易软件服务合同', 3, 250000.00, 1, 15000.00, 265000.00,
    2, '2026-07-10', '2026-10-10', '2026-07-10', 2, 1,
    3, 2, 1,
    '软件技术服务合同，分3期付款，使用全公司通用提成方案', 1, NOW(), NOW(), 0
) ON CONFLICT (id) DO NOTHING;

-- 合同2的回款计划（3期）
INSERT INTO mxx_crm_contract_payment_plan (contract_id, stage_name, payment_type, plan_amount, plan_date, status, sort, remark, create_time, update_time, deleted)
VALUES
(2, '首付款', 1, 79500.00, '2026-07-15', 1, 1, '合同签订后支付30%', NOW(), NOW(), 0),
(2, '进度款', 2, 106000.00, '2026-08-30', 0, 2, '软件开发完成50%支付40%', NOW(), NOW(), 0),
(2, '验收款', 4, 79500.00, '2026-10-10', 0, 3, '项目验收后支付30%', NOW(), NOW(), 0)
ON CONFLICT DO NOTHING;

-- 合同3：销售经理专项方案（团队分成）| 一次性收款 | 已审批 | ¥1,356,000
-- 预期：按方案3第2档(80万-200万) 2.5% 计算，总提成 = 1,356,000 * 2.5% = 33,900
--       按团队分成规则分配给合同提成成员
INSERT INTO mxx_crm_contract (
    id, contract_no, customer_id, title, contract_type, amount, currency, tax_amount, total_amount,
    status, start_date, end_date, sign_date, payment_method_type, assigned_to,
    approval_status, commission_rule_id, commission_mode,
    remark, created_by, create_time, update_time, deleted
) VALUES (
    3, 'CON-20260703-0001', 3, '深圳电子设备采购合同', 1, 1200000.00, 1, 156000.00, 1356000.00,
    2, '2026-07-15', '2026-09-15', '2026-07-15', 1, 1,
    3, 3, 2,
    '大额设备采购合同，使用销售经理专项提成方案（团队分成），手动指定成员分配', 1, NOW(), NOW(), 0
) ON CONFLICT (id) DO NOTHING;

-- 合同3的手动提成成员（3人，比例合计100%）
INSERT INTO mxx_crm_contract_commission_member (contract_id, user_id, user_name, role_type, share_ratio, sort, created_by, create_time, updated_by, update_time)
VALUES
(3, 1, '管理员', 1, 0.60, 1, 1, NOW(), 1, NOW()),
(3, 2, '张三', 2, 0.25, 2, 1, NOW(), 1, NOW()),
(3, 4, '王五', 3, 0.15, 3, 1, NOW(), 1, NOW())
ON CONFLICT DO NOTHING;

-- 合同4：手动指定分成模式（无指定规则，用默认方案）| 分期收款 | 已审批 | ¥565,000
-- 预期：使用默认方案(方案1)，按个人业绩计算
INSERT INTO mxx_crm_contract (
    id, contract_no, customer_id, title, contract_type, amount, currency, tax_amount, total_amount,
    status, start_date, end_date, sign_date, payment_method_type, assigned_to,
    approval_status, commission_rule_id, commission_mode,
    remark, created_by, create_time, update_time, deleted
) VALUES (
    4, 'CON-20260704-0001', 4, '广州制造生产线改造合同', 1, 500000.00, 1, 65000.00, 565000.00,
    2, '2026-07-20', '2026-12-20', '2026-07-20', 2, 2,
    3, NULL, 1,
    '生产线改造项目，使用默认提成方案', 1, NOW(), NOW(), 0
) ON CONFLICT (id) DO NOTHING;

-- 合同4的回款计划（2期）
INSERT INTO mxx_crm_contract_payment_plan (contract_id, stage_name, payment_type, plan_amount, plan_date, status, sort, remark, create_time, update_time, deleted)
VALUES
(4, '预付款', 1, 226000.00, '2026-07-25', 1, 1, '合同签订后支付40%', NOW(), NOW(), 0),
(4, '验收款', 4, 339000.00, '2026-12-20', 0, 2, '项目验收后支付60%', NOW(), NOW(), 0)
ON CONFLICT DO NOTHING;

-- 合同5：草稿状态 | 默认方案 | 一次性收款 | ¥339,000
-- 用于测试新建/编辑合同功能
INSERT INTO mxx_crm_contract (
    id, contract_no, customer_id, title, contract_type, amount, currency, tax_amount, total_amount,
    status, start_date, end_date, sign_date, payment_method_type, assigned_to,
    approval_status, commission_rule_id, commission_mode,
    remark, created_by, create_time, update_time, deleted
) VALUES (
    5, 'CON-20260705-0001', 1, '北京科技二期扩展合同', 1, 300000.00, 1, 39000.00, 339000.00,
    1, '2026-08-01', '2026-11-01', NULL, 1, 1,
    0, NULL, 1,
    '草稿状态，用于测试新建编辑功能', 1, NOW(), NOW(), 0
) ON CONFLICT (id) DO NOTHING;

-- 合同6：团队分成 + 手动指定成员 | 大额合同 | 已审批 | ¥2,500,000
-- 预期：按方案3第3档(200万+) 3.5% 计算，总提成 = 2,500,000 * 3.5% = 87,500
--       按手动指定的2人分成
INSERT INTO mxx_crm_contract (
    id, contract_no, customer_id, title, contract_type, amount, currency, tax_amount, total_amount,
    status, start_date, end_date, sign_date, payment_method_type, assigned_to,
    approval_status, commission_rule_id, commission_mode,
    remark, created_by, create_time, update_time, deleted
) VALUES (
    6, 'CON-20260706-0001', 3, '深圳电子二期战略合作合同', 1, 2212389.38, 1, 287610.62, 2500000.00,
    2, '2026-07-25', '2027-07-25', '2026-07-25', 2, 2,
    3, 3, 2,
    '年度战略合作合同，团队分成，2人按比例分配', 1, NOW(), NOW(), 0
) ON CONFLICT (id) DO NOTHING;

-- 合同6的手动提成成员（2人，比例合计100%）
INSERT INTO mxx_crm_contract_commission_member (contract_id, user_id, user_name, role_type, share_ratio, sort, created_by, create_time, updated_by, update_time)
VALUES
(6, 2, '张三', 1, 0.70, 1, 1, NOW(), 1, NOW()),
(6, 3, '李四', 2, 0.30, 2, 1, NOW(), 1, NOW())
ON CONFLICT DO NOTHING;

-- 合同6的回款计划（4期）
INSERT INTO mxx_crm_contract_payment_plan (contract_id, stage_name, payment_type, plan_amount, plan_date, status, sort, remark, create_time, update_time, deleted)
VALUES
(6, '预付款', 1, 500000.00, '2026-08-01', 0, 1, '合同签订后支付20%', NOW(), NOW(), 0),
(6, '进度款1', 2, 750000.00, '2026-11-01', 0, 2, '第一阶段完成支付30%', NOW(), NOW(), 0),
(6, '进度款2', 2, 750000.00, '2027-03-01', 0, 3, '第二阶段完成支付30%', NOW(), NOW(), 0),
(6, '验收款', 4, 500000.00, '2027-07-25', 0, 4, '项目验收后支付20%', NOW(), NOW(), 0)
ON CONFLICT DO NOTHING;

-- ============================================================
-- 测试数据说明：
--
-- 【提成规则】5个：
--   规则1：销售部2026方案 - 个人业绩(1) - 销售部适用 - 3档阶梯 - 默认方案
--   规则2：全公司通用方案 - 个人业绩(1) - 全公司适用 - 2档阶梯
--   规则3：销售经理专项方案 - 团队分成(2) - 销售部适用 - 3档阶梯 - 3个默认成员角色
--   规则4：部门经理提成方案 - 部门经理(3) - 销售部适用 - 3档阶梯
--   规则5：销售总监提成方案 - 总监(4) - 全公司适用 - 2档阶梯
--
-- 【合同】6个：
--   合同1：方案1(个人业绩) + 一次性 + 90.4万 + 已审批 → 第2档2% → ¥18,080 → 1人
--   合同2：方案2(个人业绩) + 分期 + 26.5万 + 已审批 → 第1档1% → ¥2,650 → 1人
--   合同3：方案3(团队分成) + 手动3人 + 135.6万 + 已审批 → 第2档2.5% → ¥33,900 → 3人(60%/25%/15%)
--   合同4：默认方案 + 分期 + 56.5万 + 已审批 → 第2档2% → ¥11,300 → 1人
--   合同5：草稿 + 默认 + 33.9万 + 草稿 → 用于测试新建编辑
--   合同6：方案3(团队分成) + 手动2人 + 250万 + 已审批 → 第3档3.5% → ¥87,500 → 2人(70%/30%)
--
-- 【关联关系验证】
--   - 合同 → commission_rule_id → 提成规则（1:1）
--   - 合同 → commission_mode → 提成模式（1=自动 2=手动）
--   - 合同 → contract_commission_member → 提成成员（1:N，手动模式用）
--   - 提成规则 → commission_tier → 阶梯配置（1:N）
--   - 提成规则 → commission_rule_member → 默认成员（1:N，团队分成用）
--
-- 【计算逻辑验证】
--   预览接口 /api/system/finance/commission/preview?id=合同ID
--   可验证各合同的提成计算结果是否符合预期
-- ============================================================
