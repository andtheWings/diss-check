from pathlib import Path
from diss_check.spec import load_spec
from diss_check.engine import Engine
from diss_check.report import Report


def test_dissertation_produces_results(test_dissertation_path):
    """Run the full check suite against a known-accepted dissertation."""
    if not test_dissertation_path.exists():
        import pytest
        pytest.skip("Test dissertation not found")

    spec = load_spec(Path("specs/iu.yaml"))
    engine = Engine(spec)
    results = engine.run(test_dissertation_path)
    report = Report(results=results)

    assert len(report.results) == len(spec.checks)
    assert report.summary.error == 0

    print(f"\nResults: {report.summary.pass_} PASS, {report.summary.fail} FAIL, {report.summary.manual} MANUAL")
    for r in report.results:
        if r.status != "PASS":
            print(f"  {r.status}: {r.check_id} — {r.detail}")
