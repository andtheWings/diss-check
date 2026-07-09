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

fn dominant_cluster(values: &[f32], proximity: f32, min_count: usize) -> Option<f32> {
    if values.is_empty() {
        return None;
    }

    let mut sorted: Vec<f32> = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mut best_center: f32 = sorted[0];
    let mut best_count: usize = 0;

    // Anchor on first value of each cluster (prevents drift; tighter than running-mean center)
    let mut cluster_start: f32 = sorted[0];
    let mut cluster_sum: f32 = 0.0;
    let mut cluster_count: usize = 0;

    for &v in &sorted {
        if (v - cluster_start).abs() <= proximity {
            cluster_sum += v;
            cluster_count += 1;
        } else {
            if cluster_count > best_count {
                best_count = cluster_count;
                best_center = cluster_sum / cluster_count as f32;
            }
            cluster_start = v;
            cluster_sum = v;
            cluster_count = 1;
        }
    }

    if cluster_count > best_count {
        best_count = cluster_count;
        best_center = cluster_sum / cluster_count as f32;
    }

    if best_count >= min_count {
        Some(best_center)
    } else {
        None
    }
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

        let mut excluded_pages: std::collections::HashSet<usize> = std::collections::HashSet::new();
        excluded_pages.insert(1);
        for pg in super::sections::find_section_pages(doc, &["accepted by"]) {
            excluded_pages.insert(pg);
        }
        for pg in super::sections::find_section_pages(doc, &["©", "copyright"]) {
            excluded_pages.insert(pg);
        }
        for pg in super::sections::find_section_pages(doc, &["dedication"]) {
            excluded_pages.insert(pg);
        }

        let mut left_edges: Vec<f32> = Vec::new();
        let mut right_margins: Vec<f32> = Vec::new();
        let mut page_first_tops: Vec<f32> = Vec::new();
        let mut page_last_bottoms: Vec<f32> = Vec::new();

        for page in &doc.pages {
            if excluded_pages.contains(&page.page_number) {
                continue;
            }

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

            let x0s: Vec<f32> = body.iter().map(|s| s.bbox.2).collect();
            if let Some(e) = dominant_cluster(&x0s, 4.0, 5) {
                left_edges.push(e);
            }

            let right_gaps: Vec<f32> = body
                .iter()
                .map(|s| (page.width - s.bbox.3).max(0.0))
                .collect();
            if let Some(e) = dominant_cluster(&right_gaps, 4.0, 5) {
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

        let mut excluded_pages: std::collections::HashSet<usize> = std::collections::HashSet::new();
        excluded_pages.insert(1);
        for pg in super::sections::find_section_pages(doc, &["accepted by"]) {
            excluded_pages.insert(pg);
        }
        for pg in super::sections::find_section_pages(doc, &["©", "copyright"]) {
            excluded_pages.insert(pg);
        }
        for pg in super::sections::find_section_pages(doc, &["dedication"]) {
            excluded_pages.insert(pg);
        }

        let mut evidence: Vec<EvidenceItem> = Vec::new();
        let mut asymmetrical_pages = 0usize;

        for page in &doc.pages {
            if excluded_pages.contains(&page.page_number) {
                continue;
            }

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
                rights.push((page.width - x1).max(0.0));
            }

            let left_cluster = dominant_cluster(&lefts, 4.0, 10);
            let right_cluster = dominant_cluster(&rights, 4.0, 10);

            if let (Some(lc), Some(rc)) = (left_cluster, right_cluster) {
                let diff = lc - rc;
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
                            lc,
                            rc,
                            direction
                        )),
                    });
                }
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
    use crate::document::{Document, Page, TextSpan};

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

    fn build_page_with_mixed_spans(
        body_left: f32,
        body_right: f32,
        body_count: usize,
        centered_lefts: &[f32],
        centered_rights: &[f32],
    ) -> Document {
        let height = 792.0;
        let mut all_spans: Vec<(f32, f32, f32, f32)> = Vec::new();

        for i in 0..body_count {
            let top = 80.0 + i as f32 * 24.0;
            all_spans.push((top, top + 12.0, body_left, body_right));
        }

        for (i, &cl) in centered_lefts.iter().enumerate() {
            let top = 80.0 + (body_count + i) as f32 * 24.0;
            let cr = centered_rights.get(i).copied().unwrap_or(cl + 300.0);
            all_spans.push((top, top + 12.0, cl, cr));
        }

        Document {
            pages: vec![Page {
                page_number: 2,
                width: 612.0,
                height,
                spans: all_spans
                    .iter()
                    .map(|&b| make_span("some text here", b))
                    .collect(),
                images: vec![],
                paths: vec![],
            }],
        }
    }

    // --- dominant_cluster unit tests ---

    #[test]
    fn test_cluster_basic() {
        let mut values: Vec<f32> = vec![];
        for _ in 0..20 {
            values.push(90.0);
        }
        for _ in 0..10 {
            values.push(180.0);
        }
        let result = dominant_cluster(&values, 4.0, 5);
        assert_eq!(result, Some(90.0));
    }

    #[test]
    fn test_cluster_mixed_indent() {
        let mut values: Vec<f32> = vec![];
        for _ in 0..15 {
            values.push(90.0);
        }
        for _ in 0..5 {
            values.push(120.0);
        }
        for _ in 0..5 {
            values.push(200.0);
        }
        let result = dominant_cluster(&values, 4.0, 5);
        assert_eq!(result, Some(90.0));
    }

    #[test]
    fn test_cluster_all_centered() {
        let values: Vec<f32> = (0..15).map(|_| 180.0).collect();
        let result = dominant_cluster(&values, 4.0, 5);
        assert_eq!(result, Some(180.0));
    }

    #[test]
    fn test_cluster_too_small() {
        let values: Vec<f32> = vec![90.0, 91.0, 92.0, 93.0];
        let result = dominant_cluster(&values, 4.0, 5);
        assert_eq!(result, None);
    }

    #[test]
    fn test_cluster_exact_min() {
        let values: Vec<f32> = vec![90.0, 90.5, 91.0, 91.5, 92.0];
        let result = dominant_cluster(&values, 4.0, 5);
        assert!(result.is_some());
    }

    #[test]
    fn test_cluster_empty() {
        let result = dominant_cluster(&[], 4.0, 5);
        assert_eq!(result, None);
    }

    // --- MarginsChecker clustered tests ---

    #[test]
    fn test_margins_clustered_pass() {
        let doc = build_page_with_mixed_spans(94.0, 518.0, 27, &[180.0, 200.0, 220.0, 250.0, 270.0], &[432.0, 412.0, 392.0, 362.0, 342.0]);
        let r = MarginsChecker.check(&doc, &default_params());
        assert_eq!(r.status, Status::Pass, "{}", r.detail);
    }

    #[test]
    fn test_margins_clustered_fail_left() {
        let doc = build_page_with_mixed_spans(72.0, 518.0, 27, &[180.0, 200.0], &[432.0, 412.0]);
        let r = MarginsChecker.check(&doc, &default_params());
        assert_eq!(r.status, Status::Fail, "{}", r.detail);
    }

    #[test]
    fn test_margins_clustered_fail_right() {
        let doc = build_page_with_mixed_spans(94.0, 542.0, 27, &[180.0, 200.0], &[432.0, 412.0]);
        let r = MarginsChecker.check(&doc, &default_params());
        assert_eq!(r.status, Status::Fail, "{}", r.detail);
    }

    #[test]
    fn test_margins_page_exclusions() {
        let pages = vec![
            body_spans(5, 72.0, 518.0, 40.0, 24.0),
            body_spans(20, 94.0, 518.0, 80.0, 24.0),
            body_spans(27, 94.0, 518.0, 80.0, 24.0),
        ];
        let mut doc = multi_page_doc(pages);
        doc.pages[1].spans[5].text = "accepted by the faculty".to_string();

        let r = MarginsChecker.check(&doc, &default_params());
        assert_eq!(r.status, Status::Pass, "page 1 (title) and page 2 (acceptance) should be excluded; only page 3's body margin (94pt) matters. got: {}", r.detail);
    }

    #[test]
    fn test_margins_error_empty() {
        let doc = Document { pages: vec![] };
        let r = MarginsChecker.check(&doc, &default_params());
        assert_eq!(r.status, Status::Error);
    }

    // --- MarginSymmetryChecker clustered tests ---

    #[test]
    fn test_symmetry_clustered_pass() {
        let doc = build_page_with_mixed_spans(90.0, 522.0, 30, &[180.0, 200.0], &[432.0, 412.0]);
        let r = MarginSymmetryChecker.check(&doc, &default_params());
        assert_eq!(r.status, Status::Pass, "{}", r.detail);
    }

    #[test]
    fn test_symmetry_clustered_fail() {
        let doc = build_page_with_mixed_spans(90.0, 502.0, 30, &[180.0], &[432.0]);
        let r = MarginSymmetryChecker.check(&doc, &default_params());
        assert_eq!(r.status, Status::Fail, "{}", r.detail);
    }

    #[test]
    fn test_symmetry_page_exclusions() {
        let pages = vec![
            body_spans(20, 82.0, 502.0, 80.0, 24.0),
            body_spans(20, 90.0, 522.0, 80.0, 24.0),
            body_spans(20, 90.0, 522.0, 80.0, 24.0),
        ];
        let mut doc = multi_page_doc(pages);
        doc.pages[1].spans[5].text = "accepted by the faculty".to_string();

        let r = MarginSymmetryChecker.check(&doc, &default_params());
        assert_eq!(r.status, Status::Pass,
            "page 1 (title) and page 2 (acceptance) should be excluded; page 3 is symmetric. got: {}",
            r.detail);
    }

    #[test]
    fn test_cluster_body_minority() {
        let mut values: Vec<f32> = vec![];
        for _ in 0..20 {
            values.push(180.0);
        }
        for _ in 0..5 {
            values.push(90.0);
        }
        let result = dominant_cluster(&values, 4.0, 5);
        assert_eq!(result, Some(180.0),
            "centered cluster (20) outnumbers body (5); returns centered position");
    }
}
