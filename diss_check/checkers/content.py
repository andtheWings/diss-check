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


@register_checker(category="content", name="committee_order")
class CommitteeOrderChecker(BaseChecker):
    requires = ["pdfplumber"]

    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        doc = ctx.document
        chair_first = params.get("chair_first", True)
        page_filter = params.get("page")

        for page in doc.pages:
            if page_filter is not None and page.page_number != page_filter:
                continue

            committee = self._find_committee(page)
            if not committee:
                continue

            violations: list[EvidenceItem] = []

            if chair_first:
                for i, (name, is_chair) in enumerate(committee):
                    if is_chair and i > 0:
                        violations.append(EvidenceItem(
                            page=page.page_number,
                            excerpt=f"Chair '{name}' listed at position {i + 1}, should be first",
                        ))

                if violations:
                    return CheckResult(
                        status="FAIL",
                        evidence=violations,
                        detail=f"Committee chair not listed first",
                    )

            if not any(is_chair for _, is_chair in committee):
                return CheckResult(
                    status="FAIL",
                    evidence=[EvidenceItem(
                        page=page.page_number,
                        excerpt="Chair label missing — IU requires 'Chair' after chair's degrees",
                    )],
                    detail="Chair not explicitly labeled",
                )

            return CheckResult(
                status="PASS",
                detail=f"Committee chair listed first ({len(committee)} members)",
            )

        return CheckResult(status="ERROR", detail="Committee not found on specified page")

    def _find_committee(self, page) -> list[tuple[str, bool]]:
        lines = _page_lines(page)
        committee: list[tuple[str, bool]] = []
        in_committee = False

        for line in lines:
            low = line.lower()
            if "doctoral committee" in low or "committee" in low:
                in_committee = True
                continue

            if not in_committee:
                continue

            if "date of defense" in low or "defense date" in low:
                break

            if re.match(r"^[_\-\s]+$", line):
                continue

            if re.match(r"^(january|february|march|april|may|june|july|august|september|october|november|december)\s+\d{1,2},?\s+\d{4}$", low):
                continue

            is_name = bool(re.search(r"ph\.?\s*d\.?|m\.\s*p\.\s*a\.?|m\.\s*a\.?|m\.\s*s\.?|j\.?\s*d\.?|ed\.?\s*d\.?", low))
            if not is_name:
                if committee and not re.match(r"^[_\-\s]+$", line):
                    break
                continue

            is_chair = "chair" in low
            committee.append((line.strip(), is_chair))

        return committee


def _extract_toc_entries(page) -> list[tuple[str, int]]:
    lines = defaultdict(list)
    for s in page.spans:
        if s.text.strip():
            lines[round(s.top)].append(s)

    entries = []
    for top in sorted(lines.keys()):
        spans = sorted(lines[top], key=lambda s: s.x0)
        text = " ".join(s.text for s in spans).strip()
        m = re.match(r"^(.+?)\.{2,}\s*(\d+)\s*$", text)
        if m:
            entries.append((m.group(1).strip(), int(m.group(2))))
    return entries


def _extract_page_heading(page) -> str:
    lines = defaultdict(list)
    for s in page.spans:
        if s.text.strip() and s.top >= 72 and s.bottom <= page.height - 72:
            lines[round(s.top)].append(s)

    heading_parts = []
    started = False
    for top in sorted(lines.keys()):
        spans = sorted(lines[top], key=lambda s: s.x0)
        line = " ".join(s.text for s in spans).strip()

        is_chapter_line = bool(re.match(r"^chapter\s+\d+", line.lower())) or \
                          bool(re.match(r"^appendix\s+[a-z]", line.lower()))
        is_short = len(line) < 100 and not line.startswith("http")

        if is_chapter_line:
            started = True
            heading_parts.append(line)
        elif started and is_short and not re.match(r"^\d+$", line):
            heading_parts.append(line)
        elif started:
            break

    heading = " ".join(heading_parts)
    return heading


@register_checker(category="content", name="toc_title_parity")
class TocTitleParityChecker(BaseChecker):
    requires = ["pdfplumber"]

    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        doc = ctx.document

        toc_page = None
        for page in doc.pages:
            text = " ".join(s.text for s in page.spans).lower()
            if "table of contents" in text:
                toc_page = page
                break

        if toc_page is None:
            return CheckResult(status="ERROR", detail="TOC page not found")

        entries = _extract_toc_entries(toc_page)

        violations: list[EvidenceItem] = []
        checked = 0

        for entry_title, _entry_pg in entries:
            if not _contains_chapter_keyword(entry_title):
                continue

            entry_norm = _normalize_title(entry_title)

            found = False
            for page in doc.pages:
                if page.page_number == toc_page.page_number:
                    continue
                body_heading = _extract_page_heading(page)
                body_norm = _normalize_title(body_heading)
                if _titles_match_norm(entry_norm, body_norm):
                    found = True
                    break

            checked += 1
            if not found:
                violations.append(EvidenceItem(
                    page=0,
                    excerpt=f"TOC: \"{entry_title[:60]}\" — no matching heading found in body",
                ))

        if violations:
            return CheckResult(
                status="FAIL",
                evidence=violations,
                detail=f"{len(violations)}/{checked} chapter title(s) mismatch between TOC and body",
            )

        return CheckResult(
            status="PASS",
            detail=f"All {checked} chapter titles match between TOC and body",
        )


def _contains_chapter_keyword(title: str) -> bool:
    low = title.lower()
    return bool(re.match(r"^chapter\s+\d+", low)) or bool(re.match(r"^appendix\s+[a-z]", low))


def _normalize_title(title: str) -> str:
    t = re.sub(r"\s+", " ", title).strip().lower()
    t = re.sub(r"[–—\-—]+", "-", t)
    t = re.sub(r"[^a-z0-9\s\-:]", "", t)
    return t


def _titles_match_norm(a: str, b: str) -> bool:
    if a == b:
        return True
    if len(a) > 20 and a in b:
        return True
    if len(b) > 20 and b in a:
        return True

    a_words = set(a.split())
    b_words = set(b.split())
    if not a_words or not b_words:
        return False
    common = a_words & b_words
    if len(common) >= max(len(a_words), len(b_words)) * 0.7:
        return True

    return False
