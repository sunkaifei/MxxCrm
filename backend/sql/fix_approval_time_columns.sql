-- 按项目规则修正审批模块时间字段: TIMESTAMPTZ->TIMESTAMP, created_at/updated_at->create_time/update_time
BEGIN;

-- mxx_system_approval_flow
ALTER TABLE mxx_system_approval_flow ALTER COLUMN create_time TYPE TIMESTAMP;
ALTER TABLE mxx_system_approval_flow ALTER COLUMN update_time TYPE TIMESTAMP;

-- mxx_system_approval_flow_node
ALTER TABLE mxx_system_approval_flow_node ALTER COLUMN create_time TYPE TIMESTAMP;

-- mxx_system_approval_flow_edge
ALTER TABLE mxx_system_approval_flow_edge ALTER COLUMN create_time TYPE TIMESTAMP;

-- mxx_system_approval_instance
ALTER TABLE mxx_system_approval_instance ALTER COLUMN submitted_at TYPE TIMESTAMP;
ALTER TABLE mxx_system_approval_instance ALTER COLUMN finished_at TYPE TIMESTAMP;
ALTER TABLE mxx_system_approval_instance RENAME COLUMN created_at TO create_time;
ALTER TABLE mxx_system_approval_instance RENAME COLUMN updated_at TO update_time;
ALTER TABLE mxx_system_approval_instance ALTER COLUMN create_time TYPE TIMESTAMP;
ALTER TABLE mxx_system_approval_instance ALTER COLUMN update_time TYPE TIMESTAMP;

-- mxx_system_approval_log
ALTER TABLE mxx_system_approval_log RENAME COLUMN created_at TO create_time;
ALTER TABLE mxx_system_approval_log ALTER COLUMN create_time TYPE TIMESTAMP;

COMMIT;
