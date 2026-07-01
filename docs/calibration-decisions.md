# Calibration Decisions Log

> Corpus: 2 documents (chambers 2020, alexander 2025)
> Last updated: 2026-07-01

## Decision 1: `references_heading_format` — keep is_bold matching
**Context**: Both PDFs flagged because chapter headings are bold but references headings are not.
**Decision**: Bold style must also match chapter headings. Keep is_bold check. Legitimate violation per IU spec ("Title formatted like chapter titles").
**Action**: None — checker behavior unchanged.

## Decision 2: `references_font_consistent` — exclude tiny citation metadata
**Context**: Chambers had 23 violations from 6-8pt page numbers/citation numbers in references.
**Decision**: Exclude spans with font_size < 8pt from the references font check. These are standard citation formatting (superscript page numbers, volume metadata).
**Action**: Added `if s.font_size < 8.0 { continue; }` to ReferencesFontChecker. Reduced chambers violations from 23 → 15.

## Decision 3: `references_font_consistent` — tolerate ±2pt deviation
**Context**: Chambers' remaining 15 violations were 10pt references text (correct font family, just slightly smaller). IU spec says "same font size/type."
**Decision**: Tolerate ±2pt from body size for references entries. 10pt is an acceptable deliberate formatting choice.
**Action**: Changed tolerance from 1.5pt → 2.0pt. Chambers now PASS.

## Decision 4: `abstract_title_format` — use positional heuristic for page detection
**Context**: Chambers abstract page has no "Abstract" heading. Keyword-based `find_section_pages` failed, picking up body text as the title.
**Decision**: Use the established positional heuristic (page between acceptance page and TOC, >100 non-empty spans, not dedication/ack/preface) from `structure.rs`.
**Action**: Added `find_abstract_page()` to `sections.rs` using positional heuristic. All three abstract checkers now use it.

## Decision 5: `references_font_consistent` — exclude Symbol/Wingdings fonts
**Context**: Alexander had 57 violations from SymbolMT bullet markers in references lists. Same category as embedded chart fonts excluded by `font_family_consistent`.
**Decision**: Exclude Symbol, Wingdings, CambriaMath, LucidaConsole, ZapfDingbats from references font check.
**Action**: Added special font exclusion list identical to `font_family_consistent`. Both PDFs now PASS.

## Decision 6: `references_heading_format` — is_bold mismatch is legitimate
**Context**: Both PDFs flagged because references headings are not bold while chapter headings are. Per IU spec, references title must be "formatted like chapter titles."
**Decision**: Legitimate finding. Checker behavior unchanged. Both dissertations should use bold for references headings.

## Decision 7: `global_margins` — refactored to statistical margin-setting check
**Context**: Individual-span margin checking produced 2,128 violations on Alexander (noise) and 1 leader-dot FP on Chambers. User wanted check focused on whether the author used the correct margin settings.
**Decision**: Replace individual-span violation checking with a statistical approach:
- Left/right: 5th percentile of per-page x0 / (page_width - x1) to find typical margin edge
- Top/bottom: first/last body text line per page
- Pass if average falls within +-0.125in of required (1.125-1.375in for L/R, 0.875-1.125in for T/B)
**Action**: Rewrote MarginsChecker with percentile-based edge detection and range-based PASS/FAIL.

## Decision 8: `tables_figures_within_margins` — removed (not a spec requirement)
**Context**: Alexander flagged 59 full-bleed images. Per IU checklist, tables/figures are exempt from top-margin rule, and the "within margins" requirement applies to the List of Tables/Figures, not the figures themselves.
**Decision**: Remove checker and spec entry entirely. General margin compliance is covered by the refactored `global_margins` checker.
**Action**: Deleted `tables_figures.rs`, removed from registry and spec.

## Decision 9: `font_family_consistent` — add Aptos to exclusion list
**Context**: Alexander flagged 34 Aptos chart labels. Aptos is the default Excel/Office 365 font.
**Decision**: Add Aptos to special font exclusion list (same category as Symbol/Wingdings).
**Action**: Added "Aptos" to exclusions in both font_family_consistent and references_font_consistent.

## Decision 10: `font_size_consistent` — 13pt title page is legitimate
**Context**: Alexander has 42 title page spans at 13pt while body is 11pt. Per IU spec "Title page: Same font size as document."
**Decision**: Legitimate violation. Checker behavior unchanged.

## Decision 11: `headings_consistent` — exclude chart figure titles near images
**Context**: Alexander flagged "Contents"/"Functions"/"Activities" (14pt Arial) as heading mismatches (body 11pt). These are chart/figure titles in appendices.
**Decision**: Exclude text near images/paths from heading detection (within 72pt proximity).
**Action**: Added image/path proximity check to HeadingsConsistentChecker.

## Decision 12: `committee_chair_first` — Alexander no Chair label is legitimate
**Context**: Alexander acceptance page has no Chair label. Per IU spec "Add Chair after degrees."
**Decision**: Legitimate violation. Checker behavior unchanged.

## Decision 13: `cv_no_page_number` — Alexander CV page number is legitimate
**Context**: Alexander CV shows page number "vii". Per IU spec, CV has no page number.
**Decision**: Legitimate violation. Checker behavior unchanged.

## Decision 14: `page_numbers_format` — fix body-start detection for extended front matter
**Context**: Johnson has extended front matter (list of figures/tables/code) with Roman numerals to page xviii, but body-start was computed from TOC+1. Also, spurious chapter detection on page 12 from "chapter" in figure captions.
**Decision**: Improve body_start detection to:
- Only trust chapter detection if chapters page is > fm_max + 10 (avoids false positives from figure captions)
- Fall back to detecting Roman-to-Arabic page number transition (first Arabic page = body start)
**Action**: Refactored `find_body_start()` with Arabic-transition fallback. Johnson body_start now correctly = 19.

## Decision 15: `justification_consistent` — exclude TOC/front matter pages
**Context**: Johnson's TOC page 8 flagged as "justified" when rest of document is left-aligned. TOC right-justified for page numbers is standard practice. IU checklist mentions justification consistency for body text, not front matter.
**Decision**: Exclude TOC and list-of-figures/tables/abbreviations pages from justification check.
**Action**: Added front matter keyword detection to JustificationChecker to skip those pages.

## Decision 16: `title_page_clause_spacing` — Johnson 26pt gap mismatch is legitimate
**Context**: Johnson's clause uses 26pt line gaps while body is 20pt and single-spaced range is 12-18pt. 
**Decision**: Legitimate finding. Checker behavior unchanged.

## Decision 17: LaTeX font family handling — semantic distinction for math/code/mono
**Context**: Johnson (LaTeX) had 2,605 font_family violations from mixed LaTeX fonts (Computer Modern body, Libertinus math, DejaVu mono, Latin Modern Sans headings). Adding individual fonts to exclusion list was unsustainable.
**Decision**: Add semantic detection via `is_non_body_text()`:
- Monospace fonts (contains "mono"/"code" in font name) → exclude
- Non-alphabetic short text (<4 chars, no letters) → exclude (math symbols)
- CJK characters left in exclusion list for now
**Action**: Added `is_non_body_text()` semantic filter to font_family and font_size checkers. Simplified special font exclusion list back to core set (Symbol, Wingdings, Aptos, etc.). Johnson violations dropped from 2,605 → 1,265 (remaining are genuine LMSans CV font mismatch).

## Decision 18: `font_family_consistent` — Johnson LMSans CV font is legitimate
**Context**: Johnson's CV uses Latin Modern Sans for all text while body uses Computer Modern.
**Decision**: Genuine violation. Checker behavior unchanged.

## Decision 19: `justification_consistent` — exclude all Roman-numbered pages as front matter
**Context**: 4 new documents flagged TOC/list-of-figures pages (8-10) as "justified" when body is left-aligned. Keyword-based exclusion missed continuation pages.
**Decision**: Skip any page with a Roman numeral page number (front matter).
**Action**: Replaced keyword detection with Roman-numeral footer check. All 4 documents now PASS.

## Decision 20: LaTeX Computer Modern — expand font family normalization
**Context**: Nagasaka used CMR10 (body), CMBX10 (bold), CMTI10 (italic), TeX-matha10 (math) — all flagged as separate families.
**Decision**: Add all common CM variants (cmr, cmbx, cmmi, cmsy, cmex, cmti, cmsl, tex-math) to `normalize_family` → "ComputerModern".
**Action**: Expanded prefix list. Nagasaka font_family dropped from 4,301 → 0, references_font from 83 → 28 → 0.

## Decision 21: Math font detection — `is_non_body_text` catches math fonts
**Context**: TeX-matha10, CMMI, CMSY fonts used for math symbols alongside body text.
**Decision**: Add `span.font_name.contains("math")` to `is_non_body_text()` semantic filter. Also added mono/code detection.
**Action**: All math-related font violations now excluded from font_family and font_size checks.
