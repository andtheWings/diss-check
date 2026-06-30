#set page(margin: 1in)
#set text(font: "DejaVu Sans", size: 10pt)

= Project Roadmap
#text(size: 9pt, fill: gray)[
  Updated: June 30, 2026 \
  Current round: font_size checker \
  Test document: `tests/fixtures/2020-12-chambers.pdf`
]

== Phase 1 — MVP (end-to-end pipeline with 1 checker)

#figure(
  table(
    columns: (0.6fr, 1fr, 6fr, 0.6fr),
    align: (center, left, left, center),
    stroke: 0.5pt,
    inset: 6pt,
    table.header(
      [*Round*], [*Feature*], [*Detail*], [*Tests*],
    ),
    [1], [Project scaffold],
    [Spec: — \ Plan: `plans/...plan.md`],
    [—],
    [2], [Spec models + IU spec],
    [Design: `specs/...design.md` \ Plan: `plans/...plan.md`],
    [2],
    [3], [Document IR],
    [TextSpan / Page / Document pydantic models],
    [—],
    [4], [Pdfplumber extractor],
    [Word-level extraction, builds Document IR in ~1s],
    [—],
    [5], [Engine + checker registry],
    [Orchestrates spec → extract → check → report],
    [—],
    [6], [Margins checker],
    [0.25in tolerance, page-number zone exclusion. 33 violations on test doc (all TOC leader dots)],
    [6],
    [7], [CLI + report (text)],
    [`diss-check --spec <path> <pdf>`],
    [2],
    [8], [Integration test],
    [End-to-end pipeline test against Chambers dissertation],
    [1],
  ),
  caption: none,
)

== Phase 2 — Typography, structure, and content checkers

#figure(
  table(
    columns: (0.6fr, 1fr, 3fr, 1.5fr, 0.6fr),
    align: (center, left, center, left, center),
    stroke: 0.5pt,
    inset: 6pt,
    table.header(
      [*Round*], [*Feature*], [*Status*], [*Design*], [*Tests*],
    ),
    [9], [font_size checker], [☐], [Design: `specs/...design.md` \ Plan: `plans/...plan.md`], [—],
    [10], [font_weight checker], [☐], [Same design spec], [—],
    [11], [font_family checker], [☐], [Same design spec], [—],
    [12], [justification checker], [☐], [Same design spec], [—],
    [13], [section_presence checker], [☐], [Same design spec], [—],
    [14], [section_order checker], [☐], [Same design spec], [—],
    [15], [text_match checker], [☐], [Same design spec], [—],
    [16], [committee_order checker], [☐], [Same design spec], [—],
    [17], [toc_title_parity checker], [☐], [Same design spec], [—],
    [18], [human checker], [☐], [Same design spec], [—],
    [19], [JSON report output], [☐], [Same design spec], [—],
    [20], [Full IU spec], [☐], [Same design spec], [—],
  ),
  caption: none,
)

== Phase 3 — Calibration

#figure(
  table(
    columns: (0.6fr, 1fr, 3fr, 1.5fr, 0.6fr),
    align: (center, left, center, left, center),
    stroke: 0.5pt,
    inset: 6pt,
    table.header(
      [*Round*], [*Feature*], [*Status*], [*Design*], [*Tests*],
    ),
    [21], [Calibration workflow], [☐], [Design: `specs/...design.md`], [—],
    [22], [veraPDF extractor], [☐], [—], [—],
  ),
  caption: none,
)
