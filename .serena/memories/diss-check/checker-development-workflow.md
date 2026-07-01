## Checker Development & Validation Workflow (Rust)

### Codebase
- Rust project on `rust-rewrite` branch
- Source in `src/checkers/` — each file contains related checkers
- Trait: `Checker` with `category()`, `name()`, `check(doc, params) -> CheckResult`
- Registration: factory functions + `REGISTRY` HashMap in `src/checkers/mod.rs`

### When to check with user
Only pause for user feedback between rounds if:
- Confidence in the solution is < 8/10
- There are diverging decision pathways you want feedback on

Otherwise proceed autonomously — implement, test, validate, and move to the next round.

### Round lifecycle
1. **Design** based on `tests/fixtures/2020-12-chambers.pdf`
2. **Implement** the checker in `src/checkers/<module>.rs`
3. **Register** in `src/checkers/mod.rs` — add factory + `REGISTRY` entry
4. **Validate** against `tests/fixtures/2025-06-alexander.pdf`
5. **Add check** to `specs/iu.yaml`
6. **Write tests** in `src/checkers/<module>.rs` (inline `#[cfg(test)] mod tests`)
7. **Update** `docs/roadmap/roadmap.md` — mark round ✅

### PDF extraction details (pdf_oxide)
- `extract_chars()` gives per-character data: font_name, font_size, is_bold, is_italic, color, bbox
- Characters are grouped into word-level TextSpans by proximity (gap threshold)
- Word bboxes are union of constituent char bboxes
- `extract_images()` and `extract_paths()` give image/path bboxes on each page

### Common test helpers (inline `#[cfg(test)]`)
- `fn span(text, top) -> TextSpan` — basic span at default x-position
- `fn span_x(text, top, x0, x1) -> TextSpan` — span with custom x-bounds
- Bbox format: `(top, bottom, x0, x1)` — origin at top-left
- Font filtering: internal names like TT0/TT1 are resolved to real names

### Build & test commands
- `cargo build --release` — full build (should be 0 warnings)
- `cargo test` — run all tests
- `cargo run -- check --spec specs/iu.yaml <pdf>` — run checker
- `cargo run -- check --spec specs/iu.yaml <pdf> --json` — JSON output
