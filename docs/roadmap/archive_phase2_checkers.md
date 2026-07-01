# Phase 2 — Typography, Structure, and Content Checkers (Archived)

> 10 automated checkers built. Python/pdfplumber.

| Round | Feature | Status | Notes |
|-------|---------|--------|-------|
| 9 | font_size checker | ✅ | 10 tests. Known: < 8.5pt formatting artifacts filtered; figure content vs legend distinction not made |
| 10 | font_weight checker | ✅ | 11 tests. Detects bold/italic/bold-italic from font name. Supports page filter + invert mode |
| 11 | font_family checker | ✅ | 8 tests. Known: chart/graph embedded fonts (e.g. Aptos) flagged as mismatches; special fonts (Symbol, Wingdings) excluded |
| 12 | justification checker | ✅ | 2 tests. Right-edge stdev heuristic (sd < 8 = justified). Pages < 50 spans excluded as figures. Known FN: mixed justification within page, tight left-alignment misclassified as justified |
| 13 | section_presence checker | ✅ | 9 tests. Keyword-based detection. Promoted to automatable. Known FN: non-English docs, scanned PDFs, non-standard section naming |
| 14 | section_order checker | ✅ | 7 tests. TOC + heuristic. Abstract detected by backwards scan from TOC |
| 15 | boilerplate_match checker | ✅ | 7 tests. Verifies template text with {variable} substitution. Tolerates extra lines, multiline vars, trailing punctuation |
| 16 | committee_order checker | ✅ | 4 tests. FAILs when Chair label missing or chair not first |
| 17 | toc_title_parity checker | ✅ | 2 tests. Known: PDF glyph reordering issues |
| 18 | human checker (manual passthrough) | ✅ | 2 tests. Always returns MANUAL with configurable prompt |
| 19 | JSON report output | ✅ | 1 test. `--json` flag |
| 20 | Expand IU spec to full checklist | ✅ | 27 checks: 10 automated + 17 human review. Covers all sections of IU format review checklist (rev Sept 2025) |
