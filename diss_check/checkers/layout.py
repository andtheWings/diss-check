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
    requires = ["docling"]

    def check(self, ctx: ExtractionContext, params: dict) -> CheckResult:
        doc = ctx.docling_doc
        top_margin = _parse_measurement(params["top"])
        bottom_margin = _parse_measurement(params["bottom"])
        left_margin = _parse_measurement(params["left"])
        right_margin = _parse_measurement(params["right"])

        violations: list[EvidenceItem] = []

        for page_no, page in doc.pages.items():
            page_width = page.size.width if hasattr(page.size, 'width') else 612
            page_height = page.size.height if hasattr(page.size, 'height') else 792

            for text_item in doc.texts:
                if not text_item.prov:
                    continue
                prov = text_item.prov[0]
                # docling provenance can be nested: handle both [item] and [[item]]
                prov_item = prov[0] if hasattr(prov, '__getitem__') and not hasattr(prov, 'page_no') else prov
                if prov_item.page_no != page_no:
                    continue
                if prov_item.bbox is None:
                    continue

                bbox = prov_item.bbox
                l, t, r, b = bbox.l, bbox.t, bbox.r, bbox.b

                if l < left_margin:
                    violations.append(EvidenceItem(
                        page=page_no, bbox=(l, t, r, b),
                        excerpt=text_item.text[:100],
                    ))
                elif (page_width - r) < right_margin:
                    violations.append(EvidenceItem(
                        page=page_no, bbox=(l, t, r, b),
                        excerpt=text_item.text[:100],
                    ))
                elif t < top_margin:
                    violations.append(EvidenceItem(
                        page=page_no, bbox=(l, t, r, b),
                        excerpt=text_item.text[:100],
                    ))
                elif (page_height - b) < bottom_margin:
                    violations.append(EvidenceItem(
                        page=page_no, bbox=(l, t, r, b),
                        excerpt=text_item.text[:100],
                    ))

        if violations:
            return CheckResult(
                status="FAIL", evidence=violations,
                detail=f"{len(violations)} text block(s) violate margin requirements",
            )
        return CheckResult(
            status="PASS",
            detail="All text is within required margins",
        )
