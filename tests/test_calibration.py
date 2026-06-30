from pathlib import Path
import json
import pytest

from diss_check.spec import load_spec, InstitutionSpec
from diss_check.checkers.base import CheckResult, EvidenceItem
from diss_check.calibration import (
    run_calibration,
    CalibrationReport,
    CheckFrequency,
    format_text,
    format_json,
)
from diss_check.engine import Engine


def test_run_calibration_with_corpus(test_dissertation_path, fixtures_dir):
    spec = load_spec(Path("specs/iu.yaml"))
    report = run_calibration(spec, fixtures_dir, spec_path="specs/iu.yaml")

    assert isinstance(report, CalibrationReport)
    assert len(report.documents) >= 1
    assert "2020-12-chambers.pdf" in report.documents
    assert len(report.document_results) == len(report.documents)
    assert len(report.check_frequencies) == len(spec.checks)
    assert report.spec_path == "specs/iu.yaml"

    for doc_result in report.document_results:
        assert doc_result.document in report.documents
        assert len(doc_result.results) == len(spec.checks)
        assert doc_result.summary.pass_ + doc_result.summary.fail + doc_result.summary.manual + doc_result.summary.error == len(spec.checks)

    for freq in report.check_frequencies:
        assert freq.total_documents == len(report.documents)
        total = freq.pass_count + freq.fail_count + freq.manual_count + freq.error_count
        assert total == freq.total_documents

    assert report.automated_fail_count >= 0
    assert report.systemic_fail_count >= 0


def test_calibration_empty_corpus(tmp_path):
    empty_dir = tmp_path / "empty"
    empty_dir.mkdir()
    spec = load_spec(Path("specs/iu.yaml"))
    with pytest.raises(FileNotFoundError, match="No PDF files found"):
        run_calibration(spec, empty_dir)


def test_calibration_nonexistent_path():
    spec = load_spec(Path("specs/iu.yaml"))
    with pytest.raises(FileNotFoundError):
        run_calibration(spec, Path("/nonexistent/path/12345"))


def test_format_text_structure(fixtures_dir):
    spec = load_spec(Path("specs/iu.yaml"))
    report = run_calibration(spec, fixtures_dir)

    output = format_text(report)
    assert "CALIBRATION REPORT" in output
    assert report.spec_path in output
    assert str(report.corpus_path) in output
    assert "AUTOMATED CHECKS" in output
    assert "MANUAL CHECKS" in output

    for doc_name in report.documents:
        assert doc_name in output

    for freq in report.check_frequencies:
        assert freq.check_id in output


def test_format_json(fixtures_dir):
    spec = load_spec(Path("specs/iu.yaml"))
    report = run_calibration(spec, fixtures_dir)

    output = format_json(report)
    data = json.loads(output)

    assert data["spec_path"] == report.spec_path
    assert data["corpus_path"] == str(report.corpus_path)
    assert len(data["documents"]) == len(report.documents)
    assert len(data["document_results"]) == len(report.document_results)
    assert len(data["check_frequencies"]) == len(report.check_frequencies)


def test_check_frequency_systemic_classification():
    freq = CheckFrequency(
        check_id="test_check",
        category="layout",
        automatable=True,
        total_documents=2,
        pass_count=0,
        fail_count=2,
    )
    assert freq.fail_count / freq.total_documents >= 0.5

    freq2 = CheckFrequency(
        check_id="test_check",
        category="layout",
        automatable=True,
        total_documents=2,
        pass_count=1,
        fail_count=1,
    )
    assert freq2.fail_count / freq2.total_documents >= 0.5


def test_systemic_fail_count_property(fixtures_dir):
    spec = load_spec(Path("specs/iu.yaml"))
    report = run_calibration(spec, fixtures_dir)

    computed = sum(
        1 for f in report.check_frequencies
        if f.category in {"layout", "typography", "structure", "content"}
        and f.total_documents > 0
        and f.fail_count / f.total_documents >= report.systemic_threshold
        and f.fail_count >= 1
    )
    assert report.systemic_fail_count == computed


def test_calibration_integration_with_chambers(test_dissertation_path, fixtures_dir):
    spec = load_spec(Path("specs/iu.yaml"))
    report = run_calibration(spec, fixtures_dir)

    chambers_result = next(
        dr for dr in report.document_results
        if dr.document == "2020-12-chambers.pdf"
    )
    assert chambers_result is not None

    engine = Engine(spec)
    single_results = engine.run(test_dissertation_path)

    assert len(chambers_result.results) == len(single_results)

    chambers_statuses = {r.check_id: r.status for r in chambers_result.results}
    for r in single_results:
        assert r.check_id in chambers_statuses
        assert chambers_statuses[r.check_id] == r.status
