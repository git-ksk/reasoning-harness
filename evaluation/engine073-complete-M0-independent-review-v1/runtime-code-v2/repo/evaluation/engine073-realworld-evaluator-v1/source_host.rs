use reasoning_harness_core::{
    Evidence, EvidenceMetadata, ReasoningArtifact, ScopeCoverage,
    SourceAttributionAuthorityCeiling, SourceAttributionBinding, SourceAttributionLocator,
    SourceAttributionProposal, SourceAttributionState, SourceAttributionTargetPolicy,
    SourceAttributionTransformKind, SourceTextSpan, append_source_attributed_claim,
    finalize_source_attributed_answer, materialize_source_attributed_claim,
};
#[cfg(engine07)]
use reasoning_harness_core::{
    TrustedSourceReviewAuthority, reconcile_source_attributed_targets,
    record_trusted_source_equivalence,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};
fn main() {
    let input: Value =
        serde_json::from_slice(&fs::read(std::env::args().nth(1).expect("input")).unwrap())
            .unwrap();
    let id = input["id"].as_str().unwrap();
    let mut artifact = ReasoningArtifact {
        task: input["question"].as_str().unwrap().into(),
        ..Default::default()
    };
    artifact.source_attribution = SourceAttributionState {
        targets: vec![SourceAttributionTargetPolicy {
            policy_id: format!("engine073-realworld-independent-v1:{id}"),
            target_id: id.into(),
            target_question: artifact.task.clone(),
            authority_ceiling: SourceAttributionAuthorityCeiling::SourceLocal,
            hard_verification_required: false,
        }],
        ..Default::default()
    };
    for (n, source) in input["sources"].as_array().unwrap().iter().enumerate() {
        let full_text = fs::read_to_string(source["normalized_path"].as_str().unwrap()).unwrap();
        let start = usize::try_from(source["start_byte"].as_u64().unwrap()).unwrap();
        let end = usize::try_from(source["end_byte"].as_u64().unwrap()).unwrap();
        let text = full_text.get(start..end).expect("valid UTF-8 source span");
        assert_eq!(Some(text), source["quote"].as_str(), "quote binding mismatch");
        let eid = format!("e:{n}");
        let bid = format!("b:{n}");
        let scope = source["scope"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    ScopeCoverage::Values {
                        values: BTreeSet::from([v.as_str().unwrap().to_owned()]),
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        artifact.evidence.push(Evidence {
            id: eid.clone(),
            source: source["source_id"].as_str().unwrap().into(),
            observation: full_text.clone(),
            facts: Default::default(),
            metadata: EvidenceMetadata {
                scope: Some(scope),
                ..Default::default()
            },
        });
        artifact
            .source_attribution
            .bindings
            .push(SourceAttributionBinding {
                id: bid.clone(),
                target_id: id.into(),
                evidence_id: eid,
                source_id: source["source_id"].as_str().unwrap().into(),
                source_url: Some(source["url"].as_str().unwrap().into()),
                locator: Some(SourceAttributionLocator {
                    section: Some(source["locator"].as_str().unwrap().into()),
                    ..Default::default()
                }),
                retrieved_at_unix_seconds: Some(
                    source["retrieved_at_unix_seconds"].as_i64().unwrap(),
                ),
                source_version: Some(source["source_version"].as_str().unwrap().into()),
                span: SourceTextSpan {
                    start_byte: start,
                    end_byte: end,
                },
            });
        let (_, claim) = materialize_source_attributed_claim(
            &artifact,
            format!("c:{n}"),
            None,
            &SourceAttributionProposal {
                target_id: id.into(),
                binding_ids: vec![bid],
                transform_kind: SourceAttributionTransformKind::ExactQuote,
                transformed_statement: None,
                source_language: Some("en".into()),
                output_language: Some("en".into()),
            },
            None,
        )
        .unwrap();
        append_source_attributed_claim(&mut artifact, None, claim).unwrap();
    }
    let selected = vec![id.to_owned()];
    let frozen = serde_json::to_vec(&artifact).unwrap();
    let original = finalize_source_attributed_answer(&artifact, &selected).unwrap();
    #[cfg(engine07)]
    let (status, view, refusal, unauthorized_reviewed, replay_equal) = {
        let authority =
            TrustedSourceReviewAuthority::new("engine073-independent-agent-controlled-reference-only").unwrap();
        let mut reviews = vec![];
        let mut refusal = None;
        if input["reference_authorized"] == true && input["advice_equivalent"] == true {
            match record_trusted_source_equivalence(&artifact, &authority, "c:0", "c:1") {
                Ok(r) => reviews.push(r),
                Err(e) => refusal = Some(e.to_string()),
            }
        }
        let result =
            reconcile_source_attributed_targets(&artifact, &selected, Some(&authority), &reviews)
                .unwrap();
        let replay =
            reconcile_source_attributed_targets(&artifact, &selected, Some(&authority), &reviews)
                .unwrap();
        let untrusted =
            reconcile_source_attributed_targets(&artifact, &selected, None, &[]).unwrap();
        let unauthorized = untrusted.target_answers.iter().any(|a| {
            matches!(
                a.status,
                reasoning_harness_core::SourceReconciliationStatus::ReviewedCompatible
            )
        });
        (
            serde_json::to_value(result.target_answers[0].status).unwrap(),
            serde_json::to_value(&result).unwrap(),
            refusal,
            unauthorized,
            result == replay,
        )
    };
    #[cfg(not(engine07))]
    let (status, view, refusal, unauthorized_reviewed, replay_equal) = (
        serde_json::to_value(original.status).unwrap(),
        serde_json::to_value(&original).unwrap(),
        None::<String>,
        false,
        original == finalize_source_attributed_answer(&artifact, &selected).unwrap(),
    );
    println!(
        "{}",
        json!({"id":id,"engine_version":reasoning_harness_core::ENGINE_VERSION,"original":original,"status":status,"view":view,"review_refusal":refusal,"unauthorized_reviewed":unauthorized_reviewed,"replay_equal":replay_equal,"artifact_unchanged":frozen==serde_json::to_vec(&artifact).unwrap(),"claims_promoted":!artifact.claims.is_empty()||!artifact.verification_receipts.is_empty(),"binding_count":artifact.source_attribution.bindings.len()})
    );
}
