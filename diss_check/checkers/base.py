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
    requires: list[str] = ["pdfplumber"]

    @abstractmethod
    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        ...
