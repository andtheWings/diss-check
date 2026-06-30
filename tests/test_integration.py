import pytest
from diss_check.spec import load_spec
from diss_check.engine import Engine
from diss_check.report import Report


def test_iu_template_produces_results(iu_template_path):
    """Run the full MVP check suite against the IU template PDF."""
    if not iu_template_path.exists():
        pytest.skip("IU template PDF not found — place it at tests/fixtures/iu_template.pdf")

    spec = load_spec("specs/iu.yaml")
    engine = Engine(spec)
    results = engine.run(iu_template_path)
    report = Report(results=results)

    assert len(report.results) == len(spec.checks)
    assert report.summary.error == 0

    # Print results for debugging
    print(f"\nResults: {report.summary.pass_} PASS, {report.summary.fail} FAIL, {report.summary.manual} MANUAL")
    for r in report.results:
        if r.status != "PASS":
            print(f"  {r.status}: {r.check_id} — {r.detail}")
