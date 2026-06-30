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
