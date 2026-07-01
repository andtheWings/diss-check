# Phase 4 — Rust Rewrite (Archived)

> pdf_oxide — 44-134x faster than Python.

| Round | Feature | Status | Notes |
|-------|---------|--------|-------|
| R1 | Rust scaffold + engine + pdf_oxide extractor | ✅ | Cargo project, Document IR, spec models, engine, CLI (clap), report |
| R2 | margins checker (Rust) | ✅ | 9 clean violations vs Python's 56 noisy. Word-level span splitting from pdf_oxide |
| R3 | margin_symmetry checker (Rust, new) | ✅ | Per-page L/R margin comparison. 62/202 pages asymmetric on chambers |
| R4 | font_size checker (Rust) | ✅ | PASS with 0 violations on chambers (matches Python). 44x speedup |
| R5 | font_weight checker (Rust) | ✅ | Uses pdf_oxide is_bold/is_italic directly. Both pass on chambers |
| R6 | font_family checker (Rust) | ✅ | Matches Python (PASS). Filters internal font names (TT0) when unresolved |
| R7 | justification checker (Rust) | ✅ | Matches Python (PASS, left-aligned). 197 vs 198 pages due to span count differences |
| R8 | section_presence + section_order checkers (Rust) | ✅ | Both match Python perfectly on chambers |
| R9 | content + human checkers (Rust) | ✅ | All 4 content checkers match Python. toc_title_parity now PASS |
| R10 | calibration + optimizations (Rust) | ⬜ | Port calibration workflow, clean up warnings |
