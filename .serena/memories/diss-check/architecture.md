## Architecture (Rust)

```
src/
  main.rs           # CLI entrypoint (clap): --spec + PDF argument, --json flag
  lib.rs            # Library root
  spec.rs           # InstitutionSpec, CheckDefinition, load_spec() from YAML
  engine.rs         # Engine: runs extractor, instantiates checkers, runs checks
  document.rs       # TextSpan, Page, Document models
  extractor.rs      # pdf_oxide extractor: groups chars into word-level TextSpans
  report.rs         # Report, format_text(), format_json()
  calibration.rs    # Calibration workflow: systemic vs isolated classification
  checkers/
    mod.rs          # Checker trait, CheckResult, EvidenceItem, checker registry
    margins.rs      # MarginsChecker, MarginSymmetryChecker
    typography.rs   # FontSizeChecker, FontWeightChecker, FontFamilyChecker, JustificationChecker
    structure.rs    # SectionPresenceChecker, SectionOrderChecker
    content.rs      # BoilerplateMatchChecker, CommitteeOrderChecker, TocTitleParityChecker
    human.rs        # HumanReviewChecker (always returns MANUAL)
    page_numbers.rs # TitlePageNoPageNumber, AcceptancePageNumber, PageNumbersFormat
    headings.rs     # HeadingsConsistentChecker, NewChaptersNewPages
    hyperlinks.rs   # HyperlinksFormatChecker
    cv.rs           # CvNoPageNumberChecker
```

## Key patterns

### Checker trait
```rust
pub trait Checker {
    fn name(&self) -> &'static str;
    fn check(&self, ctx: &ExtractionContext, params: &CheckParams) -> CheckResult;
}
```
Checkers are registered in a HashMap via `register_checkers()`.

### Document model (rust)
- `TextSpan { text, font_name, font_size, is_bold, is_italic, color, bbox }`
- `Page { page_number, width, height, spans }`
- `Document { pages }`
- `ExtractionContext { document }`

### Test pattern
```rust
fn make_doc(spans_by_page: Vec<Vec<((f64,f64,f64,f64), &str, f64)>>) -> Document {
    // Each element = ((x0, top, x1, bottom), text, font_size)
    // One inner Vec per page
}
```

### Engine flow
1. Load spec YAML → InstitutionSpec
2. Run pdf_oxide extractor → populate ExtractionContext
3. For each check_def: get checker instance, call checker.check(ctx, params)
4. Non-automatable checks (check_def.automatable == false) return MANUAL directly

## Build & test commands
- Build: `cargo build --release`
- Test: `cargo test`
- Run: `cargo run -- --spec specs/iu.yaml <pdf>`
