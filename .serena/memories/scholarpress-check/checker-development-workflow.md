# Checker Development & Validation (Rust)

## Codebase
- Library crate (`scholarpress-check`), compiled as part of `scholarpress-cli`
- Source: `src/checkers/` — grouped by domain
- Trait: `Checker { category(), name(), check(doc, params) -> CheckResult }`
- Registration: factory function + `REGISTRY` entry in `src/checkers/mod.rs`

## Institution data
Spec YAML and test fixtures live in `scholarpress-catalog`. Development workflow:
- Set `CATALOG_PATH` to `../scholarpress-catalog` or use fallback
- Corpus PDFs: `{CATALOG_PATH}/institutions/iu/tests/corpus/`
- Synthetic fixtures: `{CATALOG_PATH}/institutions/iu/tests/fixtures/`

## When to check with user
Only pause between rounds if confidence < 8/10 or there are diverging pathways.

## Round lifecycle
1. **Design** based on corpus documents in catalog
2. **Implement** in `src/checkers/<module>.rs`
3. **Register** in `src/checkers/mod.rs` — factory + REGISTRY entry
4. **Validate** against multiple catalog corpus PDFs
5. **Add check** to catalog's `institutions/iu/spec.yaml`
6. **Write tests** inline (`#[cfg(test)] mod tests`)
7. **Update** `docs/roadmap/roadmap.md` — mark done

## PDF extraction (pdf_oxide)
- `extract_chars()` → per-character: font_name, font_size, is_bold, is_italic, color, bbox
- Characters → word-level TextSpans by proximity + font matching
- Word bbox = union of constituent char bboxes
- `extract_images()` / `extract_paths()` → image/path bboxes per page

## Common test helpers
- `fn span(text, top) -> TextSpan` — default x-position
- `fn span_x(text, top, x0, x1) -> TextSpan` — custom x-bounds
- Bbox: `(top, bottom, x0, x1)` — origin at page top-left
- Test spans need `color: None`

## Build & test
- `cargo build --release` — 0 warnings
- `cargo test` — 83 tests
- `cargo clippy -- -D warnings`
- `cargo fmt --check`
- Integration: `scholarpress check --spec {CATALOG_PATH}/institutions/iu/spec.yaml <pdf>`
