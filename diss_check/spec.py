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
