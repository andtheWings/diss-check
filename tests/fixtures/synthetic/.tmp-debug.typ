#set page(paper: "us-letter", margin: (left: 1.25in, right: 1.25in, top: 1in, bottom: 1in))
#set text(size: 12pt)
#set par(justify: false)

// Page 1: centered heading + body text
#align(center, text(size: 14pt)[CHAPTER ONE])
#align(center, text(size: 14pt)[INTRODUCTION AND BACKGROUND])
#v(1in)
#lorem(30)
#lorem(30)

// Page 2: body + figure + table
#pagebreak()
#lorem(30)
#v(0.5in)

#align(center, rect(width: 60%, height: 2in, stroke: black)[
  Figure 1: A synthetic test figure
])

#v(0.3in)

#align(center, table(
  columns: 3,
  [Column A], [Column B], [Column C],
  [Data 1], [Data 2], [Data 3],
  [Data 4], [Data 5], [Data 6],
))

// Page 3: sparse dedication-style page
#pagebreak()
#v(2in)
#align(center, text(weight: "regular")[
  To my family and friends,\
  for their unwavering support.
])
