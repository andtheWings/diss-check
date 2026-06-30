from pathlib import Path
from diss_check.spec import InstitutionSpec
from diss_check.document import ExtractionContext
from diss_check.checkers.base import get_checker, CheckResult
from diss_check.extractors.docling_extractor import DoclingExtractor
from diss_check.extractors.base import BaseExtractor


EXTRACTOR_MAP: dict[str, BaseExtractor] = {
    "docling": DoclingExtractor(),
}


class Engine:
    def __init__(self, spec: InstitutionSpec):
        self.spec = spec

    def run(self, pdf_path: Path) -> list[CheckResult]:
        required_extractors = self._collect_required_extractors()
        ctx = ExtractionContext()
        for ext_name in required_extractors:
            EXTRACTOR_MAP[ext_name].extract(pdf_path, ctx)

        results: list[CheckResult] = []
        for check_def in self.spec.checks:
            try:
                checker = get_checker(check_def.category, check_def.checker)
            except KeyError:
                results.append(CheckResult(
                    check_id=check_def.id,
                    status="ERROR",
                    detail=f"No checker registered for {check_def.category}/{check_def.checker}",
                ))
                continue
            if not check_def.automatable:
                results.append(CheckResult(
                    check_id=check_def.id,
                    status="MANUAL",
                    detail=check_def.review_hint or "Manual review required",
                ))
                continue
            result = checker.check(ctx, check_def.params)
            result.check_id = check_def.id
            results.append(result)

        return results

    def _collect_required_extractors(self) -> set[str]:
        required: set[str] = set()
        for check_def in self.spec.checks:
            if not check_def.automatable:
                continue
            try:
                checker_cls = type(get_checker(check_def.category, check_def.checker))
            except KeyError:
                continue
            required.update(checker_cls.requires)
        return required
