## Checker Development & Validation Workflow

For each checker round:
1. **Design** based on `tests/fixtures/2020-12-chambers.pdf`
2. **Implement** the checker
3. **Validate** against `tests/fixtures/2025-06-alexander.pdf` with human-in-the-loop feedback
4. **Reference** these IU artifacts for violation assessments:
   - `specs/artifacts/iu/format-review-checklist.pdf` (extracted via pdfplumber)
   - `specs/artifacts/iu/formatting-template.docx` (extracted via python-docx or zipfile/xml)
5. **Brainstorm potential false negatives** — edge cases the checker could miss:
   - What legitimate violations exist that the checker won't catch?
   - What assumptions does the heuristic make that could be wrong?
   - What document variations would evade detection?
   - Example: a justification checker might miss a page that switches from left-aligned to justified mid-page if the variance from the body text masks the change
6. **Present both assessments** (false positives + false negatives) to the user before marking round complete

### Round lifecycle
- Start: mark round 🚧 in `docs/ROADMAP.md`
- Implement: add checker to checkers/*.py, register in __init__.py
- Add check to `specs/iu.yaml`
- Write tests in `tests/checkers/test_*.py`
- Update `tests/test_spec.py` for check count changes (use `checks[-1]` for last check, not fixed index)
- Present assessment to user for human-in-the-loop feedback
- On approval: mark round ✅ in ROADMAP, update "Current round", commit

### Commit message format
`feat: <checker_name> checker (Round N)`

### Common pitfalls
- `_make_doc` uses bbox=(top, bottom, x0, x1) order matching pdfplumber convention
- TextSpan properties were historically wrong — fixed in Round 9 to proper mapping
- Spec test `test_spec.py` assertions need updating each round (length, checker names)
- pdfplumber extractor groups chars into words; font_name/font_size come from per-char data
- docling is too slow for practical use (~4min/PDF) — use pdfplumber only
- Running pytest: `source .venv/bin/activate && pytest tests/ -v`
