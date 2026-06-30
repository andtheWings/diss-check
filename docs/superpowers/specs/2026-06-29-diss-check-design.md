# diss-check — Design Spec

## Overview

A generalizable CLI tool and Python library that takes an institution-specific
YAML spec defining dissertation formatting requirements and checks a PDF for
compliance. Produces a report of PASS / FAIL / MANUAL per check with evidence.

First target institution: Indiana University (IU).
Architecture designed for generalization to other institutions after IU is working.

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
    │  ExtractionCtx  │  Produced by Docling + pdfplumber (+ veraPDF if requested)
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
    docling_doc: DoclingDocument
    pdfplumber_pages: list[Page] | None
    verapdf_report: dict | None
```

### Extractor selection

Each checker declares what it needs:

```python
class BaseChecker:
    requires: list[Literal["docling", "pdfplumber", "verapdf"]] = ["docling"]
```

The engine collects the union of all requirements before extracting. Extractors
that no checker needs are skipped.

| Extractor | Produces | Used for |
|---|---|---|
| Docling | `DoclingDocument` (hierarchy, bounding boxes, reading order) | Structural checks, margin checks, section order, TOC/chapter matching |
| pdfplumber | `list[Page]` (per-character font metadata) | Font family/size, bold/italic, hyperlink styling |
| veraPDF | `dict` (PDF/A validation report) | PDF/A structural compliance |

Docling is always required (it produces the primary IR). pdfplumber is
lazy-loaded only if a checker declares it. veraPDF is optional.

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

| Category | Checker examples | Primary extractor |
|---|---|---|
| `layout` | margins, content_start_position, landscape_margins | docling |
| `typography` | font_family, font_size, font_weight, justification, hyperlink_style | docling + pdfplumber |
| `structure` | page_numbering, section_presence, section_order, chapter_new_page | docling |
| `content` | text_match, committee_order, toc_title_parity, word_count | docling |
| `human` | Passthrough for `automatable: false` checks | none |

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
diss-check check --spec specs/iu.yaml dissertation.pdf
diss-check check --spec specs/iu.yaml --json dissertation.pdf
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
3. Re-run calibration.
4. Once corpus produces predominantly PASS results, spec is ready for
   production use.
5. Individual FAIL results on corpus documents are documented as known
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

- **docling**: primary document IR and extraction
- **pdfplumber**: supplementary font-level metadata extraction
- **veraPDF**: PDF/A structural validation (called as subprocess)
- **pydantic**: spec validation and data models
- **pyyaml**: YAML spec parsing
- **click** or **typer**: CLI framework
- **pytest**: test framework

## Out of Scope (v1)

- Web application frontend (library + CLI only)
- Non-PDF document formats (.docx, .tex, .typ)
- Multi-institution spec sharing/override system
- Real-time/live checking
