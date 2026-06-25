-- 合同审批流演示数据: <=5000 部门审核->结束; >5000 部门经理审核->管理员审核->结束
BEGIN;
DELETE FROM mxx_system_approval_flow_node WHERE flow_id = 1;
DELETE FROM mxx_system_approval_flow_edge WHERE flow_id = 1;

-- 节点 (approver_id: 6=销售经理, 5=销售总监, 3=超级管理员)
INSERT INTO mxx_system_approval_flow_node (flow_id, node_key, node_type, node_order, node_name, approver_type, approver_id, is_final, position_x, position_y, create_time) VALUES
(1, 'start',          1, 1, '开始',           NULL, NULL, 0, 100,  250, NOW()),
(1, 'dept_review',    2, 2, '部门审核',        1,    6,    0, 300,  250, NOW()),
(1, 'amount_check',   3, 3, '金额判断',        NULL, NULL, 0, 500,  250, NOW()),
(1, 'end_small',      4, 4, '结束',           NULL, NULL, 1, 700,  120, NOW()),
(1, 'manager_review', 2, 5, '部门经理审核',    1,    5,    0, 700,  400, NOW()),
(1, 'admin_review',   2, 6, '管理员审核',      1,    3,    0, 900,  400, NOW()),
(1, 'end_large',      4, 7, '结束',           NULL, NULL, 1, 1100, 400, NOW());

-- 连线
INSERT INTO mxx_system_approval_flow_edge (flow_id, source_node_key, target_node_key, condition_expr, label, create_time) VALUES
(1, 'start',         'dept_review',    NULL,                NULL,    NOW()),
(1, 'dept_review',   'amount_check',   NULL,                NULL,    NOW()),
(1, 'amount_check',  'end_small',      'amount <= 5000',    '<=5000', NOW()),
(1, 'amount_check',  'manager_review', 'amount > 5000',     '>5000',  NOW()),
(1, 'manager_review','admin_review',   NULL,                NULL,    NOW()),
(1, 'admin_review',  'end_large',      NULL,                NULL,    NOW());

COMMIT;
