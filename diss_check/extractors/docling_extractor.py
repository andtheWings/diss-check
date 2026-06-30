from pathlib import Path
from docling.document_converter import DocumentConverter, PdfFormatOption
from docling.datamodel.pipeline_options import PdfPipelineOptions
from docling.datamodel.base_models import InputFormat
from diss_check.document import ExtractionContext
from diss_check.extractors.base import BaseExtractor


class DoclingExtractor(BaseExtractor):
    name = "docling"

    def __init__(self):
        pipeline_options = PdfPipelineOptions(do_ocr=False, do_table_structure=False)
        self._converter = DocumentConverter(
            format_options={
                InputFormat.PDF: PdfFormatOption(pipeline_options=pipeline_options),
            }
        )

    def extract(self, source: Path, ctx: ExtractionContext) -> None:
        result = self._converter.convert(str(source))
        ctx.docling_doc = result.document
