## Architecture (Rust)

```
src/
  main.rs              # CLI (clap): check + calibrate subcommands, --quiet, --check, --category
  lib.rs               # Library root
  spec.rs              # InstitutionSpec, CheckDefinition, load_spec() from YAML
  engine.rs            # Engine: CheckOptions, run_checks() with ID/category filters
  document.rs          # TextSpan, Page (with images/paths), Document
  extractor.rs         # pdf_oxide: extract_chars → word-level TextSpans + images/paths
  report.rs            # build_report(), format_text(), format_text_quiet(), format_json()
  calibration.rs       # Calibration workflow: systemic vs isolated, text+JSON output
  checkers/
    mod.rs             # Checker trait, Status, EvidenceItem, CheckResult, REGISTRY
    layout.rs          # MarginsChecker (percentile-based), MarginSymmetryChecker
    typography.rs      # FontSizeChecker, FontWeightChecker, FontFamilyChecker, JustificationChecker
    structure.rs       # SectionPresence, SectionOrder, page numbers, headings, chapters, CV, hyperlinks
    content.rs         # BoilerplateMatch (fuzzy 70%), CommitteeOrder, TocTitleParity, HumanReview
    title_page.rs      # TitlePageAllCaps, TitlePageClauseCentered, TitlePageClauseSpacing
    optional_pages.rs  # CopyrightPageFormat
    footnotes.rs       # FootnotesFontChecker
    sections.rs        # ReferencesFont, ReferencesHeading, CvHeading, CvNamePosition, abstract checkers
    toc_details.rs     # TocPageNumbersAligned, TocNoOverhang, TocCvNoDots
```

## Key types

### Checker trait
```rust
pub trait Checker: Send + Sync {
    fn category(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn check(&self, doc: &Document, params: &serde_yaml::Value) -> CheckResult;
}
```
Registered via factory functions + `REGISTRY: LazyLock<HashMap<(String,String), CheckerFactory>>`.

### Document model
- `TextSpan { text, font_name, font_size, is_bold, is_italic, color: Option<(f32,f32,f32)>, bbox: (f32,f32,f32,f32) }`
  - bbox: `(top, bottom, x0, x1)` — origin at page top-left
- `Page { page_number, width, height, spans, images: Vec<bbox>, paths: Vec<bbox> }`
- `Document { pages: Vec<Page> }`

### Engine flow
1. Load spec YAML → InstitutionSpec
2. Run pdf_oxide extractor → Document (with spans, images, paths)
3. For each check_def (filtered by CheckOptions): get checker, call checker.check(doc, params)
4. Non-automatable checks (automatable=false) return MANUAL directly

### Build & test
- `cargo build --release` — 0 clippy warnings
- `cargo test` — 83 tests
- `cargo clippy -- -D warnings` — lint
- `cargo fmt --check` — format check
- `cargo run -- check --spec specs/iu.yaml <pdf>`
