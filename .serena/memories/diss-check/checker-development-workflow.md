## Checker Development & Validation Workflow (Rust)

### Codebase
- Rust project on `rust-rewrite` branch
- Source in `src/checkers/` — files grouped by domain (layout, typography, structure, content, title_page, sections, footnotes, optional_pages, toc_details)
- Trait: `Checker` with `category()`, `name()`, `check(doc, params) -> CheckResult`
- Registration: factory function + `REGISTRY` entry in `src/checkers/mod.rs`

### When to check with user
Only pause for user feedback between rounds if:
- Confidence in the solution is < 8/10
- There are diverging decision pathways you want feedback on

Otherwise proceed autonomously — implement, test, validate, and move to the next round.

### Round lifecycle
1. **Design** based on `tests/corpus/` documents
2. **Implement** the checker in `src/checkers/<module>.rs`
3. **Register** in `src/checkers/mod.rs` — add factory function + `REGISTRY` entry
4. **Validate** against multiple test PDFs in `tests/corpus/`
5. **Add check** to `specs/iu.yaml`
6. **Write tests** inline in `src/checkers/<module>.rs` (`#[cfg(test)] mod tests`)
7. **Update** `docs/roadmap/roadmap.md` — mark round ✅

### PDF extraction details (pdf_oxide)
- `extract_chars()` → per-character: font_name, font_size, is_bold, is_italic, color, bbox, origin_x/y
- Characters grouped into word-level TextSpans by proximity + font matching
- Word bboxes are union of constituent char bboxes
- `extract_images()` and `extract_paths()` → image/path bboxes on each page
- Color is forwarded as `Some((r, g, b))` on each TextSpan

### Common test helpers (inline)
- `fn span(text, top) -> TextSpan` — default x-position
- `fn span_x(text, top, x0, x1) -> TextSpan` — custom x-bounds
- Bbox format: `(top, bottom, x0, x1)` — origin at page top-left
- All test spans need `color: None, images: vec![], paths: vec![]`

### Build & test commands
- `cargo build --release` — 0 warnings
- `cargo test` — 83 tests
- `cargo clippy -- -D warnings` — lint check
- `cargo fmt --check` — format check
- `cargo run -- check --spec specs/iu.yaml <pdf>` — run checker
- `cargo run -- check --spec specs/iu.yaml <pdf> --json --quiet` — JSON, failures only
- `cargo run -- check --spec specs/iu.yaml --check global_margins <pdf>` — single check
