-- V3 横向报价单模板（稳定版 v5）
-- 修复：context 块内不用 # 前缀，斑马纹用 typst 原生方式

DELETE FROM mxx_system_pdf_template WHERE template_code = 'quotation_landscape_v3' AND deleted = 0;

INSERT INTO mxx_system_pdf_template
    (name, template_code, doc_type, content, header_content, footer_content,
     paper_size, orientation, margin_top, margin_bottom, margin_left, margin_right,
     font_family, is_default, status, sort, remark, create_by, create_time, update_by, update_time, deleted)
VALUES
    ('报价单V3横向（精装版）', 'quotation_landscape_v3', 'quotation',
     $tpl$#let amber = rgb("#b8860b")
#let steel = rgb("#6e6e6e")
#let zebra-color = rgb("#f7f7f7")
#let ink = rgb("#1a1a1a")
#let hair-soft = rgb("#d6d6d6")

#set page(
  paper: "a4",
  flipped: true,
  margin: (top: 60pt, bottom: 40pt, left: 30pt, right: 30pt),
  header: context {
    if counter(page).get().first() > 1 [
      #set text(size: 7pt, fill: steel)
      #grid(
        columns: (1fr, 1fr, 1fr),
        align: horizon,
        [NO. #text(fill: amber, weight: "bold")[{{ quotation.quotation_no }}] · 客户: {{ quotation.customer_name }}],
        align(center)[#text(size: 12pt, weight: "bold")[产品报价单（续）]],
        align(right)[ISSUED {{ quotation.quotation_date }}],
      )
      #v(2pt)
      #line(length: 100%, stroke: 0.5pt + ink)
    ]
  },
  footer: context {
    let cur = counter(page).get().first()
    let tot = counter(page).final().first()
    set text(size: 7pt, fill: steel)
    grid(
      columns: (1fr, 1fr, 1fr),
      align: horizon,
      [{{ company.company_name }}],
      align(center)[{{ quotation.quotation_no }}],
      align(right)[PAGE #cur / #tot],
    )
  },
)

#set text(font: "Source Han Sans SC", size: 9pt, fill: black)
#set par(leading: 0.75em, justify: false)

// 抬头
#grid(
  columns: (1.1fr, 1fr),
  column-gutter: 20pt,
  align: horizon,
  [
    #text(tracking: 1pt, fill: amber, weight: "bold", size: 7pt)[SUPPLIER]
    #v(2pt)
    #text(size: 18pt, weight: "bold")[{{ company.company_name }}]
    #v(3pt)
    #text(size: 7pt, fill: steel)[
      {{ company.register_address }} \
      T: {{ company.contact_phone }}  E: {{ company.contact_email }} \
      统一社会信用代码: {{ company.credit_code }}
    ]
  ],
  [
    #align(right)[
      #text(tracking: 1.5pt, fill: steel, weight: "bold", size: 7pt)[QUOTATION]
      #v(2pt)
      #text(size: 24pt, weight: "bold")[产品报价单]
      #v(4pt)
      #text(size: 8pt, fill: steel)[NO. ]
      #text(size: 8pt, fill: amber, weight: "bold")[{{ quotation.quotation_no }}]
      #text(size: 8pt, fill: steel)[  ISSUED {{ quotation.quotation_date }}]
    ]
  ],
)

#v(6pt)
#line(length: 100%, stroke: 1pt + ink)
#v(6pt)

// 信息三栏
#grid(
  columns: (1fr, 1fr, 1fr),
  column-gutter: 14pt,
  [
    #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[客户信息 CLIENT]
    #v(3pt)
    #text(size: 8pt)[*客户*: {{ quotation.customer_name }}] \
    #text(size: 8pt)[*联系人*: {{ quotation.contact_name }}]
    {% if customer and customer.personal_mobile %} \
    #text(size: 8pt)[*手机*: {{ customer.personal_mobile }}]
    {% endif %}
  ],
  [
    #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[项目信息 PROJECT]
    #v(3pt)
    #text(size: 8pt)[*标题*: {{ quotation.title }}] \
    #text(size: 8pt)[*有效期*: {{ quotation.valid_until }}]
    {% if quotation.delivery_date %} \
    #text(size: 8pt)[*交货*: {{ quotation.delivery_date }}]
    {% endif %}
  ],
  [
    #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[商务条款 TERMS]
    #v(3pt)
    #text(size: 8pt)[*付款*: {{ quotation.payment_terms }}] \
    #text(size: 8pt)[*交货*: {{ quotation.delivery_terms }}] \
    #text(size: 8pt)[*币种*: {{ quotation.currency }}]
  ],
)

#v(6pt)
#line(length: 100%, stroke: 0.4pt + hair-soft)
#v(4pt)

// 产品表格（斑马纹 - 使用 table 的 row 功能）
#table(
  columns: (5%, 12%, 22%, 31%, 8%, 11%, 11%),
  align: (center + horizon, left + horizon, left + horizon, left + horizon, right + horizon, right + horizon, right + horizon),
  stroke: 0.3pt + hair-soft,
  table.header(
    table.cell(fill: none)[*序号*],
    table.cell(fill: none)[*产品编号*],
    table.cell(fill: none)[*产品名称*],
    table.cell(fill: none)[*规格参数*],
    table.cell(fill: none, align: right)[*数量*],
    table.cell(fill: none, align: right)[*单价*],
    table.cell(fill: none, align: right)[*金额*],
  ),
  {% for item in items %}
  table.cell(fill: if calc.odd({{ item.index }}) { white } else { zebra-color })[{{ item.index }}],
  table.cell(fill: if calc.odd({{ item.index }}) { white } else { zebra-color })[{{ item.product_code }}],
  table.cell(fill: if calc.odd({{ item.index }}) { white } else { zebra-color })[{{ item.product_name }}],
  table.cell(fill: if calc.odd({{ item.index }}) { white } else { zebra-color })[{{ item.spec }}],
  table.cell(fill: if calc.odd({{ item.index }}) { white } else { zebra-color }, align: right)[{{ item.quantity }} {{ item.unit }}],
  table.cell(fill: if calc.odd({{ item.index }}) { white } else { zebra-color }, align: right)[{{ item.unit_price }}],
  table.cell(fill: if calc.odd({{ item.index }}) { white } else { zebra-color }, align: right)[*{{ item.subtotal }}*],
  {% endfor %}
)

#v(10pt)

// 金额汇总 + 备注
#grid(
  columns: (1.2fr, 1fr),
  column-gutter: 20pt,
  [
    #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[备注条款 TERMS]
    #v(3pt)
    #line(length: 100%, stroke: 0.3pt + hair-soft)
    #v(3pt)
    {% if quotation.remark %}
    #text(size: 7.5pt)[{{ quotation.remark }}]
    #v(2pt)
    {% endif %}
    #text(size: 7pt, fill: steel)[
      - 报价有效期内有效，逾期需重新核定 \
      - 数量正负5%属正常交付范围，按实际结算 \
      - 含增值税，不含安装调试费 \
      - 本报价单金额汇总以末页为准
    ]
  ],
  [
    #block(inset: 8pt, stroke: 0.5pt + ink, width: 100%)[
      #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[金额汇总 TOTAL]
      #v(4pt)
      #grid(
        columns: (1fr, auto),
        gutter: 4pt,
        [*小计*], [{{ quotation.total_amount }}],
        [*税额*], [{{ quotation.tax_amount }}],
        {% if quotation.discount_amount %}
        [*折扣*], [- {{ quotation.discount_amount }}],
        {% endif %}
      )
      #v(3pt)
      #line(length: 100%, stroke: 0.8pt + ink)
      #v(3pt)
      #grid(
        columns: (1fr, auto),
        [#text(size: 11pt, weight: "bold")[*合计 Grand Total*]],
        [#text(size: 14pt, weight: "bold", fill: amber)[*{{ quotation.grand_total }}*]],
      )
      #v(2pt)
      #text(size: 7pt, fill: steel)[大写: {{ grand_total_cn }}]
    ]
  ],
)

#v(16pt)

// 签章区
#grid(
  columns: (1fr, 1fr),
  column-gutter: 30pt,
  [
    #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[供方签章 SUPPLIER]
    #v(24pt)
    #line(length: 100%, stroke: 0.4pt + ink)
    #v(3pt)
    #text(size: 7pt, fill: steel)[DATE: {{ quotation.quotation_date }}]
  ],
  [
    #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[客户签章 CLIENT]
    #v(24pt)
    #line(length: 100%, stroke: 0.4pt + ink)
    #v(3pt)
    #text(size: 7pt, fill: steel)[DATE: ____________]
  ],
)
$tpl$,
     NULL, NULL,
     'a4', 'landscape', 60, 40, 30, 30,
     'Source Han Sans SC', 1, 1, 50, 'V3横向精装版-稳定复刻v5',
     NULL, NOW(), NULL, NOW(), 0
);

UPDATE mxx_system_pdf_template SET is_default = 0
  WHERE doc_type = 'quotation' AND template_code != 'quotation_landscape_v3' AND deleted = 0;

SELECT id, name, template_code, is_default FROM mxx_system_pdf_template
  WHERE doc_type = 'quotation' AND deleted = 0 ORDER BY is_default DESC;
