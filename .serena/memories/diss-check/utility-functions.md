## Key Types and Functions (Rust)

### Document model (`src/document.rs`)
- `TextSpan { text, font_name, font_size, is_bold, is_italic, color: Option<Color>, bbox: (f64,f64,f64,f64) }`
  - Bbox: `(x0, top, x1, bottom)` — matches pdf_oxide convention
- `Page { page_number: usize, width: f64, height: f64, spans: Vec<TextSpan> }`
- `Document { pages: Vec<Page> }`
- `ExtractionContext { document: Document }`

### Checker types (`src/checkers/mod.rs`)
- `Checker` trait: `name() -> &'static str`, `check(ctx, params) -> CheckResult`
- `CheckResult { status: CheckStatus, evidence: Vec<EvidenceItem>, summary }`
- `CheckStatus`: Pass, Fail, Manual, Error
- `EvidenceItem { page, bbox, text, message }`

### Spec model (`src/spec.rs`)
- `InstitutionSpec { name, checks: Vec<CheckDefinition>, margin_spec, font_spec, ... }`
- `CheckDefinition { id, name, description, checker, params, automatable }`
- `load_spec(path) -> InstitutionSpec`

### Measurement parsing
- `parse_measurement("1in") -> 72.0`, `parse_measurement("12pt") -> 12.0`

### Font helpers
- `is_bold(font_name) -> bool`, `is_italic(font_name) -> bool`
- `normalize_family(font_name) -> String` — strips prefix, PS/MT suffixes, style variants

### Build & test
- `cargo build --release` — 0 warnings
- `cargo test` — 59 tests
- `cargo run -- --spec specs/iu.yaml <pdf>`
- `cargo run -- --spec specs/iu.yaml <pdf> --json`
