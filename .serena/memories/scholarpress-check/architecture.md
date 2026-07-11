# scholarpress-check — Architecture (Rust)

Library crate for PDF dissertation formatting validation. Originally `diss-check`; renamed in Phase 3 of the ScholarPress ecosystem migration. **Library-only** — binary entry point lives in `scholarpress-cli`.

Part of the ecosystem DAG: `catalog → check → publish`. `cli` binds all modules.

## Module structure

```
src/
  lib.rs               # Public module declarations
  engine.rs            # CheckOptions, run_checks() — filtering by ID/category
  spec.rs              # InstitutionSpec, CheckDef, load_spec(path) from YAML
  document.rs          # TextSpan, Page (with images/paths), Document
  extractor.rs         # pdf_oxide extraction: chars → word-level TextSpans
  report.rs            # build_report(), format_text(), format_text_quiet(), format_json()
  calibration.rs       # Calibration workflow: systemic vs isolated, text+JSON output
  checkers/
    mod.rs             # Checker trait, Status, EvidenceItem, CheckResult, REGISTRY (36 checkers)
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
pub trait Checker: Send + Sync { ... }
```
Registered via factory functions + `REGISTRY: LazyLock<HashMap<(String,String), CheckerFactory>>`.

### Document model
- `TextSpan { text, font_name, font_size, is_bold, is_italic, color, bbox: (top,bottom,x0,x1) }`
- `Page { page_number, width, height, spans, images, paths }`
- `Document { pages: Vec<Page> }`

### Engine flow
1. Load spec YAML → InstitutionSpec
2. pdf_oxide extraction → Document
3. For each check (filtered by CheckOptions): checker.check(doc, params) → CheckResult
4. Non-automatable checks return MANUAL directly

## Institution data

Institution specs and test fixtures live in `scholarpress-catalog`. Consumed via:
- **Development:** `CATALOG_PATH` env var or `../scholarpress-catalog/` sibling-directory fallback
- **Production:** `rust-embed` bakes spec/template files into the binary (cli concern)
- **Library:** Accepts `&Path` to a spec file — agnostic about how data is sourced

## Build & verify
- `cargo build --release` — 0 clippy warnings
- `cargo test` — 83 tests
- `cargo clippy -- -D warnings`
- `cargo fmt --check`
- `rtk cargo test` — run tests with output filtering

## Reference graph
- `mem:scholarpress-check/project-status` — current state, version
- `mem:scholarpress-check/checker-development-workflow` — how to add checkers
- `mem:scholarpress-check/checker-validation-workflow` — calibration process
- `mem:scholarpress-check/utility-functions` — helper API reference
- `mem:scholarpress-catalog/` — catalog (institution data, test fixtures, templates)
- `mem:scholarpress-cli/` — CLI wrapper
