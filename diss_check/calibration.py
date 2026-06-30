from pathlib import Path

from pydantic import BaseModel

from diss_check.checkers.base import CheckResult
from diss_check.engine import Engine
from diss_check.report import Report, Summary
from diss_check.spec import InstitutionSpec


class DocumentResult(BaseModel):
    document: str
    results: list[CheckResult]
    summary: Summary


class CheckFrequency(BaseModel):
    check_id: str
    category: str
    automatable: bool
    pass_count: int = 0
    fail_count: int = 0
    manual_count: int = 0
    error_count: int = 0
    total_documents: int = 0
    fail_documents: list[str] = []
    fail_details: list[str] = []


class CalibrationReport(BaseModel):
    spec_path: str
    corpus_path: str
    documents: list[str]
    document_results: list[DocumentResult]
    check_frequencies: list[CheckFrequency]

    @property
    def systemic_threshold(self) -> float:
        return 0.5

    @property
    def automated_checks(self):
        automated_categories = {"layout", "typography", "structure", "content"}
        return [f for f in self.check_frequencies if f.category in automated_categories]

    @property
    def systemic_fail_count(self) -> int:
        return sum(
            1 for f in self.automated_checks
            if f.total_documents > 0
            and f.fail_count / f.total_documents >= self.systemic_threshold
            and f.fail_count >= 1
        )

    @property
    def automated_fail_count(self) -> int:
        return sum(
            1 for f in self.automated_checks
            if f.fail_count > 0
        )


def run_calibration(
    spec: InstitutionSpec, corpus_path: Path, spec_path: Path | str | None = None
) -> CalibrationReport:
    pdf_files = sorted(corpus_path.glob("*.pdf"))
    if not pdf_files:
        raise FileNotFoundError(f"No PDF files found in {corpus_path}")

    check_freqs: dict[str, CheckFrequency] = {}
    for check_def in spec.checks:
        check_freqs[check_def.id] = CheckFrequency(
            check_id=check_def.id,
            category=check_def.category,
            automatable=check_def.automatable,
        )

    engine = Engine(spec)
    document_results: list[DocumentResult] = []

    for pdf_path in pdf_files:
        results = engine.run(pdf_path)
        report = Report(results=results)
        document_results.append(DocumentResult(
            document=pdf_path.name,
            results=results,
            summary=report.summary,
        ))
        for result in results:
            freq = check_freqs[result.check_id]
            freq.total_documents += 1
            if result.status == "PASS":
                freq.pass_count += 1
            elif result.status == "FAIL":
                freq.fail_count += 1
                freq.fail_documents.append(pdf_path.name)
                freq.fail_details.append(f"[{pdf_path.name}] {result.detail}")
            elif result.status == "MANUAL":
                freq.manual_count += 1
            elif result.status == "ERROR":
                freq.error_count += 1

    return CalibrationReport(
        spec_path=str(spec_path) if spec_path else "N/A",
        corpus_path=str(corpus_path),
        documents=[p.name for p in pdf_files],
        document_results=document_results,
        check_frequencies=list(check_freqs.values()),
    )


def format_text(report: CalibrationReport) -> str:
    lines = [
        "=" * 70,
        "CALIBRATION REPORT",
        "=" * 70,
        f"Spec:     {report.spec_path}",
        f"Corpus:   {report.corpus_path} ({len(report.documents)} documents)",
        "",
        "Documents:",
    ]
    for doc in report.documents:
        lines.append(f"  - {doc}")

    lines.extend(["", "-" * 70])

    for doc_result in report.document_results:
        s = doc_result.summary
        summaries = []
        if s.pass_:
            summaries.append(f"{s.pass_} PASS")
        if s.fail:
            summaries.append(f"{s.fail} FAIL")
        if s.manual:
            summaries.append(f"{s.manual} MANUAL")
        if s.error:
            summaries.append(f"{s.error} ERROR")
        lines.append(f"  {doc_result.document}: {', '.join(summaries)}")

    lines.extend(["", "=" * 70, "", "AUTOMATED CHECKS", ""])

    automated_categories = {"layout", "typography", "structure", "content"}
    auto_checks = [f for f in report.check_frequencies if f.category in automated_categories]
    for freq in auto_checks:
        lines.append(f"{freq.check_id} [{freq.category}]")
        lines.append(f"  PASS={freq.pass_count} FAIL={freq.fail_count} ERROR={freq.error_count}")
        if freq.fail_count > 0:
            fail_ratio = freq.fail_count / freq.total_documents
            if fail_ratio >= report.systemic_threshold:
                lines.append(f"  STATUS: SYSTEMIC ({freq.fail_count}/{freq.total_documents} documents)")
            else:
                lines.append(f"  STATUS: isolated ({freq.fail_count}/{freq.total_documents} documents)")
            for detail in freq.fail_details:
                lines.append(f"    {detail}")
        else:
            lines.append(f"  STATUS: clean ({freq.pass_count}/{freq.total_documents} documents)")
        lines.append("")

    manual_checks = [f for f in report.check_frequencies if f.category not in automated_categories]
    if manual_checks:
        lines.extend(["", "MANUAL CHECKS", ""])
        for freq in manual_checks:
            lines.append(f"{freq.check_id} [{freq.category}] — MANUAL review ({freq.manual_count}/{freq.total_documents} docs)")
        lines.append("")

    lines.append("=" * 70)
    lines.append(f"Automated checks with \u22651 FAIL: {report.automated_fail_count}")
    systemic_pct = int(report.systemic_threshold * 100)
    lines.append(f"Systemic FAILs (\u2265{systemic_pct}% of corpus): {report.systemic_fail_count}")
    lines.append("=" * 70)

    return "\n".join(lines)


def format_json(report: CalibrationReport) -> str:
    return report.model_dump_json(indent=2)
