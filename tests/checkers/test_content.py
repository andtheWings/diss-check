from diss_check.checkers.content import BoilerplateMatchChecker
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


TEMPLATE_CLAUSE = (
    "Submitted to the faculty of the {school}\n"
    "in partial fulfillment of the requirements\n"
    "for the degree\n"
    "{degree}\n"
    "Indiana University\n"
    "{month} {year}"
)


def test_matches_exact():
    doc = _make_doc([
        [
            ((72, 85, 90, 522), "Submitted to the faculty of the Graduate School"),
            ((100, 115, 90, 522), "in partial fulfillment of the requirements"),
            ((128, 143, 90, 522), "for the degree"),
            ((156, 171, 90, 522), "Doctor of Philosophy"),
            ((184, 199, 90, 522), "Indiana University"),
            ((212, 227, 90, 522), "May 2026"),
        ],
    ])
    ctx = ExtractionContext(document=doc)
    result = BoilerplateMatchChecker().check(ctx, {"template": TEMPLATE_CLAUSE, "page": 1})
    assert result.status == "PASS"


def test_matches_with_extra_lines():
    doc = _make_doc([
        [
            ((50, 65, 90, 522), "TITLE"),
            ((72, 85, 90, 522), "Submitted to the faculty of the Graduate School"),
            ((100, 115, 90, 522), "in partial fulfillment of the requirements"),
            ((128, 143, 90, 522), "for the degree"),
            ((156, 171, 90, 522), "Doctor of Philosophy"),
            ((184, 199, 90, 522), "Indiana University"),
            ((212, 227, 90, 522), "May 2026"),
        ],
    ])
    ctx = ExtractionContext(document=doc)
    result = BoilerplateMatchChecker().check(ctx, {"template": TEMPLATE_CLAUSE, "page": 1})
    assert result.status == "PASS"


def test_fails_when_template_not_found():
    doc = _make_doc([
        [
            ((72, 85, 90, 522), "Completely different text"),
        ],
    ])
    ctx = ExtractionContext(document=doc)
    result = BoilerplateMatchChecker().check(ctx, {"template": TEMPLATE_CLAUSE, "page": 1})
    assert result.status == "FAIL"


def test_passes_with_no_template():
    doc = _make_doc([
        [
            ((72, 85, 90, 522), "anything"),
        ],
    ])
    ctx = ExtractionContext(document=doc)
    result = BoilerplateMatchChecker().check(ctx, {"template": ""})
    assert result.status == "PASS"


def test_page_filter():
    doc = _make_doc([
        [
            ((72, 85, 90, 522), "wrong page text"),
        ],
        [
            ((72, 85, 90, 522), "Submitted to the faculty of the Graduate School"),
            ((100, 115, 90, 522), "in partial fulfillment of the requirements"),
            ((128, 143, 90, 522), "for the degree"),
            ((156, 171, 90, 522), "PhD"),
            ((184, 199, 90, 522), "Indiana University"),
            ((212, 227, 90, 522), "May 2026"),
        ],
    ])
    ctx = ExtractionContext(document=doc)
    result = BoilerplateMatchChecker().check(ctx, {"template": TEMPLATE_CLAUSE, "page": 2})
    assert result.status == "PASS"


def test_variables_match_anything():
    doc = _make_doc([
        [
            ((72, 85, 90, 522), "Submitted to the faculty of the Any School Name"),
            ((100, 115, 90, 522), "in partial fulfillment of the requirements"),
            ((128, 143, 90, 522), "for the degree"),
            ((156, 171, 90, 522), "Master of Science"),
            ((184, 199, 90, 522), "Indiana University"),
            ((212, 227, 90, 522), "December 2025"),
        ],
    ])
    ctx = ExtractionContext(document=doc)
    result = BoilerplateMatchChecker().check(ctx, {"template": TEMPLATE_CLAUSE, "page": 1})
    assert result.status == "PASS"


def test_multiline_variable():
    doc = _make_doc([
        [
            ((72, 85, 90, 522), "Submitted to the faculty of the Graduate School"),
            ((100, 115, 90, 522), "in partial fulfillment of the requirements"),
            ((128, 143, 90, 522), "for the degree"),
            ((156, 171, 90, 522), "Doctor of Philosophy"),
            ((184, 199, 90, 522), "in the Department of Speech"),
            ((212, 227, 90, 522), "and Hearing Sciences"),
            ((240, 255, 90, 522), "Indiana University"),
            ((268, 283, 90, 522), "May 2026"),
        ],
    ])
    ctx = ExtractionContext(document=doc)
    result = BoilerplateMatchChecker().check(ctx, {"template": TEMPLATE_CLAUSE, "page": 1})
    assert result.status == "PASS"


from diss_check.checkers.content import CommitteeOrderChecker


def test_committee_chair_first_pass():
    doc = _make_doc([
        [
            ((72, 85, 90, 522), "Doctoral Committee"),
            ((130, 145, 90, 522), "Jane Smith, PhD, Chair"),
            ((170, 185, 90, 522), "John Doe, PhD"),
            ((210, 225, 90, 522), "Alice Jones, PhD"),
        ],
    ])
    ctx = ExtractionContext(document=doc)
    result = CommitteeOrderChecker().check(ctx, {"chair_first": True, "page": 1})
    assert result.status == "PASS"


def test_committee_chair_not_first_fail():
    doc = _make_doc([
        [
            ((72, 85, 90, 522), "Doctoral Committee"),
            ((130, 145, 90, 522), "John Doe, PhD"),
            ((170, 185, 90, 522), "Jane Smith, PhD, Chair"),
            ((210, 225, 90, 522), "Alice Jones, PhD"),
        ],
    ])
    ctx = ExtractionContext(document=doc)
    result = CommitteeOrderChecker().check(ctx, {"chair_first": True, "page": 1})
    assert result.status == "FAIL"


def test_committee_no_chair_label_fail():
    doc = _make_doc([
        [
            ((72, 85, 90, 522), "Doctoral Committee"),
            ((130, 145, 90, 522), "Jane Smith, PhD"),
            ((170, 185, 90, 522), "John Doe, PhD"),
        ],
    ])
    ctx = ExtractionContext(document=doc)
    result = CommitteeOrderChecker().check(ctx, {"chair_first": True, "page": 1})
    assert result.status == "FAIL"


def test_committee_skips_signature_lines():
    doc = _make_doc([
        [
            ((72, 85, 90, 522), "Doctoral Committee"),
            ((130, 145, 90, 522), "__________"),
            ((170, 185, 90, 522), "Jane Smith, PhD, Chair"),
            ((210, 225, 90, 522), "__________"),
            ((250, 265, 90, 522), "John Doe, PhD"),
        ],
    ])
    ctx = ExtractionContext(document=doc)
    result = CommitteeOrderChecker().check(ctx, {"chair_first": True, "page": 1})
    assert result.status == "PASS"


from diss_check.checkers.content import TocTitleParityChecker


def test_toc_parity_matches_chapter_headings():
    doc = _make_doc([
        [  # p1: TOC
            ((72, 85, 90, 522), "Table of Contents"),
            ((144, 156, 90, 500), "Chapter 1: Introduction ..................... 5"),
        ],
        [  # p2-p4: filler
            ((72, 85, 90, 522), "filler"),
        ],
        [  # p5: body chapter 1
            ((72, 85, 90, 522), "Chapter 1: Introduction"),
            ((100, 115, 90, 522), "Body text starts here..."),
        ],
    ])
    ctx = ExtractionContext(document=doc)
    result = TocTitleParityChecker().check(ctx, {})
    assert result.status == "PASS"


def test_toc_parity_fails_when_mismatch():
    doc = _make_doc([
        [
            ((72, 85, 90, 522), "Table of Contents"),
            ((144, 156, 90, 500), "Chapter 1: Wrong Name ..................... 5"),
        ],
        [
            ((72, 85, 90, 522), "filler"),
        ],
        [
            ((72, 85, 90, 522), "Chapter 1: Different Name"),
            ((100, 115, 90, 522), "Body text..."),
        ],
    ])
    ctx = ExtractionContext(document=doc)
    result = TocTitleParityChecker().check(ctx, {})
    assert result.status == "FAIL"
