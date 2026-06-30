from diss_check.report import Report, format_text
from diss_check.checkers.base import CheckResult


def test_report_summary_counts():
    results = [
        CheckResult(check_id="c1", status="PASS", detail="ok"),
        CheckResult(check_id="c2", status="FAIL", detail="bad"),
        CheckResult(check_id="c3", status="MANUAL", detail="check"),
    ]
    report = Report(results=results)
    assert report.summary.pass_ == 1
    assert report.summary.fail == 1
    assert report.summary.manual == 1
    assert report.summary.error == 0


def test_format_text_includes_statuses():
    results = [
        CheckResult(check_id="c1", status="PASS", detail="ok"),
        CheckResult(check_id="c2", status="FAIL", detail="bad margin on page 3"),
    ]
    report = Report(results=results)
    output = format_text(report)
    assert "c1" in output
    assert "c2" in output
    assert "PASS" in output
    assert "FAIL" in output
    assert "bad margin on page 3" in output
