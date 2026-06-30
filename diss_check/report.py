from pydantic import BaseModel
from diss_check.checkers.base import CheckResult


class Summary(BaseModel):
    pass_: int = 0
    fail: int = 0
    manual: int = 0
    error: int = 0


class Report(BaseModel):
    results: list[CheckResult]
    summary: Summary | None = None

    def model_post_init(self, __context):
        summary = Summary()
        for r in self.results:
            if r.status == "PASS":
                summary.pass_ += 1
            elif r.status == "FAIL":
                summary.fail += 1
            elif r.status == "MANUAL":
                summary.manual += 1
            elif r.status == "ERROR":
                summary.error += 1
        self.summary = summary


def format_text(report: Report) -> str:
    lines = ["=" * 60, "DISSERTATION FORMAT CHECK REPORT", "=" * 60]
    for result in report.results:
        marker = {"PASS": "[PASS]", "FAIL": "[FAIL]", "MANUAL": "[MANUAL]", "ERROR": "[ERROR]"}[result.status]
        lines.append(f"\n{marker} {result.check_id}")
        if result.detail:
            lines.append(f"  {result.detail}")
        for ev in result.evidence:
            page_info = f"page {ev.page}"
            if ev.bbox:
                page_info += f" @ ({ev.bbox[0]:.0f},{ev.bbox[1]:.0f},{ev.bbox[2]:.0f},{ev.bbox[3]:.0f})"
            lines.append(f"    [{page_info}] {ev.excerpt or ''}")
    s = report.summary
    lines.append(f"\n{'─' * 60}")
    lines.append(f"Summary: {s.pass_} PASS, {s.fail} FAIL, {s.manual} MANUAL, {s.error} ERROR")
    lines.append("=" * 60)
    return "\n".join(lines)
