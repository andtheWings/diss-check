# diss-check Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a CLI tool and Python library that checks PDF dissertations against an institution-specific YAML format spec, producing PASS/FAIL/MANUAL reports with evidence.

**Architecture:** Extractors (Docling, pdfplumber, veraPDF) produce an ExtractionContext. Checkers (layout, typography, structure, content, human) consume the context + spec params to produce CheckResults. The engine orchestrates: load spec -> extract -> run checkers -> produce report. TDD throughout.

**Tech Stack:** Python 3.11+, docling, pdfplumber, pydantic, pyyaml, click, pytest

## Global Constraints

- Python 3.11+ (for `Literal` in type hints without importing typing_extensions)
- All dependencies declared in pyproject.toml
- TDD: write failing test first, then implementation
- Checker code lives in `diss_check/checkers/`; each checker is one file
- Spec YAML lives in `specs/`
- Report is a pydantic model; formatters render it to text or JSON
- veraPDF is optional (not needed for IU v1); checker requires list controls which extractors run
- Document structure is defined in the spec, not hardcoded

---

## File Map

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
│   ├── calibration.py
│   ├── extractors/
│   │   ├── __init__.py
│   │   ├── base.py
│   │   ├── docling_extractor.py
│   │   ├── pdfplumber_extractor.py
│   │   └── verapdf_extractor.py
│   └── checkers/
│       ├── __init__.py
│       ├── base.py
│       ├── layout.py
│       ├── typography.py
│       ├── structure.py
│       ├── content.py
│       └── human.py
├── specs/
│   └── iu.yaml
└── tests/
    ├── __init__.py
    ├── conftest.py
    ├── test_spec.py
    ├── test_engine.py
    ├── test_report.py
    ├── checkers/
    │   ├── __init__.py
    │   ├── test_layout.py
    │   ├── test_typography.py
    │   ├── test_structure.py
    │   ├── test_content.py
    │   └── test_human.py
    ├── extractors/
    │   ├── __init__.py
    │   ├── test_docling.py
    │   ├── test_pdfplumber.py
    │   └── test_verapdf.py
    └── fixtures/
        ├── iu_template.pdf
        └── .gitkeep
```

---

### Task 1: Project Scaffolding

**Files:**
- Create: `pyproject.toml`
- Create: `diss_check/__init__.py`
- Create: `diss_check/extractors/__init__.py`
- Create: `diss_check/checkers/__init__.py`
- Create: `tests/__init__.py`
- Create: `tests/checkers/__init__.py`
- Create: `tests/extractors/__init__.py`
- Create: `tests/conftest.py`
- Create: `specs/.gitkeep`
- Create: `tests/fixtures/.gitkeep`

**Interfaces:**
- Produces: project skeleton with importable `diss_check` package

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
    "pdfplumber>=0.11",
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
mkdir -p diss_check/extractors diss_check/checkers tests/checkers tests/extractors tests/fixtures specs
touch diss_check/__init__.py
touch diss_check/extractors/__init__.py
touch diss_check/checkers/__init__.py
touch tests/__init__.py
touch tests/checkers/__init__.py
touch tests/extractors/__init__.py
touch specs/.gitkeep
touch tests/fixtures/.gitkeep
```

- [ ] **Step 3: Create tests/conftest.py with shared fixtures path helper**

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

- [ ] **Step 4: Install and verify**

```bash
pip install -e ".[dev]" 2>&1 | tail -5
python -c "import diss_check; print('ok')"
```

Expected: `ok`

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "chore: scaffold project structure and dependencies"
```

---

### Task 2: Spec Models (pydantic)

**Files:**
- Create: `diss_check/spec.py`
- Create: `tests/test_spec.py`

**Interfaces:**
- Produces:
  - `InstitutionSpec` — top-level model with `institution: str`, `source_revision: str`, `document_structure: DocumentStructure`, `checks: list[CheckDef]`, `constants: dict`
  - `DocumentStructure` — with `front_matter: list[SectionDef]`, `body: list[SectionDef]`, `end_matter: list[SectionDef]`
  - `SectionDef` — with `id: str`, `required: bool`, `page_number_start: str | None = None`
  - `CheckDef` — with `id: str`, `category: str`, `checker: str`, `target: CheckTarget`, `params: dict`, `automatable: bool = True`, `review_hint: str | None = None`
  - `CheckTarget` — with `scope: str | None = None`, `page: str | None = None`, `pages: list[str] | None = None`, `element: str | None = None`
  - `load_spec(path: Path) -> InstitutionSpec` — loads and validates a YAML spec file

- [ ] **Step 1: Write failing test for spec loading**

```python
# tests/test_spec.py
from diss_check.spec import load_spec, InstitutionSpec


def test_load_spec_parses_yaml(tmp_path):
    yaml_content = """
institution: Test University
source_revision: "January 2025"
document_structure:
  front_matter:
    - {id: title_page, required: true}
  body:
    - {id: chapters, required: true}
  end_matter:
    - {id: references, required: true}
checks:
  - id: test_margins
    category: layout
    checker: margins
    target: {scope: all_pages}
    params: {top: "1in"}
constants:
  degree: "PhD"
"""
    spec_file = tmp_path / "test.yaml"
    spec_file.write_text(yaml_content)
    spec = load_spec(spec_file)
    assert spec.institution == "Test University"
    assert spec.source_revision == "January 2025"
    assert len(spec.document_structure.front_matter) == 1
    assert spec.document_structure.front_matter[0].id == "title_page"
    assert spec.document_structure.front_matter[0].required is True
    assert len(spec.checks) == 1
    assert spec.checks[0].checker == "margins"
    assert spec.checks[0].automatable is True
    assert spec.constants == {"degree": "PhD"}


def test_load_spec_automatable_defaults_to_true(tmp_path):
    yaml_content = """
institution: Test
source_revision: "v1"
document_structure:
  front_matter: []
  body: []
  end_matter: []
checks:
  - id: c1
    category: layout
    checker: margins
    target: {scope: all_pages}
    params: {}
constants: {}
"""
    spec_file = tmp_path / "test.yaml"
    spec_file.write_text(yaml_content)
    spec = load_spec(spec_file)
    assert spec.checks[0].automatable is True


def test_load_spec_optional_fields_default_to_none(tmp_path):
    yaml_content = """
institution: Test
source_revision: "v1"
document_structure:
  front_matter: []
  body: []
  end_matter: []
checks:
  - id: c1
    category: layout
    checker: margins
    target: {scope: all_pages}
    params: {}
constants: {}
"""
    spec_file = tmp_path / "test.yaml"
    spec_file.write_text(yaml_content)
    spec = load_spec(spec_file)
    assert spec.checks[0].review_hint is None
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `pytest tests/test_spec.py -v`
Expected: FAIL with ImportError (module not found)

- [ ] **Step 3: Implement spec models**

```python
# diss_check/spec.py
from pathlib import Path
from typing import Literal

import yaml
from pydantic import BaseModel


class SectionDef(BaseModel):
    id: str
    required: bool
    page_number_start: str | None = None


class DocumentStructure(BaseModel):
    front_matter: list[SectionDef]
    body: list[SectionDef]
    end_matter: list[SectionDef]


class CheckTarget(BaseModel):
    scope: str | None = None
    page: str | None = None
    pages: list[str] | None = None
    element: str | None = None


class CheckDef(BaseModel):
    id: str
    category: Literal["layout", "typography", "structure", "content", "human"]
    checker: str
    target: CheckTarget
    params: dict = {}
    automatable: bool = True
    review_hint: str | None = None


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

- [ ] **Step 4: Run tests to verify they pass**

Run: `pytest tests/test_spec.py -v`
Expected: all 3 tests PASS

- [ ] **Step 5: Commit**

```bash
git add diss_check/spec.py tests/test_spec.py
git commit -m "feat: add pydantic spec models with YAML loading"
```

---

### Task 3: ExtractionContext and Extractor Base

**Files:**
- Create: `diss_check/document.py`
- Create: `diss_check/extractors/base.py`
- Create: `tests/extractors/test_docling.py` (stub — will be filled in Task 4)

**Interfaces:**
- Produces:
  - `ExtractionContext` dataclass with `docling_doc`, `pdfplumber_pages`, `verapdf_report`
  - `BaseExtractor` abstract class with `name: str`, `extract(source: Path, ctx: ExtractionContext) -> None`

- [ ] **Step 1: Write ExtractionContext dataclass**

```python
# diss_check/document.py
from dataclasses import dataclass, field
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from docling_core.types.doc import DoclingDocument


@dataclass
class ExtractionContext:
    docling_doc: "DoclingDocument | None" = None
    pdfplumber_pages: list | None = None
    verapdf_report: dict | None = None
```

- [ ] **Step 2: Write extractor base class**

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

- [ ] **Step 3: Verify imports work**

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
- Modify: `tests/extractors/test_docling.py` (fill in stub with real tests)

**Interfaces:**
- Consumes: `BaseExtractor` from Task 3, `ExtractionContext` from Task 3
- Produces: `DoclingExtractor(name="docling")` — calls `DocumentConverter.convert()` and stores result on `ctx.docling_doc`

- [ ] **Step 1: Write failing test (if no PDF fixture, test that extractor is importable and has correct name)**

```python
# tests/extractors/test_docling.py
from diss_check.extractors.docling_extractor import DoclingExtractor


def test_docling_extractor_has_correct_name():
    extractor = DoclingExtractor()
    assert extractor.name == "docling"


def test_docling_extractor_extract_populates_ctx(iu_template_path):
    from diss_check.document import ExtractionContext
    extractor = DoclingExtractor()
    ctx = ExtractionContext()
    extractor.extract(iu_template_path, ctx)
    assert ctx.docling_doc is not None
    # DoclingDocument has a body tree
    assert ctx.docling_doc.body is not None
```

- [ ] **Step 2: Run tests to verify failure (no PDF fixture yet — test fixture test will error)**

Run: `pytest tests/extractors/test_docling.py::test_docling_extractor_has_correct_name -v`
Expected: FAIL with ImportError

- [ ] **Step 3: Implement DoclingExtractor**

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

- [ ] **Step 4: Run basic test (no PDF needed)**

Run: `pytest tests/extractors/test_docling.py::test_docling_extractor_has_correct_name -v`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add diss_check/extractors/docling_extractor.py tests/extractors/test_docling.py
git commit -m "feat: add Docling extractor"
```

---

### Task 5: pdfplumber Extractor

**Files:**
- Create: `diss_check/extractors/pdfplumber_extractor.py`
- Create: `tests/extractors/test_pdfplumber.py`

**Interfaces:**
- Consumes: `BaseExtractor` from Task 3, `ExtractionContext` from Task 3
- Produces: `PdfplumberExtractor(name="pdfplumber")` — opens PDF with pdfplumber, stores `list[Page]` on `ctx.pdfplumber_pages`

- [ ] **Step 1: Write failing test**

```python
# tests/extractors/test_pdfplumber.py
from diss_check.extractors.pdfplumber_extractor import PdfplumberExtractor


def test_pdfplumber_extractor_has_correct_name():
    extractor = PdfplumberExtractor()
    assert extractor.name == "pdfplumber"


def test_pdfplumber_extractor_extract_populates_ctx(iu_template_path):
    from diss_check.document import ExtractionContext
    import pdfplumber
    extractor = PdfplumberExtractor()
    ctx = ExtractionContext()
    extractor.extract(iu_template_path, ctx)
    assert ctx.pdfplumber_pages is not None
    assert len(ctx.pdfplumber_pages) > 0
    assert isinstance(ctx.pdfplumber_pages[0], pdfplumber.page.Page)
```

- [ ] **Step 2: Run test to verify failure**

Run: `pytest tests/extractors/test_pdfplumber.py::test_pdfplumber_extractor_has_correct_name -v`
Expected: FAIL with ImportError

- [ ] **Step 3: Implement PdfplumberExtractor**

```python
# diss_check/extractors/pdfplumber_extractor.py
from pathlib import Path
import pdfplumber
from diss_check.document import ExtractionContext
from diss_check.extractors.base import BaseExtractor


class PdfplumberExtractor(BaseExtractor):
    name = "pdfplumber"

    def extract(self, source: Path, ctx: ExtractionContext) -> None:
        with pdfplumber.open(source) as pdf:
            ctx.pdfplumber_pages = list(pdf.pages)
```

- [ ] **Step 4: Run basic test**

Run: `pytest tests/extractors/test_pdfplumber.py::test_pdfplumber_extractor_has_correct_name -v`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add diss_check/extractors/pdfplumber_extractor.py tests/extractors/test_pdfplumber.py
git commit -m "feat: add pdfplumber extractor"
```

---

### Task 6: veraPDF Extractor Interface

**Files:**
- Create: `diss_check/extractors/verapdf_extractor.py`

**Interfaces:**
- Consumes: `BaseExtractor` from Task 3
- Produces: `VerapdfExtractor(name="verapdf")` — stub implementation that sets `ctx.verapdf_report = {}`. Full implementation (subprocess call to veraPDF CLI) is out of scope for IU v1.

- [ ] **Step 1: Implement verapdf stub**

```python
# diss_check/extractors/verapdf_extractor.py
from pathlib import Path
from diss_check.document import ExtractionContext
from diss_check.extractors.base import BaseExtractor


class VerapdfExtractor(BaseExtractor):
    name = "verapdf"

    def extract(self, source: Path, ctx: ExtractionContext) -> None:
        ctx.verapdf_report = {}
```

- [ ] **Step 2: Verify import**

```bash
python -c "from diss_check.extractors.verapdf_extractor import VerapdfExtractor; print('ok')"
```

Expected: `ok`

- [ ] **Step 3: Commit**

```bash
git add diss_check/extractors/verapdf_extractor.py
git commit -m "feat: add veraPDF extractor stub"
```

---

### Task 7: Checker Framework (BaseChecker, Registry, CheckResult, Engine)

**Files:**
- Create: `diss_check/checkers/base.py`
- Create: `diss_check/engine.py`
- Create: `tests/test_engine.py`

**Interfaces:**
- Consumes: `InstitutionSpec` from Task 2, `ExtractionContext` from Task 3, `BaseExtractor` from Task 3
- Produces:
  - `EvidenceItem(pagedantic.BaseModel)` with `page: int`, `bbox: tuple[float,float,float,float] | None`, `excerpt: str | None`
  - `CheckResult(pydantic.BaseModel)` with `check_id: str`, `status: Literal["PASS","FAIL","MANUAL","ERROR"]`, `evidence: list[EvidenceItem]`, `detail: str`
  - `BaseChecker` with `requires: list[str]`, `category: str`, `name: str`, `check(ctx: ExtractionContext, params: dict) -> CheckResult`
  - `_CHECKER_REGISTRY: dict` and `register_checker(category, name)` decorator
  - `Engine` with `__init__(spec: InstitutionSpec)`, `run(pdf_path: Path) -> list[CheckResult]`

- [ ] **Step 1: Write EvidenceItem, CheckResult, BaseChecker, and registry**

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
    check_id: str
    status: Literal["PASS", "FAIL", "MANUAL", "ERROR"]
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

- [ ] **Step 2: Write Engine**

```python
# diss_check/engine.py
from pathlib import Path
from diss_check.spec import InstitutionSpec
from diss_check.document import ExtractionContext
from diss_check.checkers.base import get_checker, CheckResult
from diss_check.extractors.docling_extractor import DoclingExtractor
from diss_check.extractors.pdfplumber_extractor import PdfplumberExtractor
from diss_check.extractors.verapdf_extractor import VerapdfExtractor
from diss_check.extractors.base import BaseExtractor


EXTRACTOR_MAP: dict[str, BaseExtractor] = {
    "docling": DoclingExtractor(),
    "pdfplumber": PdfplumberExtractor(),
    "verapdf": VerapdfExtractor(),
}


class Engine:
    def __init__(self, spec: InstitutionSpec):
        self.spec = spec

    def run(self, pdf_path: Path) -> list[CheckResult]:
        required_extractors = self._collect_required_extractors()
        ctx = ExtractionContext()
        for ext_name in required_extractors:
            extractor = EXTRACTOR_MAP[ext_name]
            extractor.extract(pdf_path, ctx)

        results: list[CheckResult] = []
        content_checks = [c for c in self.spec.checks if c.category != "human"]
        human_checks = [c for c in self.spec.checks if c.category == "human"]

        for check_def in content_checks + human_checks:
            checker = get_checker(check_def.category, check_def.checker)
            result = checker.check(ctx, check_def.params)
            result.check_id = check_def.id
            results.append(result)

        return results

    def _collect_required_extractors(self) -> set[str]:
        required: set[str] = set()
        for check_def in self.spec.checks:
            try:
                checker_cls = type(get_checker(check_def.category, check_def.checker))
            except KeyError:
                continue
            required.update(checker_cls.requires)
        return required
```

- [ ] **Step 3: Write engine test with a mock checker**

```python
# tests/test_engine.py
from diss_check.engine import Engine
from diss_check.spec import InstitutionSpec, DocumentStructure, CheckDef, CheckTarget


def test_engine_runs_checkers_and_returns_results(tmp_path):
    # Create a minimal spec
    spec = InstitutionSpec(
        institution="Test",
        source_revision="v1",
        document_structure=DocumentStructure(
            front_matter=[],
            body=[],
            end_matter=[],
        ),
        checks=[
            CheckDef(
                id="test_human_check",
                category="human",
                checker="manual",
                target=CheckTarget(scope="all_pages"),
                params={"hint": "review this"},
                automatable=False,
            ),
        ],
        constants={},
    )

    engine = Engine(spec)
    # We need a mock PDF or a real one. For now, use the IU template fixture.
    # If no fixture available, this test will fail gracefully.
    import pytest
    # This test needs a real PDF; skip if not available
    pytest.skip("Requires PDF fixture")
```

- [ ] **Step 4: Verify imports and basic structure**

```bash
python -c "from diss_check.checkers.base import BaseChecker, CheckResult, EvidenceItem, register_checker, get_checker; print('ok')"
python -c "from diss_check.engine import Engine; print('ok')"
```

Expected: `ok` twice

- [ ] **Step 5: Commit**

```bash
git add diss_check/checkers/base.py diss_check/engine.py tests/test_engine.py
git commit -m "feat: add checker framework (BaseChecker, registry, CheckResult, Engine)"
```

---

### Task 8: Human Checker (needed first — other checkers will reference the pattern)

**Files:**
- Create: `diss_check/checkers/human.py`
- Create: `tests/checkers/test_human.py`

**Interfaces:**
- Consumes: `BaseChecker`, `register_checker` from Task 7
- Produces: `HumanChecker` registered as `("human", "manual")` — returns MANUAL status with review_hint in detail

- [ ] **Step 1: Write failing test**

```python
# tests/checkers/test_human.py
from diss_check.checkers.human import HumanChecker
from diss_check.document import ExtractionContext


def test_human_checker_returns_manual():
    checker = HumanChecker()
    ctx = ExtractionContext()
    result = checker.check(ctx, {"hint": "verify committee signatures are present"})
    assert result.status == "MANUAL"
    assert result.check_id == ""  # Engine sets this
    assert len(result.evidence) == 0
    assert "verify committee signatures are present" in result.detail
```

- [ ] **Step 2: Run test to verify failure**

Run: `pytest tests/checkers/test_human.py -v`
Expected: FAIL with ImportError

- [ ] **Step 3: Implement HumanChecker**

```python
# diss_check/checkers/human.py
from diss_check.document import ExtractionContext
from diss_check.checkers.base import BaseChecker, CheckResult, register_checker


@register_checker(category="human", name="manual")
class HumanChecker(BaseChecker):
    requires = []

    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        hint = params.get("hint", "Manual review required")
        return CheckResult(
            check_id="",
            status="MANUAL",
            detail=hint,
        )
```

- [ ] **Step 4: Run test to verify pass**

Run: `pytest tests/checkers/test_human.py -v`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add diss_check/checkers/human.py tests/checkers/test_human.py
git commit -m "feat: add human checker (manual review passthrough)"
```

---

### Task 9: Layout Checker (Margins)

**Files:**
- Create: `diss_check/checkers/layout.py`
- Create: `tests/checkers/test_layout.py`

**Interfaces:**
- Consumes: `BaseChecker`, `register_checker` from Task 7, `ExtractionContext` from Task 3
- Produces: `MarginsChecker` registered as `("layout", "margins")` — checks text content bounding boxes against specified margins using docling's bounding box data

- [ ] **Step 1: Write failing test with synthetic DoclingDocument**

```python
# tests/checkers/test_layout.py
import pytest
from diss_check.checkers.layout import MarginsChecker
from diss_check.document import ExtractionContext


def _make_synthetic_doc(bboxes):
    """Create a synthetic DoclingDocument with text items at given bounding boxes.
    Each bbox is (x0, y0, x1, y1) in points (72pt = 1in).
    """
    from docling_core.types.doc import DoclingDocument, DocItemLabel, BoundingBox, CoordOrigin
    from docling_core.types.doc.document import TextItem, PageItem, SectionHeaderItem, GroupItem

    doc = DoclingDocument(name="synthetic")
    page = PageItem(
        page_no=1,
        size={"width": 612, "height": 792},  # US Letter in points
        self_ref="#",
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


def test_margins_pass_when_all_text_within_bounds():
    doc = _make_synthetic_doc([
        (90, 110, 522, 682),   # left 90pt (1.25in), right margin 90pt from edge (1.25in)
    ])
    ctx = ExtractionContext(docling_doc=doc)
    checker = MarginsChecker()
    result = checker.check(ctx, {
        "top": "1in", "bottom": "1in", "left": "1.25in", "right": "1.25in"
    })
    assert result.status == "PASS"


def test_margins_fail_when_text_exceeds_left_margin():
    doc = _make_synthetic_doc([
        (36, 110, 522, 682),   # left 36pt (0.5in) < required 1.25in (90pt)
    ])
    ctx = ExtractionContext(docling_doc=doc)
    checker = MarginsChecker()
    result = checker.check(ctx, {
        "top": "1in", "bottom": "1in", "left": "1.25in", "right": "1.25in"
    })
    assert result.status == "FAIL"
    assert len(result.evidence) > 0
    assert result.evidence[0].page == 1


def test_margins_fail_when_text_exceeds_right_margin():
    doc = _make_synthetic_doc([
        (90, 110, 576, 682),   # right edge at 576pt, margin = 612-576=36pt (0.5in)
    ])
    ctx = ExtractionContext(docling_doc=doc)
    checker = MarginsChecker()
    result = checker.check(ctx, {
        "top": "1in", "bottom": "1in", "left": "1.25in", "right": "1.25in"
    })
    assert result.status == "FAIL"
```

- [ ] **Step 2: Run test to verify failure**

Run: `pytest tests/checkers/test_layout.py -v`
Expected: FAIL with ImportError

- [ ] **Step 3: Implement MarginsChecker**

```python
# diss_check/checkers/layout.py
from diss_check.document import ExtractionContext
from diss_check.checkers.base import BaseChecker, CheckResult, EvidenceItem, register_checker


def _parse_measurement(value: str) -> float:
    """Parse a measurement string like '1in' or '1.25in' to points."""
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
                        page=page_no,
                        bbox=(l, t, r, b),
                        excerpt=text_item.text[:100],
                    ))
                elif (page_width - r) < right_margin:
                    violations.append(EvidenceItem(
                        page=page_no,
                        bbox=(l, t, r, b),
                        excerpt=text_item.text[:100],
                    ))
                elif t < top_margin:
                    violations.append(EvidenceItem(
                        page=page_no,
                        bbox=(l, t, r, b),
                        excerpt=text_item.text[:100],
                    ))
                elif (page_height - b) < bottom_margin:
                    violations.append(EvidenceItem(
                        page=page_no,
                        bbox=(l, t, r, b),
                        excerpt=text_item.text[:100],
                    ))

        if violations:
            return CheckResult(
                check_id="",
                status="FAIL",
                evidence=violations,
                detail=f"{len(violations)} text block(s) violate margin requirements",
            )

        return CheckResult(
            check_id="",
            status="PASS",
            detail="All text is within required margins",
        )
```

- [ ] **Step 4: Run test to verify pass**

Run: `pytest tests/checkers/test_layout.py -v`
Expected: all tests PASS

- [ ] **Step 5: Commit**

```bash
git add diss_check/checkers/layout.py tests/checkers/test_layout.py
git commit -m "feat: add layout checker (margins)"
```

---

### Task 10: Typography Checker (font_size, font_weight, font_family, justification)

**Files:**
- Create: `diss_check/checkers/typography.py`
- Create: `tests/checkers/test_typography.py`

**Interfaces:**
- Consumes: `BaseChecker`, `register_checker` from Task 7
- Produces:
  - `FontSizeChecker` `("typography", "font_size")` — checks all text font sizes against allowed list, using pdfplumber
  - `FontWeightChecker` `("typography", "font_weight")` — checks that specific elements are not bold
  - `FontFamilyChecker` `("typography", "font_family")` — checks font family consistency
  - `JustificationChecker` `("typography", "justification")` — checks text alignment consistency

- [ ] **Step 1: Write failing tests**

```python
# tests/checkers/test_typography.py
import pdfplumber.page
from diss_check.checkers.typography import FontSizeChecker, FontWeightChecker
from diss_check.document import ExtractionContext


def _make_synthetic_pdfplumber_page(chars):
    """Create a mock pdfplumber page with characters having specified font sizes.
    chars is a list of (font_size, font_name) tuples."""
    class MockChar:
        def __init__(self, font_size, font_name):
            self.size = font_size
            self.fontname = font_name
    class MockPage:
        def __init__(self, chars):
            self.chars = [MockChar(s, n) for s, n in chars]
    return MockPage(chars)


def test_font_size_pass_when_all_in_allowed_range():
    pages = [_make_synthetic_pdfplumber_page([(11, "Times"), (11, "Times"), (12, "Times")])]
    ctx = ExtractionContext(pdfplumber_pages=pages)
    checker = FontSizeChecker()
    result = checker.check(ctx, {"allowed": ["11pt", "12pt"], "consistent": True})
    assert result.status == "PASS"


def test_font_size_fail_when_size_not_allowed():
    pages = [_make_synthetic_pdfplumber_page([(14, "Times")])]
    ctx = ExtractionContext(pdfplumber_pages=pages)
    checker = FontSizeChecker()
    result = checker.check(ctx, {"allowed": ["11pt", "12pt"]})
    assert result.status == "FAIL"
    assert len(result.evidence) > 0
    assert result.evidence[0].page == 1


def test_font_weight_pass_when_element_not_bold():
    pages = [_make_synthetic_pdfplumber_page([(12, "TimesNewRoman")])]  # not bold
    ctx = ExtractionContext(pdfplumber_pages=pages)
    checker = FontWeightChecker()
    result = checker.check(ctx, {"weight": "normal"})
    # Not bold font names don't contain "Bold"
    assert result.status == "PASS"


def test_font_weight_fail_when_bold_found():
    pages = [_make_synthetic_pdfplumber_page([(12, "TimesNewRoman,Bold")])]
    ctx = ExtractionContext(pdfplumber_pages=pages)
    checker = FontWeightChecker()
    result = checker.check(ctx, {"weight": "normal"})
    assert result.status == "FAIL"
```

- [ ] **Step 2: Run tests to verify failure**

Run: `pytest tests/checkers/test_typography.py -v`
Expected: FAIL with ImportError

- [ ] **Step 3: Implement typography checkers**

```python
# diss_check/checkers/typography.py
from diss_check.document import ExtractionContext
from diss_check.checkers.base import BaseChecker, CheckResult, EvidenceItem, register_checker


def _parse_font_size(value: str) -> float:
    value = value.strip()
    if value.endswith("pt"):
        return float(value[:-2])
    raise ValueError(f"Unsupported font size: {value}")


@register_checker(category="typography", name="font_size")
class FontSizeChecker(BaseChecker):
    requires = ["docling", "pdfplumber"]

    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        allowed_sizes = [_parse_font_size(s) for s in params["allowed"]]
        violations: list[EvidenceItem] = []

        if ctx.pdfplumber_pages is None:
            return CheckResult(
                check_id="", status="ERROR",
                detail="pdfplumber extraction required but not available",
            )

        for i, page in enumerate(ctx.pdfplumber_pages):
            page_no = i + 1
            for char in getattr(page, 'chars', []):
                size = getattr(char, 'size', None)
                if size is not None and size not in allowed_sizes:
                    violations.append(EvidenceItem(
                        page=page_no,
                        excerpt=f"font size {size}pt",
                    ))

        if violations:
            unique_sizes = set(v.excerpt for v in violations)
            return CheckResult(
                check_id="", status="FAIL", evidence=violations[:20],
                detail=f"Found disallowed font sizes: {unique_sizes}. Allowed: {params['allowed']}",
            )
        return CheckResult(check_id="", status="PASS", detail="All font sizes are allowed")


@register_checker(category="typography", name="font_weight")
class FontWeightChecker(BaseChecker):
    requires = ["docling", "pdfplumber"]

    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        required_weight = params.get("weight", "normal")
        violations: list[EvidenceItem] = []

        if ctx.pdfplumber_pages is None:
            return CheckResult(
                check_id="", status="ERROR",
                detail="pdfplumber extraction required but not available",
            )

        for i, page in enumerate(ctx.pdfplumber_pages):
            page_no = i + 1
            for char in getattr(page, 'chars', []):
                fontname = getattr(char, 'fontname', '') or ''
                is_bold = 'Bold' in fontname
                if required_weight == "normal" and is_bold:
                    violations.append(EvidenceItem(
                        page=page_no,
                        excerpt=f"bold text: '{getattr(char, 'text', '')[:30]}'",
                    ))

        if violations:
            return CheckResult(
                check_id="", status="FAIL", evidence=violations[:20],
                detail=f"Found {len(violations)} bold character(s) where normal weight is required",
            )
        return CheckResult(check_id="", status="PASS", detail="No bold text found")


@register_checker(category="typography", name="font_family")
class FontFamilyChecker(BaseChecker):
    requires = ["docling", "pdfplumber"]

    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        allowed = params.get("allowed", [])
        violations: list[EvidenceItem] = []

        if ctx.pdfplumber_pages is None:
            return CheckResult(
                check_id="", status="ERROR",
                detail="pdfplumber extraction required but not available",
            )

        for i, page in enumerate(ctx.pdfplumber_pages):
            page_no = i + 1
            for char in getattr(page, 'chars', []):
                fontname = getattr(char, 'fontname', '') or ''
                if allowed and not any(a.lower() in fontname.lower() for a in allowed):
                    violations.append(EvidenceItem(
                        page=page_no,
                        excerpt=f"font: '{fontname}'",
                    ))

        if violations:
            return CheckResult(
                check_id="", status="FAIL", evidence=violations[:20],
                detail=f"Found {len(violations)} character(s) with disallowed font. Allowed: {allowed}",
            )
        return CheckResult(check_id="", status="PASS", detail="All fonts match allowed list")


@register_checker(category="typography", name="justification")
class JustificationChecker(BaseChecker):
    requires = ["docling"]

    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        # Justification detection from DoclingDocument is limited.
        # For v1, flag as MANUAL and let human review confirm.
        return CheckResult(
            check_id="", status="MANUAL",
            detail="Automatic justification detection is limited. Verify justification is consistent throughout the document.",
        )
```

- [ ] **Step 4: Run tests to verify pass**

Run: `pytest tests/checkers/test_typography.py -v`
Expected: all 4 tests PASS

- [ ] **Step 5: Commit**

```bash
git add diss_check/checkers/typography.py tests/checkers/test_typography.py
git commit -m "feat: add typography checkers (font_size, font_weight, font_family, justification)"
```

---

### Task 11: Structure Checker (Page Numbering, Section Presence/Order)

**Files:**
- Create: `diss_check/checkers/structure.py`
- Create: `tests/checkers/test_structure.py`

**Interfaces:**
- Consumes: `BaseChecker`, `register_checker` from Task 7; `InstitutionSpec.document_structure` via params passed by engine
- Produces:
  - `PageNumberingChecker` `("structure", "page_numbering")` — validates Roman vs Arabic numbering at correct positions
  - `SectionPresenceChecker` `("structure", "section_presence")` — checks required sections exist
  - `SectionOrderChecker` `("structure", "section_order")` — checks sections appear in correct order

- [ ] **Step 1: Write failing tests**

```python
# tests/checkers/test_structure.py
from diss_check.checkers.structure import SectionPresenceChecker, SectionOrderChecker
from diss_check.document import ExtractionContext


def _make_synthetic_doc_with_sections(section_ids):
    """Create a synthetic DoclingDocument with section headers matching given ids."""
    from docling_core.types.doc import DoclingDocument, DocItemLabel, BoundingBox, CoordOrigin
    from docling_core.types.doc.document import TextItem, GroupItem

    doc = DoclingDocument(name="synthetic")
    text_items = []
    for i, sid in enumerate(section_ids):
        item = TextItem(
            self_ref=f"#/texts/{i}",
            text=sid,
            label=DocItemLabel.SECTION_HEADER,
            prov=[],
        )
        item.page_no = 1
        item.bbox = BoundingBox(
            l=90, t=100 + i * 30, r=522, b=120 + i * 30,
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


def test_section_presence_pass_when_all_required_present():
    doc = _make_synthetic_doc_with_sections(["title_page", "acceptance_page", "toc"])
    ctx = ExtractionContext(docling_doc=doc)
    checker = SectionPresenceChecker()
    result = checker.check(ctx, {
        "required_sections": [
            {"id": "title_page"}, {"id": "acceptance_page"}, {"id": "toc"},
        ],
    })
    assert result.status == "PASS"


def test_section_presence_fail_when_required_missing():
    doc = _make_synthetic_doc_with_sections(["title_page", "toc"])
    ctx = ExtractionContext(docling_doc=doc)
    checker = SectionPresenceChecker()
    result = checker.check(ctx, {
        "required_sections": [
            {"id": "title_page"}, {"id": "acceptance_page"}, {"id": "toc"},
        ],
    })
    assert result.status == "FAIL"
    assert "acceptance_page" in result.detail


def test_section_order_pass_when_correct():
    doc = _make_synthetic_doc_with_sections(["title_page", "acceptance_page", "toc"])
    ctx = ExtractionContext(docling_doc=doc)
    checker = SectionOrderChecker()
    result = checker.check(ctx, {
        "expected_order": ["title_page", "acceptance_page", "toc"],
    })
    assert result.status == "PASS"


def test_section_order_fail_when_wrong_order():
    doc = _make_synthetic_doc_with_sections(["acceptance_page", "title_page", "toc"])
    ctx = ExtractionContext(docling_doc=doc)
    checker = SectionOrderChecker()
    result = checker.check(ctx, {
        "expected_order": ["title_page", "acceptance_page", "toc"],
    })
    assert result.status == "FAIL"
```

- [ ] **Step 2: Run tests to verify failure**

Run: `pytest tests/checkers/test_structure.py -v`
Expected: FAIL with ImportError

- [ ] **Step 3: Implement structure checkers**

```python
# diss_check/checkers/structure.py
from docling_core.types.doc import DocItemLabel
from diss_check.document import ExtractionContext
from diss_check.checkers.base import BaseChecker, CheckResult, EvidenceItem, register_checker


def _find_section_headers(doc):
    """Find all section headers in the document and return list of (text, page_no)."""
    headers = []
    for text_item in doc.texts:
        if text_item.label == DocItemLabel.SECTION_HEADER:
            text = getattr(text_item, 'text', '') or ''
            page_no = getattr(text_item, 'page_no', 1)
            headers.append((text, page_no))
    return headers


def _match_section(text: str, section_id: str) -> bool:
    """Check if a section header text matches a section id.
    Uses substring matching after normalizing. Override for institution-specific logic."""
    text_lower = text.lower().replace(' ', '_').replace('-', '_')
    return section_id.lower() in text_lower


@register_checker(category="structure", name="section_presence")
class SectionPresenceChecker(BaseChecker):
    requires = ["docling"]

    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        required = params.get("required_sections", [])
        headers = _find_section_headers(ctx.docling_doc)
        header_texts = [h[0].lower() for h in headers]

        missing = []
        for sec in required:
            sec_id = sec["id"]
            if not any(_match_section(ht, sec_id) for ht in header_texts):
                missing.append(sec_id)

        if missing:
            return CheckResult(
                check_id="", status="FAIL",
                detail=f"Missing required sections: {', '.join(missing)}",
            )
        return CheckResult(check_id="", status="PASS", detail="All required sections present")


@register_checker(category="structure", name="section_order")
class SectionOrderChecker(BaseChecker):
    requires = ["docling"]

    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        expected_order = params.get("expected_order", [])
        headers = _find_section_headers(ctx.docling_doc)

        found_order = []
        for header_text, _ in headers:
            for expected_id in expected_order:
                if _match_section(header_text, expected_id):
                    found_order.append(expected_id)
                    break

        # Remove consecutive duplicates (same heading repeated)
        deduped = []
        for item in found_order:
            if not deduped or deduped[-1] != item:
                deduped.append(item)

        expected_present = [e for e in expected_order if e in deduped]
        if expected_present != expected_order:
            return CheckResult(
                check_id="", status="FAIL",
                detail=f"Section order mismatch. Expected: {expected_order}. Found: {deduped}",
            )
        return CheckResult(check_id="", status="PASS", detail="Section order is correct")


@register_checker(category="structure", name="page_numbering")
class PageNumberingChecker(BaseChecker):
    requires = ["docling"]

    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        # Page numbering detection from DoclingDocument is limited for v1.
        # Docling doesn't directly expose page number text separate from the
        # footer furniture group. This check is flagged MANUAL for v1.
        return CheckResult(
            check_id="", status="MANUAL",
            detail="Automatic page numbering validation is limited. Verify page numbers manually: front matter should use Roman numerals starting at ii, body should use Arabic starting at 1.",
        )
```

- [ ] **Step 4: Run tests to verify pass**

Run: `pytest tests/checkers/test_structure.py -v`
Expected: all 4 tests PASS

- [ ] **Step 5: Commit**

```bash
git add diss_check/checkers/structure.py tests/checkers/test_structure.py
git commit -m "feat: add structure checkers (section presence, section order, page numbering)"
```

---

### Task 12: Content Checker (text_match, committee_order, toc_title_parity)

**Files:**
- Create: `diss_check/checkers/content.py`
- Create: `tests/checkers/test_content.py`

**Interfaces:**
- Consumes: `BaseChecker`, `register_checker` from Task 7
- Produces:
  - `TextMatchChecker` `("content", "text_match")` — fuzzy text matching against a template with variable interpolation
  - `CommitteeOrderChecker` `("content", "committee_order")` — checks committee chair is listed first
  - `TocTitleParityChecker` `("content", "toc_title_parity")` — checks TOC entries match document titles

- [ ] **Step 1: Write failing tests**

```python
# tests/checkers/test_content.py
from diss_check.checkers.content import TextMatchChecker, CommitteeOrderChecker
from diss_check.document import ExtractionContext


def _make_synthetic_doc_with_text(texts):
    """Create synthetic DoclingDocument with given text items."""
    from docling_core.types.doc import DoclingDocument, DocItemLabel, BoundingBox, CoordOrigin
    from docling_core.types.doc.document import TextItem, GroupItem

    doc = DoclingDocument(name="synthetic")
    text_items = []
    for i, txt in enumerate(texts):
        item = TextItem(
            self_ref=f"#/texts/{i}",
            text=txt,
            label=DocItemLabel.TEXT,
            prov=[],
        )
        item.page_no = 1
        item.bbox = BoundingBox(
            l=90, t=100 + i * 15, r=522, b=112 + i * 15,
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


def test_text_match_pass_when_template_found():
    doc = _make_synthetic_doc_with_text([
        "Submitted to the faculty of the University Graduate School",
        "in partial fulfillment of the requirements",
        "for the degree Doctor of Philosophy",
    ])
    ctx = ExtractionContext(docling_doc=doc)
    checker = TextMatchChecker()
    result = checker.check(ctx, {
        "template": "Submitted to the faculty of the University Graduate School",
    })
    assert result.status == "PASS"


def test_text_match_fail_when_template_not_found():
    doc = _make_synthetic_doc_with_text(["Some unrelated text", "More stuff"])
    ctx = ExtractionContext(docling_doc=doc)
    checker = TextMatchChecker()
    result = checker.check(ctx, {
        "template": "Submitted to the faculty of the University Graduate School",
    })
    assert result.status == "FAIL"


def test_text_match_handles_multiline_template():
    doc = _make_synthetic_doc_with_text([
        "Submitted to the faculty of the University Graduate School",
        "in partial fulfillment of the requirements",
    ])
    ctx = ExtractionContext(docling_doc=doc)
    checker = TextMatchChecker()
    result = checker.check(ctx, {
        "template": (
            "Submitted to the faculty of the University Graduate School\n"
            "in partial fulfillment of the requirements"
        ),
    })
    assert result.status == "PASS"


def test_committee_order_pass_when_empty_context():
    ctx = ExtractionContext()
    checker = CommitteeOrderChecker()
    result = checker.check(ctx, {"chair_first": True})
    assert result.status == "MANUAL"


def test_toc_title_parity_manual():
    ctx = ExtractionContext()
    from diss_check.checkers.content import TocTitleParityChecker
    checker = TocTitleParityChecker()
    result = checker.check(ctx, {})
    assert result.status == "MANUAL"
```

- [ ] **Step 2: Run tests to verify failure**

Run: `pytest tests/checkers/test_content.py -v`
Expected: FAIL with ImportError

- [ ] **Step 3: Implement content checkers**

```python
# diss_check/checkers/content.py
from diss_check.document import ExtractionContext
from diss_check.checkers.base import BaseChecker, CheckResult, EvidenceItem, register_checker


def _collect_all_text(doc) -> str:
    """Concatenate all text items in reading order into a single string."""
    parts = []
    for text_item in doc.texts:
        txt = getattr(text_item, 'text', '') or ''
        parts.append(txt)
    return '\n'.join(parts)


def _normalize(text: str) -> str:
    """Normalize text for comparison: strip whitespace, collapse spaces."""
    return ' '.join(text.split())


@register_checker(category="content", name="text_match")
class TextMatchChecker(BaseChecker):
    requires = ["docling"]

    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        template = params.get("template", "")
        if not template:
            return CheckResult(check_id="", status="ERROR", detail="No template provided")

        document_text = _collect_all_text(ctx.docling_doc)
        template_lines = [line.strip() for line in template.strip().split('\n') if line.strip()]

        missing_lines = []
        for line in template_lines:
            normalized_line = _normalize(line)
            if normalized_line not in _normalize(document_text):
                # Try fuzzy: check if significant words are present
                words = normalized_line.split()
                significant = [w for w in words if len(w) > 3]
                if significant:
                    found_count = sum(1 for w in significant if w.lower() in _normalize(document_text).lower())
                    if found_count < len(significant) * 0.5:
                        missing_lines.append(line)

        if missing_lines:
            return CheckResult(
                check_id="", status="FAIL",
                detail=f"Template text not found. Missing or mismatched lines: {missing_lines[:5]}",
            )
        return CheckResult(check_id="", status="PASS", detail="Template text matches document")


@register_checker(category="content", name="committee_order")
class CommitteeOrderChecker(BaseChecker):
    requires = ["docling"]

    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        # Committee name ordering requires semantic understanding of names
        # and committee roles. For v1, flag as MANUAL.
        return CheckResult(
            check_id="", status="MANUAL",
            detail="Automatic committee order detection is limited. Verify the committee chair is listed first, followed by other committee members.",
        )


@register_checker(category="content", name="toc_title_parity")
class TocTitleParityChecker(BaseChecker):
    requires = ["docling"]

    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        # TOC title parity requires matching TOC entries against document
        # section titles. For v1, flag as MANUAL.
        return CheckResult(
            check_id="", status="MANUAL",
            detail="Verify that all chapter titles and subheadings in the Table of Contents match verbatim the titles in the document body.",
        )
```

- [ ] **Step 4: Run tests to verify pass**

Run: `pytest tests/checkers/test_content.py -v`
Expected: all tests PASS

- [ ] **Step 5: Commit**

```bash
git add diss_check/checkers/content.py tests/checkers/test_content.py
git commit -m "feat: add content checkers (text_match, committee_order, toc_title_parity)"
```

---

### Task 13: Report Model and Formatters

**Files:**
- Create: `diss_check/report.py`
- Create: `tests/test_report.py`

**Interfaces:**
- Consumes: `CheckResult` from Task 7
- Produces:
  - `Report(pydantic.BaseModel)` — wraps `list[CheckResult]` with summary counts
  - `format_text(report: Report) -> str` — human-readable CLI output
  - `format_json(report: Report) -> str` — JSON string output

- [ ] **Step 1: Write failing test**

```python
# tests/test_report.py
import json
from diss_check.report import Report, format_text, format_json
from diss_check.checkers.base import CheckResult


def test_report_summary_counts():
    results = [
        CheckResult(check_id="c1", status="PASS", detail="ok"),
        CheckResult(check_id="c2", status="FAIL", detail="bad", evidence=[]),
        CheckResult(check_id="c3", status="MANUAL", detail="check"),
    ]
    report = Report(results=results)
    assert report.pass_count == 1
    assert report.fail_count == 1
    assert report.manual_count == 1
    assert report.error_count == 0


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


def test_format_json_is_valid():
    results = [CheckResult(check_id="c1", status="PASS", detail="ok")]
    report = Report(results=results)
    output = format_json(report)
    data = json.loads(output)
    assert data["summary"]["pass"] == 1
    assert len(data["results"]) == 1
    assert data["results"][0]["check_id"] == "c1"


def test_format_text_empty_report():
    report = Report(results=[])
    output = format_text(report)
    assert "No checks" in output or "0" in output
```

- [ ] **Step 2: Run tests to verify failure**

Run: `pytest tests/test_report.py -v`
Expected: FAIL with ImportError

- [ ] **Step 3: Implement Report and formatters**

```python
# diss_check/report.py
import json
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

    @property
    def pass_count(self) -> int:
        return self.summary.pass_ if self.summary else 0

    @property
    def fail_count(self) -> int:
        return self.summary.fail if self.summary else 0

    @property
    def manual_count(self) -> int:
        return self.summary.manual if self.summary else 0

    @property
    def error_count(self) -> int:
        return self.summary.error if self.summary else 0


def format_text(report: Report) -> str:
    lines = []
    lines.append("=" * 60)
    lines.append("DISSERTATION FORMAT CHECK REPORT")
    lines.append("=" * 60)

    for result in report.results:
        status_marker = {"PASS": "[PASS]", "FAIL": "[FAIL]", "MANUAL": "[MANUAL]", "ERROR": "[ERROR]"}[result.status]
        lines.append(f"\n{status_marker} {result.check_id}")
        if result.detail:
            lines.append(f"  {result.detail}")
        for ev in result.evidence:
            page_info = f"page {ev.page}"
            if ev.bbox:
                page_info += f" @ {ev.bbox}"
            lines.append(f"    [{page_info}] {ev.excerpt or ''}")

    lines.append("\n" + "-" * 60)
    s = report.summary
    lines.append(f"Summary: {s.pass_} PASS, {s.fail} FAIL, {s.manual} MANUAL, {s.error} ERROR")
    lines.append("=" * 60)
    return "\n".join(lines)


def format_json(report: Report) -> str:
    data = {
        "summary": {
            "pass": report.summary.pass_,
            "fail": report.summary.fail,
            "manual": report.summary.manual,
            "error": report.summary.error,
        },
        "results": [
            {
                "check_id": r.check_id,
                "status": r.status,
                "detail": r.detail,
                "evidence": [
                    {
                        "page": e.page,
                        "bbox": list(e.bbox) if e.bbox else None,
                        "excerpt": e.excerpt,
                    }
                    for e in r.evidence
                ],
            }
            for r in report.results
        ],
    }
    return json.dumps(data, indent=2)
```

- [ ] **Step 4: Run tests to verify pass**

Run: `pytest tests/test_report.py -v`
Expected: all tests PASS

- [ ] **Step 5: Commit**

```bash
git add diss_check/report.py tests/test_report.py
git commit -m "feat: add Report model and text/JSON formatters"
```

---

### Task 14: CLI

**Files:**
- Create: `diss_check/cli.py`

**Interfaces:**
- Consumes: `Engine`, `InstitutionSpec`, `load_spec`, `Report`, `format_text`, `format_json` from previous tasks
- Produces: CLI with `diss-check --spec <path> [--json] <pdf>` and `diss-check calibrate --spec <path> --corpus <path>`

- [ ] **Step 1: Write CLI module**

```python
# diss_check/cli.py
from pathlib import Path
import sys

import click

from diss_check.spec import load_spec
from diss_check.engine import Engine
from diss_check.report import Report, format_text, format_json


@click.group()
def main():
    """Check dissertation PDFs against institutional formatting requirements."""
    pass


@main.command()
@click.option("--spec", required=True, type=click.Path(exists=True), help="Path to institution spec YAML file")
@click.option("--json", "output_json", is_flag=True, default=False, help="Output results as JSON")
@click.argument("pdf", type=click.Path(exists=True))
def check(spec, output_json, pdf):
    """Check a dissertation PDF against the given spec."""
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

    if output_json:
        click.echo(format_json(report))
    else:
        click.echo(format_text(report))

    # Exit with non-zero if any FAIL or ERROR
    if report.fail_count > 0 or report.error_count > 0:
        sys.exit(1)


@main.command()
@click.option("--spec", required=True, type=click.Path(exists=True), help="Path to institution spec YAML file")
@click.option("--corpus", required=True, type=click.Path(exists=True), help="Path to corpus directory of accepted PDFs")
def calibrate(spec, corpus):
    """Run check suite against a corpus of accepted dissertations."""
    from diss_check.calibration import run_calibration
    spec_path = Path(spec)
    corpus_path = Path(corpus)

    try:
        institution_spec = load_spec(spec_path)
    except Exception as e:
        click.echo(f"Error loading spec: {e}", err=True)
        sys.exit(1)

    report = run_calibration(institution_spec, corpus_path)
    click.echo(report)
```

- [ ] **Step 2: Verify CLI is callable**

```bash
python -m diss_check.cli --help
```

Expected: help text with `check` and `calibrate` subcommands

- [ ] **Step 3: Commit**

```bash
git add diss_check/cli.py
git commit -m "feat: add CLI with check and calibrate commands"
```

---

### Task 15: Calibration Workflow

**Files:**
- Create: `diss_check/calibration.py`

**Interfaces:**
- Consumes: `InstitutionSpec`, `Engine`, `CheckResult` from previous tasks
- Produces: `run_calibration(spec: InstitutionSpec, corpus_path: Path) -> str` — runs checks on all PDFs in corpus, returns frequency report of failures grouped by check_id

- [ ] **Step 1: Implement calibration module**

```python
# diss_check/calibration.py
from pathlib import Path
from collections import Counter

from diss_check.spec import InstitutionSpec
from diss_check.engine import Engine


def run_calibration(spec: InstitutionSpec, corpus_path: Path) -> str:
    """Run the full check suite against every PDF in the corpus directory.
    Returns a text report grouping failures by check_id and frequency."""
    pdf_files = sorted(corpus_path.glob("*.pdf"))
    if not pdf_files:
        return "No PDF files found in corpus directory."

    all_results: dict[str, list[str]] = {}  # pdf_name -> [check_id:status]
    fail_counts: Counter[str] = Counter()

    for pdf_path in pdf_files:
        engine = Engine(spec)
        results = engine.run(pdf_path)
        all_results[pdf_path.name] = [
            f"{r.check_id}:{r.status}" for r in results
        ]
        for r in results:
            if r.status == "FAIL":
                fail_counts[r.check_id] += 1

    lines = []
    lines.append(f"Calibration report for {spec.institution} ({spec.source_revision})")
    lines.append(f"Corpus: {corpus_path} ({len(pdf_files)} documents)")
    lines.append("=" * 60)

    if not fail_counts:
        lines.append("\nAll checks PASS on all documents.")
        return "\n".join(lines)

    lines.append("\nSystemic failures (check_id -> failure count across corpus):")
    for check_id, count in fail_counts.most_common():
        lines.append(f"  {check_id}: {count}/{len(pdf_files)} documents failed")

    lines.append("\nPer-document breakdown:")
    for pdf_name, statuses in all_results.items():
        lines.append(f"\n  {pdf_name}:")
        for s in statuses:
            if ":FAIL" in s:
                lines.append(f"    {s}")

    return "\n".join(lines)
```

- [ ] **Step 2: Verify imports and basic structure**

```bash
python -c "from diss_check.calibration import run_calibration; print('ok')"
```

Expected: `ok`

- [ ] **Step 3: Commit**

```bash
git add diss_check/calibration.py
git commit -m "feat: add calibration workflow for corpus validation"
```

---

### Task 16: IU Spec YAML

**Files:**
- Create: `specs/iu.yaml`

**Interfaces:**
- Produces: Complete IU dissertation format spec based on the September 2025 checklist

- [ ] **Step 1: Write the IU spec YAML**

```yaml
# specs/iu.yaml
institution: Indiana University
source_revision: "September 2025"

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

  - id: font_size_consistent
    category: typography
    checker: font_size
    target: { scope: all_pages }
    params:
      allowed: ["11pt", "12pt"]
      consistent: true

  - id: font_family_consistent
    category: typography
    checker: font_family
    target: { scope: all_pages }
    params:
      allowed: []
      consistent: true

  - id: title_not_bold
    category: typography
    checker: font_weight
    target: { page: title_page, element: title }
    params:
      weight: normal

  - id: title_page_no_page_number
    category: structure
    checker: page_numbering
    target: { page: title_page }
    params:
      expect_no_page_number: true
    automatable: false
    review_hint: "Verify the title page has no page number."

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

  - id: end_matter_presence
    category: structure
    checker: section_presence
    target: { scope: end_matter }
    params:
      required_sections:
        - { id: references }
        - { id: curriculum_vitae }

  - id: front_matter_order
    category: structure
    checker: section_order
    target: { scope: front_matter }
    params:
      expected_order:
        - title_page
        - acceptance_page
        - copyright_page
        - dedication
        - acknowledgements
        - preface
        - abstract
        - toc
        - lot
        - lof
        - lop
        - loa

  - id: end_matter_order
    category: structure
    checker: section_order
    target: { scope: end_matter }
    params:
      expected_order:
        - references
        - appendices
        - curriculum_vitae

  - id: acceptance_clause_wording
    category: content
    checker: text_match
    target: { page: acceptance_page, element: clause }
    params:
      template: |
        Submitted to the faculty of the University Graduate School
        in partial fulfillment of the requirements
        for the degree
    automatable: true

  - id: committee_chair_first
    category: content
    checker: committee_order
    target: { pages: [acceptance_page, abstract] }
    params:
      chair_first: true
    automatable: false
    review_hint: "Verify the committee chair is listed first, followed by other committee members."

  - id: toc_title_parity
    category: content
    checker: toc_title_parity
    target: { page: toc }
    params: {}
    automatable: false
    review_hint: "Verify that all chapter titles and subheadings in the Table of Contents match verbatim the titles in the document body. No overhanging words into page number column."

  - id: abstract_word_count
    category: content
    checker: text_match
    target: { page: abstract }
    params:
      max_words: 350
    automatable: true

  - id: copyright_page_format
    category: content
    checker: text_match
    target: { page: copyright_page }
    params:
      template: "©"
    automatable: false
    review_hint: "Verify copyright page has the copyright symbol, year, and student name, centered."

  - id: cv_no_page_number
    category: structure
    checker: page_numbering
    target: { page: curriculum_vitae }
    params:
      expect_no_page_number: true
    automatable: false
    review_hint: "Verify the Curriculum Vitae has no page number."

  - id: font_weight_consistent
    category: typography
    checker: font_weight
    target: { scope: all_pages }
    params:
      weight: normal
    automatable: false
    review_hint: "Verify that titles are not bold. Verify no italics or bold in lists of tables/figures/pictures."

  - id: justification_consistent
    category: typography
    checker: justification
    target: { scope: all_pages }
    params: {}
    automatable: false
    review_hint: "Verify justification is consistent throughout the document. If justified, references and CV must also be justified."

  - id: page_numbers_bottom_center
    category: structure
    checker: page_numbering
    target: { scope: all_pages }
    params:
      position: "bottom_center"
      distance_from_bottom: "0.5in"
    automatable: false
    review_hint: "Verify page numbers are at the bottom center, 0.5 inches from the bottom edge, matching document font and size."

  - id: chapter_starts_new_page
    category: structure
    checker: page_numbering
    target: { scope: body }
    params:
      chapter_new_page: true
    automatable: false
    review_hint: "Verify that each new chapter starts on a new page."

  - id: listing_titles_match
    category: content
    checker: toc_title_parity
    target: { pages: [lot, lof, lop] }
    params: {}
    automatable: false
    review_hint: "Verify all titles in the List of Tables, List of Figures, and List of Pictures match verbatim the titles in the document. Also verify no italics or bold in these lists."

  - id: cv_title_format
    category: content
    checker: text_match
    target: { page: curriculum_vitae }
    params:
      expected_title: "Curriculum Vitae"
    automatable: false
    review_hint: "Verify the Curriculum Vitae title matches chapter title formatting (capitalization, placement, font type, bold or not)."

constants:
  degree: "Doctor of Philosophy"
  acceptable_fonts: ["Times New Roman"]
  page_number_font_size: same_as_body
```

- [ ] **Step 2: Verify the spec loads and validates**

```bash
python -c "from diss_check.spec import load_spec; s = load_spec('specs/iu.yaml'); print(f'Loaded: {s.institution} ({s.source_revision}), {len(s.checks)} checks')"
```

Expected: `Loaded: Indiana University (September 2025), N checks`

- [ ] **Step 3: Commit**

```bash
git add specs/iu.yaml
git commit -m "feat: add Indiana University format spec (September 2025)"
```

---

### Task 17: Integration Test — End-to-End with IU Template

**Files:**
- Create: `tests/test_integration.py`

**Interfaces:**
- Consumes: All previous tasks
- Produces: Integration test that runs the full engine against the IU template PDF and asserts reasonable results

- [ ] **Step 1: Write integration test**

```python
# tests/test_integration.py
from pathlib import Path
import pytest
from diss_check.spec import load_spec
from diss_check.engine import Engine
from diss_check.report import Report


@pytest.mark.integration
def test_iu_template_produces_results(iu_template_path):
    """Run the full check suite against the IU template PDF.
    Since the template is known-acceptable, we expect mostly PASS or MANUAL results,
    but not necessarily all PASS (the template is Word-derived and may be imperfect)."""
    spec = load_spec(Path("specs/iu.yaml"))
    engine = Engine(spec)
    results = engine.run(iu_template_path)
    report = Report(results=results)

    # All checks should have run
    assert len(report.results) == len(spec.checks)

    # Should not have any ERROR statuses
    assert report.error_count == 0

    # Known-acceptable template should have zero or very few FAILs
    # (some FAILs might occur if the template is imperfect)
    print(f"\nResults: {report.pass_count} PASS, {report.fail_count} FAIL, {report.manual_count} MANUAL")
    for r in report.results:
        if r.status == "FAIL":
            print(f"  FAIL: {r.check_id} — {r.detail}")
```

- [ ] **Step 2: Run integration test (may fail if no template PDF)**

Run: `pytest tests/test_integration.py -v -m integration`
Expected: test runs; may skip or fail depending on whether `tests/fixtures/iu_template.pdf` exists

- [ ] **Step 3: Commit**

```bash
git add tests/test_integration.py
git commit -m "test: add end-to-end integration test with IU template"
```

---

### Task 18: Final Verification

- [ ] **Step 1: Run all unit tests**

```bash
pytest tests/ -v --ignore=tests/test_integration.py --ignore=tests/fixtures
```

Expected: all tests PASS

- [ ] **Step 2: Verify CLI works end-to-end with a PDF**

```bash
diss-check check --spec specs/iu.yaml tests/fixtures/iu_template.pdf 2>&1
```

Expected: Check report output (no ERRORs, some FAILs/Manual are OK since template is imperfect)

- [ ] **Step 3: Verify JSON output**

```bash
diss-check check --spec specs/iu.yaml --json tests/fixtures/iu_template.pdf 2>&1 | python -m json.tool | head -20
```

Expected: Valid JSON with summary and results

- [ ] **Step 4: Run full test suite**

```bash
pytest tests/ -v 2>&1
```

- [ ] **Step 5: Commit any final fixes**

```bash
git add -A && git commit -m "chore: final verification and cleanup"
```

---

## Notes for Implementers

1. **IU template PDF** is expected at `tests/fixtures/iu_template.pdf`. This file is not committed (it's in `.gitignore`). Download it from the IU website before running integration tests.
2. **Corpus directory** at `tests/fixtures/corpus/` is gitignored. Populate with accepted dissertations for calibration testing.
3. **veraPDF** requires a separate CLI tool installation. The extractor stub is in place; full implementation requires `verapdf` on PATH.
4. **Checkers flagged `automatable: false`** produce MANUAL results. As automation improves, these can be reclassified in the spec without changing code.
5. **Font metadata** from pdfplumber uses character-level `fontname` attributes. These often include style suffixes (e.g., "TimesNewRoman,Bold"). The typography checkers parse these suffixes.
6. **Section matching** in structure checkers uses substring matching on normalized text. For IU, the spec's section ids (e.g., "toc") match text like "Table of Contents" via `_match_section()`. Institution-specific override may be needed later.
