use std::collections::HashMap;
use std::sync::LazyLock;
use crate::document::Document;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Pass,
    Fail,
    Manual,
    Error,
}

impl Status {
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Pass => "PASS",
            Status::Fail => "FAIL",
            Status::Manual => "MANUAL",
            Status::Error => "ERROR",
        }
    }
}

#[derive(Debug, Clone)]
pub struct EvidenceItem {
    pub page: usize,
    pub bbox: Option<(f32, f32, f32, f32)>,
    pub excerpt: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub check_id: String,
    pub status: Status,
    pub evidence: Vec<EvidenceItem>,
    pub detail: String,
}

pub trait Checker: Send + Sync {
    fn category(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn check(&self, doc: &Document, params: &serde_yaml::Value) -> CheckResult;
}

type CheckerFactory = fn() -> Box<dyn Checker>;

fn make_margins() -> Box<dyn Checker> {
    Box::new(crate::checkers::layout::MarginsChecker)
}

fn make_margin_symmetry() -> Box<dyn Checker> {
    Box::new(crate::checkers::layout::MarginSymmetryChecker)
}

fn make_font_size() -> Box<dyn Checker> {
    Box::new(crate::checkers::typography::FontSizeChecker)
}

fn make_font_weight() -> Box<dyn Checker> {
    Box::new(crate::checkers::typography::FontWeightChecker)
}

fn make_font_family() -> Box<dyn Checker> {
    Box::new(crate::checkers::typography::FontFamilyChecker)
}

fn make_justification() -> Box<dyn Checker> {
    Box::new(crate::checkers::typography::JustificationChecker)
}

fn make_section_presence() -> Box<dyn Checker> {
    Box::new(crate::checkers::structure::SectionPresenceChecker)
}

fn make_section_order() -> Box<dyn Checker> {
    Box::new(crate::checkers::structure::SectionOrderChecker)
}

static REGISTRY: LazyLock<HashMap<(String, String), CheckerFactory>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(
        ("layout".to_string(), "margins".to_string()),
        make_margins as CheckerFactory,
    );
    m.insert(
        ("layout".to_string(), "margin_symmetry".to_string()),
        make_margin_symmetry as CheckerFactory,
    );
    m.insert(
        ("typography".to_string(), "font_size".to_string()),
        make_font_size as CheckerFactory,
    );
    m.insert(
        ("typography".to_string(), "font_weight".to_string()),
        make_font_weight as CheckerFactory,
    );
    m.insert(
        ("typography".to_string(), "font_family".to_string()),
        make_font_family as CheckerFactory,
    );
    m.insert(
        ("typography".to_string(), "justification".to_string()),
        make_justification as CheckerFactory,
    );
    m.insert(
        ("structure".to_string(), "section_presence".to_string()),
        make_section_presence as CheckerFactory,
    );
    m.insert(
        ("structure".to_string(), "section_order".to_string()),
        make_section_order as CheckerFactory,
    );
    m
});

pub fn get_checker(category: &str, name: &str) -> Option<Box<dyn Checker>> {
    REGISTRY.get(&(category.to_string(), name.to_string())).map(|f| f())
}

pub mod layout;
pub mod typography;
pub mod structure;
