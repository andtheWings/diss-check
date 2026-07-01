# Phase 7 — Manual Check Automation Assessment (Archived)

> Promoted 10 human-review checks to 16 automated checkers. 5 remain human (spacing/credentials).

### Audit (7.1)

10 `checker: review` entries assessed against pdf_oxide capabilities (text, bbox, font_name, font_size, is_bold, is_italic, color, extract_images, extract_paths). Produced feasibility matrix showing 7 fully automatable, 2 partially automatable, 1 must stay manual.

### Checkers implemented

| Round | Checker | Category | Notes |
|-------|---------|----------|-------|
| 7.2 | `title_page_all_caps` | typography | First text block on page 1 = title. Check all-caps. |
| 7.3a | `title_page_clause_centered` | typography | Clause spans horizontally centered (36pt tolerance) |
| 7.3b | `title_page_clause_spacing` | typography | Line spacing single-spaced OR matches body |
| 7.4 | `copyright_page_format` | content | © + year + name centered. PASS if no copyright page |
| 7.5a | `footnotes_font_consistent` | typography | Footnote text font/size near page bottom |
| 7.6a | `tables_figures_within_margins` | layout | Image bboxes vs margins — later removed (Decision 8) |
| 7.7a | `references_font_consistent` | typography | Font family + size on references pages |
| 7.7b | `references_heading_format` | structure | References heading matches chapter style |
| 7.8a | `cv_heading_format` | structure | CV heading matches chapter style |
| 7.8b | `cv_name_position` | structure | Name centered/left on CV page |
| 7.9a | `abstract_text_centered` | typography | Student name + title centered |
| 7.9b | `abstract_word_count` | content | Abstract ≤ 350 words |
| 7.9c | `abstract_title_format` | typography | Title all-caps or title case |
| 7.10a | `toc_page_numbers_aligned` | structure | Page numbers at consistent x-position |
| 7.10b | `toc_no_overhang` | structure | No entries overhang page number column |
| 7.10c | `toc_cv_no_dots` | structure | CV entry no leader dots or page number |

### Remaining human checks (5)
`clause_spacing`, `footnotes_spacing`, `references_spacing`, `cv_no_credentials`, `toc_spacing`, `tables_figures_legend_font`, `final_manual_review`

### Extractor changes
- `TextSpan` gained `color: Option<(f32,f32,f32)>`
- `Page` gained `images` and `paths` bbox vecs from pdf_oxide
