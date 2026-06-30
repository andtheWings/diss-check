from diss_check.checkers.human import HumanReviewChecker
from diss_check.document import ExtractionContext


def test_human_review_returns_manual():
    ctx = ExtractionContext()
    result = HumanReviewChecker().check(ctx, {})
    assert result.status == "MANUAL"
    assert result.detail == "Manual review required"


def test_human_review_returns_custom_prompt():
    ctx = ExtractionContext()
    result = HumanReviewChecker().check(ctx, {
        "prompt": "Check margins on landscape pages",
    })
    assert result.status == "MANUAL"
    assert "landscape" in result.detail
