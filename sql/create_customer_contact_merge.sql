-- ============================================================
-- 创建 mxx_crm_customer_contact_merge 表
-- 用于管理客户与联系人的绑定关系（支持历史记录）
-- ============================================================

BEGIN;

CREATE TABLE IF NOT EXISTS public.mxx_crm_customer_contact_merge (
    id bigint NOT NULL,
    customer_id bigint NOT NULL,
    contact_id bigint NOT NULL,
    title character varying(128),
    role_type character varying(32),
    is_current boolean DEFAULT true,
    is_primary boolean DEFAULT false,
    is_billing boolean DEFAULT false,
    is_shipping boolean DEFAULT false,
    bound_at timestamp without time zone,
    unbound_at timestamp without time zone,
    notes character varying(512),
    create_time timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    update_time timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);

CREATE SEQUENCE IF NOT EXISTS public.mxx_crm_customer_contact_merge_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.mxx_crm_customer_contact_merge_id_seq OWNED BY public.mxx_crm_customer_contact_merge.id;

ALTER TABLE ONLY public.mxx_crm_customer_contact_merge
    ADD CONSTRAINT mxx_crm_customer_contact_merge_pkey PRIMARY KEY (id);

CREATE INDEX IF NOT EXISTS idx_customer_contact_merge_customer_id ON public.mxx_crm_customer_contact_merge(customer_id);
CREATE INDEX IF NOT EXISTS idx_customer_contact_merge_contact_id ON public.mxx_crm_customer_contact_merge(contact_id);
CREATE INDEX IF NOT EXISTS idx_customer_contact_merge_current ON public.mxx_crm_customer_contact_merge(is_current);

COMMIT;
