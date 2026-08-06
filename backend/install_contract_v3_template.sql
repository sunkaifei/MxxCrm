-- 合同 A4 竖版模板（1:1 复刻 contract-a4-portrait.html）
-- 完整实现：封面标题 / 甲乙方信息 / 鉴于条款 / 分条编号条款 / 产品表格 / 付款计划 / 签署页

DELETE FROM mxx_system_pdf_template WHERE template_code = 'contract_portrait_v3' AND deleted = 0;

INSERT INTO mxx_system_pdf_template
    (name, template_code, doc_type, content, header_content, footer_content,
     paper_size, orientation, margin_top, margin_bottom, margin_left, margin_right,
     font_family, is_default, status, sort, remark, create_by, create_time, update_by, update_time, deleted)
VALUES
    ('合同V3竖版（精装版）', 'contract_portrait_v3', 'contract',
     $tpl$#let amber = rgb("#96680a")
#let steel = rgb("#666666")
#let zebra-color = rgb("#f7f7f7")
#let ink = rgb("#1a1a1a")
#let hair-soft = rgb("#cccccc")
#let mist = rgb("#999999")

#set page(
  paper: "a4",
  margin: (top: 40pt, bottom: 40pt, left: 50pt, right: 50pt),
  header: context {
    if counter(page).get().first() > 1 [
      #set text(size: 7pt, fill: steel)
      #grid(
        columns: (1fr, auto),
        align: horizon,
        [#text(size: 11pt, weight: "bold", fill: ink)[产品购销合同]],
        [#text(size: 7.5pt, fill: steel)[NO. #text(fill: amber, weight: "bold")[{{ contract.contract_no }}]]],
      )
      #v(2pt)
      #line(length: 100%, stroke: 0.5pt + hair-soft)
    ]
  },
  footer: context {
    let cur = counter(page).get().first()
    let tot = counter(page).final().first()
    set text(size: 7pt, fill: mist)
    grid(
      columns: (1fr, auto),
      align: horizon,
      [产品购销合同 {{ contract.contract_no }}],
      align(right)[PAGE #text(fill: amber, weight: "bold")[#cur] / #tot],
    )
  },
)

#set text(font: "Source Han Sans SC", size: 9pt, fill: ink)
#set par(leading: 0.85em, justify: true)

// ========== 封面标题 ==========
#align(center)[
  #text(tracking: 1pt, fill: amber, size: 7.5pt)[CONTRACT]
  #v(3pt)
  #text(size: 28pt, font: "Source Han Serif SC", weight: "bold", tracking: 2pt)[产品购销合同]
  #v(2pt)
  #text(size: 8pt, fill: steel, tracking: 1.5pt)[Product Purchase & Sales Agreement]
  #v(4pt)
  #text(size: 8pt, fill: ink)[合同编号 NO. ]
  #text(size: 8pt, fill: amber, weight: "bold")[{{ contract.contract_no }}]
  #text(size: 8pt, fill: steel)[  签订日期: {{ contract.sign_date }}]
]

#v(4pt)
#line(length: 100%, stroke: 1pt + ink)
#v(6pt)

// ========== 甲乙方信息 ==========
#grid(
  columns: (1fr, 1fr),
  column-gutter: 0pt,
  [
    #text(size: 11pt, font: "Source Han Serif SC", weight: "bold", fill: amber)[甲方（买方）]
    #v(3pt)
    #text(size: 10pt, weight: "bold")[{% if customer and customer.company_name %}{{ customer.company_name }}{% endif %}]
    #v(2pt)
    #text(size: 8pt, fill: steel)[联系人 / 电话] \
    #text(size: 8pt)[{{ contract.their_signer_name }}  {{ contract.their_signer_phone }}]
  ],
  [
    #text(size: 11pt, font: "Source Han Serif SC", weight: "bold", fill: amber)[乙方（卖方）]
    #v(3pt)
    #text(size: 10pt, weight: "bold")[{{ company.company_name }}]
    #v(2pt)
    #text(size: 8pt, fill: steel)[联系人 / 电话] \
    #text(size: 8pt)[{{ contract.our_signer_name }}  {% if company %}{{ company.contact_phone }}{% endif %}]
  ],
)

#v(4pt)

// ========== 鉴于条款 ==========
#block(
  inset: 8pt,
  fill: zebra-color,
  stroke: (left: 2pt + amber),
  width: 100%,
)[
  #text(size: 9pt)[
    *鉴于* 甲方拟向乙方采购相关产品及服务，乙方具备相应供货能力。甲乙双方本着平等自愿、诚实信用的原则，根据《中华人民共和国民法典》及相关法律法规，经友好协商，就产品买卖事宜达成如下协议，以兹共同遵守。
  ]
]

#v(6pt)

// ========== 第一条 合同标的 ==========
#text(size: 10pt, font: "Source Han Serif SC", weight: "bold", fill: amber)[第一条]
#text(size: 10pt, weight: "bold")[  合同标的]
#line(length: 100%, stroke: 0.3pt + hair-soft)
#v(2pt)
#text(size: 8.5pt)[乙方向甲方供应相关产品，合同总金额为 *{{ contract.total_amount }}* 元（大写：{{ grand_total_cn }}），含增值税。]

#v(3pt)

// ========== 合同条款（动态描述） ==========
{{ contract_description_typst }}

#v(4pt)

// ========== 付款计划 ==========
{% if payment_plans and payment_plans|length > 0 %}
#text(size: 10pt, font: "Source Han Serif SC", weight: "bold", fill: amber)[付款计划]
#line(length: 100%, stroke: 0.3pt + hair-soft)
#v(2pt)

#table(
  columns: (8%, 1fr, auto, auto),
  align: (center + horizon, left + horizon, right + horizon, left + horizon),
  stroke: 0.3pt + hair-soft,
  table.header(
    [*序号*], [*阶段*], [*金额*], [*计划日期*],
  ),
  {% for plan in payment_plans %}
  [{{ plan.index }}],
  [{{ plan.stage_name }}],
  [{{ plan.plan_amount }}],
  [{{ plan.plan_date }}],
  {% endfor %}
)
#v(4pt)
{% endif %}

// ========== 备注 ==========
{% if contract.remark %}
#text(size: 10pt, font: "Source Han Serif SC", weight: "bold", fill: amber)[其他约定]
#line(length: 100%, stroke: 0.3pt + hair-soft)
#v(2pt)
#text(size: 8.5pt)[{{ contract.remark }}]
#v(4pt)
{% endif %}

#v(10pt)

// ========== 签署区 ==========
#align(center)[
  #text(size: 13pt, font: "Source Han Serif SC", weight: "bold", tracking: 2pt)[合 同 签 署]
  #v(1pt)
  #text(size: 7.5pt, fill: steel, tracking: 1pt)[SIGNATURE & SEAL]
]
#v(8pt)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 20pt,
  [
    #text(size: 12pt, font: "Source Han Serif SC", weight: "bold", fill: amber)[甲方（买方）]
    #v(8pt)
    #text(size: 8pt, fill: steel)[单位名称] \
    #v(1pt)
    #line(length: 100%, stroke: 0.4pt + hair-soft)
    #v(8pt)
    #text(size: 8pt, fill: steel)[授权代表（签字）] \
    #v(1pt)
    #line(length: 100%, stroke: 0.4pt + hair-soft)
    #v(8pt)
    #text(size: 8pt, fill: steel)[签署日期] \
    #v(1pt)
    #line(length: 100%, stroke: 0.4pt + hair-soft)
    #v(8pt)
    #text(size: 8pt, fill: steel)[单位盖章] \
    #v(2pt)
    #block(inset: 30pt, stroke: 0.5pt + mist, width: 100%)[#align(center)[#text(size: 7.5pt, fill: mist)[（盖章处）]]]
  ],
  [
    #text(size: 12pt, font: "Source Han Serif SC", weight: "bold", fill: amber)[乙方（卖方）]
    #v(8pt)
    #text(size: 8pt, fill: steel)[单位名称] \
    #v(1pt)
    #line(length: 100%, stroke: 0.4pt + hair-soft)
    #v(8pt)
    #text(size: 8pt, fill: steel)[授权代表（签字）] \
    #v(1pt)
    #line(length: 100%, stroke: 0.4pt + hair-soft)
    #v(8pt)
    #text(size: 8pt, fill: steel)[签署日期] \
    #v(1pt)
    #line(length: 100%, stroke: 0.4pt + hair-soft)
    #v(8pt)
    #text(size: 8pt, fill: steel)[单位盖章] \
    #v(2pt)
    #block(inset: 30pt, stroke: 0.5pt + mist, width: 100%)[#align(center)[#text(size: 7.5pt, fill: mist)[（盖章处）]]]
  ],
)
$tpl$,
     NULL, NULL,
     'a4', 'portrait', 40, 40, 50, 50,
     'Source Han Sans SC', 1, 1, 50, 'V3竖版-1:1复刻HTML设计',
     NULL, NOW(), NULL, NOW(), 0
);

-- 取消其他合同模板的默认状态
UPDATE mxx_system_pdf_template SET is_default = 0
  WHERE doc_type = 'contract' AND template_code != 'contract_portrait_v3' AND deleted = 0;

SELECT id, name, template_code, doc_type, paper_size, orientation, is_default, status
  FROM mxx_system_pdf_template
  WHERE doc_type = 'contract' AND deleted = 0
  ORDER BY is_default DESC, sort ASC;
