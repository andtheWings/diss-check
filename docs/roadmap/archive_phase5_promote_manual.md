# Phase 5 — Promote Manual Checks to Automated (Archived)

> pdf_oxide provides per-span `color`, `is_bold`, `is_italic`, `font_name`, `font_size`, `bbox` — enabling checks that were MANUAL with pdfplumber.

| Round | Feature | Status | Notes |
|-------|---------|--------|-------|
| R11 | `title_page_no_page_number` | ✅ | PASS on both chambers and alexander |
| R12 | `acceptance_page_page_number_ii` | ✅ | PASS — detects 'ii' Roman numeral |
| R13 | `page_numbers_format` | ✅ | PASS — Roman in front matter, Arabic in body |
| R14 | `headings_consistent` | ✅ | PASS — heading font matches body modal |
| R15 | `new_chapters_new_pages` | ✅ | PASS — chapters at top of pages |
| R16 | `hyperlinks_format` | ✅ | PASS — URL text uses body font |
| R17 | `cv_no_page_number` | ✅ | PASS on chambers, ERROR on alexander (no CV heading found) |
| R18 | calibration (Rust) | ✅ | 0.80s for 3-doc corpus (Python: ~107s, 134x faster) |
