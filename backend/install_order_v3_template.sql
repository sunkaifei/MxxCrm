-- 订单 V3 横向模板（1:1 复刻 order-a4-landscape.html）
-- 完整实现：买方/卖方双栏 / 订单元信息条 / 斑马纹 / 页码X/Y / 续页header / 金额汇总

DELETE FROM mxx_system_pdf_template WHERE template_code = 'order_landscape_v3' AND deleted = 0;

INSERT INTO mxx_system_pdf_template
    (name, template_code, doc_type, content, header_content, footer_content,
     paper_size, orientation, margin_top, margin_bottom, margin_left, margin_right,
     font_family, is_default, status, sort, remark, create_by, create_time, update_by, update_time, deleted)
VALUES
    ('订单V3横向（精装版）', 'order_landscape_v3', 'order',
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
        [NO. #text(fill: amber, weight: "bold")[{{ order.order_no }}] · 买方: {{ order.customer_name }}],
        align(center)[#text(size: 12pt, weight: "bold")[产品订单（续）]],
        align(right)[ISSUED {{ order.order_date }}],
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
      align(center)[{{ order.order_no }}],
      align(right)[PAGE #cur / #tot],
    )
  },
)

#set text(font: "Source Han Sans SC", size: 9pt, fill: black)
#set par(leading: 0.75em, justify: false)

// ========== 抬头 ==========
#grid(
  columns: (1.1fr, 1fr),
  column-gutter: 20pt,
  align: horizon,
  [
    #text(tracking: 1pt, fill: amber, weight: "bold", size: 7pt)[SUPPLIER 供方]
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
      #text(tracking: 1.5pt, fill: steel, weight: "bold", size: 7pt)[PURCHASE ORDER]
      #v(2pt)
      #text(size: 24pt, weight: "bold")[产品订单]
      #v(4pt)
      #text(size: 8pt, fill: steel)[NO. ]
      #text(size: 8pt, fill: amber, weight: "bold")[{{ order.order_no }}]
      #text(size: 8pt, fill: steel)[  ISSUED {{ order.order_date }}]
    ]
  ],
)

#v(4pt)
#line(length: 100%, stroke: 1pt + ink)
#v(4pt)

// ========== 买方/卖方 双栏 ==========
#grid(
  columns: (1fr, 1fr),
  column-gutter: 0pt,
  [
    #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[买方 BUYER]
    #v(2pt)
    #text(size: 9pt, weight: "bold")[{{ order.buyer_company_name }}]
    #v(2pt)
    #text(size: 7.5pt, fill: steel)[CONTACT]
    #text(size: 7.5pt)[  {{ order.contact_name }}] \
    #text(size: 7.5pt, fill: steel)[TEL]
    #text(size: 7.5pt)[  {% if customer and customer.personal_mobile %}{{ customer.personal_mobile }}{% endif %}] \
    #text(size: 7.5pt, fill: steel)[ADDR]
    #text(size: 7.5pt)[  {% if customer and customer.address %}{{ customer.address }}{% endif %}]
  ],
  [
    #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[卖方 SELLER]
    #v(2pt)
    #text(size: 9pt, weight: "bold")[{{ order.seller_company_name }}]
    #v(2pt)
    #text(size: 7.5pt, fill: steel)[BANK]
    #text(size: 7.5pt)[  {{ order.seller_bank_name }}] \
    #text(size: 7.5pt, fill: steel)[ACT]
    #text(size: 7.5pt)[  {{ order.seller_account_name }}] \
    #text(size: 7.5pt, fill: steel)[NO]
    #text(size: 7.5pt)[  {{ order.seller_account_number }}]
  ],
)

#v(4pt)

// ========== 订单元信息条 ==========
#table(
  columns: (1fr, 1fr, 1fr, 1fr),
  align: left + horizon,
  stroke: (top: 0.4pt + hair-soft, bottom: 0.4pt + hair-soft),
  [.ORDER DATE 下单日期 #text(size: 9pt, weight: "bold")[{{ order.order_date }}]],
  [DELIVERY 交货期 #text(size: 9pt, weight: "bold", fill: amber)[{{ order.delivery_date }}]],
  [PAYMENT 付款方式 #text(size: 9pt, weight: "bold")[{{ order.currency }}]],
  [CURRENCY 币种 #text(size: 9pt, weight: "bold")[{{ order.currency }}]],
)

#v(4pt)

// ========== 产品表格（斑马纹） ==========
#table(
  columns: (5%, 12%, 22%, 24%, 8%, 11%, 11%),
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
  table.cell(fill: if calc.odd({{ item.index }}) { white } else { zebra-color }, align: right)[*{{ item.amount }}*],
  {% endfor %}
)

#v(10pt)

// ========== 金额汇总 + 条款 ==========
#grid(
  columns: (1.2fr, 1fr),
  column-gutter: 20pt,
  [
    #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[交货与验收条款 TERMS]
    #v(3pt)
    #line(length: 100%, stroke: 0.3pt + hair-soft)
    #v(3pt)
    {% if order.remark %}
    #text(size: 7.5pt)[{{ order.remark }}]
    #v(2pt)
    {% endif %}
    #text(size: 7pt, fill: steel)[
      - 卖方须于约定交货期内送达指定地点，运费由卖方承担 \
      - 到货后买方7个工作日内完成验收，异议需书面提出 \
      - 数量正负5%属正常交付范围，按实际结算 \
      - 含增值税专用发票，质保期12个月 \
      - 本订单经双方签章后生效，金额汇总以末页为准
    ]
  ],
  [
    #block(inset: 8pt, stroke: 0.5pt + ink, width: 100%)[
      #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[金额汇总 TOTAL]
      #v(4pt)
      #grid(
        columns: (1fr, auto),
        gutter: 4pt,
        [*产品小计*], [{{ order.product_amount }}],
        [*税额*], [{{ order.tax_amount }}],
        {% if order.discount_amount %}
        [*折扣*], [- {{ order.discount_amount }}],
        {% endif %}
      )
      #v(3pt)
      #line(length: 100%, stroke: 0.8pt + ink)
      #v(3pt)
      #grid(
        columns: (1fr, auto),
        [#text(size: 11pt, weight: "bold")[*合计 Grand Total*]],
        [#text(size: 14pt, weight: "bold", fill: amber)[*{{ order.total_amount }}*]],
      )
      #v(2pt)
      #text(size: 7pt, fill: steel)[大写: {{ grand_total_cn }}]
    ]
  ],
)

#v(16pt)

// ========== 签章区 ==========
#grid(
  columns: (1fr, 1fr),
  column-gutter: 30pt,
  [
    #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[卖方签章 SELLER]
    #v(24pt)
    #line(length: 100%, stroke: 0.4pt + ink)
    #v(3pt)
    #text(size: 7pt, fill: steel)[DATE: {{ order.order_date }}]
  ],
  [
    #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[买方签章 BUYER]
    #v(24pt)
    #line(length: 100%, stroke: 0.4pt + ink)
    #v(3pt)
    #text(size: 7pt, fill: steel)[DATE: ____________]
  ],
)
$tpl$,
     NULL, NULL,
     'a4', 'landscape', 60, 40, 30, 30,
     'Source Han Sans SC', 1, 1, 50, 'V3横向精装版-1:1复刻HTML设计',
     NULL, NOW(), NULL, NOW(), 0
);

-- 取消其他订单模板的默认状态
UPDATE mxx_system_pdf_template SET is_default = 0
  WHERE doc_type = 'order' AND template_code != 'order_landscape_v3' AND deleted = 0;

SELECT id, name, template_code, doc_type, paper_size, orientation, is_default, status
  FROM mxx_system_pdf_template
  WHERE doc_type = 'order' AND deleted = 0
  ORDER BY is_default DESC, sort ASC;
