# Project Roadmap

> Updated: 2026-07-01
> Current phase: Phase 8 — Large-corpus calibration
> Branch: `rust-rewrite` — Rust + pdf_oxide (134x faster than Python)
> Test documents: chambers (2020), alexander (2025)

## Status

**18 automated checks** + **10 human-review checks** — 59 tests passing, 0 build warnings.

---

## Phase History

Previous phases are fully complete and archived:

| Phase | Description | Rounds | Archive |
|-------|-------------|--------|---------|
| 1 | MVP — end-to-end pipeline with 1 checker (Python/pdfplumber) | 1–8 | [archive_phase1_mvp.md](archive_phase1_mvp.md) |
| 2 | Typography, structure, and content checkers (Python) | 9–20 | [archive_phase2_checkers.md](archive_phase2_checkers.md) |
| 3 | Calibration workflow (Python) | 21–22 | [archive_phase3_calibration.md](archive_phase3_calibration.md) |
| 4 | Rust rewrite — engine + all checkers ported (pdf_oxide) | R1–R10 | [archive_phase4_rust_rewrite.md](archive_phase4_rust_rewrite.md) |
| 5 | Promote 7 manual checks to automated (pdf_oxide capabilities) | R11–R18 | [archive_phase5_promote_manual.md](archive_phase5_promote_manual.md) |
| 6 | Cleanup & hardening | R19–R23 | [archive_phase6_cleanup.md](archive_phase6_cleanup.md) |

---

## Phase 7 — Manual check automation assessment

> **Goal**: Assess all 10 remaining human-review checks against pdf_oxide capabilities. Automate where possible, document why the rest must stay manual.

| Round | Feature | Status | Notes |
|-------|---------|--------|-------|
| 7.1 | Audit remaining manual checks | ✅ | Feasibility matrix complete. 10 human checks → 16 automated + 5 human (spacing/credentials stay manual). pdf_oxide exposes color, images, paths beyond current usage |
| 7.2 | `title_page_all_caps` | ✅ | First text block on page 1 = title. Check if all-caps. Chambers: PASS "TAKEN SPACES:", Alexander: PASS "SILENT STRATEGIES: INNER SPEECH AND PROBLEM SOLVING IN APHASIA" |
| 7.3a | `title_page_clause_centered` | ✅ | Clause spans on title page are horizontally centered. Chambers: 7/7 centered, Alexander: 8/8 centered |
| 7.3b | `title_page_clause_spacing` | ✅ | Clause line spacing is single-spaced OR matches body. Chambers: single-spaced (16pt), Alexander: single-spaced (14pt) |
| 7.4 | `copyright_page_format` | ✅ | © + year present, centered. Student name centered. Chambers: 3 lines centered, Alexander: no copyright page (optional — PASS) |
| 7.5a | `footnotes_font_consistent` | ✅ | Footnote font matches body, size ≥ 10pt ≤ body. Chambers: PASS (no footnotes detected), Alexander: PASS |
| 7.5b | `footnotes_spacing` | ⬜ | Human review: single-spaced or matches document line spacing |
| 7.6a | `tables_figures_within_margins` | ✅ | Image bboxes checked against margins. Chambers: PASS (no edge images), Alexander: FAIL (59 full-bleed images) |
| 7.6b | `tables_figures_legend_font` | ⬜ | Human review: legend/descriptor font ≥ 10pt or matches body |
| 7.7a | `references_font_consistent` | ✅ | References use same font as body. Chambers: PASS, Alexander: PASS |
| 7.7b | `references_heading_format` | ✅ | References heading matches chapter style. Chambers: PASS, Alexander: PASS |
| 7.7c | `references_spacing` | ⬜ | Human review: single-spaced or matches document |
| 7.8a | `cv_heading_format` | ✅ | CV heading matches chapter style. Chambers: PASS, Alexander: PASS |
| 7.8b | `cv_name_position` | ✅ | Name centered/left on CV page. Chambers: PASS (centered), Alexander: PASS |
| 7.8c | `cv_no_credentials` | ⬜ | Human review: no credentials/PII on CV |
| 7.9a | `abstract_text_centered` | ✅ | Student name + title centered on abstract. Chambers: PASS, Alexander: PASS |
| 7.9b | `abstract_word_count` | ✅ | Abstract ≤ 350 words. Chambers: PASS, Alexander: PASS |
| 7.9c | `abstract_title_format` | ✅ | Title is all-caps or title case. Chambers: PASS, Alexander: PASS |
| 7.10a | `toc_page_numbers_aligned` | ✅ | Page numbers at consistent right-edge position. Chambers: PASS, Alexander: PASS |
| 7.10b | `toc_no_overhang` | ✅ | No entries overhang page number column. Chambers: PASS, Alexander: PASS |
| 7.10c | `toc_cv_no_dots` | ✅ | CV entry has no leader dots/page number. Chambers: PASS, Alexander: PASS |
| 7.10d | `toc_spacing` | ⬜ | Human review: single-spaced or matches document |
| 7.11 | `final_manual_review` | ⬜ | Intentionally manual catch-all — stays as `checker: review` |
| 7.12 | Update spec + register new checkers | ✅ | All 16 automated checkers registered, 5 human reviews kept, spec updated |

---

## Phase 8 — Large-corpus calibration

> **Goal**: Run automated checkers and calibration workflow over a larger PDF corpus (~10+ dissertations). Human-in-the-loop review of results to tune thresholds, surface false positives, and validate checker decisions.

| Round | Feature | Status | Notes |
|-------|---------|--------|-------|
| 8.1 | Gather calibration corpus | ⬜ | Target 10+ real dissertation PDFs (IU or comparable). Add to `tests/fixtures/` or separate corpus directory |
| 8.2 | Run full check suite on corpus | ⬜ | Run all automated checkers against every PDF. Collect results in structured format for review |
| 8.3 | Human review: layout checkers | ⬜ | Review margins, margin_symmetry results across corpus. Tune tolerance thresholds. Identify systemic false positives (e.g. TOC leader dots, figure edge cases) |
| 8.4 | Human review: typography checkers | ⬜ | Review font_size, font_weight, font_family, justification. Check for embedded-font false positives, mixed-font documents, variant families |
| 8.5 | Human review: structure checkers | ⬜ | Review section_presence, section_order, page numbers, headings_consistent, new_chapters_new_pages. Test non-standard naming conventions |
| 8.6 | Human review: content checkers | ⬜ | Review boilerplate_match, committee_order, toc_title_parity, hyperlinks. Test unusual title page layouts |
| 8.7 | Human review: Phase 7 new automations | ⬜ | Review results from any newly automated checks from Phase 7 across the corpus |
| 8.8 | Threshold tuning + heuristic refinement | ⬜ | Apply lessons from rounds 8.3–8.7. Update checker defaults based on corpus data. Re-run and verify |
| 8.9 | Calibration report | ⬜ | Document per-checker false positive/negative rates, known limitations, and recommended thresholds. Output to `docs/calibration-report.md` |

---

## Phase 9 — CLI & report UX review

> **Goal**: Polish the user-facing interface — command-line ergonomics, report readability, and output flexibility.

| Round | Feature | Status | Notes |
|-------|---------|--------|-------|
| 9.1 | CLI audit | ✅ | Added `--version`, improved help text, proper exit codes (0=PASS, 1=FAIL/ERROR, 2=usage error) |
| 9.2 | `--check` / `--category` filter | ✅ | `--check <id>` runs single check, `--category <name>` filters by category |
| 9.3 | `--quiet` flag | ✅ | `--quiet` shows only FAIL/ERROR + summary |
| 9.7 | Multiple PDF batch mode | ⬜ | Deferred — calibration command covers batch needs |
| 9.8 | Exit codes | ✅ | 0 = all pass, 1 = failures found, 2 = usage/resource error |
| 9.9 | Documentation | ⬜ | Update README with usage, spec-writing guide |

---

## Phase 10 — Initial formal release prep

> **Goal**: Package for public release — versioning, changelog, distribution, and polish.

| Round | Feature | Status | Notes |
|-------|---------|--------|-------|
| 10.1 | Version + CHANGELOG | ✅ | v0.1.0 in Cargo.toml, CHANGELOG.md written |
| 10.2 | README polish | ✅ | Installation, usage, checker catalog, architecture diagram |
| 10.3 | CI pipeline | ✅ | GitHub Actions: build, test, clippy, fmt |
| 10.4 | Distribution | ⬜ | crates.io publish deferred — ready when you are |
| 10.5 | Example specs | ⬜ | IU spec is the reference; more institutions deferred |
| 10.6 | Tag release | ⬜ | Ready for `git tag v0.1.0` when you want |
| 10.7 | Post-release plan | ⬜ | Roadmap includes Phase 11+ for future work |

---

## Future (post-v0.1.0)

- Support for additional university specs (contributor-driven or curated)
- Web UI / GitHub Action for CI-based dissertation checking
- Custom check scripting (user-defined checks via Lua/Rhai or YAML expressions)
- Multi-language support for non-English dissertations
- Incremental checking (re-check only changed pages)
