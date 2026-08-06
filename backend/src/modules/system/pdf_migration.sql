-- ================================================================
-- PDF 模块数据库迁移脚本
-- 创建时间: 2026-08-05
-- 说明:
--   1. 创建 PDF 模板表 mxx_system_pdf_template
--   2. 创建 PDF 生成记录表 mxx_system_pdf_record
--   3. 为报价单、销售订单表添加 PDF 相关字段
--   4. 插入 PDF 模块菜单与权限码（mxx_system_menu）
--   5. 插入 3 个默认 PDF 模板数据（quotation_standard / order_standard / contract_formal）
-- 数据库: PostgreSQL
-- 编码: UTF-8
-- ================================================================


-- ================================================================
-- 一、创建 PDF 模板表 mxx_system_pdf_template
-- ================================================================
CREATE TABLE IF NOT EXISTS mxx_system_pdf_template (
    id              BIGSERIAL PRIMARY KEY,
    name            VARCHAR(64)  NOT NULL,
    template_code   VARCHAR(64)  NOT NULL,
    doc_type        VARCHAR(32)  NOT NULL,
    content         TEXT         NOT NULL,
    header_content  TEXT,
    footer_content  TEXT,
    paper_size      VARCHAR(16)  NOT NULL DEFAULT 'a4',
    orientation     VARCHAR(16)  NOT NULL DEFAULT 'portrait',
    margin_top      INT          NOT NULL DEFAULT 20,
    margin_bottom   INT          NOT NULL DEFAULT 20,
    margin_left     INT          NOT NULL DEFAULT 40,
    margin_right    INT          NOT NULL DEFAULT 40,
    font_family     VARCHAR(64)  NOT NULL DEFAULT 'Source Han Sans SC',
    is_default      INT          NOT NULL DEFAULT 0,
    status          INT          NOT NULL DEFAULT 1,
    sort            INT          NOT NULL DEFAULT 0,
    remark          VARCHAR(255),
    create_by       BIGINT,
    create_time     TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_by       BIGINT,
    update_time     TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted         INT          NOT NULL DEFAULT 0
);

-- 表注释
COMMENT ON TABLE mxx_system_pdf_template IS 'PDF模板表';

-- 字段注释
COMMENT ON COLUMN mxx_system_pdf_template.id IS '主键ID';
COMMENT ON COLUMN mxx_system_pdf_template.name IS '模板名称';
COMMENT ON COLUMN mxx_system_pdf_template.template_code IS '模板编码（唯一标识，如 quotation_standard）';
COMMENT ON COLUMN mxx_system_pdf_template.doc_type IS '文档类型（quotation/order/contract 等）';
COMMENT ON COLUMN mxx_system_pdf_template.content IS '模板内容（Typst 模板源码）';
COMMENT ON COLUMN mxx_system_pdf_template.header_content IS '页眉内容';
COMMENT ON COLUMN mxx_system_pdf_template.footer_content IS '页脚内容';
COMMENT ON COLUMN mxx_system_pdf_template.paper_size IS '纸张大小（a4/a3/letter 等）';
COMMENT ON COLUMN mxx_system_pdf_template.orientation IS '纸张方向（portrait/landscape）';
COMMENT ON COLUMN mxx_system_pdf_template.margin_top IS '上边距（pt）';
COMMENT ON COLUMN mxx_system_pdf_template.margin_bottom IS '下边距（pt）';
COMMENT ON COLUMN mxx_system_pdf_template.margin_left IS '左边距（pt）';
COMMENT ON COLUMN mxx_system_pdf_template.margin_right IS '右边距（pt）';
COMMENT ON COLUMN mxx_system_pdf_template.font_family IS '字体（默认 Source Han Sans SC）';
COMMENT ON COLUMN mxx_system_pdf_template.is_default IS '是否默认模板（0否 1是）';
COMMENT ON COLUMN mxx_system_pdf_template.status IS '状态（1正常 0停用）';
COMMENT ON COLUMN mxx_system_pdf_template.sort IS '排序值';
COMMENT ON COLUMN mxx_system_pdf_template.remark IS '备注';
COMMENT ON COLUMN mxx_system_pdf_template.create_by IS '创建人ID';
COMMENT ON COLUMN mxx_system_pdf_template.create_time IS '创建时间';
COMMENT ON COLUMN mxx_system_pdf_template.update_by IS '更新人ID';
COMMENT ON COLUMN mxx_system_pdf_template.update_time IS '更新时间';
COMMENT ON COLUMN mxx_system_pdf_template.deleted IS '删除标志（0未删除 1已删除）';

-- 索引
CREATE INDEX IF NOT EXISTS idx_pdf_template_code ON mxx_system_pdf_template(template_code);
CREATE INDEX IF NOT EXISTS idx_pdf_template_doc_type ON mxx_system_pdf_template(doc_type) WHERE deleted = 0;


-- ================================================================
-- 二、创建 PDF 生成记录表 mxx_system_pdf_record
-- ================================================================
CREATE TABLE IF NOT EXISTS mxx_system_pdf_record (
    id              BIGSERIAL PRIMARY KEY,
    doc_type        VARCHAR(32)  NOT NULL,
    doc_id          BIGINT       NOT NULL,
    doc_no          VARCHAR(64),
    template_id     BIGINT       NOT NULL,
    template_name   VARCHAR(64),
    file_name       VARCHAR(255) NOT NULL,
    file_path       VARCHAR(512) NOT NULL,
    file_url        VARCHAR(512),
    file_size       BIGINT,
    page_count      INT,
    trigger_type    VARCHAR(32)  NOT NULL DEFAULT 'manual',
    status          INT          NOT NULL DEFAULT 1,
    error_msg       TEXT,
    create_by       BIGINT,
    create_time     TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted         INT          NOT NULL DEFAULT 0
);

-- 表注释
COMMENT ON TABLE mxx_system_pdf_record IS 'PDF生成记录表';

-- 字段注释
COMMENT ON COLUMN mxx_system_pdf_record.id IS '主键ID';
COMMENT ON COLUMN mxx_system_pdf_record.doc_type IS '文档类型（quotation/order/contract 等）';
COMMENT ON COLUMN mxx_system_pdf_record.doc_id IS '文档ID（业务主键）';
COMMENT ON COLUMN mxx_system_pdf_record.doc_no IS '文档编号（如报价单号、订单号）';
COMMENT ON COLUMN mxx_system_pdf_record.template_id IS '使用的模板ID';
COMMENT ON COLUMN mxx_system_pdf_record.template_name IS '使用的模板名称';
COMMENT ON COLUMN mxx_system_pdf_record.file_name IS '生成文件名';
COMMENT ON COLUMN mxx_system_pdf_record.file_path IS '文件存储路径';
COMMENT ON COLUMN mxx_system_pdf_record.file_url IS '文件访问URL';
COMMENT ON COLUMN mxx_system_pdf_record.file_size IS '文件大小（字节）';
COMMENT ON COLUMN mxx_system_pdf_record.page_count IS '页数';
COMMENT ON COLUMN mxx_system_pdf_record.trigger_type IS '触发方式（manual 手动 / auto 自动）';
COMMENT ON COLUMN mxx_system_pdf_record.status IS '状态（1成功 0失败）';
COMMENT ON COLUMN mxx_system_pdf_record.error_msg IS '错误信息（失败时记录）';
COMMENT ON COLUMN mxx_system_pdf_record.create_by IS '创建人ID';
COMMENT ON COLUMN mxx_system_pdf_record.create_time IS '创建时间';
COMMENT ON COLUMN mxx_system_pdf_record.deleted IS '删除标志（0未删除 1已删除）';

-- 索引
CREATE INDEX IF NOT EXISTS idx_pdf_record_doc ON mxx_system_pdf_record(doc_type, doc_id);
CREATE INDEX IF NOT EXISTS idx_pdf_record_template ON mxx_system_pdf_record(template_id);
CREATE INDEX IF NOT EXISTS idx_pdf_record_create_time ON mxx_system_pdf_record(create_time);


-- ================================================================
-- 三、为现有业务表添加 PDF 相关字段
-- ================================================================

-- 报价单表添加 PDF 字段
ALTER TABLE mxx_sale_quotation ADD COLUMN IF NOT EXISTS pdf_file_url VARCHAR(512);
ALTER TABLE mxx_sale_quotation ADD COLUMN IF NOT EXISTS pdf_template_id BIGINT;
COMMENT ON COLUMN mxx_sale_quotation.pdf_file_url IS 'PDF文件访问URL';
COMMENT ON COLUMN mxx_sale_quotation.pdf_template_id IS 'PDF模板ID';

-- 销售订单表添加 PDF 字段
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS pdf_file_url VARCHAR(512);
ALTER TABLE mxx_sale_order ADD COLUMN IF NOT EXISTS pdf_template_id BIGINT;
COMMENT ON COLUMN mxx_sale_order.pdf_file_url IS 'PDF文件访问URL';
COMMENT ON COLUMN mxx_sale_order.pdf_template_id IS 'PDF模板ID';


-- ================================================================
-- 四、插入权限码（mxx_system_menu 表）
-- ================================================================
-- 说明：
--   权限码格式遵循 {module}:{resource}:{action}
--   parent_id 说明：
--     - system 相关菜单的 parent_id = 67（系统管理文件夹 /system）
--     - sale:quotation:pdf 按钮的 parent_id = 301（报价单菜单 /sale/quotation）
--     - sale:order:pdf 按钮的 parent_id = 167（销售订单菜单 /sale/order）
--     - crm:contract:pdf 按钮的 parent_id = 164（合同菜单 /sale/contract）
--   使用 ON CONFLICT (id) DO NOTHING 避免重复执行报错
--   ID 从 900 开始，避免与现有菜单冲突

-- PDF 模板管理 - MENU
INSERT INTO mxx_system_menu (id, parent_id, name, type, route_name, path, component, perm, status, sort)
VALUES (900, 67, 'page.system.pdfTemplate.title', 'MENU', 'SystemPdfTemplate', '/system/pdf-template', 'system/pdf-template/index', 'system:pdf-template:list', 1, 20)
ON CONFLICT (id) DO NOTHING;

-- PDF 模板按钮权限
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, component, status, sort) VALUES
(901, 900, '查看PDF模板', 'BUTTON', 'system:pdf-template:view', 'system/pdf-template/index', 1, 1),
(902, 900, '新增PDF模板', 'BUTTON', 'system:pdf-template:create', 'system/pdf-template/index', 1, 2),
(903, 900, '编辑PDF模板', 'BUTTON', 'system:pdf-template:edit', 'system/pdf-template/index', 1, 3),
(904, 900, '删除PDF模板', 'BUTTON', 'system:pdf-template:delete', 'system/pdf-template/index', 1, 4)
ON CONFLICT (id) DO NOTHING;

-- PDF 生成记录 - MENU
INSERT INTO mxx_system_menu (id, parent_id, name, type, route_name, path, component, perm, status, sort)
VALUES (905, 67, 'page.system.pdfRecord.title', 'MENU', 'SystemPdfRecord', '/system/pdf-record', 'system/pdf-record/index', 'system:pdf-record:list', 1, 21)
ON CONFLICT (id) DO NOTHING;

-- 报价单生成PDF按钮 - BUTTON（挂在报价单菜单下，parent_id=301）
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, component, status, sort)
VALUES (906, 301, '生成报价单PDF', 'BUTTON', 'sale:quotation:pdf', 'sale/quotation/index', 1, 10)
ON CONFLICT (id) DO NOTHING;

-- 销售订单生成PDF按钮 - BUTTON（挂在销售订单菜单下，parent_id=167）
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, component, status, sort)
VALUES (907, 167, '生成订单PDF', 'BUTTON', 'sale:order:pdf', 'sale/order/index', 1, 10)
ON CONFLICT (id) DO NOTHING;

-- 合同生成PDF按钮 - BUTTON（挂在合同菜单下，parent_id=164）
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, component, status, sort)
VALUES (908, 164, '生成合同PDF', 'BUTTON', 'crm:contract:pdf', 'sale/contract/index', 1, 10)
ON CONFLICT (id) DO NOTHING;


-- ================================================================
-- 五、插入默认 PDF 模板数据
-- ================================================================
-- 说明：content 字段为简化版 Typst 模板源码，使用 $tpl$ ... $tpl$ 美元引用避免转义问题
--       模板中使用 #变量名 作为业务数据占位符，由后端渲染时替换

-- 1. 报价单标准模板 quotation_standard
INSERT INTO mxx_system_pdf_template (name, template_code, doc_type, content, paper_size, orientation, is_default, status, sort, remark)
SELECT '报价单标准模板', 'quotation_standard', 'quotation', $tpl$#set page(
  paper: "a4",
  margin: (top: 20pt, bottom: 20pt, left: 40pt, right: 40pt),
)
#set text(font: "Source Han Sans SC", size: 10pt)

#align(center)[
  #text(size: 18pt, weight: "bold")[报价单]
]

#v(1em)

#grid(
  columns: (1fr, 1fr),
  [*客户名称*: #customer_name],
  [*报价单号*: #quotation_no],
  [*联系人*: #contact_name],
  [*报价日期*: #quotation_date],
  [*有效期至*: #valid_until],
  [*币种*: #currency],
)

#v(1em)

#table(
  columns: (auto, 1fr, auto, auto, auto),
  align: center + horizon,
  [*序号*], [*产品名称*], [*数量*], [*单价*], [*金额*],
  ..#items,
)

#v(1em)

#align(right)[
  合计金额: *#grand_total*
]

#v(2em)

#text(size: 9pt, fill: gray)[
  备注: #remark
]

#v(1em)

#align(center)[
  #text(size: 9pt, fill: gray)[本报价单仅供参考，最终价格以合同为准。]
]
$tpl$, 'a4', 'portrait', 1, 1, 1, '报价单默认模板，适用于销售报价单 PDF 生成'
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_pdf_template WHERE template_code = 'quotation_standard' AND deleted = 0);

-- 2. 销售订单标准模板 order_standard
INSERT INTO mxx_system_pdf_template (name, template_code, doc_type, content, paper_size, orientation, is_default, status, sort, remark)
SELECT '销售订单标准模板', 'order_standard', 'order', $tpl$#set page(
  paper: "a4",
  margin: (top: 20pt, bottom: 20pt, left: 40pt, right: 40pt),
)
#set text(font: "Source Han Sans SC", size: 10pt)

#align(center)[
  #text(size: 18pt, weight: "bold")[销售订单]
]

#v(1em)

#grid(
  columns: (1fr, 1fr),
  [*客户名称*: #customer_name],
  [*订单编号*: #order_no],
  [*联系人*: #contact_name],
  [*下单日期*: #order_date],
  [*交货日期*: #delivery_date],
  [*币种*: #currency],
)

#v(1em)

#table(
  columns: (auto, 1fr, auto, auto, auto),
  align: center + horizon,
  [*序号*], [*产品名称*], [*数量*], [*单价*], [*金额*],
  ..#items,
)

#v(1em)

#grid(
  columns: (1fr, 1fr),
  [*合计金额*: #grand_total],
  [*付款方式*: #payment_method],
)

#v(2em)

#text(size: 9pt, fill: gray)[
  收货地址: #shipping_address
]

#v(1em)

#align(center)[
  #text(size: 9pt, fill: gray)[本订单一经双方确认即具法律效力。]
]
$tpl$, 'a4', 'portrait', 1, 1, 2, '销售订单默认模板，适用于销售订单 PDF 生成'
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_pdf_template WHERE template_code = 'order_standard' AND deleted = 0);

-- 3. 合同正式模板 contract_formal
INSERT INTO mxx_system_pdf_template (name, template_code, doc_type, content, paper_size, orientation, is_default, status, sort, remark)
SELECT '合同正式模板', 'contract_formal', 'contract', $tpl$#set page(
  paper: "a4",
  margin: (top: 20pt, bottom: 20pt, left: 40pt, right: 40pt),
)
#set text(font: "Source Han Sans SC", size: 10pt)

#align(center)[
  #text(size: 18pt, weight: "bold")[销售合同]
]

#v(1em)

#grid(
  columns: (1fr, 1fr),
  [*合同编号*: #contract_no],
  [*签订日期*: #sign_date],
  [*甲方*: #party_a],
  [*乙方*: #party_b],
)

#v(1em)

= 合同标的

#items

#v(1em)

= 合同金额

合计金额（含税）: *#grand_total*
币种: #currency

#v(1em)

= 付款条款

#payment_terms

#v(1em)

= 交货条款

#delivery_terms

#v(1em)

= 双方权利义务

#rights_obligations

#v(2em)

#grid(
  columns: (1fr, 1fr),
  align: center,
  [*甲方（盖章）*],
  [*乙方（盖章）*],
  #v(3em),
  #v(3em),
  日期: #sign_date,
  日期: #sign_date,
)
$tpl$, 'a4', 'portrait', 1, 1, 3, '合同正式模板，适用于销售合同 PDF 生成'
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_pdf_template WHERE template_code = 'contract_formal' AND deleted = 0);


-- ================================================================
-- 迁移结束
-- ================================================================
