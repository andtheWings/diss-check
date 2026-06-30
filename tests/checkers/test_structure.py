from diss_check.checkers.structure import SectionPresenceChecker
from diss_check.document import Document, ExtractionContext, Page, TextSpan


def _make_doc(spans_by_page):
    pages = []
    for page_spans in spans_by_page:
        spans = [
            TextSpan(text=t, font_name="Times", font_size=12.0, bbox=bbox)
            for bbox, t in page_spans
        ]
        pages.append(Page(page_number=len(pages) + 1, width=612, height=792, spans=spans))
    return Document(pages=pages)


def test_detects_title_page_when_no_page_number():
    doc = _make_doc([
        [((72, 85, 90, 522), "TITLE TEXT",)],
    ])
    ctx = ExtractionContext(document=doc)
    result = SectionPresenceChecker().check(ctx, {
        "required_sections": [{"id": "title_page"}],
    })
    assert result.status == "PASS"


def test_fails_title_page_when_page_number_present():
    doc = _make_doc([
        [((72, 85, 90, 522), "TITLE TEXT"), ((750, 760, 90, 522), "ii")],
    ])
    ctx = ExtractionContext(document=doc)
    result = SectionPresenceChecker().check(ctx, {
        "required_sections": [{"id": "title_page"}],
    })
    assert result.status == "FAIL"


def test_detects_abstract_by_heuristic():
    abstract_spans = [((72 + i*7, 85 + i*7, 90, 500), f"line{i}") for i in range(110)]
    doc = _make_doc([
        [((72, 85, 90, 522), "TITLE TEXT")],
        [((72, 85, 90, 522), "Accepted by the Graduate Faculty")],
        [((72, 85, 90, 522), "ACKNOWLEDGEMENTS")],
        abstract_spans,
        [((72, 85, 90, 522), "Table of Contents")],
    ])
    ctx = ExtractionContext(document=doc)
    result = SectionPresenceChecker().check(ctx, {
        "required_sections": [{"id": "abstract"}],
    })
    assert result.status == "PASS"


def test_detects_toc_by_leader_dots():
    doc = _make_doc([
        [((72, 85, 90, 522), "Table of Contents")],
        [((144, 156, 222, 540), "Chapter 1 .................. 5")],
    ])
    ctx = ExtractionContext(document=doc)
    result = SectionPresenceChecker().check(ctx, {
        "required_sections": [{"id": "toc"}],
    })
    assert result.status == "PASS"


def test_detects_acceptance_page_by_keyword():
    doc = _make_doc([
        [((72, 85, 90, 522), "Accepted by the Graduate Faculty")],
    ])
    ctx = ExtractionContext(document=doc)
    result = SectionPresenceChecker().check(ctx, {
        "required_sections": [{"id": "acceptance_page"}],
    })
    assert result.status == "PASS"


def test_reports_missing_sections():
    doc = _make_doc([
        [((72, 85, 90, 522), "Just some body text")],
    ])
    ctx = ExtractionContext(document=doc)
    result = SectionPresenceChecker().check(ctx, {
        "required_sections": [{"id": "abstract"}, {"id": "toc"}],
    })
    assert result.status == "FAIL"
    assert "abstract" in result.detail.lower()
    assert "toc" in result.detail.lower()


def test_detects_references():
    doc = _make_doc([
        [((72, 85, 90, 522), "References")],
        [((100, 115, 90, 522), "Smith, J. (2020)...")],
    ])
    ctx = ExtractionContext(document=doc)
    result = SectionPresenceChecker().check(ctx, {
        "required_sections": [{"id": "references"}],
    })
    assert result.status == "PASS"


def test_detects_curriculum_vitae():
    doc = _make_doc([
        [((72, 85, 90, 522), "CURRICULUM VITAE")],
    ])
    ctx = ExtractionContext(document=doc)
    result = SectionPresenceChecker().check(ctx, {
        "required_sections": [{"id": "curriculum_vitae"}],
    })
    assert result.status == "PASS"


def test_no_required_sections_passes():
    doc = _make_doc([
        [((72, 85, 90, 522), "anything")],
    ])
    ctx = ExtractionContext(document=doc)
    result = SectionPresenceChecker().check(ctx, {
        "required_sections": [],
    })
    assert result.status == "PASS"
