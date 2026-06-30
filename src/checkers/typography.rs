use std::collections::{HashMap, HashSet};
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

fn normalize_family(font_name: &str) -> String {
    let name = if let Some(idx) = font_name.find('+') {
        &font_name[idx + 1..]
    } else {
        font_name
    };

    let suffixes = [
        "PS", "MT", "-Regular", "-BoldItalic", "-Bold", "-Italic", "-Oblique",
    ];

    let mut result = name.to_string();
    for suffix in &suffixes {
        result = result.replace(suffix, "");
    }
    result.trim_matches('-').to_string()
}

fn is_internal_font_name(name: &str) -> bool {
    if name.len() < 4 {
        return true;
    }
    name.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()) && name.len() <= 6
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

pub struct FontWeightChecker;

impl Checker for FontWeightChecker {
    fn category(&self) -> &'static str {
        "typography"
    }

    fn name(&self) -> &'static str {
        "font_weight"
    }

    fn check(&self, doc: &Document, params: &Value) -> CheckResult {
        let expected = params
            .get("weight")
            .and_then(|v| v.as_str())
            .unwrap_or("normal");
        let page_filter = params.get("page").and_then(|v| v.as_u64()).map(|p| p as usize);
        let invert = params
            .get("invert")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut violations: Vec<EvidenceItem> = Vec::new();

        for page in &doc.pages {
            if let Some(target) = page_filter {
                if page.page_number != target {
                    continue;
                }
            }

            for span in &page.spans {
                if span.text.trim().is_empty() {
                    continue;
                }
                let (top, bottom, _x0, _x1) = span.bbox;
                if bottom >= (page.height - 53.0) {
                    continue;
                }
                if top < 36.0 {
                    continue;
                }

                let detected = match (span.is_bold, span.is_italic) {
                    (true, true) => "bold-italic",
                    (true, false) => "bold",
                    (false, true) => "italic",
                    (false, false) => "normal",
                };

                let is_violation = if invert {
                    detected == expected
                } else {
                    detected != expected
                };

                if is_violation {
                    let detail = if invert {
                        format!(
                            "{} ({}, should not be {})",
                            span.text, detected, expected,
                        )
                    } else {
                        format!(
                            "{} ({}, expected {})",
                            span.text, detected, expected,
                        )
                    };
                    violations.push(EvidenceItem {
                        page: page.page_number,
                        bbox: Some(span.bbox),
                        excerpt: Some(detail),
                    });
                }
            }
        }

        if violations.is_empty() {
            CheckResult {
                check_id: String::new(),
                status: Status::Pass,
                evidence: vec![],
                detail: "All text conforms to font weight requirements".to_string(),
            }
        } else {
            CheckResult {
                check_id: String::new(),
                status: Status::Fail,
                detail: format!(
                    "{} span(s) violate font weight requirements",
                    violations.len(),
                ),
                evidence: violations,
            }
        }
    }
}

pub struct FontFamilyChecker;

impl Checker for FontFamilyChecker {
    fn category(&self) -> &'static str {
        "typography"
    }

    fn name(&self) -> &'static str {
        "font_family"
    }

    fn check(&self, doc: &Document, params: &Value) -> CheckResult {
        let allowed: HashSet<String> = params
            .get("allowed")
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();
        let consistent = params
            .get("consistent")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let special_fonts: HashSet<&str> = [
            "Symbol", "Wingdings", "CambriaMath", "LucidaConsole", "ZapfDingbats",
        ]
        .iter()
        .cloned()
        .collect();

        let mut violations: Vec<EvidenceItem> = Vec::new();
        let mut family_counts: HashMap<String, usize> = HashMap::new();

        for page in &doc.pages {
            for span in &page.spans {
                if span.text.trim().is_empty() {
                    continue;
                }
                let (top, bottom, _x0, _x1) = span.bbox;
                if bottom >= (page.height - 53.0) {
                    continue;
                }
                if top < 36.0 {
                    continue;
                }

                let family = normalize_family(&span.font_name);

                if is_internal_font_name(&family) || special_fonts.contains(family.as_str()) {
                    continue;
                }

                if consistent {
                    *family_counts.entry(family.clone()).or_insert(0) += 1;
                }

                if allowed.is_empty() {
                    continue;
                }

                if !allowed.contains(&family) {
                    violations.push(EvidenceItem {
                        page: page.page_number,
                        bbox: Some(span.bbox),
                        excerpt: Some(format!("{} ({})", span.text, family)),
                    });
                }
            }
        }

        if consistent && !family_counts.is_empty() && family_counts.len() > 1 {
            let modal_family = family_counts
                .iter()
                .max_by_key(|(_, count)| *count)
                .map(|(k, _)| k.clone())
                .unwrap_or_default();

            for page in &doc.pages {
                for span in &page.spans {
                    if span.text.trim().is_empty() {
                        continue;
                    }
                    let (top, bottom, _x0, _x1) = span.bbox;
                    if bottom >= (page.height - 53.0) || top < 36.0 {
                        continue;
                    }
                    let family = normalize_family(&span.font_name);
                    if is_internal_font_name(&family) || special_fonts.contains(family.as_str()) {
                        continue;
                    }
                    if family != modal_family && (allowed.is_empty() || !allowed.contains(&family)) {
                        violations.push(EvidenceItem {
                            page: page.page_number,
                            bbox: Some(span.bbox),
                            excerpt: Some(format!(
                                "{} ({}, expected {})",
                                span.text, family, modal_family,
                            )),
                        });
                    }
                }
            }
        }

        if violations.is_empty() {
            CheckResult {
                check_id: String::new(),
                status: Status::Pass,
                evidence: vec![],
                detail: "All text conforms to font family requirements".to_string(),
            }
        } else {
            CheckResult {
                check_id: String::new(),
                status: Status::Fail,
                detail: format!(
                    "{} span(s) violate font family requirements",
                    violations.len(),
                ),
                evidence: violations,
            }
        }
    }
}
