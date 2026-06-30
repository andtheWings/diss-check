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


def _page_text(page) -> str:
    return " ".join(s.text for s in page.spans).lower()


def _contains_keyword(text: str, section_id: str) -> bool:
    patterns = SECTION_KEYWORDS.get(section_id, section_id)
    for pattern in patterns.split("|"):
        if pattern in text:
            return True
    return False


@register_checker(category="structure", name="section_presence")
class SectionPresenceChecker(BaseChecker):
    requires = ["pdfplumber"]

    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        doc = ctx.document
        required = params.get("required_sections", [])

        found: set[str] = set()
        missing: list[str] = []

        for sec in required:
            sec_id = sec.get("id", "") if isinstance(sec, dict) else str(sec)
            if self._detect_section(doc, sec_id):
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

    def _detect_section(self, doc, section_id: str) -> bool:
        if section_id == "title_page":
            page1 = doc.pages[0]
            text = _page_text(page1)
            for span in page1.spans:
                if span.bottom > (page1.height - 50) and span.text.strip():
                    return False
            return bool(text.strip())

        for page in doc.pages:
            text = _page_text(page)
            if _contains_keyword(text, section_id):
                return True

        return False
