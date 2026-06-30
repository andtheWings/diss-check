# diss-check — Design Spec

> **Last updated:** 2026-06-30
> **Status key:** ✅ implemented &nbsp; 🚧 planned &nbsp; ⬜ not started

## Overview

A generalizable CLI tool and Python library that takes an institution-specific
YAML spec defining dissertation formatting requirements and checks a PDF for
compliance. Produces a report of PASS / FAIL / MANUAL per check with evidence.

First target institution: Indiana University (IU).
Architecture designed for generalization to other institutions after IU is working.

## Implementation Status

| Component | Status | Notes |
|---|---|---|
| **Spec models** (pydantic) | ✅ | `spec.py` — `InstitutionSpec`, `CheckDef`, `DocumentStructure`, `load_spec()` |
| **Document IR** (custom pydantic) | ✅ | `document.py` — `Document`, `Page`, `TextSpan`; decoupled from docling |
| **Pdfplumber extractor** | ✅ | `extractors/pdfplumber_extractor.py` — ~1s for 34-page PDF |
| **Docling extractor** | ✅ (disabled) | `extractors/docling_extractor.py` — preserved for structural checks, not default |
| **Engine** | ✅ | `engine.py` — loads spec, runs extractors+checkers, returns results |
| **Checker registry** | ✅ | `checkers/base.py` — `@register_checker`, `get_checker()`, `CheckResult` |
| **Layout checker** | ✅ | `checkers/layout.py` — `margins` (5 unit tests) |
| **CLI** | ✅ | `cli.py` — `diss-check --spec <path> <pdf>` (text output only) |
| **Report (text)** | ✅ | `report.py` — `Report`, `format_text()` (2 unit tests) |
| **Integration test** | ✅ | `tests/test_integration.py` — IU template pipeline in 2.9s |
| **IU spec (minimal)** | ✅ | `specs/iu.yaml` — 2 checks (margins + front_matter_presence) |
| **veraPDF extractor** | 🚧 | Stub; no IU checks need PDF/A yet |
| **Typography checker** | 🚧 | `font_size`, `font_weight`, `font_family`, `justification` |
| **Structure checker** | 🚧 | `section_presence`, `section_order`, `page_numbering` |
| **Content checker** | 🚧 | `text_match`, `committee_order`, `toc_title_parity` |
| **Human checker** | 🚧 | `automatable: false` passthrough |
| **Report (JSON)** | 🚧 | `format_json()` |
| **Calibration workflow** | 🚧 | Corpus-based spec validation |
| **IU spec (full)** | 🚧 | 2 of ~22 checks from checklist implemented |

## Testing Coverage (IU spec)

10 tests total across 4 files:

| Test file | Tests | Covers |
|---|---|---|
| `tests/test_spec.py` | 2 | Spec loading, YAML validation, pydantic model correctness |
| `tests/checkers/test_layout.py` | 5 | Margins: PASS, top/bottom/left/right violation |
| `tests/test_report.py` | 2 | Report summary counts, text formatter output |
| `tests/test_integration.py` | 1 | Full pipeline: spec → extraction → check → report against IU template |

**IU spec checks covered by automated tests:** 1 of 2 (`global_margins`).
`front_matter_presence` is `automatable: false` (MANUAL) and has no checker yet.

## Inputs

- **Institution spec** (`specs/iu.yaml`): YAML defining document structure,
  formatting rules, and which checks are automatable.
- **Dissertation PDF**: the document under review.
- **Calibration corpus** (optional): a directory of accepted dissertations
  used to validate the spec and checkers.

## Output

A report (text or JSON) listing each check with:

- `status`: PASS | FAIL | MANUAL | ERROR
- `evidence`: list of page references, bounding boxes, and text excerpts
- `detail`: human-readable explanation

## Architecture

```
diss-check/
├── diss_check/
│   ├── cli.py              # CLI entry point
│   ├── engine.py           # Orchestrator
│   ├── spec.py             # Pydantic models for YAML spec
│   ├── document.py         # ExtractionContext dataclass
│   ├── extractors/
│   │   ├── base.py         # Abstract Extractor interface
│   │   ├── docling_extractor.py
│   │   ├── pdfplumber_extractor.py
│   │   └── verapdf_extractor.py
│   ├── checkers/
│   │   ├── base.py         # BaseChecker + registry
│   │   ├── layout.py       # Margins, page dimensions
│   │   ├── typography.py   # Font family/size/weight, justification
│   │   ├── structure.py    # Page numbering, section presence/order
│   │   ├── content.py      # Text matching, clause wording, title parity
│   │   └── human.py        # MANUAL_REVIEW passthrough
│   ├── report.py           # Report model + formatters
│   └── calibration.py      # Corpus-based calibration workflow
├── specs/
│   └── iu.yaml             # IU format spec
├── tests/
│   ├── conftest.py
│   ├── checkers/
│   ├── extractors/
│   ├── test_engine.py
│   ├── test_spec.py
│   └── fixtures/
│       ├── iu_template.pdf
│       └── corpus/          # Accepted dissertations (gitignored)
└── pyproject.toml
```

### Data flow

```
spec.yaml + dissertation.pdf
         │
    ┌────▼────┐
    │  Engine  │
    └────┬────┘
         │ loads spec, resolves extractors needed by declared checkers
    ┌────▼────────────┐
    │  ExtractionCtx  │  Primary: Document IR from pdfplumber (~1s)
    │                 │  Optional: DoclingDocument from docling (structural checks)
    └────┬────────────┘
         │
    ┌────▼────┐
    │ Checkers │  Each checker receives ExtractionCtx + its spec params slice
    └────┬────┘
         │
    ┌────▼────┐
    │  Report  │  PASS / FAIL / MANUAL per check, with evidence
    └─────────┘
```

## Extraction Layer

### ExtractionContext

```python
@dataclass
class ExtractionContext:
    document: Document | None          # primary IR (pydantic: Document/Page/TextSpan)
    docling_doc: DoclingDocument | None  # optional, for structural checks only
    verapdf_report: dict | None
```

### Document IR (custom pydantic)

pdfplumber populates a lightweight `Document` IR:

```python
class TextSpan(BaseModel):
    text: str
    font_name: str
    font_size: float
    bbox: tuple[float, float, float, float]  # (top, bottom, x0, x1)

class Page(BaseModel):
    page_number: int
    width: float
    height: float
    spans: list[TextSpan]

class Document(BaseModel):
    pages: list[Page]
```

Built in ~1s for a 34-page dissertation. Serializable to JSON for caching.
Checkers consume `ctx.document`; never touch extractor internals.

### Extractor selection

Each checker declares what it needs:

```python
class BaseChecker:
    requires: list[Literal["pdfplumber", "docling", "verapdf"]] = ["pdfplumber"]
```

The engine collects the union of all requirements before extracting. Extractors
that no checker needs are skipped.

| Extractor | Produces | Used for | Status |
|---|---|---|---|
| pdfplumber | `Document` (pages, spans with position + font) | Margins, font size/family/weight, justification | ✅ Default |
| Docling | `DoclingDocument` (hierarchy, reading order, furniture) | Section detection, TOC/chapter matching | 🚧 Optional |
| veraPDF | `dict` (PDF/A validation report) | PDF/A structural compliance | ⬜ Not needed yet |

## Spec Format (YAML)

### Institution metadata

```yaml
institution: Indiana University
source_revision: "September 2025"
```

`source_revision` maps to the revision date on the source checklist document
(e.g., "Revised September 2025" footnote). This is not a semantic version;
each institution revision produces a new spec file.

### Document structure

```yaml
document_structure:
  front_matter:
    - { id: title_page, required: true }
    - { id: acceptance_page, required: true, page_number_start: ii }
    - { id: copyright_page, required: false }
    - { id: dedication, required: false }
    - { id: acknowledgements, required: false }
    - { id: preface, required: false }
    - { id: abstract, required: true }
    - { id: toc, required: true }
    - { id: lot, required: false }
    - { id: lof, required: false }
    - { id: lop, required: false }
    - { id: loa, required: false }
  body:
    - { id: chapters, required: true }
  end_matter:
    - { id: references, required: true }
    - { id: appendices, required: false }
    - { id: curriculum_vitae, required: true }
```

### Check definitions

```yaml
checks:
  - id: global_margins
    category: layout
    checker: margins
    target: { scope: all_pages }
    params: { top: "1in", bottom: "1in", left: "1.25in", right: "1.25in" }

  - id: font_size_consistent
    category: typography
    checker: font_size
    target: { scope: all_pages }
    params: { allowed: ["11pt", "12pt"], consistent: true }

  - id: title_not_bold
    category: typography
    checker: font_weight
    target: { page: title_page, element: title }
    params: { weight: normal }

  - id: clause_wording
    category: content
    checker: text_match
    target: { page: acceptance_page, element: clause }
    params:
      template: |
        Submitted to the faculty of the University Graduate School
        in partial fulfillment of the requirements
        for the degree {degree}
        in the {department},
        Indiana University
        {month} {year}
    automatable: true

  - id: committee_chair_first
    category: content
    checker: committee_order
    target: { pages: [acceptance_page, abstract] }
    params: { chair_first: true }
    automatable: false
    review_hint: "Verify the committee chair is listed first, followed by other members"
```

### Shared constants

```yaml
constants:
  degree: "Doctor of Philosophy"
  acceptable_fonts: ["Times New Roman"]
  page_number_font_size: same_as_body
```

Constants are referenced by checks via `{key}` interpolation in `params.template`
and directly by checkers that need global configuration.

### Pydantic validation

The spec is validated on load via pydantic models. Invalid spec files fail
before any extraction or checking occurs.

## Checker Engine

### Registry pattern

```python
@register_checker(category="typography", name="font_size")
class FontSizeChecker(BaseChecker):
    requires = ["docling", "pdfplumber"]

    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        ...
```

### CheckResult model

```python
class EvidenceItem:
    page: int
    bbox: tuple[float, float, float, float] | None
    excerpt: str | None

class CheckResult:
    check_id: str
    status: Literal["PASS", "FAIL", "MANUAL", "ERROR"]
    evidence: list[EvidenceItem]
    detail: str
```

### Check categories

| Category | Checker examples | Primary extractor | Status |
|---|---|---|---|---|
| `layout` | margins, content_start_position, landscape_margins | pdfplumber | ✅ margins done |
| `typography` | font_family, font_size, font_weight, justification, hyperlink_style | pdfplumber | 🚧 |
| `structure` | page_numbering, section_presence, section_order, chapter_new_page | pdfplumber + docling | 🚧 |
| `content` | text_match, committee_order, toc_title_parity, word_count | pdfplumber + docling | 🚧 |
| `human` | Passthrough for `automatable: false` checks | none | 🚧 |

### Execution order

Content checks run before human checks so automated results can inform the
reviewer. For example, if TOC/chapter title parity already passes
automatically, the reviewer doesn't need to re-verify it manually.

## Reporting

Two output formats:

- **Text** (default CLI output): grouped by category, status markers,
  evidence excerpts.
- **JSON**: full structured output, suitable for CI pipelines or future
  web frontend.

CLI usage:
```
diss-check --spec specs/iu.yaml dissertation.pdf
diss-check --spec specs/iu.yaml --json dissertation.pdf
```

## Calibration Workflow

### Purpose

Iteratively validate the spec and checkers against known-good documents to
surface false positives and spec errors.

### Sources of ground truth

1. **IU template PDF**: the official Word-derived template (may be imperfect;
   discrepancies between template and checklist must be flagged).
2. **Corpus of accepted dissertations**: PDFs that passed human format review,
   providing a broader validation set.

### Calibration tool

```
diss-check calibrate --spec specs/iu.yaml --corpus path/to/corpus/
```

For each document in the corpus, runs the full check suite. Compares results
against expected outcomes (all PASS for accepted dissertations). Produces a
diff report:

- **Systemic FAILs across the corpus**: likely spec issues (thresholds too
  strict, incorrect expected values).
- **Isolated FAILs**: likely real issues in that specific document or edge
  cases the spec doesn't handle.

Output groups failures by check_id and frequency, highlighting which checks
and which documents need attention.

### Iterative loop

1. Run calibration against corpus.
2. Review systemic FAILs → adjust spec or checker.
3. Capture version-control artifact of modified spec.
4. Re-run calibration.
5. Once corpus produces predominantly PASS results, spec is ready for
   production use.
6. Individual FAIL results on corpus documents are documented as known
   exceptions or flagged for human review.

## Testing Strategy

### Unit tests (checkers)

Each checker is tested in isolation with synthetic `DoclingDocument` fragments
(no real PDF required). Hand-crafted fragments trigger both PASS and FAIL
paths for each check.

### Integration tests (extractors)

Verify that pdfplumber and veraPDF correctly populate the ExtractionContext
when run against real PDFs (IU template and corpus samples).

### Calibration/snapshot tests

Full engine run against the accepted-dissertation corpus. Results captured as
snapshots. Assert that known-acceptable documents produce predominantly PASS
results. FAILs become either spec fixes or documented known exceptions.

Corpus files are gitignored (potentially large and may contain PII).

### Test structure

```
tests/
├── conftest.py              # fixtures: synthetic docs, IU template path
├── checkers/
│   ├── test_layout.py
│   ├── test_typography.py
│   ├── test_structure.py
│   ├── test_content.py
│   └── test_human.py
├── extractors/
│   ├── test_docling.py
│   ├── test_pdfplumber.py
│   └── test_verapdf.py
├── test_engine.py
├── test_spec.py
└── fixtures/
    ├── iu_template.pdf
    └── corpus/              # gitignored
```

## Dependencies

| Dependency | Role | Status |
|---|---|---|
| **pdfplumber** | Primary extractor — text, position, font metadata in ~1s | ✅ Active |
| **pydantic** | Spec validation, Document IR, result models | ✅ Active |
| **pyyaml** | YAML spec parsing | ✅ Active |
| **click** | CLI framework | ✅ Active |
| **pytest** | Test framework | ✅ Active |
| **docling** | Optional structural extraction (section detection, TOC matching) | ✅ Installed, not default |
| **veraPDF** | PDF/A structural validation (called as subprocess) | ⬜ Not needed yet |

## Out of Scope (v1)

- Web application frontend (library + CLI only)
- Non-PDF document formats (.docx, .tex, .typ)
- Multi-institution spec sharing/override system
- Real-time/live checking
