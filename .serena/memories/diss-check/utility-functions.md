## Key Types and Functions (Rust)

### Document model (`src/document.rs`)
- `TextSpan { text: String, font_name: String, font_size: f32, bbox: (f32,f32,f32,f32), is_bold: bool, is_italic: bool, color: Option<(f32,f32,f32)> }`
  - bbox: `(top, bottom, x0, x1)` — origin at page top-left
- `Page { page_number: usize, width: f32, height: f32, spans: Vec<TextSpan>, images: Vec<(f32,f32,f32,f32)>, paths: Vec<(f32,f32,f32,f32)> }`
  - images/paths: bboxes in same coord system as text spans
- `Document { pages: Vec<Page> }`

### Checker types (`src/checkers/mod.rs`)
- `Checker` trait: `category() -> &'static str`, `name() -> &'static str`, `check(doc: &Document, params: &Value) -> CheckResult`
- `CheckResult { check_id: String, status: Status, evidence: Vec<EvidenceItem>, detail: String }`
- `Status`: Pass, Fail, Manual, Error
- `EvidenceItem { page: usize, bbox: Option<(f32,f32,f32,f32)>, excerpt: Option<String> }`

### Spec model (`src/spec.rs`)
- `InstitutionSpec { institution, source_revision, document_structure, checks, constants }`
- `CheckDef { id, category, checker, target, params, automatable, review_hint }`
- `load_spec(path: &Path) -> Result<InstitutionSpec>`

### Engine (`src/engine.rs`)
- `CheckOptions { check_id: Option<String>, category: Option<String> }` — derives Default
- `run_checks(spec, pdf_path, options) -> Result<Vec<CheckResult>>`

### Font helpers (`src/checkers/typography.rs`)
- `normalize_family(font_name) -> String` — strips `+` prefix, PS/MT/Identity-H/Book suffixes, maps CM/NewCM to "ComputerModern"
- `is_internal_font_name(name) -> bool` — detects TT0-style internal names
- `is_non_body_text(span) -> bool` — math/mono/code font semantic detection
- `is_near_image(page, span) -> bool` — span overlaps image/path bbox
- `parse_measurement("1in") -> 72.0`, `parse_measurement("12pt") -> 12.0`

### Structure helpers (`src/checkers/structure.rs`)
- `find_all_sections(doc) -> HashMap<String, usize>` — detects title, acceptance, abstract, toc, chapters, references, CV
- `find_body_start(doc, sections) -> usize` — chapters > Arabic transition > fm_max+1
- `page_text(page) -> String`, `page_text_no_citations(page) -> String`

### Content helpers (`src/checkers/content.rs`)
- `page_lines(page) -> Vec<String>` — groups spans by top into text lines
- `match_template(template_lines, page_lines) -> usize` — fuzzy match count
- `extract_toc_entries(page) -> Vec<(String, usize)>` — parses TOC leader dots

### Test pattern
```rust
fn span_x(text, top, x0, x1) -> TextSpan {
    TextSpan { text: text.into(), font_name: "Times".into(), font_size: 12.0,
        bbox: (top, top+12.0, x0, x1), is_bold: false, is_italic: false, color: None }
}
```
