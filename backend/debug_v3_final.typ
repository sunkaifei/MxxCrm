#set text(font: "Source Han Sans SC", size: 10.5pt)
#set par(leading: 0.8em, justify: true)

#let amber = rgb("#b8860b")
#let steel = rgb("#6e6e6e")
#let zebra = rgb("#f7f7f7")
#let ink = rgb("#1a1a1a")
#let hair-soft = rgb("#d6d6d6")

#let zebra-row(..cells) = {
  let i = counter("row").get().first()
  counter("row").step()
  let fill = if calc.even(i) { zebra } else { white }
  for c in cells {
    table.cell(fill: fill, c)
  }
}

#set page(
  paper: "a4",
  flipped: true,
  margin: (top: 60pt, bottom: 40pt, left: 30pt, right: 30pt),
  header: context {
    if counter(page).get().first() > 1 [
      #set text(size: 7pt, fill: steel)
      #align(left)[NO. #text(fill: amber, weight: "bold")[QUO-20260731-0110] · 客户: 测试公司]
      #align(center)[#text(size: 12pt, weight: "bold")[产品报价单（续）]]
      #align(right)[ISSUED 2026-07-31]
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
      [测试公司],
      align(center)[QUO-20260731-0110],
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
    #text(tracking: 1pt, fill: amber, weight: "bold", size: 7pt)[// SUPPLIER]
    #v(2pt)
    #text(size: 18pt, weight: "bold")[测试公司]
    #v(3pt)
    #text(size: 7pt, fill: steel)[
      上海 \
      T: 021-1234  E: test@test.com \
      统一社会信用代码: 91310000XXX
    ]
  ],
  [
    #align(right)[
      #text(tracking: 1.5pt, fill: steel, weight: "bold", size: 7pt)[QUOTATION]
      #v(2pt)
      #text(size: 24pt, weight: "bold")[产品报价单]
      #v(4pt)
      #text(size: 8pt, fill: steel)[NO. ]
      #text(size: 8pt, fill: amber, weight: "bold")[QUO-20260731-0110]
      #text(size: 8pt, fill: steel)[  ISSUED 2026-07-31]
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
    #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[// 客户信息 CLIENT]
    #v(3pt)
    #text(size: 8pt)[*客户*: 测试公司] \
    #text(size: 8pt)[*联系人*: 张三]
    
  ],
  [
    #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[// 项目信息 PROJECT]
    #v(3pt)
    #text(size: 8pt)[*标题*: 测试报价] \
    #text(size: 8pt)[*有效期*: 2026-12-31]
     \
    #text(size: 8pt)[*交货*: 2026-08-15]
    
  ],
  [
    #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[// 商务条款 TERMS]
    #v(3pt)
    #text(size: 8pt)[*付款*: 月结30天] \
    #text(size: 8pt)[*交货*: 送货上门] \
    #text(size: 8pt)[*币种*: 人民币]
  ],
)

#v(6pt)
#line(length: 100%, stroke: 0.4pt + hair-soft)
#v(4pt)

// ========== 产品表格（斑马纹） ==========
#counter("row").update(0)
#table(
  columns: (5%, 12%, 22%, 31%, 8%, 11%, 11%),
  align: (center + horizon, left + horizon, left + horizon, left + horizon, right + horizon, right + horizon, right + horizon),
  stroke: 0.3pt + hair-soft,
  table.header(
    [*序号*], [*产品编号*], [*产品名称*], [*规格参数*], [*数量*], [*单价*], [*金额*],
  ),
  
  #zebra-row(
    [1],
    [P001],
    [产品1],
    [规格1],
    [1 个],
    [100.00],
    [*100*],
  )
  
  #zebra-row(
    [2],
    [P002],
    [产品2],
    [规格2],
    [2 个],
    [100.00],
    [*200*],
  )
  
  #zebra-row(
    [3],
    [P003],
    [产品3],
    [规格3],
    [3 个],
    [100.00],
    [*300*],
  )
  
  #zebra-row(
    [4],
    [P004],
    [产品4],
    [规格4],
    [4 个],
    [100.00],
    [*400*],
  )
  
  #zebra-row(
    [5],
    [P005],
    [产品5],
    [规格5],
    [5 个],
    [100.00],
    [*500*],
  )
  
)

#v(10pt)

// ========== 金额汇总 + 备注 ==========
#grid(
  columns: (1.2fr, 1fr),
  column-gutter: 20pt,
  [
    #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[// 备注条款 TERMS & CONDITIONS]
    #v(3pt)
    #line(length: 100%, stroke: 0.3pt + hair-soft)
    #v(3pt)
    
    #text(size: 7.5pt)[测试备注]
    #v(2pt)
    
    #text(size: 7pt, fill: steel)[
      - 报价有效期内有效，逾期需重新核定 \
      - 数量正负5%属正常交付范围，按实际结算 \
      - 含增值税，不含安装调试费 \
      - 本报价单金额汇总以末页为准
    ]
  ],
  [
    #block(inset: 8pt, stroke: 0.5pt + ink, width: 100%)[
      #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[// 金额汇总 TOTAL]
      #v(4pt)
      #grid(
        columns: (1fr, auto),
        gutter: 4pt,
        [*小计 Subtotal*], [1000],
        [*税额 VAT*], [130],
        
      )
      #v(3pt)
      #line(length: 100%, stroke: 0.8pt + ink)
      #v(3pt)
      #grid(
        columns: (1fr, auto),
        [#text(size: 11pt, weight: "bold")[*合计 Grand Total*]],
        [#text(size: 14pt, weight: "bold", fill: amber)[*1130*]],
      )
      #v(2pt)
      #text(size: 7pt, fill: steel)[大写: 壹仟壹佰叁拾元整]
    ]
  ],
)

#v(16pt)

// ========== 签章区 ==========
#grid(
  columns: (1fr, 1fr),
  column-gutter: 30pt,
  [
    #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[// 供方签章 SUPPLIER]
    #v(24pt)
    #line(length: 100%, stroke: 0.4pt + ink)
    #v(3pt)
    #text(size: 7pt, fill: steel)[DATE: 2026-07-31]
  ],
  [
    #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[// 客户签章 CLIENT]
    #v(24pt)
    #line(length: 100%, stroke: 0.4pt + ink)
    #v(3pt)
    #text(size: 7pt, fill: steel)[DATE: ____________]
  ],
)