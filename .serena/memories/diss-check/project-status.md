## Current State (end of Round 20, Phase 2 complete)

- 67 tests passing
- 10 automated checkers: margins, font_size, font_weight, font_family, justification, section_presence, section_order, boilerplate_match, committee_order, toc_title_parity
- 27 total spec checks (10 automated + 17 human review)
- Full IU format review checklist coverage
- CLI: `diss-check --spec specs/iu.yaml <pdf>` (text output), `--json` for JSON
- 2 test dissertations: 2020-12-chambers.pdf, 2025-06-alexander.pdf

### Known limitations (from checker assessments)
- Margins: TOC leader dots flagged as right-margin violations
- Font size: figure content vs legend distinction not made
- Font family: embedded chart fonts (e.g. Aptos) flagged as mismatches
- Justification: right-edge stdev heuristic, pages <50 spans excluded, mixed within-page justification missed
- Section detection: abstract has no heading, detected by heuristic (backwards scan from TOC)
- Docling: explored but rejected (~4min/PDF, too slow)
- PDF extraction: word-order issues in some chapter headings (chambers Ch3)

### Next: Phase 3 — Calibration
- Round 21: calibration workflow
- Round 22: veraPDF extractor (if needed)
