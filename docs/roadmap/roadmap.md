# Project Roadmap

> Updated: 2026-07-01
> Current phase: complete (all phases through 10)
> Branch: `rust-rewrite` — Rust + pdf_oxide (134x faster than Python)
> Test corpus: 16 documents

## Status

**33 automated checkers** + **7 human-review checks** — 83 tests passing, 0 clippy warnings.

---

## Phase History

| Phase | Description | Rounds | Archive |
|-------|-------------|--------|---------|
| 1 | MVP — end-to-end pipeline with 1 checker (Python/pdfplumber) | 1–8 | [archive_phase1_mvp.md](archive_phase1_mvp.md) |
| 2 | Typography, structure, and content checkers (Python) | 9–20 | [archive_phase2_checkers.md](archive_phase2_checkers.md) |
| 3 | Calibration workflow (Python) | 21–22 | [archive_phase3_calibration.md](archive_phase3_calibration.md) |
| 4 | Rust rewrite — engine + all checkers ported (pdf_oxide) | R1–R10 | [archive_phase4_rust_rewrite.md](archive_phase4_rust_rewrite.md) |
| 5 | Promote 7 manual checks to automated (pdf_oxide capabilities) | R11–R18 | [archive_phase5_promote_manual.md](archive_phase5_promote_manual.md) |
| 6 | Cleanup & hardening | R19–R23 | [archive_phase6_cleanup.md](archive_phase6_cleanup.md) |
| 7 | Manual check automation assessment (10 → 16 checked) | 7.1–7.12 | [archive_phase7_manual_automation.md](archive_phase7_manual_automation.md) |

---

## Phase 8 — Large-corpus calibration

> **Goal**: Run checkers over expanding corpus, HITL review, tune thresholds, document findings. **22 calibration decisions logged.**

| Round | Feature | Status | Notes |
|-------|---------|--------|-------|
| 8.1 | Gather calibration corpus | ✅ | 16 documents over 5 rounds (chambers → wang) |
| 8.2 | Run full check suite on corpus | ✅ | 5 calibration rounds, systemic vs isolated classification |
| 8.3–8.7 | Human review: per-category | ✅ | 22 decisions logged in `docs/calibration-decisions.md` |
| 8.8 | Threshold tuning + heuristic refinement | ✅ | Margin refactor (percentile-based), semantic font detection, body_start fallback, justification front-matter exclusion, fuzzy boilerplate matching, Computer Modern normalization |
| 8.9 | Calibration report | ✅ | `docs/calibration-decisions.md` — 22 decisions across 5 rounds |

---

## Phase 9 — CLI & report UX review

| Round | Feature | Status | Notes |
|-------|---------|--------|-------|
| 9.1 | CLI audit | ✅ | `--version`, improved help text, proper exit codes (0=PASS, 1=FAIL/ERROR, 2=usage error) |
| 9.2 | `--check` / `--category` filter | ✅ | `--check <id>` runs single check, `--category <name>` filters by category |
| 9.3 | `--quiet` flag | ✅ | Shows only FAIL/ERROR + summary |
| 9.4 | Text report format review | ✅ | Clean format with per-check detail |
| 9.5 | JSON report schema | ✅ | Stable schema with check_id, status, detail, evidence |
| 9.6 | Diff-friendly output | ⬜ | Not implemented — JSON output supports external diff tools |
| 9.7 | Multiple PDF batch mode | ⬜ | Calibration command covers batch needs |
| 9.8 | Exit codes | ✅ | 0 = all pass, 1 = failures found, 2 = usage error |
| 9.9 | Documentation | ✅ | README with installation, usage, checker catalog, architecture |

---

## Phase 10 — Initial formal release prep

| Round | Feature | Status | Notes |
|-------|---------|--------|-------|
| 10.1 | Version + CHANGELOG | ✅ | v0.1.0 in Cargo.toml, CHANGELOG.md written |
| 10.2 | README polish | ✅ | Installation, usage, checker catalog, architecture diagram |
| 10.3 | CI pipeline | ✅ | GitHub Actions: build, test, clippy, fmt |
| 10.4 | Distribution | ⬜ | crates.io publish deferred |
| 10.5 | Example specs | ⬜ | IU spec is reference; more institutions deferred |
| 10.6 | Tag release | ⬜ | Ready for `git tag v0.1.0` |
| 10.7 | Post-release plan | ⬜ | v0.2.0+: community contributions, additional institutions |
