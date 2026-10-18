// Isolated exact-quote renderer control. This deliberately makes no relevance
// decision and cannot stand in for a successful end-to-end relevance gate.
use reasoning_harness_core::{
    Evidence, EvidenceMetadata, ReasoningArtifact, SourceAttributionAuthorityCeiling,
    SourceAttributionBinding, SourceAttributionLocator, SourceAttributionProposal,
    SourceAttributionState, SourceAttributionTargetPolicy, SourceAttributionTransformKind,
    SourceTextSpan, append_source_attributed_claim, finalize_source_attributed_answer,
    materialize_source_attributed_claim,
};
use serde_json::{Value, json};

fn main() {
    let input: Value = serde_json::from_slice(&std::fs::read(std::env::args().nth(1).expect("input manifest")).unwrap()).unwrap();
    assert_eq!(input["contract"], "active-source-host-development-v1");
    let source = &input["source"];
    let body = std::fs::read_to_string(source["normalized_path"].as_str().unwrap()).unwrap();
    let start = usize::try_from(source["start_byte"].as_u64().unwrap()).unwrap();
    let end = usize::try_from(source["end_byte"].as_u64().unwrap()).unwrap();
    assert_eq!(body.get(start..end), source["quote"].as_str(), "quote differs from retained full source");
    let target = input["policy"]["target_id"].as_str().unwrap();
    let eid = source["evidence_id"].as_str().unwrap();
    let sid = source["source_id"].as_str().unwrap();
    let metadata: EvidenceMetadata = serde_json::from_value(source["metadata"].clone()).unwrap();
    let mut artifact = ReasoningArtifact { task: input["policy"]["target_question"].as_str().unwrap().into(), ..Default::default() };
    artifact.evidence.push(Evidence { id:eid.into(), source:sid.into(), observation:body, facts:Default::default(), metadata });
    artifact.source_attribution = SourceAttributionState {
        targets:vec![SourceAttributionTargetPolicy {
            policy_id:"isolated-exact-quote-renderer-control".into(), target_id:target.into(), target_question:artifact.task.clone(),
            authority_ceiling:SourceAttributionAuthorityCeiling::SourceLocal, hard_verification_required:false,
        }],
        bindings:vec![SourceAttributionBinding {
            id:"isolated-source-binding".into(), target_id:target.into(), evidence_id:eid.into(), source_id:sid.into(),
            source_url:Some(source["url"].as_str().unwrap().into()),
            locator:Some(SourceAttributionLocator { section:Some(source["locator"].as_str().unwrap().into()), ..Default::default() }),
            retrieved_at_unix_seconds:Some(source["retrieved_at_unix_seconds"].as_i64().unwrap()),
            source_version:Some(source["source_version"].as_str().unwrap().into()), span:SourceTextSpan { start_byte:start, end_byte:end },
        }], ..Default::default()
    };
    let (assessment, claim) = materialize_source_attributed_claim(&artifact, "isolated-source-claim", None, &SourceAttributionProposal {
        target_id:target.into(), binding_ids:vec!["isolated-source-binding".into()],
        transform_kind:SourceAttributionTransformKind::ExactQuote, transformed_statement:None,
        source_language:Some("en".into()), output_language:Some("en".into()),
    }, None).unwrap();
    append_source_attributed_claim(&mut artifact, None, claim).unwrap();
    let finalization = finalize_source_attributed_answer(&artifact, &[target.into()]).unwrap();
    println!("{}", json!({"contract":"isolated-exact-quote-renderer-control-v1", "run_id":format!("{}:isolated-renderer",input["run_id"].as_str().unwrap()),
        "engine_version":reasoning_harness_core::ENGINE_VERSION,"assessment":assessment,"finalization":finalization,
        "direct_relevance_api_calls":0,"manual_source_binding_control":true,"normal_cli_changed":false,
        "truth_claims":artifact.claims.len(),"verification_receipts":artifact.verification_receipts.len(),
        "model_provider_calls":0,"actual_answer_f1":Value::Null,"acceptance":"NOT_EVALUATED"}));
}
