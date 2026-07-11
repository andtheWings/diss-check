# Project Status — 2026-07-11

## Current state

- **Library-only crate** (no `main.rs`) — binary extracted to `scholarpress-cli`
- **36 checkers** (33 automated + 3 human-review), 41 checks in IU spec
- **83 Rust tests**, 0 clippy warnings
- **16-document calibration corpus** — 5 rounds, 22 calibration decisions
- **7 systemic failures** confirmed legitimate
- **v0.1.0**, branch: `rust-rewrite`
- **Ecosystem migration (Phases 1-5) complete** — catalog, rename, repoint, cli extraction done

## Migration status

| Phase | Status | Detail |
|-------|--------|--------|
| 1 | ✅ | `scholarpress-catalog` created with IU profile |
| 2 | ✅ | Paths repointed to catalog via CATALOG_PATH |
| 3 | ✅ | Renamed diss-check → scholarpress-check |
| 4 | ⚠️ | publish renamed (publish wiring incomplete) |
| 5 | ✅ | CLI extracted to scholarpress-cli |
| 6 | ❌ | Cleanup pending |

## Phase history (pre-migration)

| Phase | Status |
|-------|--------|
| 1-3 | Archived (Python) |
| 4 | ✅ Rust rewrite (pdf_oxide, 134x faster) |
| 5 | ✅ 7 manual → automated |
| 6 | ✅ Cleanup, 0 warnings |
| 7 | ✅ 10 manual → 16 automated |
| 8 | ✅ Calibration: 16 docs, 22 decisions |
| 9 | ✅ CLI: --quiet, --check, --category |
| 10 | ✅ README, CHANGELOG, CI, v0.1.0 |

## Path conventions

- Spec: `CATALOG_PATH/institutions/iu/spec.yaml` or `../scholarpress-catalog/institutions/iu/spec.yaml`
- Corpus: `../scholarpress-catalog/institutions/iu/tests/corpus/`
- Fixtures: `../scholarpress-catalog/institutions/iu/tests/fixtures/`
- Binary: `scholarpress check --spec <catalog-path>/institutions/iu/spec.yaml <pdf>`
