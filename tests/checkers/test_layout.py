import pytest
from diss_check.checkers.layout import MarginsChecker
from diss_check.document import ExtractionContext


def _make_synthetic_doc(bboxes):
    """Create a synthetic DoclingDocument with text items at given bounding boxes.
    Each bbox is (x0, y0, x1, y1) in points (72pt = 1in). Page is US Letter (612x792pt)."""
    from docling_core.types.doc import DoclingDocument, DocItemLabel, BoundingBox, CoordOrigin
    from docling_core.types.doc.document import ProvenanceItem

    doc = DoclingDocument(name="synthetic")
    doc.add_page(page_no=1, size={"width": 612, "height": 792})
    for i, bbox in enumerate(bboxes):
        prov = ProvenanceItem(
            page_no=1,
            bbox=BoundingBox(
                l=bbox[0], t=bbox[1], r=bbox[2], b=bbox[3],
                coord_origin=CoordOrigin.TOPLEFT,
            ),
            charspan=(0, len(f"Text block {i}")),
        )
        doc.add_text(prov=[prov], text=f"Text block {i}", label=DocItemLabel.TEXT)
    return doc


def test_margins_pass_when_text_within_bounds():
    # margins: left=1.25in=90pt, right=1.25in→right_edge≥522pt, top=1in=72pt, bottom=1in→bottom_edge≤720pt
    doc = _make_synthetic_doc([(90, 72, 522, 720)])
    ctx = ExtractionContext(docling_doc=doc)
    checker = MarginsChecker()
    result = checker.check(ctx, {"top": "1in", "bottom": "1in", "left": "1.25in", "right": "1.25in"})
    assert result.status == "PASS"


def test_margins_fail_when_left_margin_violated():
    doc = _make_synthetic_doc([(36, 72, 522, 720)])  # left=36pt=0.5in < 1.25in
    ctx = ExtractionContext(docling_doc=doc)
    checker = MarginsChecker()
    result = checker.check(ctx, {"top": "1in", "bottom": "1in", "left": "1.25in", "right": "1.25in"})
    assert result.status == "FAIL"
    assert len(result.evidence) > 0
    assert result.evidence[0].page == 1


def test_margins_fail_when_right_margin_violated():
    doc = _make_synthetic_doc([(90, 72, 576, 720)])  # right=576pt→right_margin=612-576=36pt < 1.25in=90pt
    ctx = ExtractionContext(docling_doc=doc)
    checker = MarginsChecker()
    result = checker.check(ctx, {"top": "1in", "bottom": "1in", "left": "1.25in", "right": "1.25in"})
    assert result.status == "FAIL"
