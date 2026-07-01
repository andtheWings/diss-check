# Phase 1 — MVP (Archived)

> End-to-end pipeline with 1 checker. Python/pdfplumber.

| Round | Feature | Status | Notes |
|-------|---------|--------|-------|
| 1 | Project scaffold | ✅ | |
| 2 | Spec models + IU spec | ✅ | 2 tests |
| 3 | Document IR (TextSpan/Page/Document) | ✅ | |
| 4 | Pdfplumber extractor | ✅ | |
| 5 | Engine + checker registry | ✅ | |
| 6 | Margins checker | ✅ | 6 tests. Word-level bboxes, 0.25in tolerance, page-number zone exclusion. 33 violations on test doc (all TOC leader dots) |
| 7 | CLI + report (text) | ✅ | 2 tests |
| 8 | Integration test | ✅ | 1 test |
