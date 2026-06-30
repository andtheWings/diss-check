# Project Roadmap

> Updated: 2026-06-30
> Current round: font_weight checker (Rust)
> Test document: [2020-12-chambers.pdf](../tests/fixtures/2020-12-chambers.pdf)
> Branch: `rust-rewrite` — Rust + pdf_oxide rewrite (44x faster than Python)

## Phases

### Phase 1 — MVP (end-to-end pipeline with 1 checker)

| Round | Feature | Status | Spec | Plan | Tests | Notes |
|-------|---------|--------|------|------|-------|-------|
| 1 | Project scaffold | ✅ | — | [plan](../docs/superpowers/plans/2026-06-29-diss-check-plan.md) | — | |
| 2 | Spec models + IU spec | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | [plan](../docs/superpowers/plans/2026-06-29-diss-check-plan.md) | 2 | |
| 3 | Document IR (TextSpan/Page/Document) | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | [plan](../docs/superpowers/plans/2026-06-29-diss-check-plan.md) | — | |
| 4 | Pdfplumber extractor | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | [plan](../docs/superpowers/plans/2026-06-29-diss-check-plan.md) | — | |
| 5 | Engine + checker registry | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | [plan](../docs/superpowers/plans/2026-06-29-diss-check-plan.md) | — | |
| 6 | Margins checker | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | [plan](../docs/superpowers/plans/2026-06-29-diss-check-plan.md) | 6 | Word-level bboxes, 0.25in tolerance, page-number zone exclusion. 33 violations on test doc (all TOC leader dots) |
| 7 | CLI + report (text) | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | [plan](../docs/superpowers/plans/2026-06-29-diss-check-plan.md) | 2 | |
| 8 | Integration test | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | [plan](../docs/superpowers/plans/2026-06-29-diss-check-plan.md) | 1 | |

### Phase 2 — Typography, structure, and content checkers

| Round | Feature | Status | Spec | Plan | Tests | Notes |
|-------|---------|--------|------|------|-------|-------|
| 9 | font_size checker | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | [plan](../docs/superpowers/plans/2026-06-29-diss-check-plan.md) | 10 | Known: < 8.5pt formatting artifacts filtered; figure content vs legend distinction not made (< 10pt figure content flagged as violation when it may be legible per IU template) |
| 10 | font_weight checker | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | — | 11 | Detects bold/italic/bold-italic from font name. Supports page filter + invert mode |
| 11 | font_family checker | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | — | 8 | Known: chart/graph embedded fonts (e.g. Aptos on imported Excel charts) flagged as mismatches; special fonts (Symbol, Wingdings) excluded |
| 12 | justification checker | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | — | 2 | Right-edge stdev heuristic (sd < 8 = justified). Pages < 50 spans excluded as figures. Known FN: mixed justification within page, tight left-alignment misclassified as justified |
| 13 | section_presence checker | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | — | 9 | Keyword-based detection. Promoted to automatable. Known FN: non-English docs, scanned PDFs, non-standard section naming |
| 14 | section_order checker | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | — | 7 | TOC + heuristic. Abstract detected by backwards scan from TOC (>100 spans, exclude dedication/ack headings). docling explored but too slow (4 min/PDF) |
| 15 | text_match -> boilerplate_match checker | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | — | 7 | Renamed. Verifies template text with {variable} substitution matches on target page. Tolerates extra lines, multiline vars, trailing punctuation |
| 16 | committee_order checker | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | — | 4 | FAILs when Chair label missing (IU requirement: "Add Chair after degrees") or chair not first |
| 17 | toc_title_parity checker | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | — | 2 | Known: PDF glyph reordering issues (e.g. chambers Ch3 "TakesDevelopment" concat + overlap). Alexander 4/4 clean. |
| 18 | human checker (manual passthrough) | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | — | 2 | Always returns MANUAL with configurable prompt. Enables explicit manual-review checks in spec. |
| 19 | JSON report output | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | — | 1 | `--json` flag, Pydantic model_dump_json, bbox tuples serialize as arrays |
| 20 | Expand IU spec to full checklist | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | — | — | 27 checks: 10 automated + 17 human review. Covers all sections of IU format review checklist (rev Sept 2025) |

### Phase 3 — Calibration

| Round | Feature | Status | Spec | Plan | Tests | Notes |
|-------|---------|--------|------|------|-------|-------|
| 21 | Calibration workflow | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | — | 8 | Corpus aggregation, systemic vs isolated classification, text+JSON output. Revealed alexander has 2,579 margin violations vs chambers' 56 — genuine left-margin discrepancies (3.2% of alexander spans below threshold) |
| 22 | veraPDF extractor (if needed) | ⬜ | — | — | — | |

### Phase 4 — Rust rewrite (pdf_oxide, 44x faster)

| Round | Feature | Status | Notes |
|-------|---------|--------|-------|
| R1 | Rust scaffold + engine + pdf_oxide extractor | ✅ | Cargo project, Document IR, spec models, engine, CLI (clap), report |
| R2 | margins checker (Rust) | ✅ | 9 clean violations vs Python's 56 noisy. Word-level span splitting from pdf_oxide |
| R3 | margin_symmetry checker (Rust, new) | ✅ | Per-page L/R margin comparison. 62/202 pages asymmetric on chambers |
| R4 | font_size checker (Rust) | ✅ | PASS with 0 violations on chambers (matches Python). 44x speedup |
| R5 | font_weight checker (Rust) | ⬜ | Next round |
