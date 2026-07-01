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
