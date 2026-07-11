use diss_check::checkers::Status;
use diss_check::engine::{run_checks, CheckOptions};
use diss_check::report::build_report;
use diss_check::spec::load_spec;
use std::path::PathBuf;

fn catalog_path() -> PathBuf {
    std::env::var("CATALOG_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("../scholarpress-catalog"))
}

#[test]
fn test_synthetic_margin_variants() {
    let spec_path = catalog_path().join("institutions/iu/spec.yaml");
    let spec = load_spec(&spec_path).expect("Should load spec");

    // Expected: (name, global_margins_status, describes_what)
    // Bottom margin is inherently noisy with typeset text (line-grid alignment);
    // bottom-narrow is excluded because a 0.5in margin can't be distinguished from
    // text that simply doesn't fill to the bottom.
    let variants: Vec<(&str, Status, &str)> = vec![
        ("baseline", Status::Fail, "baseline: bottom margin artifact (known typeset limitation)"),
        ("left-narrow", Status::Fail, "left 0.75in < 1.25in-0.125in"),
        ("right-narrow", Status::Fail, "right 0.75in < 1.25in-0.125in"),
        ("left-wide", Status::Fail, "left 1.75in > 1.25in+0.125in"),
        ("right-wide", Status::Fail, "right 1.75in > 1.25in+0.125in"),
        ("top-narrow", Status::Fail, "top 0.5in < 1in-0.125in"),
        ("top-wide", Status::Fail, "top 2.0in > 1in+0.125in"),
        ("asymmetric", Status::Fail, "L=1.5,R=1.0 — both out of band + asymmetric"),
    ];

    for (name, expected_margins, desc) in &variants {
        let pdf_path = catalog_path().join(format!("institutions/iu/tests/fixtures/{}.pdf", name));
        if !pdf_path.exists() {
            eprintln!("Test PDF {} not found, skipping", name);
            continue;
        }

        let results = run_checks(&spec, &pdf_path, &CheckOptions::default())
            .unwrap_or_else(|e| panic!("{}: run_checks failed: {}", name, e));
        let report = build_report(results);

        let margins = report
            .results
            .iter()
            .find(|r| r.check_id == "global_margins")
            .unwrap_or_else(|| panic!("{}: global_margins not found", name));

        assert_eq!(
            margins.status, *expected_margins,
            "{}: global_margins expected {:?}, got {:?}. detail: {} ({})",
            name, expected_margins, margins.status, margins.detail, desc
        );
    }
}

#[test]
fn test_synthetic_symmetry_fail() {
    let spec_path = catalog_path().join("institutions/iu/spec.yaml");
    let spec = load_spec(&spec_path).expect("Should load spec");

    // Documents with genuinely different left/right margins must FAIL symmetry.
    let fail_variants = ["left-narrow", "right-narrow", "asymmetric"];

    for name in &fail_variants {
        let pdf_path = catalog_path().join(format!("institutions/iu/tests/fixtures/{}.pdf", name));
        if !pdf_path.exists() {
            continue;
        }
        let results = run_checks(&spec, &pdf_path, &CheckOptions::default())
            .unwrap_or_else(|e| panic!("{}: run_checks failed: {}", name, e));
        let report = build_report(results);
        let symmetry = report
            .results
            .iter()
            .find(|r| r.check_id == "margin_symmetry")
            .unwrap_or_else(|| panic!("{}: margin_symmetry not found", name));

        assert_eq!(
            symmetry.status, Status::Fail,
            "{}: margin_symmetry expected FAIL (L!=R), got {:?}. detail: {}",
            name, symmetry.status, symmetry.detail
        );
    }
}

#[test]
fn test_synthetic_messy() {
    let spec_path = catalog_path().join("institutions/iu/spec.yaml");
    let spec = load_spec(&spec_path).expect("Should load spec");
    let pdf_path = catalog_path().join("institutions/iu/tests/fixtures/messy.pdf");
    if !pdf_path.exists() {
        eprintln!("messy.pdf not found, skipping");
        return;
    }

    let results = run_checks(&spec, &pdf_path, &CheckOptions::default())
        .expect("messy: run_checks failed");
    let report = build_report(results);

    let margins = report
        .results
        .iter()
        .find(|r| r.check_id == "global_margins")
        .unwrap();

    let symmetry = report
        .results
        .iter()
        .find(|r| r.check_id == "margin_symmetry")
        .unwrap();

    // Left/right margins should be correctly measured from body pages.
    // Top/bottom may have artifacts from sparse pages.
    let detail = &margins.detail;
    assert!(
        detail.contains("left edge: 90pt") || detail.contains("left edge: 91pt") || detail.contains("left edge: 92pt"),
        "messy: left edge should be ~90pt. detail: {}", detail
    );
    assert!(
        detail.contains("right margin: 90pt") || detail.contains("right margin: 91pt") || detail.contains("right margin: 92pt"),
        "messy: right margin should be ~90pt. detail: {}", detail
    );
    assert_eq!(
        symmetry.status, Status::Pass,
        "messy: symmetry should PASS (body pages are symmetric, sparse page 3 skipped). detail: {}",
        symmetry.detail
    );
}

#[test]
fn test_synthetic_baseline_measures_correctly() {
    let spec_path = catalog_path().join("institutions/iu/spec.yaml");
    let spec = load_spec(&spec_path).expect("Should load spec");
    let pdf_path = catalog_path().join("institutions/iu/tests/fixtures/baseline.pdf");
    if !pdf_path.exists() {
        return;
    }
    let results = run_checks(&spec, &pdf_path, &CheckOptions::default())
        .expect("baseline: run_checks failed");
    let report = build_report(results);
    let margins = report
        .results
        .iter()
        .find(|r| r.check_id == "global_margins")
        .unwrap();

    let detail = &margins.detail;
    // Verify left and right margins are measured at the correct 90pt (1.25in)
    assert!(
        detail.contains("left edge: 90pt") || detail.contains("left edge: 91pt") || detail.contains("left edge: 92pt"),
        "baseline: left edge should be ~90pt (1.25in). detail: {}", detail
    );
    assert!(
        detail.contains("right margin: 90pt") || detail.contains("right margin: 91pt") || detail.contains("right margin: 92pt"),
        "baseline: right margin should be ~90pt (1.25in). detail: {}", detail
    );
    // Top margin should be ~72pt (1in), but 68pt is within typical range
    assert!(
        detail.contains("top edge: 68pt") || detail.contains("top edge: 69pt") || detail.contains("top edge: 70pt") || detail.contains("top edge: 71pt") || detail.contains("top edge: 72pt"),
        "baseline: top edge should be near 72pt (1in). detail: {}", detail
    );
}
