-- v46: 系统预置审批流模板
-- 使用 approver_type=7（部门主管及上级链），优先走部门树，无需手动维护 direct_manager_id
-- 支持金额阈值条件分支

-- ============================================================
-- 1. 合同审批流（含金额条件分支）
-- ============================================================
INSERT INTO mxx_system_approval_flow (flow_code, flow_name, business_type, description, enabled, is_system, create_time, update_time)
SELECT 'contract_approval', '合同审批', 'contract', '合同审批流程：金额≤10000部门主管审批，>10000部门主管→上级主管', 1, 1, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow WHERE flow_code = 'contract_approval');

-- 获取 flow_id（使用 flow_code 定位）
-- 节点
INSERT INTO mxx_system_approval_flow_node (flow_id, node_key, node_type, node_order, node_name, approver_type, approver_id, approve_mode, is_final, create_time)
SELECT f.id, 'start',     1, 0, '开始',          NULL, NULL, NULL, NULL, NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'contract_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_node n WHERE n.flow_id = f.id AND n.node_key = 'start');

INSERT INTO mxx_system_approval_flow_node (flow_id, node_key, node_type, node_order, node_name, approver_type, approver_id, approve_mode, is_final, create_time)
SELECT f.id, 'cond_amount', 3, 1, '金额条件分支',   NULL, NULL, NULL, NULL, NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'contract_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_node n WHERE n.flow_id = f.id AND n.node_key = 'cond_amount');

INSERT INTO mxx_system_approval_flow_node (flow_id, node_key, node_type, node_order, node_name, approver_type, approver_id, approve_mode, is_final, create_time)
SELECT f.id, 'dept_leader', 2, 2, '部门主管审批',   7, 1, 1, NULL, NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'contract_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_node n WHERE n.flow_id = f.id AND n.node_key = 'dept_leader');

INSERT INTO mxx_system_approval_flow_node (flow_id, node_key, node_type, node_order, node_name, approver_type, approver_id, approve_mode, is_final, create_time)
SELECT f.id, 'upper_leader', 2, 3, '上级主管审批',   7, 2, 1, NULL, NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'contract_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_node n WHERE n.flow_id = f.id AND n.node_key = 'upper_leader');

INSERT INTO mxx_system_approval_flow_node (flow_id, node_key, node_type, node_order, node_name, approver_type, approver_id, approve_mode, is_final, create_time)
SELECT f.id, 'end',        4, 4, '结束',           NULL, NULL, NULL, 1,    NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'contract_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_node n WHERE n.flow_id = f.id AND n.node_key = 'end');

-- 边
INSERT INTO mxx_system_approval_flow_edge (flow_id, source_node_key, target_node_key, condition_expr, label, create_time)
SELECT f.id, 'start',        'cond_amount',   NULL,          NULL, NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'contract_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_edge e WHERE e.flow_id = f.id AND e.source_node_key = 'start' AND e.target_node_key = 'cond_amount');

INSERT INTO mxx_system_approval_flow_edge (flow_id, source_node_key, target_node_key, condition_expr, label, create_time)
SELECT f.id, 'cond_amount',  'dept_leader',   'amount<=10000', '≤10000', NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'contract_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_edge e WHERE e.flow_id = f.id AND e.source_node_key = 'cond_amount' AND e.target_node_key = 'dept_leader');

INSERT INTO mxx_system_approval_flow_edge (flow_id, source_node_key, target_node_key, condition_expr, label, create_time)
SELECT f.id, 'cond_amount',  'upper_leader',  'amount>10000',  '>10000', NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'contract_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_edge e WHERE e.flow_id = f.id AND e.source_node_key = 'cond_amount' AND e.target_node_key = 'upper_leader');

INSERT INTO mxx_system_approval_flow_edge (flow_id, source_node_key, target_node_key, condition_expr, label, create_time)
SELECT f.id, 'dept_leader',  'end',           NULL,          NULL, NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'contract_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_edge e WHERE e.flow_id = f.id AND e.source_node_key = 'dept_leader' AND e.target_node_key = 'end');

INSERT INTO mxx_system_approval_flow_edge (flow_id, source_node_key, target_node_key, condition_expr, label, create_time)
SELECT f.id, 'upper_leader', 'end',           NULL,          NULL, NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'contract_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_edge e WHERE e.flow_id = f.id AND e.source_node_key = 'upper_leader' AND e.target_node_key = 'end');

-- ============================================================
-- 2. 报销审批流（含金额条件分支）
-- ============================================================
INSERT INTO mxx_system_approval_flow (flow_code, flow_name, business_type, description, enabled, is_system, create_time, update_time)
SELECT 'expense_approval', '报销审批', 'expense', '报销审批流程：金额≤5000部门主管审批，>5000部门主管→上级主管', 1, 1, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow WHERE flow_code = 'expense_approval');

INSERT INTO mxx_system_approval_flow_node (flow_id, node_key, node_type, node_order, node_name, approver_type, approver_id, approve_mode, is_final, create_time)
SELECT f.id, 'start',       1, 0, '开始',          NULL, NULL, NULL, NULL, NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'expense_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_node n WHERE n.flow_id = f.id AND n.node_key = 'start');

INSERT INTO mxx_system_approval_flow_node (flow_id, node_key, node_type, node_order, node_name, approver_type, approver_id, approve_mode, is_final, create_time)
SELECT f.id, 'cond_amount', 3, 1, '金额条件分支',   NULL, NULL, NULL, NULL, NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'expense_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_node n WHERE n.flow_id = f.id AND n.node_key = 'cond_amount');

INSERT INTO mxx_system_approval_flow_node (flow_id, node_key, node_type, node_order, node_name, approver_type, approver_id, approve_mode, is_final, create_time)
SELECT f.id, 'dept_leader', 2, 2, '部门主管审批',   7, 1, 1, NULL, NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'expense_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_node n WHERE n.flow_id = f.id AND n.node_key = 'dept_leader');

INSERT INTO mxx_system_approval_flow_node (flow_id, node_key, node_type, node_order, node_name, approver_type, approver_id, approve_mode, is_final, create_time)
SELECT f.id, 'upper_leader', 2, 3, '上级主管审批',   7, 2, 1, NULL, NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'expense_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_node n WHERE n.flow_id = f.id AND n.node_key = 'upper_leader');

INSERT INTO mxx_system_approval_flow_node (flow_id, node_key, node_type, node_order, node_name, approver_type, approver_id, approve_mode, is_final, create_time)
SELECT f.id, 'end',         4, 4, '结束',           NULL, NULL, NULL, 1,    NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'expense_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_node n WHERE n.flow_id = f.id AND n.node_key = 'end');

INSERT INTO mxx_system_approval_flow_edge (flow_id, source_node_key, target_node_key, condition_expr, label, create_time)
SELECT f.id, 'start',        'cond_amount',   NULL,          NULL, NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'expense_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_edge e WHERE e.flow_id = f.id AND e.source_node_key = 'start' AND e.target_node_key = 'cond_amount');

INSERT INTO mxx_system_approval_flow_edge (flow_id, source_node_key, target_node_key, condition_expr, label, create_time)
SELECT f.id, 'cond_amount',  'dept_leader',   'amount<=5000', '≤5000', NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'expense_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_edge e WHERE e.flow_id = f.id AND e.source_node_key = 'cond_amount' AND e.target_node_key = 'dept_leader');

INSERT INTO mxx_system_approval_flow_edge (flow_id, source_node_key, target_node_key, condition_expr, label, create_time)
SELECT f.id, 'cond_amount',  'upper_leader',  'amount>5000',  '>5000', NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'expense_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_edge e WHERE e.flow_id = f.id AND e.source_node_key = 'cond_amount' AND e.target_node_key = 'upper_leader');

INSERT INTO mxx_system_approval_flow_edge (flow_id, source_node_key, target_node_key, condition_expr, label, create_time)
SELECT f.id, 'dept_leader',  'end',           NULL,          NULL, NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'expense_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_edge e WHERE e.flow_id = f.id AND e.source_node_key = 'dept_leader' AND e.target_node_key = 'end');

INSERT INTO mxx_system_approval_flow_edge (flow_id, source_node_key, target_node_key, condition_expr, label, create_time)
SELECT f.id, 'upper_leader', 'end',           NULL,          NULL, NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'expense_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_edge e WHERE e.flow_id = f.id AND e.source_node_key = 'upper_leader' AND e.target_node_key = 'end');

-- ============================================================
-- 3. 请假审批流（简单单级：部门主管审批）
-- ============================================================
INSERT INTO mxx_system_approval_flow (flow_code, flow_name, business_type, description, enabled, is_system, create_time, update_time)
SELECT 'leave_approval', '请假审批', 'leave', '请假审批流程：部门主管审批（3天以上转上级主管）', 1, 1, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow WHERE flow_code = 'leave_approval');

INSERT INTO mxx_system_approval_flow_node (flow_id, node_key, node_type, node_order, node_name, approver_type, approver_id, approve_mode, is_final, create_time)
SELECT f.id, 'start',       1, 0, '开始',          NULL, NULL, NULL, NULL, NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'leave_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_node n WHERE n.flow_id = f.id AND n.node_key = 'start');

INSERT INTO mxx_system_approval_flow_node (flow_id, node_key, node_type, node_order, node_name, approver_type, approver_id, approve_mode, is_final, create_time)
SELECT f.id, 'cond_days',  3, 1, '天数条件分支',   NULL, NULL, NULL, NULL, NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'leave_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_node n WHERE n.flow_id = f.id AND n.node_key = 'cond_days');

INSERT INTO mxx_system_approval_flow_node (flow_id, node_key, node_type, node_order, node_name, approver_type, approver_id, approve_mode, is_final, create_time)
SELECT f.id, 'dept_leader', 2, 2, '部门主管审批',   7, 1, 1, NULL, NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'leave_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_node n WHERE n.flow_id = f.id AND n.node_key = 'dept_leader');

INSERT INTO mxx_system_approval_flow_node (flow_id, node_key, node_type, node_order, node_name, approver_type, approver_id, approve_mode, is_final, create_time)
SELECT f.id, 'upper_leader', 2, 3, '上级主管审批',   7, 2, 1, NULL, NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'leave_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_node n WHERE n.flow_id = f.id AND n.node_key = 'upper_leader');

INSERT INTO mxx_system_approval_flow_node (flow_id, node_key, node_type, node_order, node_name, approver_type, approver_id, approve_mode, is_final, create_time)
SELECT f.id, 'end',         4, 4, '结束',           NULL, NULL, NULL, 1,    NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'leave_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_node n WHERE n.flow_id = f.id AND n.node_key = 'end');

INSERT INTO mxx_system_approval_flow_edge (flow_id, source_node_key, target_node_key, condition_expr, label, create_time)
SELECT f.id, 'start',        'cond_days',    NULL,         NULL, NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'leave_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_edge e WHERE e.flow_id = f.id AND e.source_node_key = 'start' AND e.target_node_key = 'cond_days');

INSERT INTO mxx_system_approval_flow_edge (flow_id, source_node_key, target_node_key, condition_expr, label, create_time)
SELECT f.id, 'cond_days',   'dept_leader',  'days<=3',    '≤3天', NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'leave_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_edge e WHERE e.flow_id = f.id AND e.source_node_key = 'cond_days' AND e.target_node_key = 'dept_leader');

INSERT INTO mxx_system_approval_flow_edge (flow_id, source_node_key, target_node_key, condition_expr, label, create_time)
SELECT f.id, 'cond_days',   'upper_leader', 'days>3',     '>3天', NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'leave_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_edge e WHERE e.flow_id = f.id AND e.source_node_key = 'cond_days' AND e.target_node_key = 'upper_leader');

INSERT INTO mxx_system_approval_flow_edge (flow_id, source_node_key, target_node_key, condition_expr, label, create_time)
SELECT f.id, 'dept_leader',  'end',          NULL,         NULL, NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'leave_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_edge e WHERE e.flow_id = f.id AND e.source_node_key = 'dept_leader' AND e.target_node_key = 'end');

INSERT INTO mxx_system_approval_flow_edge (flow_id, source_node_key, target_node_key, condition_expr, label, create_time)
SELECT f.id, 'upper_leader', 'end',          NULL,         NULL, NOW() FROM mxx_system_approval_flow f WHERE f.flow_code = 'leave_approval'
AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_edge e WHERE e.flow_id = f.id AND e.source_node_key = 'upper_leader' AND e.target_node_key = 'end');

-- ============================================================
-- 验证
-- ============================================================
SELECT f.flow_code, f.flow_name, f.business_type, f.enabled, f.is_system,
       (SELECT COUNT(*) FROM mxx_system_approval_flow_node n WHERE n.flow_id = f.id) AS node_count,
       (SELECT COUNT(*) FROM mxx_system_approval_flow_edge e WHERE e.flow_id = f.id) AS edge_count
FROM mxx_system_approval_flow f
WHERE f.is_system = 1
ORDER BY f.flow_code;