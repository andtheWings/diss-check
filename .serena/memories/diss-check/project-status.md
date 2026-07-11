## Current State (all phases complete)

- **83 Rust tests passing**, 0 clippy warnings
- **33 automated checkers** + **7 human-review checks**
- **16-document calibration corpus** — 5 rounds, 22 calibration decisions logged
- **7 systemic failures** — all confirmed legitimate (no bugs remain)
- **CLI**: `cargo run -- check --spec specs/iu.yaml <pdf>`, `--json`, `--quiet`, `--check <id>`, `--category <name>`
- **Version**: v0.1.0, ready for release
- **Branch**: `rust-rewrite`

### Phase summary
| Phase | Status | Key deliverable |
|-------|--------|----------------|
| 1-2 | Archived (Python) | MVP + 10 checkers |
| 3 | Archived (Python) | Calibration |
| 4 | ✅ | Rust rewrite (pdf_oxide, 134x faster) |
| 5 | ✅ | 7 manual checks promoted |
| 6 | ✅ | Cleanup, 0 warnings |
| 7 | ✅ | 10 manual → 16 automated checkers |
| 8 | ✅ | Calibration: 16 docs, 22 decisions |
| 9 | ✅ | CLI: --quiet, --check, --category, exit codes |
| 10 | ✅ | README, CHANGELOG, CI, hex sticker |

### Roadmap
Archived phases in `docs/roadmap/archive_*.md`. Active roadmap at `docs/roadmap/roadmap.md`.
