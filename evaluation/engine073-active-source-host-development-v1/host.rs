use reasoning_harness_core::{
    Evidence, EvidenceLocalQualificationV6, EvidenceMetadata, EvidenceRelevanceBindingProposal,
    EvidenceRelevanceCandidate, EvidenceRelevanceDisposition, EvidenceRelevanceSignal,
    EvidenceRelevanceSignalKind, EvidenceRelevanceTargetPolicy, ReasoningArtifact,
    SourceAttributionAuthorityCeiling, SourceAttributionBinding, SourceAttributionLocator,
    SourceAttributionProposal, SourceAttributionState, SourceAttributionTargetPolicy,
    SourceAttributionTransformKind, SourceTextSpan, append_source_attributed_claim,
    derive_effective_evidence_local_qualification_v17, finalize_source_attributed_answer,
    materialize_evidence_relevance_v30, materialize_source_attributed_claim,
};
use serde_json::{Value, json};

fn main() {
    let bytes = std::fs::read(std::env::args().nth(1).expect("input manifest")).unwrap();
    let input: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(input["contract"], "active-source-host-development-v1");
    let policy: EvidenceRelevanceTargetPolicy = serde_json::from_value(input["policy"].clone()).unwrap();
    let proposal: EvidenceRelevanceBindingProposal = serde_json::from_value(input["proposal"].clone()).unwrap();
    let raw: EvidenceLocalQualificationV6 = serde_json::from_value(input["controlled_raw"].clone()).unwrap();
    let source = &input["source"];
    let body = std::fs::read_to_string(source["normalized_path"].as_str().unwrap()).unwrap();
    let start = usize::try_from(source["start_byte"].as_u64().unwrap()).unwrap();
    let end = usize::try_from(source["end_byte"].as_u64().unwrap()).unwrap();
    let quote = body.get(start..end).expect("exact UTF-8 quote span");
    assert_eq!(Some(quote), source["quote"].as_str(), "quote differs from retained full source");
    let eid = source["evidence_id"].as_str().unwrap();
    let sid = source["source_id"].as_str().unwrap();
    let metadata: EvidenceMetadata = serde_json::from_value(source["metadata"].clone()).unwrap();
    let candidate = EvidenceRelevanceCandidate {
        evidence_id: eid.into(), source_id: sid.into(),
        signals: vec![EvidenceRelevanceSignal { kind: EvidenceRelevanceSignalKind::Excerpt, text: quote.into() }],
    };
    let effective = derive_effective_evidence_local_qualification_v17(&policy, &candidate, Some(&proposal), Some(&raw)).unwrap();
    let relevance = materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw)).unwrap();
    let mut artifact = ReasoningArtifact { task: policy.target_question.clone(), ..Default::default() };
    artifact.source_attribution = SourceAttributionState {
        targets: vec![SourceAttributionTargetPolicy {
            policy_id: format!("{}:source-view", policy.policy_id), target_id: policy.target_id.clone(),
            target_question: policy.target_question.clone(), authority_ceiling: SourceAttributionAuthorityCeiling::SourceLocal,
            hard_verification_required: false,
        }], ..Default::default()
    };
    if relevance.disposition == EvidenceRelevanceDisposition::Relevant {
        artifact.evidence.push(Evidence { id: eid.into(), source: sid.into(), observation: body,
                                         facts: Default::default(), metadata });
        artifact.source_attribution.bindings.push(SourceAttributionBinding {
            id: "active-source-binding".into(), target_id: policy.target_id.clone(), evidence_id: eid.into(), source_id: sid.into(),
            source_url: Some(source["url"].as_str().unwrap().into()),
            locator: Some(SourceAttributionLocator { section: Some(source["locator"].as_str().unwrap().into()), ..Default::default() }),
            retrieved_at_unix_seconds: Some(source["retrieved_at_unix_seconds"].as_i64().unwrap()),
            source_version: Some(source["source_version"].as_str().unwrap().into()),
            span: SourceTextSpan { start_byte: start, end_byte: end },
        });
        let (_, claim) = materialize_source_attributed_claim(&artifact, "active-source-claim", None, &SourceAttributionProposal {
            target_id: policy.target_id.clone(), binding_ids: vec!["active-source-binding".into()],
            transform_kind: SourceAttributionTransformKind::ExactQuote, transformed_statement: None,
            source_language: Some("en".into()), output_language: Some("en".into()),
        }, None).unwrap();
        append_source_attributed_claim(&mut artifact, None, claim).unwrap();
    }
    let finalization = finalize_source_attributed_answer(&artifact, &[policy.target_id.clone()]).unwrap();
    let display = finalization.text.clone();
    let events = json!([
        {"sequence":0,"kind":"captured_source","evidence_id":eid,"source_id":sid,"parent_evidence_id":source["parent_evidence_id"],"source_version":source["source_version"],"quote_span":[start,end],"quote":source["quote"]},
        {"sequence":1,"kind":"api_result","api":"derive_effective_evidence_local_qualification_v17","policy":policy,"candidate":candidate,"proposal":proposal,"controlled_raw":raw,"output":effective},
        {"sequence":2,"kind":"api_result","api":"materialize_evidence_relevance_v30","output":relevance},
        {"sequence":3,"kind":"source_local_finalization","output":finalization}
    ]);
    println!("{}", json!({"contract":"active-source-host-development-v1","run_id":input["run_id"],
        "engine_version":reasoning_harness_core::ENGINE_VERSION,"events":events,"finalization":finalization,
        "display_artifacts":display.map(|text|json!([{"id":"source-answer","kind":"final_answer","text":text}])).unwrap_or(json!([])),
        "direct_host_api_calls":{"derive_effective_evidence_local_qualification_v17":1,"materialize_evidence_relevance_v30":1},
        "normal_cli_changed":false,"controlled_qualification_not_model_generated":true,"truth_claims":artifact.claims.len(),
        "verification_receipts":artifact.verification_receipts.len(),"model_provider_calls":0,"actual_answer_f1":Value::Null,"acceptance":"NOT_EVALUATED"}));
}
