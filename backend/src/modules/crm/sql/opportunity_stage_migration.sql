-- 商机表迁移：添加新字段 + 修改阶段值
-- 1. 添加 contact_id, lead_id 列（之前是 #[sea_orm(ignore)]，现在需要入库）
-- 2. 添加阶段内容字段
-- 3. 添加后续跟踪状态字段
-- 4. 迁移 stage 值：0→1, 1→2, 2→3, 3→4, 4→5, 5→5

-- 添加关联字段
ALTER TABLE mxx_crm_opportunity ADD COLUMN IF NOT EXISTS contact_id BIGINT;
ALTER TABLE mxx_crm_opportunity ADD COLUMN IF NOT EXISTS lead_id BIGINT;

-- 添加阶段内容字段
ALTER TABLE mxx_crm_opportunity ADD COLUMN IF NOT EXISTS requirement_summary TEXT;
ALTER TABLE mxx_crm_opportunity ADD COLUMN IF NOT EXISTS solution_summary TEXT;

-- 添加后续跟踪状态字段
ALTER TABLE mxx_crm_opportunity ADD COLUMN IF NOT EXISTS quote_status INT DEFAULT 0;
ALTER TABLE mxx_crm_opportunity ADD COLUMN IF NOT EXISTS order_status INT DEFAULT 0;
ALTER TABLE mxx_crm_opportunity ADD COLUMN IF NOT EXISTS contract_status INT DEFAULT 0;
ALTER TABLE mxx_crm_opportunity ADD COLUMN IF NOT EXISTS shipment_status INT DEFAULT 0;
ALTER TABLE mxx_crm_opportunity ADD COLUMN IF NOT EXISTS payment_status INT DEFAULT 0;
ALTER TABLE mxx_crm_opportunity ADD COLUMN IF NOT EXISTS invoice_status INT DEFAULT 0;

-- 迁移阶段值：旧 0-5 → 新 1-5
-- 0(资格审查) → 1(初步沟通)
-- 1(需求分析) → 2(需求确认)
-- 2(方案报价) → 3(方案沟通)
-- 3(商务谈判) → 4(已报价)
-- 4(已成交)   → 5(成交)
-- 5(已输单)   → 5(丢单，通过 loss_reason 判断)
UPDATE mxx_crm_opportunity SET stage = stage + 1 WHERE stage IS NOT NULL AND stage < 5 AND deleted = 0;
-- stage=5(已输单) 保持为5，通过 loss_reason 区分

-- 添加索引
CREATE INDEX IF NOT EXISTS idx_opportunity_contact_id ON mxx_crm_opportunity(contact_id);
CREATE INDEX IF NOT EXISTS idx_opportunity_lead_id ON mxx_crm_opportunity(lead_id);
