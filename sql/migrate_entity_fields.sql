BEGIN TRANSACTION;

-- ============================================
-- mxx_crm_followup 表
-- 添加缺失的 updated_by 字段（deleted 已存在）
-- ============================================
ALTER TABLE mxx_crm_followup ADD COLUMN IF NOT EXISTS updated_by BIGINT;

-- ============================================
-- mxx_crm_lead 表
-- 添加缺失的 updated_by, updated_at, deleted 字段
-- ============================================
ALTER TABLE mxx_crm_lead ADD COLUMN IF NOT EXISTS updated_by BIGINT;
ALTER TABLE mxx_crm_lead ADD COLUMN IF NOT EXISTS updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP;
ALTER TABLE mxx_crm_lead ADD COLUMN IF NOT EXISTS deleted INTEGER DEFAULT 0;

-- ============================================
-- mxx_crm_customer 表
-- 添加缺失的 updated_by, updated_at, deleted 字段
-- ============================================
ALTER TABLE mxx_crm_customer ADD COLUMN IF NOT EXISTS updated_by BIGINT;
ALTER TABLE mxx_crm_customer ADD COLUMN IF NOT EXISTS updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP;
ALTER TABLE mxx_crm_customer ADD COLUMN IF NOT EXISTS deleted INTEGER DEFAULT 0;

-- ============================================
-- mxx_crm_opportunity 表
-- 添加缺失的 deleted 字段
-- ============================================
ALTER TABLE mxx_crm_opportunity ADD COLUMN IF NOT EXISTS deleted INTEGER DEFAULT 0;

-- ============================================
-- mxx_crm_contract 表
-- 添加缺失字段
-- ============================================
ALTER TABLE mxx_crm_contract ADD COLUMN IF NOT EXISTS contract_type TEXT;
ALTER TABLE mxx_crm_contract ADD COLUMN IF NOT EXISTS tax_amount DECIMAL(18,4);
ALTER TABLE mxx_crm_contract ADD COLUMN IF NOT EXISTS total_amount DECIMAL(18,4);
ALTER TABLE mxx_crm_contract ADD COLUMN IF NOT EXISTS contract_file TEXT;
ALTER TABLE mxx_crm_contract ADD COLUMN IF NOT EXISTS remark TEXT;

COMMIT;