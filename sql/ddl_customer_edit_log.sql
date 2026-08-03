--
-- PostgreSQL database dump
--

-- Dumped from database version 16.13 (Ubuntu 16.13-0ubuntu0.24.04.1)
-- Dumped by pg_dump version 16.0

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: mxx_crm_customer_edit_log; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.mxx_crm_customer_edit_log (
    id bigint NOT NULL,
    customer_id bigint NOT NULL,
    editor_id bigint NOT NULL,
    editor_name character varying(100),
    content jsonb NOT NULL,
    edit_time timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    deleted integer DEFAULT 0,
    log_type integer DEFAULT 0
);


ALTER TABLE public.mxx_crm_customer_edit_log OWNER TO postgres;

--
-- Name: TABLE mxx_crm_customer_edit_log; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON TABLE public.mxx_crm_customer_edit_log IS '客户修改日志表，按字段粒度记录每次修改';


--
-- Name: COLUMN mxx_crm_customer_edit_log.content; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.mxx_crm_customer_edit_log.content IS '变更内容 JSON 数组：[{"field":"phone","fieldLabel":"手机号","old":"13800138000","new":"13900139000"}]';


--
-- Name: COLUMN mxx_crm_customer_edit_log.log_type; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.mxx_crm_customer_edit_log.log_type IS '日志类型：0=基本信息, 1=财务信息';


--
-- PostgreSQL database dump complete
--

