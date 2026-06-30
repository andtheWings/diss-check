from collections import defaultdict

from diss_check.document import ExtractionContext
from diss_check.checkers.base import BaseChecker, CheckResult, EvidenceItem, register_checker


SECTION_KEYWORDS: dict[str, str] = {
    "title_page": "title page|title_page",
    "acceptance_page": "accepted by|acceptance",
    "abstract": "abstract",
    "toc": "table of contents|contents",
    "chapters": "chapter",
    "references": "references|bibliography|works cited",
    "curriculum_vitae": "curriculum vitae",
}

HEADING_SECTIONS = {"toc", "acceptance_page", "curriculum_vitae", "references", "chapters"}

NON_ABSTRACT_HEADINGS = {"dedication", "acknowledgement", "acknowledgments", "preface"}


def _page_text(page) -> str:
    return " ".join(s.text for s in page.spans).lower()


def _page_text_no_citations(page) -> str:
    lines = defaultdict(list)
    for s in page.spans:
        lines[round(s.top)].append(s)

    text_parts = []
    for top in sorted(lines.keys()):
        line = " ".join(s.text for s in lines[top])
        low = line.lower()
        if "doi:" in low or "http" in low or "https" in low:
            continue
        stripped = low.strip()
        if stripped and stripped[0].isdigit() and len(stripped) <= 5:
            continue
        text_parts.append(low)
    return " ".join(text_parts)


def _contains_keyword(text: str, section_id: str) -> bool:
    patterns = SECTION_KEYWORDS.get(section_id, section_id)
    for pattern in patterns.split("|"):
        if pattern in text:
            return True
    return False


def _find_all_sections(doc) -> dict[str, int]:
    sections: dict[str, int] = {}

    page1 = doc.pages[0]
    has_page_num = any(
        s.bottom > (page1.height - 50) and s.text.strip() for s in page1.spans
    )
    if not has_page_num and _page_text(page1).strip():
        sections["title_page"] = 1

    for page in doc.pages:
        text = _page_text_no_citations(page)
        for sec_id in HEADING_SECTIONS:
            if sec_id not in sections and _contains_keyword(text, sec_id):
                sections[sec_id] = page.page_number

    if "abstract" not in sections and "acceptance_page" in sections and "toc" in sections:
        acc_pg = sections["acceptance_page"]
        toc_pg = sections["toc"]
        for page in reversed(doc.pages):
            if acc_pg < page.page_number < toc_pg:
                n_spans = len([s for s in page.spans if s.text.strip()])
                text = _page_text(page)
                is_other = any(h in text[:200] for h in NON_ABSTRACT_HEADINGS)
                if n_spans > 100 and not is_other:
                    sections["abstract"] = page.page_number
                    break

    return sections


@register_checker(category="structure", name="section_presence")
class SectionPresenceChecker(BaseChecker):
    requires = ["pdfplumber"]

    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        doc = ctx.document
        required = params.get("required_sections", [])
        sections = _find_all_sections(doc)

        found: set[str] = set()
        missing: list[str] = []

        for sec in required:
            sec_id = sec.get("id", "") if isinstance(sec, dict) else str(sec)
            if sec_id in sections:
                found.add(sec_id)
            else:
                missing.append(sec_id)

        if missing:
            return CheckResult(
                status="FAIL",
                detail=f"Missing section(s): {', '.join(missing)}",
                evidence=[
                    EvidenceItem(page=0, excerpt=f"Section '{m}' not detected")
                    for m in missing
                ],
            )

        return CheckResult(
            status="PASS",
            detail=f"All required sections detected: {', '.join(sorted(found))}",
        )


@register_checker(category="structure", name="section_order")
class SectionOrderChecker(BaseChecker):
    requires = ["pdfplumber"]

    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        doc = ctx.document
        expected = params.get("expected_order", [])
        sections = _find_all_sections(doc)

        found_pages: list[tuple[str, int]] = []
        for sec in expected:
            sec_id = sec.get("id", "") if isinstance(sec, dict) else str(sec)
            pg = sections.get(sec_id)
            if pg is not None:
                found_pages.append((sec_id, pg))

        violations: list[EvidenceItem] = []
        for i in range(1, len(found_pages)):
            prev_id, prev_pg = found_pages[i - 1]
            curr_id, curr_pg = found_pages[i]
            if curr_pg < prev_pg:
                violations.append(EvidenceItem(
                    page=curr_pg,
                    excerpt=f"'{curr_id}' (p{curr_pg}) appears before '{prev_id}' (p{prev_pg})",
                ))
            elif curr_pg == prev_pg and prev_id != curr_id:
                violations.append(EvidenceItem(
                    page=curr_pg,
                    excerpt=f"'{curr_id}' and '{prev_id}' detected on same page {curr_pg}",
                ))

        if violations:
            return CheckResult(
                status="FAIL",
                evidence=violations,
                detail=f"{len(violations)} ordering violation(s) found",
            )

        found_names = [f"{n} (p{p})" for n, p in found_pages]
        return CheckResult(
            status="PASS",
            detail=f"Sections in correct order: {', '.join(found_names)}",
        )
