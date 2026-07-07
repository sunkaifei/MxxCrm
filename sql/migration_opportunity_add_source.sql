-- 为 mxx_crm_opportunity 表添加 source 列（LeadSource 枚举类型）
ALTER TABLE public.mxx_crm_opportunity
    ADD COLUMN IF NOT EXISTS source public.mxx_lead_source;

COMMENT ON COLUMN public.mxx_crm_opportunity.source IS '商机来源';
