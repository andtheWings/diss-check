from diss_check.document import ExtractionContext
from diss_check.checkers.base import BaseChecker, CheckResult, EvidenceItem, register_checker


def _parse_measurement(value: str) -> float:
    value = value.strip()
    if value.endswith("in"):
        return float(value[:-2]) * 72
    if value.endswith("pt"):
        return float(value[:-2])
    raise ValueError(f"Unsupported measurement: {value}")


@register_checker(category="layout", name="margins")
class MarginsChecker(BaseChecker):
    requires = ["pdfplumber"]

    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        doc = ctx.document
        top_margin = _parse_measurement(params["top"])
        bottom_margin = _parse_measurement(params["bottom"])
        left_margin = _parse_measurement(params["left"])
        right_margin = _parse_measurement(params["right"])

        violations: list[EvidenceItem] = []

        for page in doc.pages:
            for span in page.spans:
                top, bottom, x0, x1 = span.bbox

                if top < top_margin or bottom > (page.height - bottom_margin) or x0 < left_margin or x1 > (page.width - right_margin):
                    violations.append(EvidenceItem(
                        page=page.page_number,
                        bbox=span.bbox,
                        excerpt=span.text,
                    ))

        if violations:
            return CheckResult(
                status="FAIL",
                evidence=violations,
                detail=f"{len(violations)} character(s) violate margin requirements",
            )
        return CheckResult(
            status="PASS",
            detail="All text is within required margins",
        )
