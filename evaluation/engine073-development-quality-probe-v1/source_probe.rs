// Development-only deterministic Core source-relevance probe.
// Never interpret these outputs as actual final-answer model quality.
// The source and case file are injected into an isolated pinned source copy.
use reasoning_harness_core::{
    derive_effective_evidence_local_qualification_v17, materialize_evidence_relevance_v30,
    EvidenceLocalQualificationV6, EvidenceLocalRelationScope as Relation,
    EvidenceRelevanceBindingProposal, EvidenceRelevanceCandidate,
    EvidenceRelevanceDisposition as Disposition, EvidenceRelevanceTargetPolicy,
};
use serde_json::{json, Value};

fn main() {
    let cases: Vec<Value> = serde_json::from_str(include_str!("source_probe_cases.json")).unwrap();
    for case in cases {
        let entity = case["entity"].as_str().unwrap();
        let policy: EvidenceRelevanceTargetPolicy = serde_json::from_value(json!({
            "policy_id":"engine073-self-authored-dev-probe-20261114",
            "target_id":"target",
            "target_question":format!("When was {entity} launched?"),
            "entity":{"canonical_id":"quartz-lantern.launch","canonical_name":entity,"aliases":[]},
            "relation":"change_or_launch",
            "identity_requirement":"require_harness_anchor",
            "assessment_budget":{"max_model_attempts":2,"max_tokens":192,"max_elapsed_ms":90000}
        }))
        .unwrap();
        let candidate: EvidenceRelevanceCandidate = serde_json::from_value(json!({
            "evidence_id":"e","source_id":"s","signals":case["signals"]
        }))
        .unwrap();
        let proposal: EvidenceRelevanceBindingProposal =
            serde_json::from_value(json!({"target_binding":"exact","relation_binding":"exact"}))
                .unwrap();
        let raw: EvidenceLocalQualificationV6 =
            serde_json::from_value(json!({"identity_scope":"exact_target","relation_scope":"requested_relation","scope_risk":"none"})).unwrap();
        let v17 = derive_effective_evidence_local_qualification_v17(
            &policy,
            &candidate,
            Some(&proposal),
            Some(&raw),
        )
        .unwrap();
        let v30 =
            materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw))
                .unwrap();
        println!(
            "{}",
            json!({
                "id":case["id"],
                "v17_requested_relation":v17.relation_scope==Relation::RequestedRelation,
                "v30_relevant":v30.disposition==Disposition::Relevant
            })
        );
    }
}
