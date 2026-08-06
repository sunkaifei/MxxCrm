#set page(
  paper: "a4",
  flipped: true,
  margin: 40pt,
  header: context {
    let pg = counter(page).get().first()
    if pg > 1 [
      #text(size: 18pt)[续页 header page=]
      #context counter(page).display("1")
    ]
  },
  footer: context {
    let cur = counter(page).get().first()
    let total = counter(page).final().first()
    [Page #context counter(page).display("1") of #context counter(page).final().first()]
  },
)

#set text(font: "Source Han Sans SC", size: 12pt)

#context [当前页: #counter(page).display("1")]

#lorem(2000)
