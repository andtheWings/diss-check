use std::collections::{BTreeMap, HashMap};
use crate::checkers::{Checker, CheckResult, EvidenceItem, Status};
use crate::document::Document;
use serde_yaml::Value;

const SECTION_KEYWORDS: &[(&str, &str)] = &[
    ("title_page", "title page|title_page"),
    ("acceptance_page", "accepted by|acceptance"),
    ("abstract", "abstract"),
    ("toc", "table of contents|contents"),
    ("chapters", "chapter"),
    ("references", "references|bibliography|works cited"),
    ("curriculum_vitae", "curriculum vitae"),
];

const HEADING_SECTIONS: &[&str] = &["toc", "acceptance_page", "curriculum_vitae", "references", "chapters"];
const NON_ABSTRACT_HEADINGS: &[&str] = &["dedication", "acknowledgement", "acknowledgments", "preface"];

fn page_text(page: &crate::document::Page) -> String {
    page.spans.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join(" ").to_lowercase()
}

fn page_text_no_citations(page: &crate::document::Page) -> String {
    let mut lines: BTreeMap<i32, Vec<&str>> = BTreeMap::new();
    for s in &page.spans {
        let top_key = s.bbox.0.round() as i32;
        lines.entry(top_key).or_default().push(&s.text);
    }

    let mut text_parts: Vec<String> = Vec::new();
    for (_top, words) in &lines {
        let line = words.join(" ");
        let low = line.to_lowercase();
        if low.contains("doi:") || low.contains("http") || low.contains("https") {
            continue;
        }
        let stripped = low.trim();
        if !stripped.is_empty() && stripped.chars().next().map_or(false, |c| c.is_ascii_digit()) && stripped.len() <= 5 {
            continue;
        }
        text_parts.push(low);
    }
    text_parts.join(" ")
}

fn contains_keyword(text: &str, section_id: &str) -> bool {
    for (sid, patterns) in SECTION_KEYWORDS {
        if *sid == section_id {
            for pattern in patterns.split('|') {
                if text.contains(pattern) {
                    return true;
                }
            }
            return false;
        }
    }
    text.contains(section_id)
}

fn find_all_sections(doc: &Document) -> HashMap<String, usize> {
    let mut sections: HashMap<String, usize> = HashMap::new();

    if let Some(page1) = doc.pages.first() {
        let has_page_num = page1.spans.iter().any(|s| {
            s.bbox.1 >= (page1.height - 53.0) && !s.text.trim().is_empty()
        });
        if !has_page_num && !page_text(page1).trim().is_empty() {
            sections.insert("title_page".to_string(), 1);
        }
    }

    for page in &doc.pages {
        let text = page_text_no_citations(page);
        for &sec_id in HEADING_SECTIONS {
            if !sections.contains_key(sec_id) && contains_keyword(&text, sec_id) {
                sections.insert(sec_id.to_string(), page.page_number);
            }
        }
    }

    if !sections.contains_key("abstract")
        && sections.contains_key("acceptance_page")
        && sections.contains_key("toc")
    {
        let acc_pg = sections["acceptance_page"];
        let toc_pg = sections["toc"];
        for page in doc.pages.iter().rev() {
            if page.page_number > acc_pg && page.page_number < toc_pg {
                let n_spans = page.spans.iter().filter(|s| !s.text.trim().is_empty()).count();
                let text = page_text(page);
                let is_other = NON_ABSTRACT_HEADINGS.iter().any(|h| text[..200.min(text.len())].contains(h));
                if n_spans > 100 && !is_other {
                    sections.insert("abstract".to_string(), page.page_number);
                    break;
                }
            }
        }
    }

    sections
}

pub struct SectionPresenceChecker;

impl Checker for SectionPresenceChecker {
    fn category(&self) -> &'static str {
        "structure"
    }

    fn name(&self) -> &'static str {
        "section_presence"
    }

    fn check(&self, doc: &Document, params: &Value) -> CheckResult {
        let required: Vec<String> = params
            .get("required_sections")
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|item| {
                        item.get("id").and_then(|v| v.as_str()).map(|s| s.to_string())
                    })
                    .collect()
            })
            .unwrap_or_default();

        let sections = find_all_sections(doc);

        let mut found: Vec<String> = Vec::new();
        let mut missing: Vec<String> = Vec::new();

        for sec_id in &required {
            if sections.contains_key(sec_id.as_str()) {
                found.push(sec_id.clone());
            } else {
                missing.push(sec_id.clone());
            }
        }

        if !missing.is_empty() {
            CheckResult {
                check_id: String::new(),
                status: Status::Fail,
                detail: format!("Missing section(s): {}", missing.join(", ")),
                evidence: missing
                    .iter()
                    .map(|m| EvidenceItem {
                        page: 0,
                        bbox: None,
                        excerpt: Some(format!("Section '{}' not detected", m)),
                    })
                    .collect(),
            }
        } else {
            found.sort();
            CheckResult {
                check_id: String::new(),
                status: Status::Pass,
                evidence: vec![],
                detail: format!("All required sections detected: {}", found.join(", ")),
            }
        }
    }
}

pub struct SectionOrderChecker;

impl Checker for SectionOrderChecker {
    fn category(&self) -> &'static str {
        "structure"
    }

    fn name(&self) -> &'static str {
        "section_order"
    }

    fn check(&self, doc: &Document, params: &Value) -> CheckResult {
        let expected: Vec<String> = params
            .get("expected_order")
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|item| {
                        item.get("id").and_then(|v| v.as_str()).map(|s| s.to_string())
                    })
                    .collect()
            })
            .unwrap_or_default();

        let sections = find_all_sections(doc);

        let mut found_pages: Vec<(String, usize)> = Vec::new();
        for sec_id in &expected {
            if let Some(&pg) = sections.get(sec_id.as_str()) {
                found_pages.push((sec_id.clone(), pg));
            }
        }

        let mut violations: Vec<EvidenceItem> = Vec::new();
        for i in 1..found_pages.len() {
            let (prev_id, prev_pg) = &found_pages[i - 1];
            let (curr_id, curr_pg) = &found_pages[i];
            if *curr_pg < *prev_pg {
                violations.push(EvidenceItem {
                    page: *curr_pg,
                    bbox: None,
                    excerpt: Some(format!(
                        "'{}' (p{}) appears before '{}' (p{})",
                        curr_id, curr_pg, prev_id, prev_pg,
                    )),
                });
            } else if curr_pg == prev_pg && prev_id != curr_id {
                violations.push(EvidenceItem {
                    page: *curr_pg,
                    bbox: None,
                    excerpt: Some(format!(
                        "'{}' and '{}' detected on same page {}",
                        curr_id, prev_id, curr_pg,
                    )),
                });
            }
        }

        if !violations.is_empty() {
            CheckResult {
                check_id: String::new(),
                status: Status::Fail,
                detail: format!("{} ordering violation(s) found", violations.len()),
                evidence: violations,
            }
        } else {
            let names: Vec<String> = found_pages.iter().map(|(n, p)| format!("{} (p{})", n, p)).collect();
            CheckResult {
                check_id: String::new(),
                status: Status::Pass,
                evidence: vec![],
                detail: format!("Sections in correct order: {}", names.join(", ")),
            }
        }
    }
}

pub struct TitlePageNoPageNumberChecker;

impl Checker for TitlePageNoPageNumberChecker {
    fn category(&self) -> &'static str { "structure" }
    fn name(&self) -> &'static str { "title_page_no_page_number" }

    fn check(&self, doc: &Document, _params: &Value) -> CheckResult {
        if let Some(page1) = doc.pages.first() {
            let pn_spans: Vec<_> = page1.spans.iter()
                .filter(|s| s.bbox.1 >= (page1.height - 53.0) && !s.text.trim().is_empty())
                .collect();
            if !pn_spans.is_empty() {
                return CheckResult {
                    check_id: String::new(), status: Status::Fail,
                    detail: format!("{} page number(s) found on title page", pn_spans.len()),
                    evidence: pn_spans.iter().map(|s| EvidenceItem {
                        page: 1, bbox: Some(s.bbox), excerpt: Some(s.text.clone()),
                    }).collect(),
                };
            }
        }
        CheckResult {
            check_id: String::new(), status: Status::Pass, evidence: vec![],
            detail: "No page number on title page".to_string(),
        }
    }
}

pub struct AcceptancePagePageNumberChecker;

impl Checker for AcceptancePagePageNumberChecker {
    fn category(&self) -> &'static str { "structure" }
    fn name(&self) -> &'static str { "acceptance_page_number" }

    fn check(&self, doc: &Document, _params: &Value) -> CheckResult {
        let sections = find_all_sections(doc);
        let acc_pg = match sections.get("acceptance_page") {
            Some(&pg) => pg,
            None => return CheckResult {
                check_id: String::new(), status: Status::Error, evidence: vec![],
                detail: "Acceptance page not found".to_string(),
            },
        };
        let page = &doc.pages[acc_pg - 1];
        let pn_spans: Vec<_> = page.spans.iter()
            .filter(|s| s.bbox.1 >= (page.height - 53.0) && !s.text.trim().is_empty())
            .collect();
        if pn_spans.is_empty() {
            return CheckResult {
                check_id: String::new(), status: Status::Fail, evidence: vec![],
                detail: "No page number found on acceptance page (expected 'ii')".to_string(),
            };
        }
        let pn_text: String = pn_spans.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join("");
        let is_roman = pn_text.chars().all(|c| "ivxlcdmIVXLCDM".contains(c));
        if !is_roman {
            return CheckResult {
                check_id: String::new(), status: Status::Fail,
                detail: format!("Page number '{}' is not a Roman numeral", pn_text.trim()),
                evidence: pn_spans.iter().map(|s| EvidenceItem {
                    page: acc_pg, bbox: Some(s.bbox), excerpt: Some(s.text.clone()),
                }).collect(),
            };
        }
        CheckResult {
            check_id: String::new(), status: Status::Pass, evidence: vec![],
            detail: format!("Acceptance page numbered '{}' (Roman numeral)", pn_text.trim()),
        }
    }
}

pub struct PageNumbersFormatChecker;

impl Checker for PageNumbersFormatChecker {
    fn category(&self) -> &'static str { "structure" }
    fn name(&self) -> &'static str { "page_numbers_format" }

    fn check(&self, doc: &Document, _params: &Value) -> CheckResult {
        let mut violations: Vec<EvidenceItem> = Vec::new();
        let sections = find_all_sections(doc);
        let body_start = sections.get("toc").copied().map(|p| p + 1).unwrap_or(6);
        for page in &doc.pages {
            let pn_spans: Vec<_> = page.spans.iter()
                .filter(|s| s.bbox.1 >= (page.height - 53.0) && !s.text.trim().is_empty())
                .collect();
            if pn_spans.is_empty() {
                continue;
            }
            let pn_text: String = pn_spans.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join("");
            let pn_trim = pn_text.trim();
            if pn_trim.is_empty() { continue; }
            let is_roman = pn_trim.chars().all(|c| "ivxlcdmIVXLCDM".contains(c));
            let is_arabic = pn_trim.chars().all(|c| c.is_ascii_digit());
            if !is_roman && !is_arabic { continue; }
            if page.page_number < body_start && is_arabic {
                violations.push(EvidenceItem {
                    page: page.page_number, bbox: None,
                    excerpt: Some(format!("Page number '{}' should be Roman numeral in front matter", pn_trim)),
                });
            }
            if page.page_number >= body_start && is_roman {
                violations.push(EvidenceItem {
                    page: page.page_number, bbox: None,
                    excerpt: Some(format!("Page number '{}' should be Arabic numeral in body", pn_trim)),
                });
            }
        }
        if violations.is_empty() {
            CheckResult { check_id: String::new(), status: Status::Pass, evidence: vec![], detail: "Page numbers correctly formatted".to_string() }
        } else {
            CheckResult { check_id: String::new(), status: Status::Fail, detail: format!("{} page number format violation(s)", violations.len()), evidence: violations }
        }
    }
}

pub struct HeadingsConsistentChecker;

impl Checker for HeadingsConsistentChecker {
    fn category(&self) -> &'static str { "structure" }
    fn name(&self) -> &'static str { "headings_consistent" }

    fn check(&self, doc: &Document, _params: &Value) -> CheckResult {
        let mut body_families: HashMap<String, usize> = HashMap::new();
        let mut body_sizes: HashMap<i32, usize> = HashMap::new();
        let sections = find_all_sections(doc);
        let body_start = sections.get("toc").copied().map(|p| p + 1).unwrap_or(6);
        for page in &doc.pages {
            if page.page_number < body_start { continue; }
            for span in &page.spans {
                let (top, bottom, _x0, _x1) = span.bbox;
                if bottom >= (page.height - 53.0) || top < 36.0 { continue; }
                if span.text.trim().len() < 3 { continue; }
                *body_families.entry(span.font_name.clone()).or_insert(0) += 1;
                let key = (span.font_size * 10.0).round() as i32;
                *body_sizes.entry(key).or_insert(0) += 1;
            }
        }
        let body_family = body_families.iter().max_by_key(|(_, c)| *c).map(|(k, _)| k.clone()).unwrap_or_default();
        let body_size_key = body_sizes.iter().max_by_key(|(_, c)| *c).map(|(k, _)| *k).unwrap_or(0);
        let body_size = body_size_key as f32 / 10.0;
        let mut violations: Vec<EvidenceItem> = Vec::new();
        for page in &doc.pages {
            for span in &page.spans {
                let (top, bottom, _x0, _x1) = span.bbox;
                if top >= 36.0 && top < 120.0 && bottom <= page.height - 53.0 && span.text.trim().len() > 3 {
                    let is_internal = span.font_name.len() < 4; let is_heading = !is_internal && (span.is_bold || span.font_size > body_size + 1.0);
                    if is_heading && (span.font_name != body_family || (span.font_size - body_size).abs() > 2.0) {
                        violations.push(EvidenceItem {
                            page: page.page_number, bbox: Some(span.bbox),
                            excerpt: Some(format!("{} ({}, {:.0}pt, expected {} {:.0}pt)", span.text, span.font_name, span.font_size, body_family, body_size)),
                        });
                    }
                }
            }
        }
        if violations.is_empty() {
            CheckResult { check_id: String::new(), status: Status::Pass, evidence: vec![], detail: format!("Headings use same font ({}, {:.0}pt) as body", body_family, body_size) }
        } else {
            CheckResult { check_id: String::new(), status: Status::Fail, detail: format!("{} heading(s) differ from body font", violations.len()), evidence: violations }
        }
    }
}

pub struct NewChaptersNewPagesChecker;

impl Checker for NewChaptersNewPagesChecker {
    fn category(&self) -> &'static str { "structure" }
    fn name(&self) -> &'static str { "new_chapters_new_pages" }

    fn check(&self, doc: &Document, _params: &Value) -> CheckResult {
        let mut violations: Vec<EvidenceItem> = Vec::new();
        for page in &doc.pages {
            for span in &page.spans {
                let (top, _bottom, _x0, _x1) = span.bbox;
                if top < 36.0 || top > 200.0 { continue; }
                let low = span.text.to_lowercase();
                if (low.contains("chapter ") || low.contains("appendix ")) && top > 100.0 {
                    violations.push(EvidenceItem {
                        page: page.page_number, bbox: Some(span.bbox),
                        excerpt: Some(format!("Chapter heading at {:.0}pt from top (not on new page)", top)),
                    });
                }
            }
        }
        if violations.is_empty() {
            CheckResult { check_id: String::new(), status: Status::Pass, evidence: vec![], detail: "All chapters start on new pages".to_string() }
        } else {
            CheckResult { check_id: String::new(), status: Status::Fail, detail: format!("{} chapter(s) not starting on new page", violations.len()), evidence: violations }
        }
    }
}

pub struct HyperlinksFormatChecker;

impl Checker for HyperlinksFormatChecker {
    fn category(&self) -> &'static str { "structure" }
    fn name(&self) -> &'static str { "hyperlinks_format" }

    fn check(&self, doc: &Document, _params: &Value) -> CheckResult {
        let mut body_families: HashMap<String, usize> = HashMap::new();
        let mut body_sizes: HashMap<i32, usize> = HashMap::new();
        for page in &doc.pages {
            for span in &page.spans {
                let (top, bottom, _x0, _x1) = span.bbox;
                if bottom >= (page.height - 53.0) || top < 36.0 { continue; }
                if span.text.trim().len() < 3 { continue; }
                *body_families.entry(span.font_name.clone()).or_insert(0) += 1;
                let key = (span.font_size * 10.0).round() as i32;
                *body_sizes.entry(key).or_insert(0) += 1;
            }
        }
        let body_family = body_families.iter().max_by_key(|(_, c)| *c).map(|(k, _)| k.clone()).unwrap_or_default();
        let body_size_key = body_sizes.iter().max_by_key(|(_, c)| *c).map(|(k, _)| *k).unwrap_or(0);
        let body_size = body_size_key as f32 / 10.0;
        let mut violations: Vec<EvidenceItem> = Vec::new();
        for page in &doc.pages {
            for span in &page.spans {
                let low = span.text.to_lowercase();
                let is_link = low.contains("http") || low.contains("https") || low.contains("doi:")
                    || low.contains("www.") || low.contains(".com") || low.contains(".org")
                    || low.contains(".edu") || low.contains(".gov");
                if !is_link || span.text.trim().len() < 5 { continue; }
                let font_mismatch = span.font_name != body_family || (span.font_size - body_size).abs() > 2.0;
                if font_mismatch {
                    violations.push(EvidenceItem {
                        page: page.page_number, bbox: Some(span.bbox),
                        excerpt: Some(format!("URL with mismatched font: {} ({}, {:.0}pt, expected {} {:.0}pt)", span.text, span.font_name, span.font_size, body_family, body_size)),
                    });
                }
            }
        }
        if violations.is_empty() {
            CheckResult { check_id: String::new(), status: Status::Pass, evidence: vec![], detail: "Hyperlinks use document body font".to_string() }
        } else {
            CheckResult { check_id: String::new(), status: Status::Fail, detail: format!("{} hyperlink(s) with non-standard formatting", violations.len()), evidence: violations }
        }
    }
}

pub struct CvNoPageNumberChecker;

impl Checker for CvNoPageNumberChecker {
    fn category(&self) -> &'static str { "structure" }
    fn name(&self) -> &'static str { "cv_no_page_number" }

    fn check(&self, doc: &Document, _params: &Value) -> CheckResult {
        let mut cv_pg: Option<usize> = None;
        for page in doc.pages.iter().rev() {
            let text: String = page.spans.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join(" ");
            let low = text.to_lowercase();
            if low.contains("curriculum vitae") || (low.contains("curriculum") && low.contains("vitae")) {
                cv_pg = Some(page.page_number);
                break;
            }
        }
        let cv_pg = match cv_pg {
            Some(pg) => pg,
            None => return CheckResult {
                check_id: String::new(), status: Status::Error, evidence: vec![],
                detail: "Curriculum Vitae page not found".to_string(),
            },
        };
        let page = &doc.pages[cv_pg - 1];
        let pn_spans: Vec<_> = page.spans.iter()
            .filter(|s| s.bbox.1 >= (page.height - 53.0) && !s.text.trim().is_empty())
            .collect();
        if !pn_spans.is_empty() {
            return CheckResult {
                check_id: String::new(), status: Status::Fail,
                detail: "Page number found on Curriculum Vitae".to_string(),
                evidence: pn_spans.iter().map(|s| EvidenceItem {
                    page: cv_pg, bbox: Some(s.bbox), excerpt: Some(s.text.clone()),
                }).collect(),
            };
        }
        CheckResult {
            check_id: String::new(), status: Status::Pass, evidence: vec![],
            detail: "No page number on Curriculum Vitae".to_string(),
        }
    }
}
