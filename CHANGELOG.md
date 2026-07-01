# Changelog

## v0.1.0 (2026-07-01)

Initial release.

### Core
- PDF extraction via pdf_oxide (134x faster than Python/pdfplumber)
- YAML-based institution spec format
- Text and JSON report output
- Calibration workflow for corpus analysis (systemic vs isolated classification)

### Checkers (33 automated + 7 human review)
**Layout (2):** `global_margins`, `margin_symmetry`
**Typography (11):** `font_size`, `font_weight`, `font_family`, `justification`, `title_page_all_caps`, `title_page_clause_centered`, `title_page_clause_spacing`, `abstract_title_format`, `footnotes_font_consistent`, `abstract_text_centered`
**Structure (13):** `section_presence`, `section_order`, `title_page_no_page_number`, `acceptance_page_number`, `page_numbers_format`, `headings_consistent`, `new_chapters_new_pages`, `hyperlinks_format`, `cv_no_page_number`, `references_heading_format`, `cv_heading_format`, `cv_name_position`, `toc_page_numbers_aligned`, `toc_no_overhang`, `toc_cv_no_dots`
**Content (7):** `boilerplate_match`, `committee_order`, `toc_title_parity`, `copyright_page_format`, `abstract_word_count`, `references_font_consistent`

### CLI
- `diss-check check --spec <yaml> <pdf>` — check single PDF
- `diss-check calibrate --spec <yaml> --corpus <dir>` — batch calibration
- `--json` — machine-readable output
- `--quiet` — FAIL/ERROR results only
- `--check <id>` / `--category <name>` — run specific checks
- Exit codes: 0=PASS, 1=FAIL/ERROR, 2=usage error

### Calibration
- 16-document corpus covering Word and LaTeX dissertations
- 22 calibration decisions logged in `docs/calibration-decisions.md`
- Semantic font detection (math, code, mono blocks)
- Statistical margin checking (percentile-based edge detection)
- Roman-to-Arabic page number transition heuristic for front matter boundary

### Known Limitations
- LaTeX documents produce font family noise from legitimate multi-font use (math, code blocks semi-filtered)
- Boilerplate template matching requires 70% line match threshold
- Abstract detection uses positional heuristic (page between acceptance and TOC)
- TOC leader dots occasionally flagged as margin violations
