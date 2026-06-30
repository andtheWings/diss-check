from pathlib import Path

import pdfplumber

from diss_check.document import Document, ExtractionContext, Page, TextSpan
from diss_check.extractors.base import BaseExtractor


class PdfplumberExtractor(BaseExtractor):
    name = "pdfplumber"

    def extract(self, source: Path, ctx: ExtractionContext) -> None:
        with pdfplumber.open(source) as pdf:
            pages = []
            for i, pdf_page in enumerate(pdf.pages):
                words = pdf_page.extract_words()
                spans = [
                    TextSpan(
                        text=w["text"],
                        font_name="",
                        font_size=0,
                        bbox=(w["top"], w["bottom"], w["x0"], w["x1"]),
                    )
                    for w in words
                ]
                pages.append(Page(
                    page_number=i + 1,
                    width=float(pdf_page.width),
                    height=float(pdf_page.height),
                    spans=spans,
                ))
            ctx.document = Document(pages=pages)
