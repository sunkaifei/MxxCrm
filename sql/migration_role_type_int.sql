-- ============================================================
-- 迁移脚本：将 role_type 从 VARCHAR 改为 INTEGER
-- 映射：decision_maker=0, influencer=1, user=2, other=3
-- 用法: psql -U <user> -d <db> -f migration_role_type_int.sql
-- ============================================================

BEGIN;

-- 1. 修改列类型，同时转换已有数据
ALTER TABLE public.mxx_crm_customer_contact_merge
ALTER COLUMN role_type TYPE integer USING (
  CASE role_type
    WHEN 'decision_maker' THEN 0
    WHEN 'influencer' THEN 1
    WHEN 'user' THEN 2
    WHEN 'other' THEN 3
    ELSE 3
  END
);

COMMIT;
