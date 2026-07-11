## Checker Validation & Calibration Workflow (Rust)

**During calibration, log every HITL decision to `docs/calibration-decisions.md`.** Reference that log before making tuning changes to avoid re-litigating settled decisions.

### Calibration process (iterative corpus expansion)

Each round:
1. Add 1-3 new PDFs to `tests/corpus/`
2. Run `cargo run -- calibrate --spec specs/iu.yaml --corpus tests/corpus/`
3. Parse systemic failures (≥50% of documents)
4. Cross-reference against `docs/calibration-decisions.md`:
   - Same check_id + already decided → skip (auto-filter)
   - Same check_id but new failure mode → present
   - New check_id → present
5. Present only NEW systemic failures to user for HITL review (one at a time)
6. Log each decision: context, decision, action taken
7. When no new issues appear for 2 consecutive rounds → calibration complete

### Current corpus (16 documents, 5 rounds)
12 Word-processed dissertations, 4 LaTeX dissertations. 22 calibration decisions logged covering:
- Margin checker refactoring (percentile-based with ±0.125in range)
- Front matter detection (Roman-to-Arabic page number transition)
- Semantic font detection (math, code, mono blocks excluded)
- Computer Modern font family normalization
- Fuzzy boilerplate matching (70% threshold)
- Justification front-matter exclusion
- Removed `tables_figures_within_margins` (not a spec requirement)

### Key calibration decisions
See `docs/calibration-decisions.md` for full log (Decisions 1-22).
