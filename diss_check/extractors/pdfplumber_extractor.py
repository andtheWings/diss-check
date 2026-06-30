from collections import Counter
from statistics import median
from pathlib import Path

import pdfplumber

from diss_check.document import Document, ExtractionContext, Page, TextSpan
from diss_check.extractors.base import BaseExtractor


def _group_chars_into_words(page) -> list[dict]:
    chars = page.chars
    if not chars:
        return []
    chars = sorted(chars, key=lambda c: (round(c["top"], 1), c["x0"]))

    char_widths = [c["width"] for c in chars if c["text"] != " "]
    gap_threshold = median(char_widths) * 1.2 if char_widths else 3.0

    words: list[dict] = []
    current_word_chars: list = []
    current_line_top = round(chars[0]["top"], 1)

    for c in chars:
        char_top = round(c["top"], 1)
        if abs(char_top - current_line_top) > 2:
            if current_word_chars:
                words.append(_make_word(current_word_chars))
                current_word_chars = []
            current_line_top = char_top
            current_word_chars = [c]
            continue

        if not current_word_chars:
            current_word_chars = [c]
            continue

        gap = c["x0"] - current_word_chars[-1]["x1"]
        if gap > gap_threshold or c["text"] == " ":
            if any(ch["text"] != " " for ch in current_word_chars):
                words.append(_make_word(current_word_chars))
            current_word_chars = [c] if c["text"] != " " else []
        else:
            current_word_chars.append(c)

    if current_word_chars and any(ch["text"] != " " for ch in current_word_chars):
        words.append(_make_word(current_word_chars))

    return words


def _make_word(chars: list) -> dict:
    text = "".join(c["text"] for c in chars)
    font_sizes = [c["size"] for c in chars]
    font_names = [c.get("fontname", "") for c in chars]
    return {
        "text": text,
        "top": min(c["top"] for c in chars),
        "bottom": max(c["bottom"] for c in chars),
        "x0": min(c["x0"] for c in chars),
        "x1": max(c["x1"] for c in chars),
        "font_name": Counter(font_names).most_common(1)[0][0],
        "font_size": median(font_sizes),
    }


class PdfplumberExtractor(BaseExtractor):
    name = "pdfplumber"

    def extract(self, source: Path, ctx: ExtractionContext) -> None:
        with pdfplumber.open(source) as pdf:
            pages = []
            for i, pdf_page in enumerate(pdf.pages):
                words = _group_chars_into_words(pdf_page)
                spans = [
                    TextSpan(
                        text=w["text"],
                        font_name=w["font_name"],
                        font_size=w["font_size"],
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
