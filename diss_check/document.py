from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from docling_core.types.doc import DoclingDocument


@dataclass
class ExtractionContext:
    docling_doc: "DoclingDocument | None" = None
