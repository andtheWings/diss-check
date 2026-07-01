use crate::checkers::{Checker, CheckResult, EvidenceItem, Status};
use crate::document::Document;
use serde_yaml::Value;

pub struct TablesFiguresMarginsChecker;

impl Checker for TablesFiguresMarginsChecker {
    fn category(&self) -> &'static str { "layout" }
    fn name(&self) -> &'static str { "tables_figures_within_margins" }

    fn check(&self, doc: &Document, _params: &Value) -> CheckResult {
        let left_margin = 90.0;
        let right_margin = 90.0;
        let top_margin = 72.0;
        let bottom_margin = 72.0;
        let tolerance = 4.0;

        let mut violations: Vec<EvidenceItem> = Vec::new();

        for page in &doc.pages {
            for &(top, bottom, x0, x1) in &page.images {
                if x0 < left_margin - tolerance {
                    violations.push(EvidenceItem {
                        page: page.page_number, bbox: Some((top, bottom, x0, x1)),
                        excerpt: Some(format!("Image extends past left margin: x0={:.0}pt (limit={:.0}pt)", x0, left_margin)),
                    });
                }
                if (page.width - x1) < right_margin - tolerance {
                    violations.push(EvidenceItem {
                        page: page.page_number, bbox: Some((top, bottom, x0, x1)),
                        excerpt: Some(format!("Image extends past right margin: right edge={:.0}pt (limit={:.0}pt)", page.width - x1, right_margin)),
                    });
                }
                if top < top_margin - tolerance {
                    violations.push(EvidenceItem {
                        page: page.page_number, bbox: Some((top, bottom, x0, x1)),
                        excerpt: Some(format!("Image extends past top margin: top={:.0}pt (limit={:.0}pt)", top, top_margin)),
                    });
                }
                if bottom > page.height - bottom_margin + tolerance {
                    violations.push(EvidenceItem {
                        page: page.page_number, bbox: Some((top, bottom, x0, x1)),
                        excerpt: Some(format!("Image extends past bottom margin: bottom={:.0}pt (limit={:.0}pt)", bottom, page.height - bottom_margin)),
                    });
                }
            }
        }

        if violations.is_empty() {
            CheckResult {
                check_id: String::new(), status: Status::Pass, evidence: vec![],
                detail: "All images within margins".to_string(),
            }
        } else {
            CheckResult {
                check_id: String::new(), status: Status::Fail,
                detail: format!("{} image(s) extend beyond margins", violations.len()),
                evidence: violations,
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{Page, Document};

    fn span(text: &str, top: f32) -> crate::document::TextSpan {
        crate::document::TextSpan {
            text: text.to_string(), font_name: "Times".to_string(), font_size: 12.0,
            bbox: (top, top + 12.0, 100.0, 200.0), is_bold: false, is_italic: false, color: None,
        }
    }

    #[test]
    fn test_no_images_pass() {
        let doc = Document { pages: vec![Page {
            page_number: 1, width: 612.0, height: 792.0,
            spans: vec![span("text", 100.0)], images: vec![], paths: vec![],
        }] };
        let r = TablesFiguresMarginsChecker.check(&doc, &Value::Null);
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn test_image_within_margins_pass() {
        let doc = Document { pages: vec![Page {
            page_number: 1, width: 612.0, height: 792.0,
            spans: vec![],
            images: vec![(100.0, 500.0, 100.0, 512.0)],
            paths: vec![],
        }] };
        let r = TablesFiguresMarginsChecker.check(&doc, &Value::Null);
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn test_image_beyond_left_margin_fail() {
        let doc = Document { pages: vec![Page {
            page_number: 1, width: 612.0, height: 792.0,
            spans: vec![],
            images: vec![(100.0, 500.0, 70.0, 512.0)],
            paths: vec![],
        }] };
        let r = TablesFiguresMarginsChecker.check(&doc, &Value::Null);
        assert_eq!(r.status, Status::Fail);
    }
}
