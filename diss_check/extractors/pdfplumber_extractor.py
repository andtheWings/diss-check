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
                chars = pdf_page.chars or []
                spans = [
                    TextSpan(
                        text=ch["text"],
                        font_name=ch.get("fontname", ""),
                        font_size=float(ch.get("size", 0)),
                        bbox=(ch["top"], ch["bottom"], ch["x0"], ch["x1"]),
                    )
                    for ch in chars
                ]
                pages.append(Page(
                    page_number=i + 1,
                    width=float(pdf_page.width),
                    height=float(pdf_page.height),
                    spans=spans,
                ))
            ctx.document = Document(pages=pages)
