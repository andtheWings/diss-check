## Current State (Phase 8 complete — 4 calibration rounds, 12-doc corpus)

- **83 Rust tests passing**, 0 build warnings
- **33 automated checkers** + **7 human-review checks**
- **12-document calibration corpus** — 4 rounds, 22 calibration decisions logged
- **6 systemic failures** — all confirmed legitimate (no checker bugs remain)
- **CLI**: `cargo run -- check --spec specs/iu.yaml <pdf>` / `--json`
- **Branch**: `rust-rewrite`