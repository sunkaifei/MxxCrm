-- 添加联系人性别字段
-- 性别：0-男，1-女，2-未知/未指定（与 system_user 表 gender 规范保持一致）

BEGIN;

ALTER TABLE public.mxx_crm_contact
ADD COLUMN IF NOT EXISTS gender INTEGER DEFAULT 2;

COMMENT ON COLUMN public.mxx_crm_contact.gender IS '性别：0-男，1-女，2-未知';

COMMIT;
