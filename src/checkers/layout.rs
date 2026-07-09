use crate::checkers::{CheckResult, Checker, EvidenceItem, Status};
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

fn mean(values: &[f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f32>() / values.len() as f32
}

fn left_edge_ptile(spans: &[&crate::document::TextSpan]) -> Option<f32> {
    let mut x0s: Vec<i32> = spans.iter().map(|s| s.bbox.2.round() as i32).collect();
    if x0s.is_empty() {
        return None;
    }
    x0s.sort();
    let idx = (x0s.len() as f32 * 0.05) as usize;
    Some(x0s[idx.min(x0s.len() - 1)] as f32)
}

fn right_margin_ptile(spans: &[&crate::document::TextSpan], page_width: f32) -> Option<f32> {
    let mut margins: Vec<i32> = spans
        .iter()
        .map(|s| (page_width - s.bbox.3).round() as i32)
        .filter(|&m| m >= 0)
        .collect();
    if margins.is_empty() {
        return None;
    }
    margins.sort();
    let idx = (margins.len() as f32 * 0.05) as usize;
    Some(margins[idx.min(margins.len() - 1)] as f32)
}

fn check_edge(
    label: &str,
    values: &[f32],
    required: f32,
    tolerance: f32,
) -> Option<(bool, String)> {
    if values.is_empty() {
        return None;
    }
    let avg = mean(values);
    let lower = required - tolerance;
    let upper = required + tolerance;
    let pass = avg >= lower && avg <= upper;
    let status = if pass { "PASS" } else { "FAIL" };
    let direction = if avg < lower {
        " too narrow"
    } else if avg > upper {
        " too wide"
    } else {
        ""
    };
    Some((
        pass,
        format!(
            "Avg {}: {:.0}pt ({:.2}in) — range [{:.2}in–{:.2}in]. {}{}",
            label,
            avg,
            avg / 72.0,
            lower / 72.0,
            upper / 72.0,
            status,
            direction
        ),
    ))
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
        let top_req = parse_measurement(params["top"].as_str().unwrap_or("1in")).unwrap_or(72.0);
        let bottom_req =
            parse_measurement(params["bottom"].as_str().unwrap_or("1in")).unwrap_or(72.0);
        let left_req =
            parse_measurement(params["left"].as_str().unwrap_or("1.25in")).unwrap_or(90.0);
        let right_req =
            parse_measurement(params["right"].as_str().unwrap_or("1.25in")).unwrap_or(90.0);
        let tolerance =
            parse_measurement(params["tolerance"].as_str().unwrap_or("0.125in")).unwrap_or(9.0);

        let mut left_edges: Vec<f32> = Vec::new();
        let mut right_margins: Vec<f32> = Vec::new();
        let mut page_first_tops: Vec<f32> = Vec::new();
        let mut page_last_bottoms: Vec<f32> = Vec::new();

        for page in &doc.pages {
            let body: Vec<&crate::document::TextSpan> = page
                .spans
                .iter()
                .filter(|s| {
                    let (top, bottom, _x0, _x1) = s.bbox;
                    top >= 36.0 && bottom <= page.height - 53.0 && s.text.trim().len() >= 3
                })
                .collect();
            if body.is_empty() {
                continue;
            }

            if let Some(e) = left_edge_ptile(&body) {
                left_edges.push(e);
            }
            if let Some(e) = right_margin_ptile(&body, page.width) {
                right_margins.push(e);
            }
            if let Some(s) = body.iter().min_by(|a, b| {
                a.bbox
                    .0
                    .partial_cmp(&b.bbox.0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }) {
                page_first_tops.push(s.bbox.0);
            }
            if let Some(s) = body.iter().max_by(|a, b| {
                a.bbox
                    .1
                    .partial_cmp(&b.bbox.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }) {
                page_last_bottoms.push(page.height - s.bbox.1);
            }
        }

        if left_edges.is_empty() {
            return CheckResult {
                check_id: String::new(),
                status: Status::Error,
                evidence: vec![],
                detail: "Insufficient body text to measure margins".to_string(),
            };
        }

        let mut lines: Vec<String> = Vec::new();
        let mut violations: Vec<EvidenceItem> = Vec::new();

        for (label, values, req) in [
            ("left edge", &left_edges, left_req),
            ("right margin", &right_margins, right_req),
            ("top edge", &page_first_tops, top_req),
            ("bottom margin", &page_last_bottoms, bottom_req),
        ] {
            if let Some((pass, line)) = check_edge(label, values, req, tolerance) {
                lines.push(line);
                if !pass {
                    violations.push(EvidenceItem {
                        page: 0,
                        bbox: None,
                        excerpt: Some(format!(
                            "{} {}pt outside [{}-{}pt]",
                            label,
                            mean(values) as i32,
                            (req - tolerance) as i32,
                            (req + tolerance) as i32
                        )),
                    });
                }
            }
        }

        if violations.is_empty() {
            CheckResult {
                check_id: String::new(),
                status: Status::Pass,
                evidence: vec![],
                detail: lines.join("; "),
            }
        } else {
            CheckResult {
                check_id: String::new(),
                status: Status::Fail,
                detail: lines.join("; "),
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
        let threshold =
            parse_measurement(params["threshold"].as_str().unwrap_or("0.25in")).unwrap_or(18.0);
        let mut evidence: Vec<EvidenceItem> = Vec::new();
        let mut asymmetrical_pages = 0usize;

        for page in &doc.pages {
            let mut lefts: Vec<f32> = Vec::new();
            let mut rights: Vec<f32> = Vec::new();
            for span in &page.spans {
                let (top, bottom, x0, x1) = span.bbox;
                if bottom >= (page.height - 53.0) || top < 36.0 {
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
                let direction = if diff > 0.0 {
                    "left wider"
                } else {
                    "right wider"
                };
                evidence.push(EvidenceItem {
                    page: page.page_number,
                    bbox: None,
                    excerpt: Some(format!(
                        "asymmetry {:.0}pt ({:.2}in): L={:.0}pt R={:.0}pt ({})",
                        diff.abs(),
                        diff.abs() / 72.0,
                        left_mean,
                        right_mean,
                        direction
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
                    threshold / 72.0
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

    fn make_span(text: &str, bbox: (f32, f32, f32, f32)) -> TextSpan {
        TextSpan {
            text: text.to_string(),
            font_name: "TimesNewRoman".to_string(),
            font_size: 12.0,
            bbox,
            is_bold: false,
            is_italic: false,
            color: None,
        }
    }

    fn multi_page_doc(pages_data: Vec<Vec<(f32, f32, f32, f32)>>) -> Document {
        Document {
            pages: pages_data
                .iter()
                .enumerate()
                .map(|(i, spans)| Page {
                    page_number: i + 1,
                    width: 612.0,
                    height: 792.0,
                    spans: spans
                        .iter()
                        .map(|&b| make_span("body text line here", b))
                        .collect(),
                    images: vec![],
                    paths: vec![],
                })
                .collect(),
        }
    }

    fn default_params() -> Value {
        serde_yaml::from_str("top: 1in\nbottom: 1in\nleft: 1.25in\nright: 1.25in\n").unwrap()
    }

    fn body_spans(
        count: usize,
        left_x: f32,
        right_x: f32,
        top_start: f32,
        gap: f32,
    ) -> Vec<(f32, f32, f32, f32)> {
        (0..count)
            .map(|i| {
                let top = top_start + i as f32 * gap;
                (top, top + 12.0, left_x, right_x)
            })
            .collect()
    }

    #[test]
    fn test_margins_pass() {
        let pages = vec![body_spans(30, 94.0, 518.0, 80.0, 24.0)];
        let doc = multi_page_doc(pages);
        let r = MarginsChecker.check(&doc, &default_params());
        assert_eq!(r.status, Status::Pass, "{}", r.detail);
    }

    #[test]
    fn test_margins_fail_left_narrow() {
        let pages = vec![body_spans(30, 72.0, 518.0, 80.0, 24.0)];
        let doc = multi_page_doc(pages);
        let r = MarginsChecker.check(&doc, &default_params());
        assert_eq!(r.status, Status::Fail, "{}", r.detail);
    }

    #[test]
    fn test_margins_fail_right_narrow() {
        let pages = vec![body_spans(30, 94.0, 542.0, 80.0, 24.0)];
        let doc = multi_page_doc(pages);
        let r = MarginsChecker.check(&doc, &default_params());
        assert_eq!(r.status, Status::Fail, "{}", r.detail);
    }

    #[test]
    fn test_margins_at_boundary_pass() {
        let pages = vec![body_spans(30, 81.0, 531.0, 80.0, 24.0)];
        let doc = multi_page_doc(pages);
        let r = MarginsChecker.check(&doc, &default_params());
        assert_eq!(r.status, Status::Pass, "{}", r.detail);
    }

    #[test]
    fn test_margins_error_empty() {
        let doc = Document { pages: vec![] };
        let r = MarginsChecker.check(&doc, &default_params());
        assert_eq!(r.status, Status::Error);
    }
}
