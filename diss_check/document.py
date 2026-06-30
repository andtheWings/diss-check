from dataclasses import dataclass
from typing import TYPE_CHECKING

from pydantic import BaseModel

if TYPE_CHECKING:
    from docling_core.types.doc import DoclingDocument


class TextSpan(BaseModel):
    text: str
    font_name: str
    font_size: float
    bbox: tuple[float, float, float, float]

    @property
    def x0(self) -> float:
        return self.bbox[0]

    @property
    def top(self) -> float:
        return self.bbox[1]

    @property
    def x1(self) -> float:
        return self.bbox[2]

    @property
    def bottom(self) -> float:
        return self.bbox[3]


class Page(BaseModel):
    page_number: int
    width: float
    height: float
    spans: list[TextSpan]


class Document(BaseModel):
    pages: list[Page]


@dataclass
class ExtractionContext:
    document: Document | None = None
    docling_doc: "DoclingDocument | None" = None
