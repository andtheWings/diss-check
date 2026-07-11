# Checker Validation & Calibration Workflow

**Log every HITL decision to `docs/calibration-decisions.md`.** Reference before tuning changes.

## Calibration process

Each round:
1. Add 1-3 new PDFs to `{CATALOG_PATH}/institutions/iu/tests/corpus/`
2. Run `scholarpress calibrate --spec {CATALOG_PATH}/institutions/iu/spec.yaml --corpus {CATALOG_PATH}/institutions/iu/tests/corpus/`
3. Parse systemic failures (≥50% of documents)
4. Cross-reference `docs/calibration-decisions.md`:
   - Already decided → skip
   - New failure mode or new check_id → present
5. Present only NEW systemic failures for HITL review
6. Log each decision
7. 2 consecutive rounds with no new issues → calibration complete

## Current corpus (16 documents, 5 rounds)
12 Word-processed + 4 LaTeX dissertations. 22 calibration decisions logged covering:
- Margin checker refactoring (percentile-based, ±0.125in range)
- Front matter detection (Roman-to-Arabic transition)
- Semantic font detection (math/code/mono excluded)
- Computer Modern font family normalization
- Fuzzy boilerplate matching (70% threshold)
- Justification front-matter exclusion
- Removed `tables_figures_within_margins` (not a spec requirement)

## CLI commands
- `scholarpress check --spec <spec> --json --quiet <pdf>` — JSON, failures only
- `scholarpress check --spec <spec> --check global_margins <pdf>` — single checker
- `scholarpress check --spec <spec> --category layout <pdf>` — category filter
- `scholarpress calibrate --spec <spec> --corpus <dir>` — corpus-wide calibration
