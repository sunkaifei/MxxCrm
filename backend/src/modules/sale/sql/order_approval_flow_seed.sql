-- 订单审批流程种子数据
-- 插入订单审批流程定义到 mxx_system_approval_flow 表
-- 前置条件：需要先通过后台管理界面创建审批流程，或手动执行以下SQL

-- 插入审批流程（如果不存在，系统内置 is_system = 1）
INSERT INTO mxx_system_approval_flow (flow_code, flow_name, business_type, description, enabled, is_system, create_time, update_time)
SELECT 'order_approval', '订单审批', 'order', '销售订单审批流程', 1, 1, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow WHERE flow_code = 'order_approval');