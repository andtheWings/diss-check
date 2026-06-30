from diss_check.document import ExtractionContext
from diss_check.checkers.base import BaseChecker, CheckResult, EvidenceItem, register_checker


@register_checker(category="human", name="review")
class HumanReviewChecker(BaseChecker):
    requires = []

    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        prompt = params.get("prompt", "Manual review required")
        return CheckResult(
            status="MANUAL",
            detail=prompt,
        )
