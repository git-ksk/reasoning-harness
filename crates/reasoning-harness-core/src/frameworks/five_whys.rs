use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhyLink {
    pub effect: String,
    pub cause: String,
    #[serde(default)]
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FiveWhysTrace {
    pub symptom: String,
    #[serde(default)]
    pub links: Vec<WhyLink>,
    pub root_cause: String,
}

pub fn validate_trace(trace: &FiveWhysTrace) -> Vec<String> {
    let mut diagnostics = Vec::new();
    for (index, link) in trace.links.iter().enumerate() {
        if normalize(&link.effect) == normalize(&link.cause) {
            diagnostics.push(format!(
                "why link {index} restates the effect instead of identifying a distinct cause"
            ));
        }
    }
    diagnostics
}

fn normalize(value: &str) -> String {
    value.split_whitespace().collect::<String>().to_lowercase()
}
