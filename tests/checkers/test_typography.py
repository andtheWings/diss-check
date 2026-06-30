from diss_check.checkers.typography import FontSizeChecker
from diss_check.document import Document, ExtractionContext, Page, TextSpan


def _make_doc(spans_by_page):
    pages = []
    for page_spans in spans_by_page:
        spans = [
            TextSpan(text=t, font_name="Times", font_size=fs, bbox=bbox)
            for bbox, t, fs in page_spans
        ]
        pages.append(Page(page_number=len(pages) + 1, width=612, height=792, spans=spans))
    return Document(pages=pages)


# bbox = (top, bottom, x0, x1) in pdfplumber coordinates (origin top-left)


def test_passes_when_all_sizes_in_allowed():
    doc = _make_doc([
        [((72, 85, 90, 522), "hello", 12.0)],
        [((72, 85, 90, 522), "world", 12.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontSizeChecker().check(ctx, {"allowed": ["10pt", "11pt", "12pt"]})
    assert result.status == "PASS"


def test_fails_when_size_not_in_allowed():
    doc = _make_doc([
        [((72, 85, 90, 522), "too big", 14.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontSizeChecker().check(ctx, {"allowed": ["10pt", "12pt"]})
    assert result.status == "FAIL"
    assert len(result.evidence) == 1
    assert "14.0pt" in result.evidence[0].excerpt


def test_passes_when_size_close_to_allowed_within_tolerance():
    doc = _make_doc([
        [((72, 85, 90, 522), "slightly off", 12.000000000000057)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontSizeChecker().check(ctx, {"allowed": ["12pt"]})
    assert result.status == "PASS"


def test_passes_when_no_allowed_specified():
    doc = _make_doc([
        [((72, 85, 90, 522), "anything", 42.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontSizeChecker().check(ctx, {"consistent": True})
    assert result.status == "PASS"


def test_consistent_passes_when_one_body_size():
    doc = _make_doc([
        [((72, 85, 90, 522), "hello", 12.0)],
        [((72, 85, 90, 522), "world", 12.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontSizeChecker().check(ctx, {"allowed": ["12pt"], "consistent": True})
    assert result.status == "PASS"


def test_consistent_allows_allowed_sizes_that_differ_from_modal():
    # 10pt is in allowed, so it should NOT be flagged even though modal is 12pt
    doc = _make_doc([
        [((72, 85, 90, 522), "body", 12.0)],
        [((72, 85, 90, 522), "body", 12.0)],
        [((72, 85, 90, 522), "reference", 10.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontSizeChecker().check(ctx, {"allowed": ["10pt", "12pt"], "consistent": True})
    assert result.status == "PASS"


def test_consistent_fails_when_unallowed_size_differs():
    doc = _make_doc([
        [((72, 85, 90, 522), "body", 12.0)],
        [((72, 85, 90, 522), "body", 12.0)],
        [((72, 85, 90, 522), "weird", 14.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontSizeChecker().check(ctx, {"allowed": ["12pt"], "consistent": True})
    assert result.status == "FAIL"
    assert "14.0pt" in result.evidence[0].excerpt


def test_skips_header_zone():
    doc = _make_doc([
        [((18, 30, 90, 522), "header", 8.0)],  # top=18 < 36
        [((72, 85, 90, 522), "body", 12.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontSizeChecker().check(ctx, {"allowed": ["12pt"]})
    assert result.status == "PASS"


def test_skips_page_number_zone():
    doc = _make_doc([
        [((72, 85, 90, 522), "body", 12.0)],
        [((750, 760, 90, 522), "pgnum", 10.0)],  # bottom=760 > 792-50=742
    ])
    ctx = ExtractionContext(document=doc)
    result = FontSizeChecker().check(ctx, {"allowed": ["12pt"]})
    assert result.status == "PASS"


def test_skips_tiny_font_sizes():
    doc = _make_doc([
        [((72, 85, 90, 522), "body", 12.0)],
        [((72, 85, 90, 522), "superscript", 6.5)],
        [((72, 85, 90, 522), "superscript", 8.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontSizeChecker().check(ctx, {"allowed": ["12pt"]})
    assert result.status == "PASS"


from diss_check.checkers.typography import FontWeightChecker


def _make_weight_doc(spans_by_page):
    pages = []
    for page_spans in spans_by_page:
        spans = [
            TextSpan(text=t, font_name=fn, font_size=fs, bbox=bbox)
            for bbox, t, fn, fs in page_spans
        ]
        pages.append(Page(page_number=len(pages) + 1, width=612, height=792, spans=spans))
    return Document(pages=pages)


def test_weight_passes_when_all_normal():
    doc = _make_weight_doc([
        [((72, 85, 90, 522), "normal text", "TimesMT", 12.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontWeightChecker().check(ctx, {"weight": "normal"})
    assert result.status == "PASS"


def test_weight_fails_when_bold_present():
    doc = _make_weight_doc([
        [((72, 85, 90, 522), "BOLD", "Times-BoldMT", 12.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontWeightChecker().check(ctx, {"weight": "normal"})
    assert result.status == "FAIL"
    assert "bold" in result.evidence[0].excerpt


def test_weight_passes_when_bold_expected():
    doc = _make_weight_doc([
        [((72, 85, 90, 522), "heading", "Times-BoldMT", 12.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontWeightChecker().check(ctx, {"weight": "bold"})
    assert result.status == "PASS"


def test_weight_skips_whitespace():
    doc = _make_weight_doc([
        [((72, 85, 90, 522), "   ", "Times-BoldMT", 12.0)],
        [((72, 85, 100, 522), "ok", "TimesMT", 12.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontWeightChecker().check(ctx, {"weight": "normal", "page": 1})
    assert result.status == "PASS"


def test_weight_detects_italic():
    doc = _make_weight_doc([
        [((72, 85, 90, 522), "italic text", "Times-ItalicMT", 12.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontWeightChecker().check(ctx, {"weight": "normal"})
    assert result.status == "FAIL"
    assert "italic" in result.evidence[0].excerpt


def test_weight_skips_header_zone():
    doc = _make_weight_doc([
        [((18, 30, 90, 522), "header", "Times-BoldMT", 12.0)],
        [((72, 85, 90, 522), "body", "TimesMT", 12.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontWeightChecker().check(ctx, {"weight": "normal"})
    assert result.status == "PASS"


def test_weight_skips_page_number_zone():
    doc = _make_weight_doc([
        [((72, 85, 90, 522), "body", "TimesMT", 12.0)],
        [((750, 760, 90, 522), "99", "Times-BoldMT", 12.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontWeightChecker().check(ctx, {"weight": "normal"})
    assert result.status == "PASS"


def test_weight_invert_flags_matching():
    doc = _make_weight_doc([
        [((72, 85, 90, 522), "bold here", "Times-BoldMT", 12.0)],
        [((72, 85, 100, 522), "normal ok", "TimesMT", 12.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontWeightChecker().check(ctx, {"weight": "bold", "invert": True})
    assert result.status == "FAIL"
    assert len(result.evidence) == 1


def test_weight_page_filter():
    doc = _make_weight_doc([
        [((72, 85, 90, 522), "p1 bold", "Times-BoldMT", 12.0)],
        [((72, 85, 90, 522), "p2 bold", "Times-BoldMT", 12.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontWeightChecker().check(ctx, {"weight": "normal", "page": 1})
    assert result.status == "FAIL"
    assert result.evidence[0].page == 1


def test_weight_title_page_not_bold_pass():
    doc = _make_weight_doc([
        [((72, 85, 90, 522), "TITLE", "TimesMT", 12.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontWeightChecker().check(ctx, {"weight": "normal", "page": 1})
    assert result.status == "PASS"


def test_weight_title_page_bold_fail():
    doc = _make_weight_doc([
        [((72, 85, 90, 522), "BOLD TITLE", "Times-BoldMT", 12.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontWeightChecker().check(ctx, {"weight": "normal", "page": 1})
    assert result.status == "FAIL"


from diss_check.checkers.typography import FontFamilyChecker


def _make_family_doc(spans_by_page):
    pages = []
    for page_spans in spans_by_page:
        spans = [
            TextSpan(text=t, font_name=fn, font_size=fs, bbox=bbox)
            for bbox, t, fn, fs in page_spans
        ]
        pages.append(Page(page_number=len(pages) + 1, width=612, height=792, spans=spans))
    return Document(pages=pages)


def test_family_passes_when_consistent():
    doc = _make_family_doc([
        [((72, 85, 90, 522), "hello", "TimesNewRomanPSMT", 12.0)],
        [((72, 85, 90, 522), "world", "TimesNewRomanPSMT", 12.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontFamilyChecker().check(ctx, {"consistent": True})
    assert result.status == "PASS"


def test_family_considers_variants_same_family():
    doc = _make_family_doc([
        [((72, 85, 90, 522), "regular", "TimesNewRomanPSMT", 12.0)],
        [((72, 85, 90, 522), "bold", "TimesNewRomanPS-BoldMT", 12.0)],
        [((72, 85, 90, 522), "italic", "TimesNewRomanPS-ItalicMT", 12.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontFamilyChecker().check(ctx, {"consistent": True})
    assert result.status == "PASS"


def test_family_fails_when_mixed():
    doc = _make_family_doc([
        [((72, 85, 90, 522), "times", "TimesNewRomanPSMT", 12.0)],
        [((72, 85, 90, 522), "arial text", "ArialMT", 12.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontFamilyChecker().check(ctx, {"consistent": True})
    assert result.status == "FAIL"


def test_family_skips_symbol_and_special_fonts():
    doc = _make_family_doc([
        [((72, 85, 90, 522), "body", "TimesNewRomanPSMT", 12.0)],
        [((72, 85, 90, 522), "*", "SymbolMT", 12.0)],
        [((72, 85, 90, 522), "+", "Wingdings-Regular", 12.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontFamilyChecker().check(ctx, {"consistent": True})
    assert result.status == "PASS"


def test_family_allowed_list():
    doc = _make_family_doc([
        [((72, 85, 90, 522), "times", "TimesNewRomanPSMT", 12.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontFamilyChecker().check(ctx, {"allowed": ["TimesNewRoman"]})
    assert result.status == "PASS"


def test_family_allowed_list_fails_other():
    doc = _make_family_doc([
        [((72, 85, 90, 522), "comic sans", "ComicSansMS", 12.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontFamilyChecker().check(ctx, {"allowed": ["TimesNewRoman"]})
    assert result.status == "FAIL"


def test_family_skips_whitespace():
    doc = _make_family_doc([
        [((72, 85, 90, 522), "   ", "ArialMT", 12.0)],
        [((72, 85, 90, 522), "ok", "TimesNewRomanPSMT", 12.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontFamilyChecker().check(ctx, {"consistent": True})
    assert result.status == "PASS"


def test_family_skips_header_and_page_number_zones():
    doc = _make_family_doc([
        [((18, 30, 90, 522), "header", "Arial-BoldMT", 12.0)],
        [((72, 85, 90, 522), "body", "TimesNewRomanPSMT", 12.0)],
        [((750, 760, 90, 522), "99", "ArialMT", 12.0)],
    ])
    ctx = ExtractionContext(document=doc)
    result = FontFamilyChecker().check(ctx, {"consistent": True})
    assert result.status == "PASS"
