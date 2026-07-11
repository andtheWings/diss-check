# Key Types and Functions (Rust)

## Document model (`src/document.rs`)
- `TextSpan { text, font_name, font_size, bbox: (top,bottom,x0,x1), is_bold, is_italic, color: Option<(f32,f32,f32)> }`
- `Page { page_number, width, height, spans, images, paths }`
- `Document { pages: Vec<Page> }`

## Checker types (`src/checkers/mod.rs`)
- `Checker` trait: `category()`, `name()`, `check(doc, params) -> CheckResult`
- `CheckResult { check_id, status: Status, evidence: Vec<EvidenceItem>, detail }`
- `Status`: Pass, Fail, Manual, Error
- `EvidenceItem { page, bbox, excerpt }`

## Spec model (`src/spec.rs`)
- `InstitutionSpec { institution, source_revision, document_structure, checks, constants }`
- `CheckDef { id, category, checker, target, params, automatable, review_hint }`
- `load_spec(path: &Path) -> Result<InstitutionSpec>` — spec files in catalog

## Engine (`src/engine.rs`)
- `CheckOptions { check_id, category }` — derives Default
- `run_checks(spec, pdf_path, options) -> Result<Vec<CheckResult>>`

## Font helpers (`src/checkers/typography.rs`)
- `normalize_family(font_name) -> String` — strips `+` prefix, PS/MT/Identity-H/Book suffixes
- `is_internal_font_name(name) -> bool` — detects TT0-style names
- `is_non_body_text(span) -> bool` — math/mono/code font detection
- `is_near_image(page, span) -> bool` — span overlaps image/path bbox
- `parse_measurement("1in") -> 72.0`, `parse_measurement("12pt") -> 12.0`

## Structure helpers (`src/checkers/structure.rs`)
- `find_all_sections(doc) -> HashMap<String, usize>` — title, acceptance, abstract, toc, chapters, references, CV
- `find_body_start(doc, sections) -> usize` — chapters > Arabic transition > fm_max+1
- `page_text(page) -> String`, `page_text_no_citations(page) -> String`

## Content helpers (`src/checkers/content.rs`)
- `page_lines(page) -> Vec<String>` — spans grouped by top into text lines
- `match_template(template_lines, page_lines) -> usize` — fuzzy match count
- `extract_toc_entries(page) -> Vec<(String, usize)>` — parses TOC leader dots

## Test pattern
```rust
fn span_x(text, top, x0, x1) -> TextSpan {
    TextSpan { text, font_name: "Times".into(), font_size: 12.0,
        bbox: (top, top+12.0, x0, x1), is_bold: false, is_italic: false, color: None }
}
```
