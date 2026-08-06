-- 修复合同模板签署区语法：将裸文本放入 [] 中
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

[*合计金额（含税）*: {{ contract.total_amount }}]
[*大写金额*: {{ grand_total_cn }}]

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
  [#v(3em)],
  [#v(3em)],
  [*签字*: {{ contract.our_signer_name }}],
  [*签字*: {{ contract.their_signer_name }}],
  [*日期*: {{ contract.sign_date }}],
  [*日期*: {{ contract.sign_date }}],
)
$tpl$
WHERE template_code = 'contract_formal' AND deleted = 0;
