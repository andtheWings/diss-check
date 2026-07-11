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
fn test_corpus_sweep() {
    let spec_path = catalog_path().join("institutions/iu/spec.yaml");
    let spec = load_spec(&spec_path).expect("Should load spec");
    let corpus_dir = catalog_path().join("institutions/iu/tests/corpus");

    let mut pdf_count = 0usize;
    let mut total_errors = 0usize;

    for entry in std::fs::read_dir(&corpus_dir).expect("corpus dir should exist") {
        let entry = entry.expect("should read entry");
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "pdf") {
            pdf_count += 1;
            let name = path.file_stem().unwrap().to_string_lossy();

            let results = run_checks(&spec, &path, &CheckOptions::default())
                .unwrap_or_else(|e| panic!("{}: run_checks failed: {}", name, e));

            let report = build_report(results);

            assert_eq!(
                report.results.len(),
                spec.checks.len(),
                "{}: expected {} checks, got {}",
                name,
                spec.checks.len(),
                report.results.len()
            );

            total_errors += report.summary.error;
        }
    }

    assert!(pdf_count > 0, "no PDFs found in corpus directory");

    // Allow a moderate number of errors across the corpus (some PDFs may be
    // incompatible with pdf_oxide or have genuine extraction issues)
    assert!(
        total_errors <= 50,
        "corpus sweep: {} total errors across {} PDFs (max 50)",
        total_errors,
        pdf_count
    );
}
