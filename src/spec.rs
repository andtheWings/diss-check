use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct SectionDef {
    pub id: String,
    pub required: bool,
}

#[derive(Debug, Deserialize)]
pub struct DocumentStructure {
    pub front_matter: Vec<SectionDef>,
    pub body: Vec<SectionDef>,
    pub end_matter: Vec<SectionDef>,
}

#[derive(Debug, Deserialize)]
pub struct CheckTarget {
    pub scope: Option<String>,
    pub page: Option<String>,
    pub element: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CheckDef {
    pub id: String,
    pub category: String,
    pub checker: String,
    pub target: CheckTarget,
    #[serde(default)]
    pub params: HashMap<String, serde_yaml::Value>,
    #[serde(default = "default_true")]
    pub automatable: bool,
    pub review_hint: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct InstitutionSpec {
    pub institution: String,
    pub source_revision: String,
    pub document_structure: DocumentStructure,
    pub checks: Vec<CheckDef>,
    #[serde(default)]
    pub constants: HashMap<String, String>,
}

pub fn load_spec(path: &Path) -> Result<InstitutionSpec, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let spec: InstitutionSpec = serde_yaml::from_str(&content)?;
    Ok(spec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_load_iu_spec() {
        let path = PathBuf::from("specs/iu.yaml");
        let spec = load_spec(&path).expect("Should load iu.yaml");
        assert_eq!(spec.institution, "Indiana University");
        assert_eq!(spec.source_revision, "September 2025");
        assert!(!spec.checks.is_empty());
        assert_eq!(spec.checks[0].id, "global_margins");
    }

    #[test]
    fn test_spec_validates_invalid_yaml() {
        let result = serde_yaml::from_str::<InstitutionSpec>("not: valid: yaml: [");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_spec_fails() {
        let result = serde_yaml::from_str::<InstitutionSpec>("");
        assert!(result.is_err());
    }
}
