# diss-check Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a CLI tool that checks dissertation PDFs against an institution-specific YAML format spec, producing PASS/FAIL/MANUAL reports.

**Phased approach:** MVP (tasks 1-8) proves the end-to-end pipeline with one extractor + one checker. Phase 2 adds remaining extractors/checkers. Phase 3 adds calibration.

**Tech Stack:** Python 3.11+, uv, docling, pdfplumber, pydantic, pyyaml, click, pytest

## Global Constraints

- Python 3.11+
- Dependencies declared in pyproject.toml, installed via `uv pip install -e ".[dev]"`
- TDD: write failing test first, then implementation
- Checker code lives in `diss_check/checkers/`; each checker is one file
- Spec YAML lives in `specs/`
- Document structure is defined in the spec, not hardcoded
- IU template PDF is NOT committed (gitignored); user provides it manually

---

## Phase 1 — MVP (end-to-end: spec → extraction → check → report)

### File Map (MVP)

```
diss-check/
├── pyproject.toml
├── diss_check/
│   ├── __init__.py
│   ├── cli.py
│   ├── engine.py
│   ├── spec.py
│   ├── document.py
│   ├── report.py
│   ├── extractors/
│   │   ├── __init__.py
│   │   ├── base.py
│   │   └── docling_extractor.py
│   └── checkers/
│       ├── __init__.py
│       ├── base.py
│       └── layout.py
├── specs/
│   └── iu.yaml
└── tests/
    ├── __init__.py
    ├── conftest.py
    ├── test_spec.py
    ├── test_engine.py
    ├── test_report.py
    ├── test_integration.py
    ├── checkers/
    │   ├── __init__.py
    │   └── test_layout.py
    └── fixtures/
        └── .gitkeep
```

---

### Task 1: Project Scaffolding (MVP deps only)

**Files:**
- Create: `pyproject.toml`
- Create: `diss_check/__init__.py`, `diss_check/extractors/__init__.py`, `diss_check/checkers/__init__.py`
- Create: `tests/__init__.py`, `tests/checkers/__init__.py`
- Create: `tests/conftest.py`, `tests/fixtures/.gitkeep`, `specs/.gitkeep`

**Interfaces:**
- Produces: importable `diss_check` package with minimal dependencies

- [ ] **Step 1: Create pyproject.toml**

```toml
[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "diss-check"
version = "0.1.0"
description = "Check dissertation PDFs against institutional formatting requirements"
requires-python = ">=3.11"
dependencies = [
    "docling>=2.0",
    "pydantic>=2.0",
    "pyyaml>=6.0",
    "click>=8.0",
]

[project.optional-dependencies]
dev = [
    "pytest>=8.0",
]

[project.scripts]
diss-check = "diss_check.cli:main"

[tool.pytest.ini_options]
testpaths = ["tests"]
```

- [ ] **Step 2: Create directory structure**

```bash
mkdir -p diss_check/extractors diss_check/checkers tests/checkers tests/fixtures specs
touch diss_check/__init__.py
touch diss_check/extractors/__init__.py
touch diss_check/checkers/__init__.py
touch tests/__init__.py
touch tests/checkers/__init__.py
touch specs/.gitkeep
touch tests/fixtures/.gitkeep
```

- [ ] **Step 3: Create tests/conftest.py**

```python
from pathlib import Path
import pytest


@pytest.fixture
def fixtures_dir():
    return Path(__file__).parent / "fixtures"


@pytest.fixture
def iu_template_path(fixtures_dir):
    return fixtures_dir / "iu_template.pdf"
```

- [ ] **Step 4: Create .gitignore**

```
*.pyc
__pycache__/
*.egg-info/
.pytest_cache/
.venv/
venv/
tests/fixtures/iu_template.pdf
tests/fixtures/corpus/
```

- [ ] **Step 5: Install and verify**

```bash
uv pip install -e ".[dev]" 2>&1 | tail -5
python -c "import diss_check; print('ok')"
```

Expected: `ok`

- [ ] **Step 6: Commit**

```bash
git add -A && git commit -m "chore: scaffold project with MVP dependencies"
```

---

### Task 2: Spec Models + Minimal IU Spec

**Files:**
- Create: `diss_check/spec.py`
- Create: `specs/iu.yaml` (minimal: 2 checks)
- Create: `tests/test_spec.py`

**Interfaces:**
- Produces:
  - `SectionDef(id: str, required: bool)`
  - `DocumentStructure(front_matter: list[SectionDef], body: list[SectionDef], end_matter: list[SectionDef])`
  - `CheckTarget(scope: str | None, page: str | None, element: str | None)`
  - `CheckDef(id: str, category: str, checker: str, target: CheckTarget, params: dict, automatable: bool = True)`
  - `InstitutionSpec(institution: str, source_revision: str, document_structure: DocumentStructure, checks: list[CheckDef], constants: dict)`
  - `load_spec(path: Path | str) -> InstitutionSpec`

- [ ] **Step 1: Write failing test**

```python
# tests/test_spec.py
from diss_check.spec import load_spec


def test_load_minimal_iu_spec():
    spec = load_spec("specs/iu.yaml")
    assert spec.institution == "Indiana University"
    assert spec.source_revision == "September 2025"
    assert len(spec.checks) == 2
    assert spec.checks[0].checker == "margins"
    assert spec.checks[0].automatable is True
    assert spec.checks[1].automatable is False  # structure check is manual for now


def test_spec_validates_invalid_yaml(tmp_path):
    yaml_content = """
institution: Test
source_revision: "v1"
checks:
  - id: bad
    category: invalid_category
    checker: x
    target: {}
    params: {}
"""
    spec_file = tmp_path / "bad.yaml"
    spec_file.write_text(yaml_content)
    import pytest
    with pytest.raises(Exception):
        load_spec(spec_file)
```

- [ ] **Step 2: Run tests to confirm failure**

Run: `pytest tests/test_spec.py -v`
Expected: FAIL (module not found / file not found)

- [ ] **Step 3: Implement spec.py**

```python
# diss_check/spec.py
from pathlib import Path
from typing import Literal

import yaml
from pydantic import BaseModel


class SectionDef(BaseModel):
    id: str
    required: bool


class DocumentStructure(BaseModel):
    front_matter: list[SectionDef]
    body: list[SectionDef]
    end_matter: list[SectionDef]


class CheckTarget(BaseModel):
    scope: str | None = None
    page: str | None = None
    element: str | None = None


class CheckDef(BaseModel):
    id: str
    category: Literal["layout", "typography", "structure", "content", "human"]
    checker: str
    target: CheckTarget
    params: dict = {}
    automatable: bool = True


class InstitutionSpec(BaseModel):
    institution: str
    source_revision: str
    document_structure: DocumentStructure
    checks: list[CheckDef]
    constants: dict = {}


def load_spec(path: Path | str) -> InstitutionSpec:
    path = Path(path)
    raw = yaml.safe_load(path.read_text())
    return InstitutionSpec.model_validate(raw)
```

- [ ] **Step 4: Create minimal IU spec**

```yaml
# specs/iu.yaml
institution: Indiana University
source_revision: "September 2025"

document_structure:
  front_matter:
    - { id: title_page, required: true }
    - { id: acceptance_page, required: true }
    - { id: abstract, required: true }
    - { id: toc, required: true }
  body:
    - { id: chapters, required: true }
  end_matter:
    - { id: references, required: true }
    - { id: curriculum_vitae, required: true }

checks:
  - id: global_margins
    category: layout
    checker: margins
    target: { scope: all_pages }
    params:
      top: "1in"
      bottom: "1in"
      left: "1.25in"
      right: "1.25in"

  - id: front_matter_presence
    category: structure
    checker: section_presence
    target: { scope: front_matter }
    params:
      required_sections:
        - { id: title_page }
        - { id: acceptance_page }
        - { id: abstract }
        - { id: toc }
    automatable: false
    review_hint: "Verify all required front matter sections are present"

constants:
  degree: "Doctor of Philosophy"
```

- [ ] **Step 5: Run tests**

Run: `pytest tests/test_spec.py -v`
Expected: all PASS

- [ ] **Step 6: Commit**

```bash
git add diss_check/spec.py specs/iu.yaml tests/test_spec.py
git commit -m "feat: add spec models and minimal IU spec"
```

---

### Task 3: ExtractionContext + Extractor Base

**Files:**
- Create: `diss_check/document.py`
- Create: `diss_check/extractors/base.py`

**Interfaces:**
- Produces:
  - `ExtractionContext` dataclass with `docling_doc: DoclingDocument | None = None`
  - `BaseExtractor` ABC with `name: str` and `extract(source: Path, ctx: ExtractionContext) -> None`

- [ ] **Step 1: Write ExtractionContext**

```python
# diss_check/document.py
from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from docling_core.types.doc import DoclingDocument


@dataclass
class ExtractionContext:
    docling_doc: "DoclingDocument | None" = None
```

- [ ] **Step 2: Write BaseExtractor**

```python
# diss_check/extractors/base.py
from abc import ABC, abstractmethod
from pathlib import Path
from diss_check.document import ExtractionContext


class BaseExtractor(ABC):
    name: str

    @abstractmethod
    def extract(self, source: Path, ctx: ExtractionContext) -> None:
        ...
```

- [ ] **Step 3: Verify imports**

```bash
python -c "from diss_check.document import ExtractionContext; from diss_check.extractors.base import BaseExtractor; print('ok')"
```

Expected: `ok`

- [ ] **Step 4: Commit**

```bash
git add diss_check/document.py diss_check/extractors/base.py
git commit -m "feat: add ExtractionContext and BaseExtractor"
```

---

### Task 4: Docling Extractor

**Files:**
- Create: `diss_check/extractors/docling_extractor.py`

**Interfaces:**
- Consumes: `BaseExtractor`, `ExtractionContext`
- Produces: `DoclingExtractor(name="docling")` — converts PDF to `DoclingDocument` and stores on ctx

- [ ] **Step 1: Implement DoclingExtractor**

```python
# diss_check/extractors/docling_extractor.py
from pathlib import Path
from docling.document_converter import DocumentConverter
from diss_check.document import ExtractionContext
from diss_check.extractors.base import BaseExtractor


class DoclingExtractor(BaseExtractor):
    name = "docling"

    def __init__(self):
        self._converter = DocumentConverter()

    def extract(self, source: Path, ctx: ExtractionContext) -> None:
        result = self._converter.convert(str(source))
        ctx.docling_doc = result.document
```

- [ ] **Step 2: Verify import**

```bash
python -c "from diss_check.extractors.docling_extractor import DoclingExtractor; print('ok')"
```

Expected: `ok`

- [ ] **Step 3: Commit**

```bash
git add diss_check/extractors/docling_extractor.py
git commit -m "feat: add docling extractor"
```

---

### Task 5: Checker Framework (BaseChecker, Registry, Engine)

**Files:**
- Create: `diss_check/checkers/base.py`
- Create: `diss_check/engine.py`

**Interfaces:**
- Consumes: `InstitutionSpec`, `ExtractionContext`, `DoclingExtractor`
- Produces:
  - `EvidenceItem(page: int, bbox: tuple|None, excerpt: str|None)` (pydantic)
  - `CheckResult(check_id: str, status: Literal["PASS","FAIL","MANUAL","ERROR"], evidence: list[EvidenceItem], detail: str)` (pydantic)
  - `BaseChecker` with `requires: list[str] = ["docling"]`, `check(ctx, params) -> CheckResult`
  - `_CHECKER_REGISTRY`, `register_checker(category, name)` decorator, `get_checker(category, name) -> BaseChecker`
  - `Engine(spec)`, `run(pdf_path) -> list[CheckResult]`

- [ ] **Step 1: Write checker base**

```python
# diss_check/checkers/base.py
from abc import ABC, abstractmethod
from typing import Literal
from pydantic import BaseModel
from diss_check.document import ExtractionContext


class EvidenceItem(BaseModel):
    page: int
    bbox: tuple[float, float, float, float] | None = None
    excerpt: str | None = None


class CheckResult(BaseModel):
    check_id: str = ""
    status: Literal["PASS", "FAIL", "MANUAL", "ERROR"] = "ERROR"
    evidence: list[EvidenceItem] = []
    detail: str = ""


_CHECKER_REGISTRY: dict[tuple[str, str], type["BaseChecker"]] = {}


def register_checker(category: str, name: str):
    def decorator(cls: type["BaseChecker"]):
        _CHECKER_REGISTRY[(category, name)] = cls
        cls.category = category
        cls.name = name
        return cls
    return decorator


def get_checker(category: str, name: str) -> "BaseChecker":
    cls = _CHECKER_REGISTRY.get((category, name))
    if cls is None:
        raise KeyError(f"No checker registered for category={category}, name={name}")
    return cls()


class BaseChecker(ABC):
    category: str
    name: str
    requires: list[str] = ["docling"]

    @abstractmethod
    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        ...
```

- [ ] **Step 2: Write engine**

```python
# diss_check/engine.py
from pathlib import Path
from diss_check.spec import InstitutionSpec
from diss_check.document import ExtractionContext
from diss_check.checkers.base import get_checker, CheckResult
from diss_check.extractors.docling_extractor import DoclingExtractor
from diss_check.extractors.base import BaseExtractor


EXTRACTOR_MAP: dict[str, BaseExtractor] = {
    "docling": DoclingExtractor(),
}


class Engine:
    def __init__(self, spec: InstitutionSpec):
        self.spec = spec

    def run(self, pdf_path: Path) -> list[CheckResult]:
        required_extractors = self._collect_required_extractors()
        ctx = ExtractionContext()
        for ext_name in required_extractors:
            EXTRACTOR_MAP[ext_name].extract(pdf_path, ctx)

        results: list[CheckResult] = []
        for check_def in self.spec.checks:
            try:
                checker = get_checker(check_def.category, check_def.checker)
            except KeyError:
                results.append(CheckResult(
                    check_id=check_def.id,
                    status="ERROR",
                    detail=f"No checker registered for {check_def.category}/{check_def.checker}",
                ))
                continue
            if not check_def.automatable:
                results.append(CheckResult(
                    check_id=check_def.id,
                    status="MANUAL",
                    detail=check_def.review_hint or "Manual review required",
                ))
                continue
            result = checker.check(ctx, check_def.params)
            result.check_id = check_def.id
            results.append(result)

        return results

    def _collect_required_extractors(self) -> set[str]:
        required: set[str] = set()
        for check_def in self.spec.checks:
            if not check_def.automatable:
                continue
            try:
                checker_cls = type(get_checker(check_def.category, check_def.checker))
            except KeyError:
                continue
            required.update(checker_cls.requires)
        return required
```

- [ ] **Step 3: Verify imports**

```bash
python -c "from diss_check.checkers.base import BaseChecker, CheckResult, EvidenceItem, register_checker; from diss_check.engine import Engine; print('ok')"
```

Expected: `ok`

- [ ] **Step 4: Commit**

```bash
git add diss_check/checkers/base.py diss_check/engine.py
git commit -m "feat: add checker framework and engine"
```

---

### Task 6: Layout Checker (Margins)

**Files:**
- Create: `diss_check/checkers/layout.py`
- Create: `tests/checkers/test_layout.py`

**Interfaces:**
- Consumes: `BaseChecker`, `register_checker`, `ExtractionContext`
- Produces: `MarginsChecker` `("layout", "margins")` — checks text bounding boxes against specified margins using docling data

- [ ] **Step 1: Write failing tests with synthetic DoclingDocument**

```python
# tests/checkers/test_layout.py
import pytest
from diss_check.checkers.layout import MarginsChecker
from diss_check.document import ExtractionContext


def _make_synthetic_doc(bboxes):
    """Create a synthetic DoclingDocument with text items at given bounding boxes.
    Each bbox is (x0, y0, x1, y1) in points (72pt = 1in). Page is US Letter (612x792pt)."""
    from docling_core.types.doc import DoclingDocument, DocItemLabel, BoundingBox, CoordOrigin
    from docling_core.types.doc.document import TextItem, GroupItem, PageItem

    doc = DoclingDocument(name="synthetic")
    page = PageItem(
        page_no=1,
        size={"width": 612, "height": 792},
        self_ref="#/pages/1",
        children=[],
        label=DocItemLabel.PAGE,
    )
    doc.pages[1] = page
    text_items = []
    for i, bbox in enumerate(bboxes):
        item = TextItem(
            self_ref=f"#/texts/{i}",
            text=f"Text block {i}",
            label=DocItemLabel.TEXT,
            prov=[],
        )
        item.page_no = 1
        item.bbox = BoundingBox(
            l=bbox[0], t=bbox[1], r=bbox[2], b=bbox[3],
            coord_origin=CoordOrigin.TOPLEFT,
        )
        text_items.append(item)
    doc.texts = text_items
    doc.body = GroupItem(
        self_ref="#/groups/body",
        label=DocItemLabel.GROUP,
        children=[t.self_ref for t in text_items],
    )
    return doc


def test_margins_pass_when_text_within_bounds():
    # margins: left=1.25in=90pt, right=1.25in→right_edge≥522pt, top=1in=72pt, bottom=1in→bottom_edge≤720pt
    doc = _make_synthetic_doc([(90, 72, 522, 720)])
    ctx = ExtractionContext(docling_doc=doc)
    checker = MarginsChecker()
    result = checker.check(ctx, {"top": "1in", "bottom": "1in", "left": "1.25in", "right": "1.25in"})
    assert result.status == "PASS"


def test_margins_fail_when_left_margin_violated():
    doc = _make_synthetic_doc([(36, 72, 522, 720)])  # left=36pt=0.5in < 1.25in
    ctx = ExtractionContext(docling_doc=doc)
    checker = MarginsChecker()
    result = checker.check(ctx, {"top": "1in", "bottom": "1in", "left": "1.25in", "right": "1.25in"})
    assert result.status == "FAIL"
    assert len(result.evidence) > 0
    assert result.evidence[0].page == 1


def test_margins_fail_when_right_margin_violated():
    doc = _make_synthetic_doc([(90, 72, 576, 720)])  # right=576pt→right_margin=612-576=36pt < 1.25in=90pt
    ctx = ExtractionContext(docling_doc=doc)
    checker = MarginsChecker()
    result = checker.check(ctx, {"top": "1in", "bottom": "1in", "left": "1.25in", "right": "1.25in"})
    assert result.status == "FAIL"
```

- [ ] **Step 2: Run tests to confirm failure**

Run: `pytest tests/checkers/test_layout.py -v`
Expected: FAIL (ImportError)

- [ ] **Step 3: Implement MarginsChecker**

```python
# diss_check/checkers/layout.py
from diss_check.document import ExtractionContext
from diss_check.checkers.base import BaseChecker, CheckResult, EvidenceItem, register_checker


def _parse_measurement(value: str) -> float:
    value = value.strip()
    if value.endswith("in"):
        return float(value[:-2]) * 72
    if value.endswith("pt"):
        return float(value[:-2])
    raise ValueError(f"Unsupported measurement: {value}")


@register_checker(category="layout", name="margins")
class MarginsChecker(BaseChecker):
    requires = ["docling"]

    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        doc = ctx.docling_doc
        top_margin = _parse_measurement(params["top"])
        bottom_margin = _parse_measurement(params["bottom"])
        left_margin = _parse_measurement(params["left"])
        right_margin = _parse_measurement(params["right"])

        violations: list[EvidenceItem] = []

        for page_no, page in doc.pages.items():
            page_width = page.size.width if hasattr(page.size, 'width') else 612
            page_height = page.size.height if hasattr(page.size, 'height') else 792

            for text_item in doc.texts:
                if getattr(text_item, 'page_no', None) != page_no:
                    continue
                if text_item.bbox is None:
                    continue

                bbox = text_item.bbox
                l, t, r, b = bbox.l, bbox.t, bbox.r, bbox.b

                if l < left_margin:
                    violations.append(EvidenceItem(
                        page=page_no, bbox=(l, t, r, b),
                        excerpt=text_item.text[:100],
                    ))
                elif (page_width - r) < right_margin:
                    violations.append(EvidenceItem(
                        page=page_no, bbox=(l, t, r, b),
                        excerpt=text_item.text[:100],
                    ))
                elif t < top_margin:
                    violations.append(EvidenceItem(
                        page=page_no, bbox=(l, t, r, b),
                        excerpt=text_item.text[:100],
                    ))
                elif (page_height - b) < bottom_margin:
                    violations.append(EvidenceItem(
                        page=page_no, bbox=(l, t, r, b),
                        excerpt=text_item.text[:100],
                    ))

        if violations:
            return CheckResult(
                status="FAIL", evidence=violations,
                detail=f"{len(violations)} text block(s) violate margin requirements",
            )
        return CheckResult(
            status="PASS",
            detail="All text is within required margins",
        )
```

- [ ] **Step 4: Run tests to confirm pass**

Run: `pytest tests/checkers/test_layout.py -v`
Expected: all PASS

- [ ] **Step 5: Commit**

```bash
git add diss_check/checkers/layout.py tests/checkers/test_layout.py
git commit -m "feat: add layout checker (margins)"
```

---

### Task 7: CLI + Report (text only)

**Files:**
- Create: `diss_check/report.py`
- Create: `diss_check/cli.py`
- Create: `tests/test_report.py`

**Interfaces:**
- Consumes: `Engine`, `CheckResult`, `load_spec`
- Produces:
  - `Report(results: list[CheckResult])` with `summary` counts, `format_text() -> str`
  - CLI: `diss-check --spec <path> <pdf>`

- [ ] **Step 1: Write Report + formatter**

```python
# diss_check/report.py
from pydantic import BaseModel
from diss_check.checkers.base import CheckResult


class Summary(BaseModel):
    pass_: int = 0
    fail: int = 0
    manual: int = 0
    error: int = 0


class Report(BaseModel):
    results: list[CheckResult]
    summary: Summary | None = None

    def model_post_init(self, __context):
        summary = Summary()
        for r in self.results:
            if r.status == "PASS":
                summary.pass_ += 1
            elif r.status == "FAIL":
                summary.fail += 1
            elif r.status == "MANUAL":
                summary.manual += 1
            elif r.status == "ERROR":
                summary.error += 1
        self.summary = summary


def format_text(report: Report) -> str:
    lines = ["=" * 60, "DISSERTATION FORMAT CHECK REPORT", "=" * 60]
    for result in report.results:
        marker = {"PASS": "[PASS]", "FAIL": "[FAIL]", "MANUAL": "[MANUAL]", "ERROR": "[ERROR]"}[result.status]
        lines.append(f"\n{marker} {result.check_id}")
        if result.detail:
            lines.append(f"  {result.detail}")
        for ev in result.evidence:
            page_info = f"page {ev.page}"
            if ev.bbox:
                page_info += f" @ ({ev.bbox[0]:.0f},{ev.bbox[1]:.0f},{ev.bbox[2]:.0f},{ev.bbox[3]:.0f})"
            lines.append(f"    [{page_info}] {ev.excerpt or ''}")
    s = report.summary
    lines.append(f"\n{'─' * 60}")
    lines.append(f"Summary: {s.pass_} PASS, {s.fail} FAIL, {s.manual} MANUAL, {s.error} ERROR")
    lines.append("=" * 60)
    return "\n".join(lines)
```

- [ ] **Step 2: Write test_report.py**

```python
# tests/test_report.py
from diss_check.report import Report, format_text
from diss_check.checkers.base import CheckResult


def test_report_summary_counts():
    results = [
        CheckResult(check_id="c1", status="PASS", detail="ok"),
        CheckResult(check_id="c2", status="FAIL", detail="bad"),
        CheckResult(check_id="c3", status="MANUAL", detail="check"),
    ]
    report = Report(results=results)
    assert report.summary.pass_ == 1
    assert report.summary.fail == 1
    assert report.summary.manual == 1
    assert report.summary.error == 0


def test_format_text_includes_statuses():
    results = [
        CheckResult(check_id="c1", status="PASS", detail="ok"),
        CheckResult(check_id="c2", status="FAIL", detail="bad margin on page 3"),
    ]
    report = Report(results=results)
    output = format_text(report)
    assert "c1" in output
    assert "c2" in output
    assert "PASS" in output
    assert "FAIL" in output
    assert "bad margin on page 3" in output
```

- [ ] **Step 3: Run report tests**

Run: `pytest tests/test_report.py -v`
Expected: PASS

- [ ] **Step 4: Write CLI**

```python
# diss_check/cli.py
from pathlib import Path
import sys
import click
from diss_check.spec import load_spec
from diss_check.engine import Engine
from diss_check.report import Report, format_text


@click.command()
@click.option("--spec", required=True, type=click.Path(exists=True), help="Path to institution spec YAML file")
@click.argument("pdf", type=click.Path(exists=True))
def main(spec, pdf):
    """Check a dissertation PDF against institutional formatting requirements."""
    spec_path = Path(spec)
    pdf_path = Path(pdf)
    try:
        institution_spec = load_spec(spec_path)
    except Exception as e:
        click.echo(f"Error loading spec: {e}", err=True)
        sys.exit(1)
    engine = Engine(institution_spec)
    results = engine.run(pdf_path)
    report = Report(results=results)
    click.echo(format_text(report))
    if report.summary.fail > 0 or report.summary.error > 0:
        sys.exit(1)
```

- [ ] **Step 5: Verify CLI**

```bash
python -m diss_check.cli --help
```

Expected: help text

- [ ] **Step 6: Commit**

```bash
git add diss_check/report.py diss_check/cli.py tests/test_report.py
git commit -m "feat: add CLI and text report formatter"
```

---

### Task 8: Integration Test (manual trigger — requires IU template PDF)

**Files:**
- Create: `tests/test_integration.py`

**Prerequisites:** User must place `iu_template.pdf` in `tests/fixtures/iu_template.pdf`

- [ ] **Step 1: Write integration test**

```python
# tests/test_integration.py
import pytest
from diss_check.spec import load_spec
from diss_check.engine import Engine
from diss_check.report import Report


def test_iu_template_produces_results(iu_template_path):
    """Run the full MVP check suite against the IU template PDF."""
    if not iu_template_path.exists():
        pytest.skip("IU template PDF not found — place it at tests/fixtures/iu_template.pdf")

    spec = load_spec("specs/iu.yaml")
    engine = Engine(spec)
    results = engine.run(iu_template_path)
    report = Report(results=results)

    assert len(report.results) == len(spec.checks)
    assert report.summary.error == 0

    # Print results for debugging
    print(f"\nResults: {report.summary.pass_} PASS, {report.summary.fail} FAIL, {report.summary.manual} MANUAL")
    for r in report.results:
        if r.status != "PASS":
            print(f"  {r.status}: {r.check_id} — {r.detail}")
```

- [ ] **Step 2: Commit**

```bash
git add tests/test_integration.py
git commit -m "test: add integration test for MVP pipeline"
```

---

## Phase 2 — Remaining Extractors and Checkers (TBD after MVP validates)

Planned files to add in Phase 2:
- `diss_check/extractors/pdfplumber_extractor.py` — font-level metadata
- `diss_check/extractors/verapdf_extractor.py` — PDF/A stub
- `diss_check/checkers/typography.py` — font_size, font_weight, font_family, justification
- `diss_check/checkers/structure.py` — section_presence, section_order, page_numbering
- `diss_check/checkers/content.py` — text_match, committee_order, toc_title_parity
- `diss_check/checkers/human.py` — manual review passthrough
- `diss_check/report.py` — add `format_json()`
- `specs/iu.yaml` — expand to full checklist
- Corresponding test files in `tests/checkers/`

## Phase 3 — Calibration (TBD after Phase 2 validates)

- `diss_check/calibration.py` — corpus-based calibration workflow
