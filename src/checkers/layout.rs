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

pub struct MarginsChecker;

impl Checker for MarginsChecker {
    fn category(&self) -> &'static str {
        "layout"
    }

    fn name(&self) -> &'static str {
        "margins"
    }

    fn check(&self, doc: &Document, params: &Value) -> CheckResult {
        let top_margin = parse_measurement(
            params["top"].as_str().unwrap_or("1in"),
        )
        .unwrap_or(72.0);
        let bottom_margin = parse_measurement(
            params["bottom"].as_str().unwrap_or("1in"),
        )
        .unwrap_or(72.0);
        let left_margin = parse_measurement(
            params["left"].as_str().unwrap_or("1.25in"),
        )
        .unwrap_or(90.0);
        let right_margin = parse_measurement(
            params["right"].as_str().unwrap_or("1.25in"),
        )
        .unwrap_or(90.0);
        let tolerance = parse_measurement(
            params["tolerance"].as_str().unwrap_or("0.25in"),
        )
        .unwrap_or(18.0);

        let mut violations: Vec<EvidenceItem> = Vec::new();

        for page in &doc.pages {
            for span in &page.spans {
                let (top, bottom, x0, x1) = span.bbox;

                if top < 36.0 {
                    continue;
                }
                if span.text.trim().len() < 3 {
                    continue;
                }

                let in_page_number_zone = bottom >= (page.height - 53.0);

                if top < top_margin - tolerance {
                    violations.push(EvidenceItem {
                        page: page.page_number,
                        bbox: Some(span.bbox),
                        excerpt: Some(span.text.clone()),
                    });
                } else if !in_page_number_zone && bottom > (page.height - bottom_margin + tolerance) {
                    violations.push(EvidenceItem {
                        page: page.page_number,
                        bbox: Some(span.bbox),
                        excerpt: Some(span.text.clone()),
                    });
                } else if x0 < left_margin - tolerance {
                    violations.push(EvidenceItem {
                        page: page.page_number,
                        bbox: Some(span.bbox),
                        excerpt: Some(span.text.clone()),
                    });
                } else if x1 > (page.width - right_margin + tolerance) {
                    violations.push(EvidenceItem {
                        page: page.page_number,
                        bbox: Some(span.bbox),
                        excerpt: Some(span.text.clone()),
                    });
                }
            }
        }

        if violations.is_empty() {
            CheckResult {
                check_id: String::new(),
                status: Status::Pass,
                evidence: vec![],
                detail: "All text is within required margins".to_string(),
            }
        } else {
            CheckResult {
                check_id: String::new(),
                status: Status::Fail,
                detail: format!("{} word(s) violate margin requirements", violations.len()),
                evidence: violations,
            }
        }
    }
}

pub struct MarginSymmetryChecker;

impl Checker for MarginSymmetryChecker {
    fn category(&self) -> &'static str {
        "layout"
    }

    fn name(&self) -> &'static str {
        "margin_symmetry"
    }

    fn check(&self, doc: &Document, params: &Value) -> CheckResult {
        let threshold = parse_measurement(
            params["threshold"].as_str().unwrap_or("0.25in"),
        )
        .unwrap_or(18.0);

        let mut evidence: Vec<EvidenceItem> = Vec::new();
        let mut asymmetrical_pages = 0usize;

        for page in &doc.pages {
            let mut lefts: Vec<f32> = Vec::new();
            let mut rights: Vec<f32> = Vec::new();

            for span in &page.spans {
                let (top, bottom, x0, x1) = span.bbox;
                if bottom >= (page.height - 53.0) {
                    continue;
                }
                if top < 36.0 {
                    continue;
                }
                if span.text.trim().len() < 3 {
                    continue;
                }
                lefts.push(x0);
                rights.push(page.width - x1);
            }

            if lefts.len() < 10 {
                continue;
            }

            let left_mean = lefts.iter().sum::<f32>() / lefts.len() as f32;
            let right_mean = rights.iter().sum::<f32>() / rights.len() as f32;
            let diff = left_mean - right_mean;

            if diff.abs() > threshold {
                asymmetrical_pages += 1;
                let direction = if diff > 0.0 { "left wider" } else { "right wider" };
                evidence.push(EvidenceItem {
                    page: page.page_number,
                    bbox: None,
                    excerpt: Some(format!(
                        "asymmetry {:.0}pt ({:.2}in): L={:.0}pt R={:.0}pt ({})",
                        diff.abs(),
                        diff.abs() / 72.0,
                        left_mean,
                        right_mean,
                        direction,
                    )),
                });
            }
        }

        if asymmetrical_pages == 0 {
            CheckResult {
                check_id: String::new(),
                status: Status::Pass,
                evidence: vec![],
                detail: "Left and right margins are symmetric".to_string(),
            }
        } else {
            CheckResult {
                check_id: String::new(),
                status: Status::Fail,
                detail: format!(
                    "{} page(s) have asymmetric margins (threshold: {:.0}pt / {:.2}in)",
                    asymmetrical_pages,
                    threshold,
                    threshold / 72.0,
                ),
                evidence,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{Page, TextSpan};

    fn make_page(spans: Vec<TextSpan>) -> Page {
        Page {
            page_number: 1,
            width: 612.0,
            height: 792.0,
            spans,
        }
    }

    fn make_span(text: &str, bbox: (f32, f32, f32, f32)) -> TextSpan {
        TextSpan {
            text: text.to_string(),
            font_name: "TimesNewRoman".to_string(),
            font_size: 12.0,
            bbox,
            is_bold: false,
            is_italic: false,
        }
    }

    fn make_doc(spans: Vec<TextSpan>) -> Document {
        Document {
            pages: vec![make_page(spans)],
        }
    }

    fn default_params() -> Value {
        serde_yaml::from_str(
            "top: 1in\nbottom: 1in\nleft: 1.25in\nright: 1.25in\n",
        )
        .unwrap()
    }

    #[test]
    fn test_margins_pass_when_text_within_bounds() {
        let checker = MarginsChecker;
        let doc = make_doc(vec![make_span("Hello", (100.0, 112.0, 92.0, 130.0))]);
        let result = checker.check(&doc, &default_params());
        assert_eq!(result.status, Status::Pass);
    }

    #[test]
    fn test_margins_pass_with_tolerance() {
        let checker = MarginsChecker;
        let doc = make_doc(vec![make_span("Hello", (100.0, 112.0, 74.0, 130.0))]);
        let result = checker.check(&doc, &default_params());
        assert_eq!(result.status, Status::Pass);
    }

    #[test]
    fn test_margins_fail_when_well_past_margin() {
        let checker = MarginsChecker;
        let doc = make_doc(vec![make_span("Hello", (100.0, 112.0, 30.0, 130.0))]);
        let result = checker.check(&doc, &default_params());
        assert_eq!(result.status, Status::Fail);
        assert_eq!(result.evidence.len(), 1);
    }

    #[test]
    fn test_margins_fail_above_top() {
        let checker = MarginsChecker;
        let doc = make_doc(vec![make_span("Hello", (40.0, 52.0, 92.0, 200.0))]);
        let result = checker.check(&doc, &default_params());
        assert_eq!(result.status, Status::Fail);
    }

    #[test]
    fn test_margins_fail_below_bottom() {
        let checker = MarginsChecker;
        let mut page = make_page(vec![make_span("Hello", (338.0, 346.5, 92.0, 200.0))]);
        page.height = 400.0;
        let doc = Document { pages: vec![page] };
        let result = checker.check(&doc, &default_params());
        assert_eq!(result.status, Status::Fail);
    }

    #[test]
    fn test_margins_fail_past_right() {
        let checker = MarginsChecker;
        let doc = make_doc(vec![make_span("Hello", (100.0, 112.0, 92.0, 550.0))]);
        let result = checker.check(&doc, &default_params());
        assert_eq!(result.status, Status::Fail);
    }

    #[test]
    fn test_margins_skips_page_number_zone() {
        let checker = MarginsChecker;
        let doc = make_doc(vec![make_span("10", (741.0, 753.0, 300.0, 310.0))]);
        let result = checker.check(&doc, &default_params());
        assert_eq!(result.status, Status::Pass);
    }

    #[test]
    fn test_margins_skips_header_zone() {
        let checker = MarginsChecker;
        let doc = make_doc(vec![make_span("Author", (10.0, 22.0, 200.0, 300.0))]);
        let result = checker.check(&doc, &default_params());
        assert_eq!(result.status, Status::Pass);
    }
}
