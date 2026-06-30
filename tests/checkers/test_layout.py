from diss_check.checkers.layout import MarginsChecker
from diss_check.document import Document, ExtractionContext, Page, TextSpan


def _make_doc(spans_by_page):
    pages = []
    for page_spans in spans_by_page:
        spans = [
            TextSpan(text=t, font_name="Times", font_size=12, bbox=bbox)
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


def test_margins_fail_when_top_violated():
    doc = _make_doc([
        [((36, 48, 90, 522), "too high")],  # top=36pt < 72pt top margin
    ])
    ctx = ExtractionContext(document=doc)
    result = MarginsChecker().check(ctx, IN_SPEC)
    assert result.status == "FAIL"
    assert result.evidence[0].page == 1


def test_margins_fail_when_bottom_violated():
    doc = _make_doc([
        [((740, 756, 90, 522), "too low")],  # bottom=756 > 720 (792-72) bottom margin
    ])
    ctx = ExtractionContext(document=doc)
    result = MarginsChecker().check(ctx, IN_SPEC)
    assert result.status == "FAIL"


def test_margins_fail_when_left_violated():
    doc = _make_doc([
        [((72, 85, 36, 522), "too left")],  # x0=36pt < 90pt left margin
    ])
    ctx = ExtractionContext(document=doc)
    result = MarginsChecker().check(ctx, IN_SPEC)
    assert result.status == "FAIL"


def test_margins_fail_when_right_violated():
    doc = _make_doc([
        [((72, 85, 90, 576), "too right")],  # x1=576 > 522 (612-90) right margin
    ])
    ctx = ExtractionContext(document=doc)
    result = MarginsChecker().check(ctx, IN_SPEC)
    assert result.status == "FAIL"
