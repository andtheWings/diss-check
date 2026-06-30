import re
from collections import defaultdict

from diss_check.document import ExtractionContext
from diss_check.checkers.base import BaseChecker, CheckResult, EvidenceItem, register_checker


def _page_lines(page) -> list[str]:
    lines = defaultdict(list)
    for s in page.spans:
        if s.text.strip():
            lines[round(s.top)].append(s)
    result = []
    for top in sorted(lines.keys()):
        spans = sorted(lines[top], key=lambda s: s.x0)
        result.append(" ".join(s.text for s in spans).strip())
    return result


def _normalize(s: str) -> str:
    return re.sub(r"\s+", " ", s).strip().lower()


@register_checker(category="content", name="boilerplate_match")
class BoilerplateMatchChecker(BaseChecker):
    requires = ["pdfplumber"]

    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        doc = ctx.document
        template = params.get("template", "")
        page_filter = params.get("page")

        if not template.strip():
            return CheckResult(status="PASS", detail="No template provided")

        template_lines = [
            _normalize(line) for line in template.strip().splitlines() if line.strip()
        ]

        violations: list[EvidenceItem] = []
        matched = False

        for page in doc.pages:
            if page_filter is not None and page.page_number != page_filter:
                continue

            page_lines = _page_lines(page)
            normed = [_normalize(l) for l in page_lines]

            if self._match_template(template_lines, normed):
                matched = True
                break

        if not matched:
            violations.append(EvidenceItem(
                page=page_filter or 0,
                excerpt="Template text not found on page",
            ))

        if violations:
            return CheckResult(
                status="FAIL",
                evidence=violations,
                detail=f"Template text not found on specified page",
            )

        return CheckResult(
            status="PASS",
            detail="Template text matches",
        )

    def _match_template(self, template_lines: list[str], page_lines: list[str]) -> bool:
        ti = 0
        pi = 0
        while ti < len(template_lines) and pi < len(page_lines):
            matched, consumed = self._line_matches_multi(
                template_lines[ti], page_lines, pi
            )
            if matched:
                ti += 1
                pi += consumed
            else:
                pi += 1
        return ti == len(template_lines)

    def _line_matches_multi(
        self, template_line: str, page_lines: list[str], start: int
    ) -> tuple[bool, int]:
        var_name = None
        if re.match(r"^\{(\w+)\}$", template_line):
            var_name = re.match(r"^\{(\w+)\}$", template_line).group(1)
        for n in range(1, min(4, len(page_lines) - start + 1)):
            joined = " ".join(page_lines[start : start + n])
            if self._line_matches(template_line, joined):
                return True, n
        return False, 1

    def _line_matches(self, template_line: str, page_line: str) -> bool:
        pattern = re.escape(template_line.rstrip(",.;:"))
        pattern = re.sub(r"\\\{(\w+)\\\}", r"(.+)", pattern)
        pattern = "^" + pattern + "[,.;:]?$"
        return bool(re.match(pattern, page_line, re.IGNORECASE))
