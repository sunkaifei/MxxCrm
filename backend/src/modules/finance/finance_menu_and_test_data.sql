-- ============================================================
-- 财务模块 - 菜单数据 + 角色授权 + 测试数据
-- ============================================================

-- ============================================================
-- 1. 菜单数据（mxx_system_menu）
-- ============================================================

-- 财务管理（顶级目录）
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, sort, icon, redirect, params, create_time, update_time, deleted)
VALUES (315, 0, '315', 'page.finance.title', 'FOLDER', 'Finance', '/finance', '', 'finance:index', 1, 8, 'lucide:account-book', NULL, NULL, NOW(), NOW(), 0);

-- 提成规则（菜单）
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, sort, icon, redirect, params, create_time, update_time, deleted)
VALUES (316, 315, '', 'page.finance.commissionRule.title', 'MENU', 'CommissionRule', '/finance/commission-rule', 'finance/commission-rule/index', 'finance:commission:list', 1, 1, 'lucide:percent', NULL, NULL, NOW(), NOW(), 0);

-- 提成规则按钮
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, sort, icon, redirect, params, create_time, update_time, deleted)
VALUES (317, 316, '', 'page.finance.commissionRule.button.create', 'BUTTON', '', '', '', 'finance:commission:manage', 1, 1, '', NULL, NULL, NOW(), NOW(), 0);
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, sort, icon, redirect, params, create_time, update_time, deleted)
VALUES (318, 316, '', 'page.finance.commissionRule.button.edit', 'BUTTON', '', '', '', 'finance:commission:manage', 1, 2, '', NULL, NULL, NOW(), NOW(), 0);
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, sort, icon, redirect, params, create_time, update_time, deleted)
VALUES (319, 316, '', 'page.finance.commissionRule.button.delete', 'BUTTON', '', '', '', 'finance:commission:manage', 1, 3, '', NULL, NULL, NOW(), NOW(), 0);
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, sort, icon, redirect, params, create_time, update_time, deleted)
VALUES (320, 316, '', 'page.finance.commissionRule.button.toggle', 'BUTTON', '', '', '', 'finance:commission:manage', 1, 4, '', NULL, NULL, NOW(), NOW(), 0);

-- 工资核算（菜单）
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, sort, icon, redirect, params, create_time, update_time, deleted)
VALUES (321, 315, '', 'page.finance.salary.title', 'MENU', 'Salary', '/finance/salary', 'finance/salary/index', 'finance:salary:list', 1, 2, 'lucide:wallet', NULL, NULL, NOW(), NOW(), 0);

-- 工资核算按钮
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, sort, icon, redirect, params, create_time, update_time, deleted)
VALUES (322, 321, '', 'page.finance.salary.button.calculate', 'BUTTON', '', '', '', 'finance:salary:calculate', 1, 1, '', NULL, NULL, NOW(), NOW(), 0);
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, sort, icon, redirect, params, create_time, update_time, deleted)
VALUES (323, 321, '', 'page.finance.salary.button.adjust', 'BUTTON', '', '', '', 'finance:salary:manage', 1, 2, '', NULL, NULL, NOW(), NOW(), 0);
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, sort, icon, redirect, params, create_time, update_time, deleted)
VALUES (324, 321, '', 'page.finance.salary.button.approve', 'BUTTON', '', '', '', 'finance:salary:approve', 1, 3, '', NULL, NULL, NOW(), NOW(), 0);
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, sort, icon, redirect, params, create_time, update_time, deleted)
VALUES (325, 321, '', 'page.finance.salary.button.pay', 'BUTTON', '', '', '', 'finance:salary:pay', 1, 4, '', NULL, NULL, NOW(), NOW(), 0);

-- 工资详情（隐藏菜单）
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, hide_in_menu, sort, icon, redirect, params, create_time, update_time, deleted)
VALUES (326, 315, '', 'page.finance.salary.detail', 'MENU', 'SalaryDetail', '/finance/salary/detail/:id', 'finance/salary/detail', 'finance:salary:list', 1, 1, 3, '', NULL, NULL, NOW(), NOW(), 0);

-- 采购付款（菜单）
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, sort, icon, redirect, params, create_time, update_time, deleted)
VALUES (332, 315, '', 'page.finance.payment.title', 'MENU', 'FinancePayment', '/finance/payment', 'finance/payment/index', 'finance:payment:list', 1, 4, 'lucide:credit-card', NULL, NULL, NOW(), NOW(), 0);

-- 采购付款按钮
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, sort, icon, redirect, params, create_time, update_time, deleted)
VALUES (333, 332, '', 'page.finance.payment.button.apply', 'BUTTON', '', '', '', 'finance:payment:apply', 1, 1, '', NULL, NULL, NOW(), NOW(), 0);
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, sort, icon, redirect, params, create_time, update_time, deleted)
VALUES (334, 332, '', 'page.finance.payment.button.approve', 'BUTTON', '', '', '', 'finance:payment:approve', 1, 2, '', NULL, NULL, NOW(), NOW(), 0);
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, sort, icon, redirect, params, create_time, update_time, deleted)
VALUES (335, 332, '', 'page.finance.payment.button.confirm', 'BUTTON', '', '', '', 'finance:payment:confirm', 1, 3, '', NULL, NULL, NOW(), NOW(), 0);
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, sort, icon, redirect, params, create_time, update_time, deleted)
VALUES (336, 332, '', 'page.finance.payment.button.cancel', 'BUTTON', '', '', '', 'finance:payment:manage', 1, 4, '', NULL, NULL, NOW(), NOW(), 0);


-- ============================================================
-- 2. 角色菜单授权（mxx_system_role_menu_merge）
--    为系统管理员（role_id=5）分配所有财务菜单
-- ============================================================

INSERT INTO mxx_system_role_menu_merge (menu_id, role_id, status, create_time, update_time)
VALUES
(315, 5, 1, NOW(), NOW()),
(316, 5, 1, NOW(), NOW()),
(317, 5, 1, NOW(), NOW()),
(318, 5, 1, NOW(), NOW()),
(319, 5, 1, NOW(), NOW()),
(320, 5, 1, NOW(), NOW()),
(321, 5, 1, NOW(), NOW()),
(322, 5, 1, NOW(), NOW()),
(323, 5, 1, NOW(), NOW()),
(324, 5, 1, NOW(), NOW()),
(325, 5, 1, NOW(), NOW()),
(326, 5, 1, NOW(), NOW()),
(332, 5, 1, NOW(), NOW()),
(333, 5, 1, NOW(), NOW()),
(334, 5, 1, NOW(), NOW()),
(335, 5, 1, NOW(), NOW()),
(336, 5, 1, NOW(), NOW());


-- ============================================================
-- 3. 测试数据 - 提成规则
-- ============================================================

-- 规则1：销售部2026提成方案（适用销售部全员）
INSERT INTO mxx_finance_commission_rule (id, rule_name, department_id, post_id, trigger_condition, effective_date, expiry_date, enabled, description, created_by, create_time, update_time, deleted)
VALUES (1, '销售部2026提成方案', 1, NULL, 1, '2026-01-01', '2026-12-31', 1, '销售部全员提成方案，按完全回款阶梯计算', 1, NOW(), NOW(), 0);

-- 规则1的阶梯
INSERT INTO mxx_finance_commission_tier (rule_id, min_amount, max_amount, commission_rate, sort)
VALUES
(1, 0, 500000, 0.0150, 1),
(1, 500000, 1000000, 0.0200, 2),
(1, 1000000, NULL, 0.0300, 3);

-- 规则2：全公司通用提成方案
INSERT INTO mxx_finance_commission_rule (id, rule_name, department_id, post_id, trigger_condition, effective_date, expiry_date, enabled, description, created_by, create_time, update_time, deleted)
VALUES (2, '全公司通用提成方案', NULL, NULL, 1, '2026-01-01', NULL, 1, '适用于所有部门的默认提成方案', 1, NOW(), NOW(), 0);

-- 规则2的阶梯
INSERT INTO mxx_finance_commission_tier (rule_id, min_amount, max_amount, commission_rate, sort)
VALUES
(2, 0, 300000, 0.0100, 1),
(2, 300000, NULL, 0.0150, 2);

-- 规则3：销售经理专项提成方案
INSERT INTO mxx_finance_commission_rule (id, rule_name, department_id, post_id, trigger_condition, effective_date, expiry_date, enabled, description, created_by, create_time, update_time, deleted)
VALUES (3, '销售经理专项提成方案', 1, 2, 1, '2026-01-01', '2026-12-31', 1, '销售经理岗位专属提成，比例更高', 1, NOW(), NOW(), 0);

-- 规则3的阶梯
INSERT INTO mxx_finance_commission_tier (rule_id, min_amount, max_amount, commission_rate, sort)
VALUES
(3, 0, 800000, 0.0200, 1),
(3, 800000, 2000000, 0.0250, 2),
(3, 2000000, NULL, 0.0350, 3);


-- ============================================================
-- 4. 测试数据 - 采购付款记录
-- ============================================================

INSERT INTO mxx_finance_payment (payment_no, purchase_order_id, purchase_order_no, supplier_name, payment_type, payment_amount, payment_method, bank_account, status, applicant_id, applicant_name, apply_time, approver_id, approver_name, approve_time, approve_remark, payment_date, remark, created_by, create_time, update_time, deleted)
VALUES
('PAY-202606-001', 1, 'PO-202606001', '北京办公用品供应商', 1, 50000.00, 1, '6228480401234567890', 3, 1, '管理员', '2026-06-10 11:00:00', 1, '管理员', '2026-06-11 09:00:00', '同意付款', '2026-06-12', '预付款30%', 1, NOW(), NOW(), 0),
('PAY-202606-002', 2, 'PO-202606002', '深圳电子元件有限公司', 3, 120000.00, 1, '6228480409876543210', 0, 2, '张三', '2026-06-22 15:30:00', NULL, NULL, NULL, NULL, NULL, '尾款70%', 2, NOW(), NOW(), 0),
('PAY-202606-003', 3, 'PO-202606003', '上海物流服务有限公司', 4, 35000.00, 2, '', 1, 1, '管理员', '2026-06-28 10:00:00', 1, '管理员', '2026-06-28 14:00:00', '同意全额付款', NULL, '物流服务费', 1, NOW(), NOW(), 0);
