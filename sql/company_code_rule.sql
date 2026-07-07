-- =============================================================
-- 公司文件编号规则配置系统 - 数据库初始化脚本
-- 包含：1) mxx_company_info 加字段  2) mxx_company_code_rule 建表
--       3) mxx_system_code_sequence 建表  4) 预置规则数据
--       5) 菜单与权限 (id 从 366 开始，company_menu.sql 已用到 361-365)
-- 日期：2026-07-04
-- =============================================================

BEGIN;

-- -------------------------------------------------------------
-- 1. 扩展企业信息表：增加 company_abbr / show_abbr 字段
-- -------------------------------------------------------------
ALTER TABLE mxx_company_info
    ADD COLUMN IF NOT EXISTS company_abbr VARCHAR(20),
    ADD COLUMN IF NOT EXISTS show_abbr INT2 DEFAULT 1;

COMMENT ON COLUMN mxx_company_info.company_abbr IS '公司简称，如 XYH，用于编号段位';
COMMENT ON COLUMN mxx_company_info.show_abbr IS '是否在编号中显示公司简称：1=是 0=否';

-- 若企业信息表无记录，插入一条默认记录
INSERT INTO mxx_company_info (id, company_name, company_abbr, show_abbr, deleted, create_time, update_time)
SELECT 1, '北京心月狐科技有限公司', 'XYH', 1, 0, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_company_info WHERE id = 1);

-- 更新已有记录的简称（若为空）
UPDATE mxx_company_info
SET company_abbr = 'XYH', show_abbr = 1
WHERE id = 1 AND company_abbr IS NULL;


-- -------------------------------------------------------------
-- 2. 编号规则配置表 mxx_company_code_rule
-- -------------------------------------------------------------
CREATE TABLE IF NOT EXISTS mxx_company_code_rule (
    id BIGSERIAL PRIMARY KEY,
    module_code VARCHAR(50) NOT NULL,
    module_name VARCHAR(100) NOT NULL,
    rule_name VARCHAR(100),
    company_abbr VARCHAR(20),
    dept_code VARCHAR(50),
    biz_type_code VARCHAR(20),
    separator VARCHAR(5) DEFAULT '-'::varchar,
    segments JSONB NOT NULL DEFAULT '[]'::jsonb,
    seq_length INT2 DEFAULT 4,
    enabled INT2 DEFAULT 1,
    remark VARCHAR(200),
    deleted INT2 DEFAULT 0,
    created_by BIGINT,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_by BIGINT,
    update_time TIMESTAMP
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_code_rule_module
    ON mxx_company_code_rule(module_code)
    WHERE deleted = 0;

COMMENT ON TABLE mxx_company_code_rule IS '公司编号规则配置表，按模块存储编号生成规则';
COMMENT ON COLUMN mxx_company_code_rule.module_code IS '模块编码，如 customer/order/contract/tech_doc';
COMMENT ON COLUMN mxx_company_code_rule.segments IS '段位配置数组 JSON，按 sort 排序拼接';
COMMENT ON COLUMN mxx_company_code_rule.seq_length IS '流水号位数，默认 4（0001-9999）';


-- -------------------------------------------------------------
-- 3. 流水号计数表 mxx_system_code_sequence
--    注意：表名沿用设计文档（mxx_system_code_sequence），
--         与规则表前缀不一致是设计文档原貌，保持兼容。
-- -------------------------------------------------------------
CREATE TABLE IF NOT EXISTS mxx_system_code_sequence (
    id BIGSERIAL PRIMARY KEY,
    module_code VARCHAR(50) NOT NULL,
    year INT4 NOT NULL,
    dept_code VARCHAR(20) DEFAULT ''::varchar,
    current_seq INT4 DEFAULT 0,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_company_code_seq_dim
    ON mxx_system_code_sequence(module_code, year, dept_code);

COMMENT ON TABLE mxx_system_code_sequence IS '编号流水号计数表，按 模块+年份+部门 维度独立计数';
COMMENT ON COLUMN mxx_system_code_sequence.current_seq IS '当前已分配的最大流水号，下一次生成编号会用 current_seq+1';


-- -------------------------------------------------------------
-- 4. 预置默认编号规则数据
--    段位 segments JSON：
--      { "type": "company|biz_type|year|dept|seq|version|fixed|date",
--        "value": "...",        // type=fixed/company/biz_type 时使用
--        "format": "yyyy|yy",   // type=year 时使用
--        "source": "current|business_date|create_time",  // type=year 时使用
--        "length": 4,           // type=seq 时使用
--        "sort": 1 }
-- -------------------------------------------------------------
INSERT INTO mxx_company_code_rule (module_code, module_name, rule_name, company_abbr, dept_code, biz_type_code, separator, segments, seq_length, enabled, remark, deleted, create_time, update_time)
SELECT 'customer', '客户管理', '客户编号规则', 'XYH', NULL, 'KH', '-',
       '[{"type":"company","sort":1},{"type":"biz_type","sort":2},{"type":"year","format":"yyyy","source":"current","sort":3},{"type":"dept","sort":4},{"type":"seq","length":4,"sort":5}]'::jsonb,
       4, 1, '公司简称-业务类型-年份-部门-流水号，例：XYH-KH-2026-XS-0001', 0, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_company_code_rule WHERE module_code = 'customer' AND deleted = 0);

INSERT INTO mxx_company_code_rule (module_code, module_name, rule_name, company_abbr, dept_code, biz_type_code, separator, segments, seq_length, enabled, remark, deleted, create_time, update_time)
SELECT 'order', '销售订单', '订单编号规则', 'XYH', NULL, 'HT', '-',
       '[{"type":"company","sort":1},{"type":"biz_type","sort":2},{"type":"year","format":"yyyy","source":"current","sort":3},{"type":"dept","sort":4},{"type":"seq","length":4,"sort":5}]'::jsonb,
       4, 1, '公司简称-业务类型-年份-部门-流水号，例：XYH-HT-2026-XS-0001', 0, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_company_code_rule WHERE module_code = 'order' AND deleted = 0);

INSERT INTO mxx_company_code_rule (module_code, module_name, rule_name, company_abbr, dept_code, biz_type_code, separator, segments, seq_length, enabled, remark, deleted, create_time, update_time)
SELECT 'contract', '合同管理', '合同编号规则', 'XYH', NULL, 'HT', '-',
       '[{"type":"company","sort":1},{"type":"year","format":"yyyy","source":"business_date","sort":2},{"type":"dept","sort":3},{"type":"seq","length":4,"sort":4}]'::jsonb,
       4, 1, '公司简称-年份-部门-流水号（合同使用业务日期年份），例：XYH-2026-XS-0001', 0, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_company_code_rule WHERE module_code = 'contract' AND deleted = 0);

INSERT INTO mxx_company_code_rule (module_code, module_name, rule_name, company_abbr, dept_code, biz_type_code, separator, segments, seq_length, enabled, remark, deleted, create_time, update_time)
SELECT 'tech_doc', '技术文档', '技术文档编号规则', 'XYH', NULL, 'JS', '-',
       '[{"type":"company","sort":1},{"type":"biz_type","sort":2},{"type":"year","format":"yyyy","source":"business_date","sort":3},{"type":"seq","length":4,"sort":4},{"type":"version","sort":5}]'::jsonb,
       4, 1, '公司简称-业务类型-年份-流水号-版本号，例：XYH-JS-2022-0001-V1', 0, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_company_code_rule WHERE module_code = 'tech_doc' AND deleted = 0);

INSERT INTO mxx_company_code_rule (module_code, module_name, rule_name, company_abbr, dept_code, biz_type_code, separator, segments, seq_length, enabled, remark, deleted, create_time, update_time)
SELECT 'mgmt_doc', '企业管理文件', '管理文件编号规则', 'XYH', NULL, 'GL', '-',
       '[{"type":"company","sort":1},{"type":"biz_type","sort":2},{"type":"year","format":"yyyy","source":"business_date","sort":3},{"type":"seq","length":4,"sort":4},{"type":"version","sort":5}]'::jsonb,
       4, 1, '公司简称-业务类型-年份-流水号-版本号，例：XYH-GL-2022-0001-V1', 0, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_company_code_rule WHERE module_code = 'mgmt_doc' AND deleted = 0);

INSERT INTO mxx_company_code_rule (module_code, module_name, rule_name, company_abbr, dept_code, biz_type_code, separator, segments, seq_length, enabled, remark, deleted, create_time, update_time)
SELECT 'standard', '执行标准', '执行标准编号规则', 'XYH', NULL, 'BZ', '-',
       '[{"type":"company","sort":1},{"type":"biz_type","sort":2},{"type":"year","format":"yyyy","source":"business_date","sort":3},{"type":"seq","length":4,"sort":4},{"type":"version","sort":5}]'::jsonb,
       4, 1, '公司简称-业务类型-年份-流水号-版本号，例：XYH-BZ-2022-0001-V1', 0, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_company_code_rule WHERE module_code = 'standard' AND deleted = 0);

INSERT INTO mxx_company_code_rule (module_code, module_name, rule_name, company_abbr, dept_code, biz_type_code, separator, segments, seq_length, enabled, remark, deleted, create_time, update_time)
SELECT 'quality_manual', '质量手册', '质量手册编号规则', 'XYH', NULL, 'ZL', '-',
       '[{"type":"company","sort":1},{"type":"biz_type","sort":2},{"type":"year","format":"yyyy","source":"business_date","sort":3},{"type":"seq","length":4,"sort":4},{"type":"version","sort":5}]'::jsonb,
       4, 1, '公司简称-业务类型-年份-流水号-版本号，例：XYH-ZL-2022-0001-V1', 0, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_company_code_rule WHERE module_code = 'quality_manual' AND deleted = 0);


-- -------------------------------------------------------------
-- 5. 菜单与按钮权限（挂在"企业"顶级菜单 id=299 下）
--    注意：原计划使用 366-370 id 并挂在 361 下，但实际数据库中
--          企业顶级菜单 id=299（page.company.title, FOLDER, /company），
--          且 366-368 已被其他模块占用，max_id=374，故改用 375-379。
-- -------------------------------------------------------------
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted)
VALUES
-- 5.1 编号规则配置（MENU，挂在 299 企业下，sort=3 排在企业信息之后）
(375, 299, '', 'page.company.codeRule.title', 'MENU', 'CompanyCodeRule', '/company/code-rule', 'company/code-rule/index', 'company:code:list', 1, 0, 0, 0, 0, 0, 1, 3, 'lucide:hash', '', NULL, NOW(), NOW(), 0),

-- 5.2 按钮权限
(376, 375, '', 'page.company.codeRule.button.add',    'BUTTON', 'CompanyCodeRuleAdd',    '', 'company/code-rule/index', 'company:code:add',     1, 0, 0, 0, 0, 0, 0, 1, '', '', NULL, NOW(), NOW(), 0),
(377, 375, '', 'page.company.codeRule.button.edit',   'BUTTON', 'CompanyCodeRuleEdit',   '', 'company/code-rule/index', 'company:code:update',  1, 0, 0, 0, 0, 0, 0, 2, '', '', NULL, NOW(), NOW(), 0),
(378, 375, '', 'page.company.codeRule.button.delete', 'BUTTON', 'CompanyCodeRuleDelete', '', 'company/code-rule/index', 'company:code:delete',  1, 0, 0, 0, 0, 0, 0, 3, '', '', NULL, NOW(), NOW(), 0),
(379, 375, '', 'page.company.codeRule.button.regenerate', 'BUTTON', 'CompanyCodeRuleRegen', '', 'company/code-rule/index', 'company:code:regenerate', 1, 0, 0, 0, 0, 0, 0, 4, '', '', NULL, NOW(), NOW(), 0);

-- 5.3 给超级管理员角色(role_id=1)分配新菜单权限
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT 1, m.id FROM (VALUES (375), (376), (377), (378), (379)) AS m(id)
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_role_menu_merge rm WHERE rm.role_id = 1 AND rm.menu_id = m.id);


COMMIT;

-- =============================================================
-- 校验 SQL（执行后可运行以下查询确认）：
-- SELECT id, parent_id, name, type, route_name, path, component, perm FROM mxx_system_menu WHERE id BETWEEN 366 AND 370 ORDER BY id;
-- SELECT module_code, module_name, rule_name, biz_type_code, separator, seq_length, enabled FROM mxx_company_code_rule WHERE deleted = 0 ORDER BY id;
-- SELECT id, company_name, company_abbr, show_abbr FROM mxx_company_info WHERE id = 1;
-- =============================================================
