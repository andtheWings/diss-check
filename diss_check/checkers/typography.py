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


def _check_bold(font_name: str) -> bool:
    return "bold" in font_name.lower()


def _check_italic(font_name: str) -> bool:
    name = font_name.lower()
    return "italic" in name or "oblique" in name


def _detect_weight(font_name: str) -> str:
    if _check_bold(font_name):
        if _check_italic(font_name):
            return "bold-italic"
        return "bold"
    if _check_italic(font_name):
        return "italic"
    return "normal"


@register_checker(category="typography", name="font_weight")
class FontWeightChecker(BaseChecker):
    requires = ["pdfplumber"]

    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        doc = ctx.document
        expected = params.get("weight", "normal")
        page_filter = params.get("page")
        invert = params.get("invert", False)

        violations: list[EvidenceItem] = []

        for page in doc.pages:
            if page_filter is not None and page.page_number != page_filter:
                continue

            for span in page.spans:
                if not span.text.strip():
                    continue

                if span.bottom > (page.height - 50):
                    continue
                if span.top < 36:
                    continue

                weight = _detect_weight(span.font_name)

                if invert:
                    if weight == expected:
                        violations.append(EvidenceItem(
                            page=page.page_number,
                            bbox=span.bbox,
                            excerpt=f"{span.text!r} ({weight}, should not be {expected})",
                        ))
                else:
                    if weight != expected:
                        violations.append(EvidenceItem(
                            page=page.page_number,
                            bbox=span.bbox,
                            excerpt=f"{span.text!r} ({weight}, expected {expected})",
                        ))

        if violations:
            return CheckResult(
                status="FAIL",
                evidence=violations,
                detail=f"{len(violations)} span(s) violate font weight requirements",
            )
        return CheckResult(
            status="PASS",
            detail="All text conforms to font weight requirements",
        )


def _normalize_family(font_name: str) -> str:
    if "+" in font_name:
        font_name = font_name.split("+", 1)[1]
    for suffix in ["PS", "MT", "-Regular", "-BoldItalic", "-Bold", "-Italic", "-Oblique"]:
        font_name = font_name.replace(suffix, "")
    return font_name.strip("-")


SPECIAL_FONTS = {"Symbol", "Wingdings", "CambriaMath", "LucidaConsole", "ZapfDingbats"}


@register_checker(category="typography", name="font_family")
class FontFamilyChecker(BaseChecker):
    requires = ["pdfplumber"]

    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        doc = ctx.document
        allowed_raw: list[str] = params.get("allowed", [])
        allowed = set(allowed_raw)
        consistent = params.get("consistent", False)

        violations: list[EvidenceItem] = []
        family_counts: Counter[str] = Counter()

        for page in doc.pages:
            for span in page.spans:
                if not span.text.strip():
                    continue
                if span.bottom > (page.height - 50):
                    continue
                if span.top < 36:
                    continue

                family = _normalize_family(span.font_name)

                if family in SPECIAL_FONTS:
                    continue

                if consistent:
                    family_counts[family] += 1

                if not allowed:
                    continue

                if family not in allowed:
                    violations.append(EvidenceItem(
                        page=page.page_number,
                        bbox=span.bbox,
                        excerpt=f"{span.text!r} ({family})",
                    ))

        if consistent and family_counts and len(family_counts) > 1:
            modal_family = family_counts.most_common(1)[0][0]
            for page in doc.pages:
                for span in page.spans:
                    if not span.text.strip():
                        continue
                    if span.bottom > (page.height - 50) or span.top < 36:
                        continue
                    family = _normalize_family(span.font_name)
                    if family in SPECIAL_FONTS:
                        continue
                    if family != modal_family and (not allowed or family not in allowed):
                        violations.append(EvidenceItem(
                            page=page.page_number,
                            bbox=span.bbox,
                            excerpt=f"{span.text!r} ({family}, expected {modal_family})",
                        ))

        if violations:
            return CheckResult(
                status="FAIL",
                evidence=violations,
                detail=f"{len(violations)} span(s) violate font family requirements",
            )
        return CheckResult(
            status="PASS",
            detail="All text conforms to font family requirements",
        )
