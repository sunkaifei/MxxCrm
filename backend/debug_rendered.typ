#set page(
  paper: "a4",
  flipped: true,
  margin: (top: 30pt, bottom: 36pt, left: 30pt, right: 30pt),
  footer: context [
    #set text(size: 7pt, fill: rgb("#6e6e6e"))
    #grid(
      columns: (1fr, 1fr, 1fr),
      align: horizon,
      [测试公司],
      #align(center)[QUO-20260731-0110],
      #align(right)[第 #context counter(page).display("1") 页],
    )
  ],
)

#set text(font: "Source Han Sans SC", size: 9pt, fill: black)
#set par(leading: 0.75em, justify: false)

// ========== 页面设置（模板自控，支持续页header和正确页码） ==========
#let amber = rgb("#b8860b")
#let steel = rgb("#6e6e6e")
#let zebra = rgb("#f7f7f7")
#let hair = rgb("#1a1a1a")
#let hair-soft = rgb("#d6d6d6")

#set page(
  paper: "a4",
  flipped: true,
  margin: (top: 28pt, bottom: 32pt, left: 28pt, right: 28pt),
  header: context {
    // 第2页起显示续页抬头
    if counter(page).get().first() > 1 [
      #set text(size: 7pt, fill: steel)
      #grid(
        columns: (1fr, 1fr, 1fr),
        align: horizon,
        [NO. *QUO-20260731-0110* · 客户: 全链路测试公司_1785511987],
        #align(center)[#text(size: 11pt, weight: "bold", fill: hair)[产品报价单（续）]],
        #align(right)[ISSUED 2026-07-31],
      )
      #v(2pt)
      #line(length: 100%, stroke: 0.5pt + hair)
      #v(4pt)
    ]
  },
  footer: context [
    #set text(size: 7pt, fill: steel)
    #grid(
      columns: (1fr, 1fr, 1fr),
      align: horizon,
      [测试公司],
      #align(center)[QUO-20260731-0110 · CONTINUED],
      #align(right)[PAGE #context counter(page).display("1") / #context counter(page).final().first().display("1")],
    )
  ],
)

#set text(font: "Source Han Sans SC", size: 9pt, fill: black)
#set par(leading: 0.75em, justify: false)

// ========== 抬头（仅首页） ==========
#context {
  if counter(page).get().first() == 1 [
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
    #line(length: 100%, stroke: 1pt + hair)
    #v(6pt)

    // 信息三栏
    #grid(
      columns: (1fr, 1fr, 1fr),
      column-gutter: 14pt,
      [
        #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[// 客户信息 CLIENT]
        #v(3pt)
        #text(size: 8pt)[*CUSTOMER*  全链路测试公司_1785511987] \
        #text(size: 8pt)[*CONTACT*  None]
        
      ],
      [
        #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[// 项目信息 PROJECT]
        #v(3pt)
        #text(size: 8pt)[*TITLE*  合同10全链路测试报价单] \
        #text(size: 8pt)[*VALID*  ]
        
      ],
      [
        #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[// 商务条款 TERMS]
        #v(3pt)
        #text(size: 8pt)[*PAYMENT*  签订后30天付款] \
        #text(size: 8pt)[*DELIVERY*  签订后60天交付] \
        #text(size: 8pt)[*CURRENCY*  人民币]
      ],
    )
    #v(6pt)
    #line(length: 100%, stroke: 0.4pt + hair-soft)
    #v(4pt)
  ]
}

// ========== 产品表格（含斑马纹） ==========
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

  
  table.cell(fill: if calc.even(1) { zebra } else { white })[1],
  table.cell(fill: if calc.even(1) { zebra } else { white })[PRD-001],
  table.cell(fill: if calc.even(1) { zebra } else { white })[测试产品1],
  table.cell(fill: if calc.even(1) { zebra } else { white })[规格1],
  table.cell(fill: if calc.even(1) { zebra } else { white }, align: right)[1 个],
  table.cell(fill: if calc.even(1) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(1) { zebra } else { white }, align: right)[*100*],
  
  table.cell(fill: if calc.even(2) { zebra } else { white })[2],
  table.cell(fill: if calc.even(2) { zebra } else { white })[PRD-002],
  table.cell(fill: if calc.even(2) { zebra } else { white })[测试产品2],
  table.cell(fill: if calc.even(2) { zebra } else { white })[规格2],
  table.cell(fill: if calc.even(2) { zebra } else { white }, align: right)[2 个],
  table.cell(fill: if calc.even(2) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(2) { zebra } else { white }, align: right)[*200*],
  
  table.cell(fill: if calc.even(3) { zebra } else { white })[3],
  table.cell(fill: if calc.even(3) { zebra } else { white })[PRD-003],
  table.cell(fill: if calc.even(3) { zebra } else { white })[测试产品3],
  table.cell(fill: if calc.even(3) { zebra } else { white })[规格3],
  table.cell(fill: if calc.even(3) { zebra } else { white }, align: right)[3 个],
  table.cell(fill: if calc.even(3) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(3) { zebra } else { white }, align: right)[*300*],
  
  table.cell(fill: if calc.even(4) { zebra } else { white })[4],
  table.cell(fill: if calc.even(4) { zebra } else { white })[PRD-004],
  table.cell(fill: if calc.even(4) { zebra } else { white })[测试产品4],
  table.cell(fill: if calc.even(4) { zebra } else { white })[规格4],
  table.cell(fill: if calc.even(4) { zebra } else { white }, align: right)[4 个],
  table.cell(fill: if calc.even(4) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(4) { zebra } else { white }, align: right)[*400*],
  
  table.cell(fill: if calc.even(5) { zebra } else { white })[5],
  table.cell(fill: if calc.even(5) { zebra } else { white })[PRD-005],
  table.cell(fill: if calc.even(5) { zebra } else { white })[测试产品5],
  table.cell(fill: if calc.even(5) { zebra } else { white })[规格5],
  table.cell(fill: if calc.even(5) { zebra } else { white }, align: right)[5 个],
  table.cell(fill: if calc.even(5) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(5) { zebra } else { white }, align: right)[*500*],
  
  table.cell(fill: if calc.even(6) { zebra } else { white })[6],
  table.cell(fill: if calc.even(6) { zebra } else { white })[PRD-006],
  table.cell(fill: if calc.even(6) { zebra } else { white })[测试产品6],
  table.cell(fill: if calc.even(6) { zebra } else { white })[规格6],
  table.cell(fill: if calc.even(6) { zebra } else { white }, align: right)[6 个],
  table.cell(fill: if calc.even(6) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(6) { zebra } else { white }, align: right)[*600*],
  
  table.cell(fill: if calc.even(7) { zebra } else { white })[7],
  table.cell(fill: if calc.even(7) { zebra } else { white })[PRD-007],
  table.cell(fill: if calc.even(7) { zebra } else { white })[测试产品7],
  table.cell(fill: if calc.even(7) { zebra } else { white })[规格7],
  table.cell(fill: if calc.even(7) { zebra } else { white }, align: right)[7 个],
  table.cell(fill: if calc.even(7) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(7) { zebra } else { white }, align: right)[*700*],
  
  table.cell(fill: if calc.even(8) { zebra } else { white })[8],
  table.cell(fill: if calc.even(8) { zebra } else { white })[PRD-008],
  table.cell(fill: if calc.even(8) { zebra } else { white })[测试产品8],
  table.cell(fill: if calc.even(8) { zebra } else { white })[规格8],
  table.cell(fill: if calc.even(8) { zebra } else { white }, align: right)[8 个],
  table.cell(fill: if calc.even(8) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(8) { zebra } else { white }, align: right)[*800*],
  
  table.cell(fill: if calc.even(9) { zebra } else { white })[9],
  table.cell(fill: if calc.even(9) { zebra } else { white })[PRD-009],
  table.cell(fill: if calc.even(9) { zebra } else { white })[测试产品9],
  table.cell(fill: if calc.even(9) { zebra } else { white })[规格9],
  table.cell(fill: if calc.even(9) { zebra } else { white }, align: right)[9 个],
  table.cell(fill: if calc.even(9) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(9) { zebra } else { white }, align: right)[*900*],
  
  table.cell(fill: if calc.even(10) { zebra } else { white })[10],
  table.cell(fill: if calc.even(10) { zebra } else { white })[PRD-010],
  table.cell(fill: if calc.even(10) { zebra } else { white })[测试产品10],
  table.cell(fill: if calc.even(10) { zebra } else { white })[规格10],
  table.cell(fill: if calc.even(10) { zebra } else { white }, align: right)[10 个],
  table.cell(fill: if calc.even(10) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(10) { zebra } else { white }, align: right)[*1000*],
  
  table.cell(fill: if calc.even(11) { zebra } else { white })[11],
  table.cell(fill: if calc.even(11) { zebra } else { white })[PRD-011],
  table.cell(fill: if calc.even(11) { zebra } else { white })[测试产品11],
  table.cell(fill: if calc.even(11) { zebra } else { white })[规格11],
  table.cell(fill: if calc.even(11) { zebra } else { white }, align: right)[11 个],
  table.cell(fill: if calc.even(11) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(11) { zebra } else { white }, align: right)[*1100*],
  
  table.cell(fill: if calc.even(12) { zebra } else { white })[12],
  table.cell(fill: if calc.even(12) { zebra } else { white })[PRD-012],
  table.cell(fill: if calc.even(12) { zebra } else { white })[测试产品12],
  table.cell(fill: if calc.even(12) { zebra } else { white })[规格12],
  table.cell(fill: if calc.even(12) { zebra } else { white }, align: right)[12 个],
  table.cell(fill: if calc.even(12) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(12) { zebra } else { white }, align: right)[*1200*],
  
  table.cell(fill: if calc.even(13) { zebra } else { white })[13],
  table.cell(fill: if calc.even(13) { zebra } else { white })[PRD-013],
  table.cell(fill: if calc.even(13) { zebra } else { white })[测试产品13],
  table.cell(fill: if calc.even(13) { zebra } else { white })[规格13],
  table.cell(fill: if calc.even(13) { zebra } else { white }, align: right)[13 个],
  table.cell(fill: if calc.even(13) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(13) { zebra } else { white }, align: right)[*1300*],
  
  table.cell(fill: if calc.even(14) { zebra } else { white })[14],
  table.cell(fill: if calc.even(14) { zebra } else { white })[PRD-014],
  table.cell(fill: if calc.even(14) { zebra } else { white })[测试产品14],
  table.cell(fill: if calc.even(14) { zebra } else { white })[规格14],
  table.cell(fill: if calc.even(14) { zebra } else { white }, align: right)[14 个],
  table.cell(fill: if calc.even(14) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(14) { zebra } else { white }, align: right)[*1400*],
  
  table.cell(fill: if calc.even(15) { zebra } else { white })[15],
  table.cell(fill: if calc.even(15) { zebra } else { white })[PRD-015],
  table.cell(fill: if calc.even(15) { zebra } else { white })[测试产品15],
  table.cell(fill: if calc.even(15) { zebra } else { white })[规格15],
  table.cell(fill: if calc.even(15) { zebra } else { white }, align: right)[15 个],
  table.cell(fill: if calc.even(15) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(15) { zebra } else { white }, align: right)[*1500*],
  
  table.cell(fill: if calc.even(16) { zebra } else { white })[16],
  table.cell(fill: if calc.even(16) { zebra } else { white })[PRD-016],
  table.cell(fill: if calc.even(16) { zebra } else { white })[测试产品16],
  table.cell(fill: if calc.even(16) { zebra } else { white })[规格16],
  table.cell(fill: if calc.even(16) { zebra } else { white }, align: right)[16 个],
  table.cell(fill: if calc.even(16) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(16) { zebra } else { white }, align: right)[*1600*],
  
  table.cell(fill: if calc.even(17) { zebra } else { white })[17],
  table.cell(fill: if calc.even(17) { zebra } else { white })[PRD-017],
  table.cell(fill: if calc.even(17) { zebra } else { white })[测试产品17],
  table.cell(fill: if calc.even(17) { zebra } else { white })[规格17],
  table.cell(fill: if calc.even(17) { zebra } else { white }, align: right)[17 个],
  table.cell(fill: if calc.even(17) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(17) { zebra } else { white }, align: right)[*1700*],
  
  table.cell(fill: if calc.even(18) { zebra } else { white })[18],
  table.cell(fill: if calc.even(18) { zebra } else { white })[PRD-018],
  table.cell(fill: if calc.even(18) { zebra } else { white })[测试产品18],
  table.cell(fill: if calc.even(18) { zebra } else { white })[规格18],
  table.cell(fill: if calc.even(18) { zebra } else { white }, align: right)[18 个],
  table.cell(fill: if calc.even(18) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(18) { zebra } else { white }, align: right)[*1800*],
  
  table.cell(fill: if calc.even(19) { zebra } else { white })[19],
  table.cell(fill: if calc.even(19) { zebra } else { white })[PRD-019],
  table.cell(fill: if calc.even(19) { zebra } else { white })[测试产品19],
  table.cell(fill: if calc.even(19) { zebra } else { white })[规格19],
  table.cell(fill: if calc.even(19) { zebra } else { white }, align: right)[19 个],
  table.cell(fill: if calc.even(19) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(19) { zebra } else { white }, align: right)[*1900*],
  
  table.cell(fill: if calc.even(20) { zebra } else { white })[20],
  table.cell(fill: if calc.even(20) { zebra } else { white })[PRD-020],
  table.cell(fill: if calc.even(20) { zebra } else { white })[测试产品20],
  table.cell(fill: if calc.even(20) { zebra } else { white })[规格20],
  table.cell(fill: if calc.even(20) { zebra } else { white }, align: right)[20 个],
  table.cell(fill: if calc.even(20) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(20) { zebra } else { white }, align: right)[*2000*],
  
  table.cell(fill: if calc.even(21) { zebra } else { white })[21],
  table.cell(fill: if calc.even(21) { zebra } else { white })[PRD-021],
  table.cell(fill: if calc.even(21) { zebra } else { white })[测试产品21],
  table.cell(fill: if calc.even(21) { zebra } else { white })[规格21],
  table.cell(fill: if calc.even(21) { zebra } else { white }, align: right)[21 个],
  table.cell(fill: if calc.even(21) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(21) { zebra } else { white }, align: right)[*2100*],
  
  table.cell(fill: if calc.even(22) { zebra } else { white })[22],
  table.cell(fill: if calc.even(22) { zebra } else { white })[PRD-022],
  table.cell(fill: if calc.even(22) { zebra } else { white })[测试产品22],
  table.cell(fill: if calc.even(22) { zebra } else { white })[规格22],
  table.cell(fill: if calc.even(22) { zebra } else { white }, align: right)[22 个],
  table.cell(fill: if calc.even(22) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(22) { zebra } else { white }, align: right)[*2200*],
  
  table.cell(fill: if calc.even(23) { zebra } else { white })[23],
  table.cell(fill: if calc.even(23) { zebra } else { white })[PRD-023],
  table.cell(fill: if calc.even(23) { zebra } else { white })[测试产品23],
  table.cell(fill: if calc.even(23) { zebra } else { white })[规格23],
  table.cell(fill: if calc.even(23) { zebra } else { white }, align: right)[23 个],
  table.cell(fill: if calc.even(23) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(23) { zebra } else { white }, align: right)[*2300*],
  
  table.cell(fill: if calc.even(24) { zebra } else { white })[24],
  table.cell(fill: if calc.even(24) { zebra } else { white })[PRD-024],
  table.cell(fill: if calc.even(24) { zebra } else { white })[测试产品24],
  table.cell(fill: if calc.even(24) { zebra } else { white })[规格24],
  table.cell(fill: if calc.even(24) { zebra } else { white }, align: right)[24 个],
  table.cell(fill: if calc.even(24) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(24) { zebra } else { white }, align: right)[*2400*],
  
  table.cell(fill: if calc.even(25) { zebra } else { white })[25],
  table.cell(fill: if calc.even(25) { zebra } else { white })[PRD-025],
  table.cell(fill: if calc.even(25) { zebra } else { white })[测试产品25],
  table.cell(fill: if calc.even(25) { zebra } else { white })[规格25],
  table.cell(fill: if calc.even(25) { zebra } else { white }, align: right)[25 个],
  table.cell(fill: if calc.even(25) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(25) { zebra } else { white }, align: right)[*2500*],
  
  table.cell(fill: if calc.even(26) { zebra } else { white })[26],
  table.cell(fill: if calc.even(26) { zebra } else { white })[PRD-026],
  table.cell(fill: if calc.even(26) { zebra } else { white })[测试产品26],
  table.cell(fill: if calc.even(26) { zebra } else { white })[规格26],
  table.cell(fill: if calc.even(26) { zebra } else { white }, align: right)[26 个],
  table.cell(fill: if calc.even(26) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(26) { zebra } else { white }, align: right)[*2600*],
  
  table.cell(fill: if calc.even(27) { zebra } else { white })[27],
  table.cell(fill: if calc.even(27) { zebra } else { white })[PRD-027],
  table.cell(fill: if calc.even(27) { zebra } else { white })[测试产品27],
  table.cell(fill: if calc.even(27) { zebra } else { white })[规格27],
  table.cell(fill: if calc.even(27) { zebra } else { white }, align: right)[27 个],
  table.cell(fill: if calc.even(27) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(27) { zebra } else { white }, align: right)[*2700*],
  
  table.cell(fill: if calc.even(28) { zebra } else { white })[28],
  table.cell(fill: if calc.even(28) { zebra } else { white })[PRD-028],
  table.cell(fill: if calc.even(28) { zebra } else { white })[测试产品28],
  table.cell(fill: if calc.even(28) { zebra } else { white })[规格28],
  table.cell(fill: if calc.even(28) { zebra } else { white }, align: right)[28 个],
  table.cell(fill: if calc.even(28) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(28) { zebra } else { white }, align: right)[*2800*],
  
  table.cell(fill: if calc.even(29) { zebra } else { white })[29],
  table.cell(fill: if calc.even(29) { zebra } else { white })[PRD-029],
  table.cell(fill: if calc.even(29) { zebra } else { white })[测试产品29],
  table.cell(fill: if calc.even(29) { zebra } else { white })[规格29],
  table.cell(fill: if calc.even(29) { zebra } else { white }, align: right)[29 个],
  table.cell(fill: if calc.even(29) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(29) { zebra } else { white }, align: right)[*2900*],
  
  table.cell(fill: if calc.even(30) { zebra } else { white })[30],
  table.cell(fill: if calc.even(30) { zebra } else { white })[PRD-030],
  table.cell(fill: if calc.even(30) { zebra } else { white })[测试产品30],
  table.cell(fill: if calc.even(30) { zebra } else { white })[规格30],
  table.cell(fill: if calc.even(30) { zebra } else { white }, align: right)[30 个],
  table.cell(fill: if calc.even(30) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(30) { zebra } else { white }, align: right)[*3000*],
  
  table.cell(fill: if calc.even(31) { zebra } else { white })[31],
  table.cell(fill: if calc.even(31) { zebra } else { white })[PRD-031],
  table.cell(fill: if calc.even(31) { zebra } else { white })[测试产品31],
  table.cell(fill: if calc.even(31) { zebra } else { white })[规格31],
  table.cell(fill: if calc.even(31) { zebra } else { white }, align: right)[31 个],
  table.cell(fill: if calc.even(31) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(31) { zebra } else { white }, align: right)[*3100*],
  
  table.cell(fill: if calc.even(32) { zebra } else { white })[32],
  table.cell(fill: if calc.even(32) { zebra } else { white })[PRD-032],
  table.cell(fill: if calc.even(32) { zebra } else { white })[测试产品32],
  table.cell(fill: if calc.even(32) { zebra } else { white })[规格32],
  table.cell(fill: if calc.even(32) { zebra } else { white }, align: right)[32 个],
  table.cell(fill: if calc.even(32) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(32) { zebra } else { white }, align: right)[*3200*],
  
  table.cell(fill: if calc.even(33) { zebra } else { white })[33],
  table.cell(fill: if calc.even(33) { zebra } else { white })[PRD-033],
  table.cell(fill: if calc.even(33) { zebra } else { white })[测试产品33],
  table.cell(fill: if calc.even(33) { zebra } else { white })[规格33],
  table.cell(fill: if calc.even(33) { zebra } else { white }, align: right)[33 个],
  table.cell(fill: if calc.even(33) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(33) { zebra } else { white }, align: right)[*3300*],
  
  table.cell(fill: if calc.even(34) { zebra } else { white })[34],
  table.cell(fill: if calc.even(34) { zebra } else { white })[PRD-034],
  table.cell(fill: if calc.even(34) { zebra } else { white })[测试产品34],
  table.cell(fill: if calc.even(34) { zebra } else { white })[规格34],
  table.cell(fill: if calc.even(34) { zebra } else { white }, align: right)[34 个],
  table.cell(fill: if calc.even(34) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(34) { zebra } else { white }, align: right)[*3400*],
  
  table.cell(fill: if calc.even(35) { zebra } else { white })[35],
  table.cell(fill: if calc.even(35) { zebra } else { white })[PRD-035],
  table.cell(fill: if calc.even(35) { zebra } else { white })[测试产品35],
  table.cell(fill: if calc.even(35) { zebra } else { white })[规格35],
  table.cell(fill: if calc.even(35) { zebra } else { white }, align: right)[35 个],
  table.cell(fill: if calc.even(35) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(35) { zebra } else { white }, align: right)[*3500*],
  
  table.cell(fill: if calc.even(36) { zebra } else { white })[36],
  table.cell(fill: if calc.even(36) { zebra } else { white })[PRD-036],
  table.cell(fill: if calc.even(36) { zebra } else { white })[测试产品36],
  table.cell(fill: if calc.even(36) { zebra } else { white })[规格36],
  table.cell(fill: if calc.even(36) { zebra } else { white }, align: right)[36 个],
  table.cell(fill: if calc.even(36) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(36) { zebra } else { white }, align: right)[*3600*],
  
  table.cell(fill: if calc.even(37) { zebra } else { white })[37],
  table.cell(fill: if calc.even(37) { zebra } else { white })[PRD-037],
  table.cell(fill: if calc.even(37) { zebra } else { white })[测试产品37],
  table.cell(fill: if calc.even(37) { zebra } else { white })[规格37],
  table.cell(fill: if calc.even(37) { zebra } else { white }, align: right)[37 个],
  table.cell(fill: if calc.even(37) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(37) { zebra } else { white }, align: right)[*3700*],
  
  table.cell(fill: if calc.even(38) { zebra } else { white })[38],
  table.cell(fill: if calc.even(38) { zebra } else { white })[PRD-038],
  table.cell(fill: if calc.even(38) { zebra } else { white })[测试产品38],
  table.cell(fill: if calc.even(38) { zebra } else { white })[规格38],
  table.cell(fill: if calc.even(38) { zebra } else { white }, align: right)[38 个],
  table.cell(fill: if calc.even(38) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(38) { zebra } else { white }, align: right)[*3800*],
  
  table.cell(fill: if calc.even(39) { zebra } else { white })[39],
  table.cell(fill: if calc.even(39) { zebra } else { white })[PRD-039],
  table.cell(fill: if calc.even(39) { zebra } else { white })[测试产品39],
  table.cell(fill: if calc.even(39) { zebra } else { white })[规格39],
  table.cell(fill: if calc.even(39) { zebra } else { white }, align: right)[39 个],
  table.cell(fill: if calc.even(39) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(39) { zebra } else { white }, align: right)[*3900*],
  
  table.cell(fill: if calc.even(40) { zebra } else { white })[40],
  table.cell(fill: if calc.even(40) { zebra } else { white })[PRD-040],
  table.cell(fill: if calc.even(40) { zebra } else { white })[测试产品40],
  table.cell(fill: if calc.even(40) { zebra } else { white })[规格40],
  table.cell(fill: if calc.even(40) { zebra } else { white }, align: right)[40 个],
  table.cell(fill: if calc.even(40) { zebra } else { white }, align: right)[100.00],
  table.cell(fill: if calc.even(40) { zebra } else { white }, align: right)[*4000*],
  
)

#v(10pt)

// ========== 金额汇总 + 备注（仅末页，context判断最后一页） ==========
#context {
  // 最后一页显示金额汇总
  if counter(page).get().first() == counter(page).final().first() [
    #grid(
      columns: (1.2fr, 1fr),
      column-gutter: 20pt,
      [
        #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[// 备注条款 TERMS & CONDITIONS]
        #v(3pt)
        #line(length: 100%, stroke: 0.3pt + hair-soft, dash: ("dot", 1pt))
        #v(3pt)
        
        #text(size: 7pt, fill: steel)[
          - 报价有效期内有效，逾期需重新核定 \
          - 数量正负5%属正常交付范围，按实际结算 \
          - 含增值税，不含安装调试费 \
          - 本报价单金额汇总以末页为准
        ]
      ],
      [
        #block(inset: 8pt, stroke: 0.5pt + hair, width: 100%)[
          #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[// 金额汇总 TOTAL]
          #v(4pt)
          #grid(
            columns: (1fr, auto),
            align: (left + horizon, right + horizon),
            gutter: 4pt,
            [*小计 Subtotal*], [1892689.20],
            [*税额 VAT*], [],
            
          )
          #v(3pt)
          #line(length: 100%, stroke: 0.8pt + hair)
          #v(3pt)
          #grid(
            columns: (1fr, auto),
            align: (left + horizon, right + horizon),
            [#text(size: 11pt, weight: "bold")[*合计 Grand Total*]],
            [#text(size: 14pt, weight: "bold", fill: amber)[**]],
          )
          #v(2pt)
          #text(size: 7pt, fill: steel)[大写: 零元整]
        ]
      ],
    )

    #v(16pt)

    // 签章区
    #grid(
      columns: (1fr, 1fr),
      column-gutter: 30pt,
      [
        #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[// 供方签章 SUPPLIER]
        #v(24pt)
        #line(length: 100%, stroke: 0.4pt + hair)
        #v(3pt)
        #text(size: 7pt, fill: steel)[DATE: 2026-07-31]
      ],
      [
        #text(tracking: 0.8pt, fill: amber, weight: "bold", size: 6.5pt)[// 客户签章 CLIENT]
        #v(24pt)
        #line(length: 100%, stroke: 0.4pt + hair)
        #v(3pt)
        #text(size: 7pt, fill: steel)[DATE: ____________]
      ],
    )
  ]
}