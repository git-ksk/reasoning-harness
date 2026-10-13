use reasoning_harness_core::{EvidenceRelevanceTargetPolicy, EvidenceRelevanceCandidate, EvidenceRelevanceBindingProposal, EvidenceLocalQualificationV6, derive_effective_evidence_local_qualification_v17, materialize_evidence_relevance_v29, materialize_evidence_relevance_v30};
use serde_json::{json, Value};
fn main() {
    let input = std::env::args().nth(1).expect("case file");
    let cases: Vec<Value> = serde_json::from_str(&std::fs::read_to_string(input).unwrap()).unwrap();
    for case in cases {
        let lambda = case["policy"] == "lambda";
        let entity = case["entity"].as_str().unwrap_or("Silver Lens");
        let policy: EvidenceRelevanceTargetPolicy = serde_json::from_value(if lambda { json!({
            "policy_id":"audit-lambda", "target_id":"lambda-preparation", "target_question":"Provisioned Concurrencyは何を準備するか？",
            "entity":{"canonical_id":"aws.lambda.provisioned-concurrency", "canonical_name":"AWS Lambda Provisioned Concurrency", "aliases":["Provisioned Concurrency"]},
            "relation":"general", "identity_requirement":"require_harness_anchor", "assessment_budget":{"max_model_attempts":2,"max_tokens":192,"max_elapsed_ms":90000}
        }) } else { json!({
            "policy_id":"audit-launch", "target_id":"launch", "target_question":format!("When was {entity} launched?"),
            "entity":{"canonical_id":"product.launch", "canonical_name":entity, "aliases":[]},
            "relation":"change_or_launch", "identity_requirement":"require_harness_anchor", "assessment_budget":{"max_model_attempts":2,"max_tokens":192,"max_elapsed_ms":90000}
        }) }).unwrap();
        let candidate: EvidenceRelevanceCandidate = serde_json::from_value(json!({"evidence_id":"e","source_id":"s","signals":case["signals"]})).unwrap();
        let proposal: EvidenceRelevanceBindingProposal = serde_json::from_value(case.get("proposal").cloned().unwrap_or(json!({"target_binding":"exact","relation_binding":"exact"}))).unwrap();
        let raw: EvidenceLocalQualificationV6 = serde_json::from_value(case.get("raw").cloned().unwrap_or(json!({"identity_scope":"exact_target","relation_scope":"requested_relation","scope_risk":"none"}))).unwrap();
        let old=serde_json::to_value(materialize_evidence_relevance_v29(&policy,&candidate,Some(&proposal),Some(&raw)).unwrap()).unwrap();
        let new=serde_json::to_value(materialize_evidence_relevance_v30(&policy,&candidate,Some(&proposal),Some(&raw)).unwrap()).unwrap();
        let q=serde_json::to_value(derive_effective_evidence_local_qualification_v17(&policy,&candidate,Some(&proposal),Some(&raw)).unwrap()).unwrap();
        let relevant=new["disposition"]=="relevant";
        println!("{}",json!({"engine_version":reasoning_harness_core::ENGINE_VERSION,"id":case["id"],"expected_relevant":case["expected_relevant"],"v29":old["disposition"],"v30":new["disposition"],"v17":q,"mismatch":relevant!=case["expected_relevant"].as_bool().unwrap(),"input":case}));
    }
}
