-- 订单审批流迁移脚本
-- 为 mxx_sale_order 表添加审批流相关字段

-- 添加审批状态字段（0-草稿, 1-待审批, 2-审批中, 3-已通过, 4-已驳回）
ALTER TABLE mxx_sale_order
  ADD COLUMN IF NOT EXISTS approval_status INTEGER DEFAULT 0;

-- 添加审批实例ID字段（关联 mxx_system_approval_instance）
ALTER TABLE mxx_sale_order
  ADD COLUMN IF NOT EXISTS instance_id BIGINT;

-- 为审批状态添加索引
CREATE INDEX IF NOT EXISTS idx_mxx_sale_order_approval_status ON mxx_sale_order(approval_status);

-- 为审批实例ID添加索引
CREATE INDEX IF NOT EXISTS idx_mxx_sale_order_instance_id ON mxx_sale_order(instance_id);

-- 为新字段添加注释
COMMENT ON COLUMN mxx_sale_order.approval_status IS '审批状态：0-草稿, 1-待审批, 2-审批中, 3-已通过, 4-已驳回';
COMMENT ON COLUMN mxx_sale_order.instance_id IS '审批实例ID，关联 mxx_system_approval_instance 表';