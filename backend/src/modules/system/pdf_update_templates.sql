-- ================================================================
-- 更新 PDF 默认模板：将 typst 变量语法改为 minijinja {{ }} 语法
-- 说明：
--   1. 模板正文使用 {{ variable }} 语法（minijinja 渲染）
--   2. 移除 #set page / #set text 指令（由 assemble_typst_source 统一添加）
--   3. 产品明细使用 {% for item in items %} 循环
-- ================================================================

-- 1. 更新报价单模板
UPDATE mxx_system_pdf_template SET content = $tpl$= 报 价 单

#grid(
  columns: (1fr, 1fr),
  column-gutter: 20pt,
  row-gutter: 8pt,
  [*客户名称*: {{ quotation.customer_name }}],
  [*报价单号*: {{ quotation.quotation_no }}],
  [*联系人*: {{ quotation.contact_name }}],
  [*报价日期*: {{ quotation.quotation_date }}],
  [*有效期至*: {{ quotation.valid_until }}],
  [*币种*: {{ quotation.currency }}],
)

#v(1em)

#table(
  columns: (auto, 1fr, auto, auto, auto, auto, auto, auto),
  align: center + horizon,
  table.header(
    [*序号*], [*产品编码*], [*产品名称*], [*规格*], [*单位*], [*数量*], [*单价*], [*小计*],
  ),
  {% for item in items %}
  [{{ item.index }}], [{{ item.product_code }}], [{{ item.product_name }}], [{{ item.spec }}], [{{ item.unit }}], [{{ item.quantity }}], [{{ item.unit_price }}], [{{ item.subtotal }}],
  {% endfor %}
)

#v(1em)

#grid(
  columns: (1fr, 1fr),
  [*合计金额*: {{ quotation.grand_total }} {{ quotation.currency }}],
  [*大写金额*: {{ grand_total_cn }}],
)

{% if quotation.payment_terms %}
#v(1em)
*付款条款:* {{ quotation.payment_terms }}
{% endif %}

{% if quotation.delivery_terms %}
#v(0.5em)
*交货条款:* {{ quotation.delivery_terms }}
{% endif %}

{% if quotation.remark %}
#v(1em)
*备注:* {{ quotation.remark }}
{% endif %}

#v(2em)

#align(center)[
  #text(size: 9pt, fill: gray)[本报价单仅供参考，最终价格以合同为准。]
]
$tpl$
WHERE template_code = 'quotation_standard' AND deleted = 0;

-- 2. 更新订单模板
UPDATE mxx_system_pdf_template SET content = $tpl$= 销 售 订 单

#grid(
  columns: (1fr, 1fr),
  column-gutter: 20pt,
  row-gutter: 8pt,
  [*客户名称*: {{ order.customer_name }}],
  [*订单编号*: {{ order.order_no }}],
  [*联系人*: {{ order.contact_name }}],
  [*下单日期*: {{ order.order_date }}],
  [*交货日期*: {{ order.delivery_date }}],
  [*币种*: {{ order.currency }}],
)

#v(1em)

#table(
  columns: (auto, 1fr, auto, auto, auto, auto, auto),
  align: center + horizon,
  table.header(
    [*序号*], [*产品名称*], [*规格*], [*单位*], [*数量*], [*单价*], [*金额*],
  ),
  {% for item in items %}
  [{{ item.index }}], [{{ item.product_name }}], [{{ item.spec }}], [{{ item.unit }}], [{{ item.quantity }}], [{{ item.unit_price }}], [{{ item.amount }}],
  {% endfor %}
)

#v(1em)

#grid(
  columns: (1fr, 1fr),
  [*合计金额*: {{ order.total_amount }} {{ order.currency }}],
  [*大写金额*: {{ grand_total_cn }}],
)

{% if order.buyer_company_name %}
#v(1em)
*买方:* {{ order.buyer_company_name }}
{% endif %}

{% if order.seller_company_name %}
#v(0.5em)
*卖方:* {{ order.seller_company_name }}
{% endif %}

{% if order.remark %}
#v(1em)
*备注:* {{ order.remark }}
{% endif %}

#v(2em)

#align(center)[
  #text(size: 9pt, fill: gray)[本订单一经双方确认即具法律效力。]
]
$tpl$
WHERE template_code = 'order_standard' AND deleted = 0;

-- 3. 更新合同模板
UPDATE mxx_system_pdf_template SET content = $tpl$= 销 售 合 同

#grid(
  columns: (1fr, 1fr),
  column-gutter: 20pt,
  row-gutter: 8pt,
  [*合同编号*: {{ contract.contract_no }}],
  [*签订日期*: {{ contract.sign_date }}],
  [*甲方*: {{ company.company_name }}],
  [*乙方*: {{ customer.company_name }}],
)

#v(1em)

= 合同标的

{{ contract.title }}

#v(1em)

= 合同金额

合计金额（含税）: *{{ contract.total_amount }}*
大写金额: {{ grand_total_cn }}

{% if payment_plans %}
#v(1em)

= 付款计划

#table(
  columns: (auto, 1fr, auto, auto),
  align: center + horizon,
  table.header(
    [*序号*], [*阶段*], [*计划金额*], [*计划日期*],
  ),
  {% for plan in payment_plans %}
  [{{ plan.index }}], [{{ plan.stage_name }}], [{{ plan.plan_amount }}], [{{ plan.plan_date }}],
  {% endfor %}
)
{% endif %}

{% if contract.payment_terms %}
#v(1em)

= 付款条款

{{ contract.payment_terms }}
{% endif %}

{% if contract.delivery_terms %}
#v(1em)

= 交货条款

{{ contract.delivery_terms }}
{% endif %}

{% if contract_description_typst %}
#v(1em)

= 合同正文

{{ contract_description_typst }}
{% endif %}

#v(2em)

#grid(
  columns: (1fr, 1fr),
  align: center,
  [*甲方（盖章）*],
  [*乙方（盖章）*],
  #v(3em),
  #v(3em),
  签字: {{ contract.our_signer_name }},
  签字: {{ contract.their_signer_name }},
  日期: {{ contract.sign_date }},
  日期: {{ contract.sign_date }},
)
$tpl$
WHERE template_code = 'contract_formal' AND deleted = 0;
