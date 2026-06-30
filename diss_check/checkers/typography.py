from collections import Counter

from diss_check.document import ExtractionContext
from diss_check.checkers.base import BaseChecker, CheckResult, EvidenceItem, register_checker


def _parse_measurement(value: str) -> float:
    value = value.strip()
    if value.endswith("in"):
        return float(value[:-2]) * 72
    if value.endswith("pt"):
        return float(value[:-2])
    raise ValueError(f"Unsupported measurement: {value}")


@register_checker(category="typography", name="font_size")
class FontSizeChecker(BaseChecker):
    requires = ["pdfplumber"]

    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        doc = ctx.document
        allowed_raw: list[str] = params.get("allowed", [])
        allowed: list[float] = [_parse_measurement(s) for s in allowed_raw]
        tolerance = 0.5
        consistent = params.get("consistent", False)

        violations: list[EvidenceItem] = []
        body_sizes: Counter[float] = Counter()

        for page in doc.pages:
            for span in page.spans:
                if span.bottom > (page.height - 50):
                    continue
                if span.top < 36:
                    continue

                size = span.font_size

                if size < 8.5:
                    continue

                if consistent:
                    body_sizes[round(size, 1)] += 1

                if not allowed:
                    continue

                matched = any(abs(size - a) <= tolerance for a in allowed)
                if not matched:
                    violations.append(EvidenceItem(
                        page=page.page_number,
                        bbox=span.bbox,
                        excerpt=f"{span.text!r} ({size:.1f}pt)",
                    ))

        if consistent and body_sizes and len(body_sizes) > 1:
            modal_size = body_sizes.most_common(1)[0][0]
            body_only_violations: list[EvidenceItem] = []
            for page in doc.pages:
                for span in page.spans:
                    if span.bottom > (page.height - 50) or span.top < 36:
                        continue
                    size = round(span.font_size, 1)

                    if size < 8.5:
                        continue

                    if any(abs(size - a) <= tolerance for a in allowed):
                        continue

                    if abs(size - modal_size) > tolerance:
                        body_only_violations.append(EvidenceItem(
                            page=page.page_number,
                            bbox=span.bbox,
                            excerpt=f"{span.text!r} ({size:.1f}pt, expected {modal_size:.0f}pt)",
                        ))

            if body_only_violations:
                violations.extend(body_only_violations)

        if violations:
            return CheckResult(
                status="FAIL",
                evidence=violations,
                detail=f"{len(violations)} span(s) violate font size requirements",
            )
        return CheckResult(
            status="PASS",
            detail="All text conforms to font size requirements",
        )
