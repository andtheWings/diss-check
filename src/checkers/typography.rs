use std::collections::HashMap;
use crate::checkers::{Checker, CheckResult, EvidenceItem, Status};
use crate::document::Document;
use serde_yaml::Value;

fn parse_measurement(value: &str) -> Result<f32, String> {
    let value = value.trim();
    if let Some(inches) = value.strip_suffix("in") {
        inches
            .trim()
            .parse::<f32>()
            .map(|v| v * 72.0)
            .map_err(|e| format!("Invalid inches: {}", e))
    } else if let Some(pts) = value.strip_suffix("pt") {
        pts.trim()
            .parse::<f32>()
            .map_err(|e| format!("Invalid points: {}", e))
    } else {
        Err(format!("Unsupported measurement: {}", value))
    }
}

pub struct FontSizeChecker;

impl Checker for FontSizeChecker {
    fn category(&self) -> &'static str {
        "typography"
    }

    fn name(&self) -> &'static str {
        "font_size"
    }

    fn check(&self, doc: &Document, params: &Value) -> CheckResult {
        let allowed: Vec<f32> = params
            .get("allowed")
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str())
                    .filter_map(|s| parse_measurement(s).ok())
                    .collect()
            })
            .unwrap_or_default();

        let tolerance: f32 = 0.5;
        let consistent = params
            .get("consistent")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut violations: Vec<EvidenceItem> = Vec::new();
        let mut body_sizes: HashMap<i32, usize> = HashMap::new();

        for page in &doc.pages {
            for span in &page.spans {
                let (top, bottom, _x0, _x1) = span.bbox;

                if bottom >= (page.height - 53.0) {
                    continue;
                }
                if top < 36.0 {
                    continue;
                }
                if span.text.trim().len() < 3 {
                    continue;
                }

                let size = span.font_size;

                if size < 8.5 {
                    continue;
                }

                if consistent {
                    let key = (size * 10.0).round() as i32;
                    *body_sizes.entry(key).or_insert(0) += 1;
                }

                if allowed.is_empty() {
                    continue;
                }

                let matched = allowed.iter().any(|a| (size - a).abs() <= tolerance);
                if !matched {
                    violations.push(EvidenceItem {
                        page: page.page_number,
                        bbox: Some(span.bbox),
                        excerpt: Some(format!(
                            "{} ({:.1}pt)",
                            span.text,
                            size,
                        )),
                    });
                }
            }
        }

        if consistent && !body_sizes.is_empty() && body_sizes.len() > 1 {
            let modal_decipt = body_sizes
                .iter()
                .max_by_key(|(_, count)| *count)
                .map(|(k, _)| *k)
                .unwrap_or(0);
            let modal_size = modal_decipt as f32 / 10.0;

            let mut body_violations: Vec<EvidenceItem> = Vec::new();
            for page in &doc.pages {
                for span in &page.spans {
                    let (top, bottom, _x0, _x1) = span.bbox;

                    if bottom >= (page.height - 53.0) || top < 36.0 {
                        continue;
                    }
                    if span.text.trim().len() < 3 {
                        continue;
                    }

                    let size = span.font_size;

                    if size < 8.5 {
                        continue;
                    }

                    if allowed.iter().any(|a| (size - a).abs() <= tolerance) {
                        continue;
                    }

                    if (size - modal_size).abs() > tolerance {
                        body_violations.push(EvidenceItem {
                            page: page.page_number,
                            bbox: Some(span.bbox),
                            excerpt: Some(format!(
                                "{} ({:.1}pt, expected {:.0}pt)",
                                span.text,
                                size,
                                modal_size,
                            )),
                        });
                    }
                }
            }

            if !body_violations.is_empty() {
                violations.extend(body_violations);
            }
        }

        if violations.is_empty() {
            CheckResult {
                check_id: String::new(),
                status: Status::Pass,
                evidence: vec![],
                detail: "All text conforms to font size requirements".to_string(),
            }
        } else {
            CheckResult {
                check_id: String::new(),
                status: Status::Fail,
                detail: format!(
                    "{} span(s) violate font size requirements",
                    violations.len(),
                ),
                evidence: violations,
            }
        }
    }
}
