from diss_check.checkers.layout import MarginsChecker
from diss_check.document import Document, ExtractionContext, Page, TextSpan


def _make_doc(spans_by_page):
    pages = []
    for page_spans in spans_by_page:
        spans = [
            TextSpan(text=t, font_name="", font_size=0, bbox=bbox)
            for bbox, t in page_spans
        ]
        pages.append(Page(page_number=len(pages) + 1, width=612, height=792, spans=spans))
    return Document(pages=pages)


# bbox = (top, bottom, x0, x1) in pdfplumber coordinates (origin top-left)
IN_SPEC = {
    "top": "1in", "bottom": "1in", "left": "1.25in", "right": "1.25in",
}


def test_margins_pass_when_text_within_bounds():
    doc = _make_doc([
        [((72, 85, 90, 522), "ok")],
    ])
    ctx = ExtractionContext(document=doc)
    result = MarginsChecker().check(ctx, IN_SPEC)
    assert result.status == "PASS"


def test_margins_pass_with_tolerance():
    # x1=535 is 13pt past margin; 0.25in (18pt) tolerance allows it
    doc = _make_doc([
        [((72, 85, 90, 535), "slightly over")],
    ])
    ctx = ExtractionContext(document=doc)
    result = MarginsChecker().check(ctx, IN_SPEC)
    assert result.status == "PASS"


def test_margins_fail_when_well_past_margin():
    doc = _make_doc([
        [((72, 85, 90, 576), "too right")],  # x1=576, past 522+18=540
    ])
    ctx = ExtractionContext(document=doc)
    result = MarginsChecker().check(ctx, IN_SPEC)
    assert result.status == "FAIL"


def test_margins_fail_when_well_above_top():
    doc = _make_doc([
        [((36, 48, 90, 522), "too high")],  # top=36, past 72-18=54
    ])
    ctx = ExtractionContext(document=doc)
    result = MarginsChecker().check(ctx, IN_SPEC)
    assert result.status == "FAIL"


def test_margins_fail_when_well_below_bottom():
    doc = _make_doc([
        [((600, 740, 90, 522), "too low")],  # bottom=740 > 738=720+18, above page-number zone (792-50=742)
    ])
    ctx = ExtractionContext(document=doc)
    result = MarginsChecker().check(ctx, IN_SPEC)
    assert result.status == "FAIL"


def test_margins_fail_when_well_past_left():
    doc = _make_doc([
        [((72, 85, 36, 522), "too left")],  # x0=36, past 90-18=72
    ])
    ctx = ExtractionContext(document=doc)
    result = MarginsChecker().check(ctx, IN_SPEC)
    assert result.status == "FAIL"
