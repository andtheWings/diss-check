# Project Roadmap

> Updated: 2026-06-30
> Current round: font_size checker
> Test document: [2020-12-chambers.pdf](../tests/fixtures/2020-12-chambers.pdf)

## Phases

### Phase 1 — MVP (end-to-end pipeline with 1 checker)

| Round | Feature | Status | Spec | Plan | Tests | Notes |
|---|---|---|---|---|---|---|---|
| 1 | Project scaffold | ✅ | — | [plan](../docs/superpowers/plans/2026-06-29-diss-check-plan.md) | — | |
| 2 | Spec models + IU spec | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | [plan](../docs/superpowers/plans/2026-06-29-diss-check-plan.md) | 2 | |
| 3 | Document IR (TextSpan/Page/Document) | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | [plan](../docs/superpowers/plans/2026-06-29-diss-check-plan.md) | — | |
| 4 | Pdfplumber extractor | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | [plan](../docs/superpowers/plans/2026-06-29-diss-check-plan.md) | — | |
| 5 | Engine + checker registry | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | [plan](../docs/superpowers/plans/2026-06-29-diss-check-plan.md) | — | |
| 6 | Margins checker | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | [plan](../docs/superpowers/plans/2026-06-29-diss-check-plan.md) | 6 | Word-level bboxes, 0.25in tolerance, page-number zone exclusion. 33 violations on test doc (all TOC leader dots). |
| 7 | CLI + report (text) | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | [plan](../docs/superpowers/plans/2026-06-29-diss-check-plan.md) | 2 | |
| 8 | Integration test | ✅ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | [plan](../docs/superpowers/plans/2026-06-29-diss-check-plan.md) | 1 | |

### Phase 2 — Typography, structure, and content checkers

| Round | Feature | Status | Spec | Plan | Tests | Notes |
|---|---|---|---|---|---|---|---|
| 9 | font_size checker | 🚧 | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | [plan](../docs/superpowers/plans/2026-06-29-diss-check-plan.md) | — | |
| 10 | font_weight checker | ⬜ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | — | — | |
| 11 | font_family checker | ⬜ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | — | — | |
| 12 | justification checker | ⬜ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | — | — | |
| 13 | section_presence checker | ⬜ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | — | — | |
| 14 | section_order checker | ⬜ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | — | — | |
| 15 | text_match checker | ⬜ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | — | — | |
| 16 | committee_order checker | ⬜ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | — | — | |
| 17 | toc_title_parity checker | ⬜ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | — | — | |
| 18 | human checker (manual passthrough) | ⬜ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | — | — | |
| 19 | JSON report output | ⬜ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | — | — | |
| 20 | Expand IU spec to full checklist | ⬜ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | — | — | |

### Phase 3 — Calibration

| Round | Feature | Status | Spec | Plan | Tests | Notes |
|---|---|---|---|---|---|---|
| 21 | Calibration workflow | ⬜ | [design](../docs/superpowers/specs/2026-06-29-diss-check-design.md) | — | — | |
| 22 | veraPDF extractor (if needed) | ⬜ | — | — | — | |
