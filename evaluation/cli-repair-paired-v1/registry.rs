//! Construct the pre-execution registry from host-owned Evidence snapshots only.
//! No question labels, gold, model outputs or previous answers are inputs.
use std::io::{self, Read};
use reasoning_harness_core::Evidence;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let evidence: Vec<Evidence> = serde_json::from_str(&input).unwrap();
    let mut sources = Vec::new();
    for item in evidence {
        let observation: Value = serde_json::from_str(&item.observation).unwrap();
        let url = observation["url"].as_str().expect("source URL");
        let version = observation["source_version"].as_str().expect("source version");
        observation["quote"].as_str().expect("complete source quotation");
        let hash = Sha256::digest(serde_json::to_vec(&item).unwrap())
            .iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        for (key, value) in &item.facts {
            sources.push(json!({
                "proposition": {"key": key, "value": value},
                "evidence_id": item.id, "source_id": item.source,
                "evidence_sha256": hash, "quote_pointer": "/quote",
                "url_pointer": "/url", "version_pointer": "/source_version",
                "url": url, "source_version": version
            }));
        }
    }
    println!("{}", json!({"schema_version":"reason-answer-context-config-v1",
        "sources":sources,"unresolved_inputs":[]}));
}
