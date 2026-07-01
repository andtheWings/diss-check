## Current State (Phase 7 complete)

- **89 Rust tests passing**, 0 build warnings
- **34 automated checkers** (16 new in Phase 7) + **5 human-review checks** + **1 final manual review**
- **Language**: Rust + pdf_oxide
- **CLI**: `cargo run -- check --spec specs/iu.yaml <pdf>` (text), `--json` for JSON
- **2 test dissertations**: 2020-12-chambers.pdf, 2025-06-alexander.pdf
- **Branch**: `rust-rewrite`

### Phase 7 accomplishments
Promoted 10 human-review checks → 16 automated + 5 remain human (spacing/credentials):
- `title_page_all_caps`, `title_page_clause_centered`, `title_page_clause_spacing`
- `copyright_page_format`, `footnotes_font_consistent`
- `tables_figures_within_margins`
- `references_font_consistent`, `references_heading_format`
- `cv_heading_format`, `cv_name_position`
- `abstract_text_centered`, `abstract_word_count`, `abstract_title_format`
- `toc_page_numbers_aligned`, `toc_no_overhang`, `toc_cv_no_dots`

### Extractor enhancements
- `TextSpan` now carries `color: Option<(f32,f32,f32)>` from pdf_oxide
- `Page` now carries `images: Vec<bbox>` and `paths: Vec<bbox>` from `extract_images`/`extract_paths`

### Next: Phase 8 — Large-corpus calibration