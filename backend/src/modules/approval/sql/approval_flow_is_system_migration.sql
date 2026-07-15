-- 审批流系统类型标记迁移脚本
-- 为 mxx_system_approval_flow 表添加 is_system 字段
-- 用于区分系统内置审批流（不可删除）和用户自定义审批流（可删除）

-- 1. 添加 is_system 列（允许为空，兼容旧数据）
ALTER TABLE mxx_system_approval_flow
  ADD COLUMN IF NOT EXISTS is_system INTEGER DEFAULT 0;

-- 2. 标记系统内置审批流（报价单、订单、合同）
UPDATE mxx_system_approval_flow
SET is_system = 1
WHERE flow_code IN ('quotation_approval', 'order_approval', 'contract_approval');

-- 3. 为 is_system 添加索引
CREATE INDEX IF NOT EXISTS idx_approval_flow_is_system ON mxx_system_approval_flow(is_system);