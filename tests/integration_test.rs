use std::path::PathBuf;
use diss_check::spec::load_spec;
use diss_check::engine::run_checks;
use diss_check::report::build_report;
use diss_check::checkers::Status;

#[test]
fn test_run_against_chambers() {
    let spec_path = PathBuf::from("specs/iu.yaml");
    let pdf_path = PathBuf::from("tests/fixtures/2020-12-chambers.pdf");

    if !pdf_path.exists() {
        eprintln!("Test PDF not found, skipping");
        return;
    }

    let spec = load_spec(&spec_path).expect("Should load spec");
    let results = run_checks(&spec, &pdf_path).expect("Should run checks");

    assert_eq!(results.len(), spec.checks.len());

    let report = build_report(results);
    assert_eq!(report.summary.error, 22);

    let justification = report.results.iter().find(|r| r.check_id == "justification_consistent").unwrap();
    assert_eq!(justification.status, Status::Pass);

    let margins = report.results.iter().find(|r| r.check_id == "global_margins").unwrap();
    assert_eq!(margins.status, Status::Fail);

    let symmetry = report.results.iter().find(|r| r.check_id == "margin_symmetry").unwrap();
    assert_eq!(symmetry.status, Status::Fail);

    let font_size = report.results.iter().find(|r| r.check_id == "font_size_consistent").unwrap();
    assert_eq!(font_size.status, Status::Pass);

    let font_weight = report.results.iter().find(|r| r.check_id == "title_page_no_bold").unwrap();
    assert_eq!(font_weight.status, Status::Pass);

    let font_family = report.results.iter().find(|r| r.check_id == "font_family_consistent").unwrap();
    assert_eq!(font_family.status, Status::Pass);
}

#[test]
fn test_run_against_alexander() {
    let spec_path = PathBuf::from("specs/iu.yaml");
    let pdf_path = PathBuf::from("tests/fixtures/2025-06-alexander.pdf");

    if !pdf_path.exists() {
        eprintln!("Test PDF not found, skipping");
        return;
    }

    let spec = load_spec(&spec_path).expect("Should load spec");
    let results = run_checks(&spec, &pdf_path).expect("Should run checks");
    let report = build_report(results);

    let margins = report.results.iter().find(|r| r.check_id == "global_margins").unwrap();
    assert_eq!(margins.status, Status::Fail);
    assert!(!margins.evidence.is_empty());
}
