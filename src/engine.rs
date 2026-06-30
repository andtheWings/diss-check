use std::path::Path;
use crate::checkers::{CheckResult, Status, get_checker};
use crate::extractor::extract_document;
use crate::spec::InstitutionSpec;

pub fn run_checks(spec: &InstitutionSpec, pdf_path: &Path) -> Result<Vec<CheckResult>, Box<dyn std::error::Error>> {
    let doc = extract_document(pdf_path)?;
    let mut results: Vec<CheckResult> = Vec::new();

    for check_def in &spec.checks {
        if !check_def.automatable {
            results.push(CheckResult {
                check_id: check_def.id.clone(),
                status: Status::Manual,
                evidence: vec![],
                detail: check_def
                    .review_hint
                    .clone()
                    .unwrap_or_else(|| "Manual review required".to_string()),
            });
            continue;
        }

        match get_checker(&check_def.category, &check_def.checker) {
            Some(checker) => {
                let params = serde_yaml::to_value(&check_def.params).unwrap_or_default();
                let mut result = checker.check(&doc, &params);
                result.check_id = check_def.id.clone();
                results.push(result);
            }
            None => {
                results.push(CheckResult {
                    check_id: check_def.id.clone(),
                    status: Status::Error,
                    evidence: vec![],
                    detail: format!(
                        "No checker registered for {}/{}",
                        check_def.category, check_def.checker
                    ),
                });
            }
        }
    }

    Ok(results)
}
