## Checker Validation & Calibration Workflow (Rust)

**During calibration (Phase 8+), log every HITL decision to `docs/calibration-decisions.md`.** Reference that log before making tuning changes to avoid re-litigating settled decisions.

### Calibration process (iterative corpus expansion)

Each round:
1. Add 1-2 new PDFs to `tests/corpus/`
2. Run `cargo run -- calibrate --spec specs/iu.yaml --corpus tests/corpus/`
3. Parse systemic failures (≥50% of documents)
4. Cross-reference against `docs/calibration-decisions.md`:
   - If same check_id + similar failure pattern already decided → skip (auto-filter)
   - If same check_id but new failure mode → present
   - If new check_id → present
5. Present only NEW systemic failures to user for HITL review
6. Log each decision in `docs/calibration-decisions.md` with: context, decision, action
7. After all rounds complete → write `docs/calibration-report.md`

### Filtering logic
- Read `docs/calibration-decisions.md` at start of each round
- Build a set of `(check_id, decision_type)` pairs from logged decisions
- Before presenting a failure, check if there's an existing decision covering it
- If the decision was "legitimate" → skip presenting (known issue)
- If the decision was "fixed" → verify the fix still holds, skip if so

### Current corpus
- `tests/corpus/2020-12-chambers.pdf`
- `tests/corpus/2025-06-alexander.pdf`
- Target: 10+ documents over multiple rounds
