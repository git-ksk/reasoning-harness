//! Precommitted, synthetic development suite for #490 target-local source output.
//! Not a fresh independent Mistral/Google/Groq holdout.
use std::{fs, path::PathBuf};

use reasoning_harness_core::{
    Evidence, EvidenceMetadata, ReasoningArtifact, SOURCE_RECONCILIATION_TARGET_ANSWER_CONTRACT_ID,
    SourceAttributionAuthorityCeiling, SourceAttributionBinding,
    SourceAttributionFinalizationStatus, SourceAttributionProposal, SourceAttributionState,
    SourceAttributionTargetPolicy, SourceAttributionTransformKind, SourceReconciliationStatus,
    SourceTextSpan, TrustedSourceCompatibilityReview, TrustedSourceReviewAuthority,
    append_source_attributed_claim, finalize_source_attributed_answer,
    materialize_source_attributed_claim, reconcile_source_attributed_targets,
    record_trusted_source_equivalence, validate_source_attribution_state,
};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
struct Suite {
    schema: String,
    issue: u64,
    kind: String,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Case {
    id: String,
    scenario: String,
    expected_target_statuses: Vec<String>,
    expected_original_status: String,
    expected_target_citations: Vec<usize>,
    expected_result: String,
}
fn suite() -> Suite {
    serde_json::from_slice(
        &fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/engine-0.7-target-presentation-dev-v1/manifest.json"),
        )
        .unwrap(),
    )
    .unwrap()
}
fn build(pairs: &[(&str, &str, &str)]) -> ReasoningArtifact {
    let mut artifact = ReasoningArtifact {
        task: "Quote only two fictional feature statements for each target.".into(),
        ..Default::default()
    };
    artifact.source_attribution = SourceAttributionState {
        targets: ["alpha", "beta"]
            .into_iter()
            .map(|id| SourceAttributionTargetPolicy {
                target_id: id.into(),
                policy_id: format!("frozen-target-{id}"),
                target_question: format!("What do admitted sources say about {id}?"),
                authority_ceiling: SourceAttributionAuthorityCeiling::SourceLocal,
                hard_verification_required: false,
            })
            .collect(),
        ..Default::default()
    };
    for (index, (target, _kind, statement)) in pairs.iter().enumerate() {
        let evidence_id = format!("e-{index}");
        let source_id = format!("s-{index}");
        let binding_id = format!("b-{index}");
        artifact.evidence.push(Evidence {
            id: evidence_id.clone(),
            source: source_id.clone(),
            observation: (*statement).into(),
            facts: Default::default(),
            metadata: EvidenceMetadata::default(),
        });
        artifact
            .source_attribution
            .bindings
            .push(SourceAttributionBinding {
                id: binding_id.clone(),
                target_id: (*target).into(),
                evidence_id,
                source_id,
                source_url: None,
                locator: None,
                retrieved_at_unix_seconds: Some(1_800_000_000),
                source_version: Some("fictional-r1".into()),
                span: SourceTextSpan {
                    start_byte: 0,
                    end_byte: statement.len(),
                },
            });
        let (_, claim) = materialize_source_attributed_claim(
            &artifact,
            format!("c-{index}"),
            None,
            &SourceAttributionProposal {
                target_id: (*target).into(),
                binding_ids: vec![binding_id],
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
    assert!(validate_source_attribution_state(&artifact).is_empty());
    artifact
}
fn cases(scenario: &str) -> (ReasoningArtifact, Vec<String>, Vec<(usize, usize)>) {
    const COMPAT: [&str; 2] = [
        "The fictional Zephyr Relay is in beta.",
        "The fictional Zephyr Relay remains in its beta phase.",
    ];
    const OPPOSED: [&str; 2] = [
        "The fictional Lynx Switch is enabled.",
        "The fictional Lynx Switch is disabled.",
    ];
    const IDENTICAL: [&str; 2] = [
        "The fictional Koala Index is active.",
        "The fictional Koala Index is active.",
    ];
    let alpha = if scenario == "qualified_plus_reviewed" || scenario == "single_qualified" {
        IDENTICAL
    } else {
        COMPAT
    };
    let beta = if scenario == "both_compatible" || scenario == "qualified_plus_reviewed" {
        COMPAT
    } else {
        OPPOSED
    };
    let multiple = matches!(
        scenario,
        "mixed"
            | "mixed_unreviewed"
            | "qualified_plus_reviewed"
            | "both_compatible"
            | "mixed_reverse"
            | "extraneous"
            | "replay"
    );
    let mut data = vec![
        ("alpha", "fiction", alpha[0]),
        ("alpha", "fiction", alpha[1]),
    ];
    if multiple {
        data.extend([("beta", "fiction", beta[0]), ("beta", "fiction", beta[1])]);
    }
    let artifact = build(&data);
    let mut chosen = if multiple {
        vec!["alpha".into(), "beta".into()]
    } else {
        vec!["alpha".into()]
    };
    let mut reviews = match scenario {
        "single_compatible" | "mixed" | "mixed_reverse" | "both_compatible" | "replay"
        | "stale" | "extraneous" => vec![(0, 1)],
        "qualified_plus_reviewed" => vec![(2, 3)],
        _ => vec![],
    };
    if scenario == "both_compatible" {
        reviews.push((2, 3));
    }
    match scenario {
        "mixed_reverse" => chosen.reverse(),
        "unresolved" => chosen = vec!["not-admitted-target".into()],
        "empty" => chosen.clear(),
        "duplicate" => chosen = vec!["alpha".into(), "alpha".into()],
        "extraneous" => chosen = vec!["beta".into()],
        _ => {}
    };
    (artifact, chosen, reviews)
}
fn status(value: SourceReconciliationStatus) -> &'static str {
    match value {
        SourceReconciliationStatus::Qualified => "qualified",
        SourceReconciliationStatus::ReviewedCompatible => "reviewed_compatible",
        SourceReconciliationStatus::Conflict => "conflict",
        SourceReconciliationStatus::Unresolved => "unresolved",
    }
}
fn original_status(value: SourceAttributionFinalizationStatus) -> &'static str {
    match value {
        SourceAttributionFinalizationStatus::Qualified => "qualified",
        SourceAttributionFinalizationStatus::Conflict => "conflict",
        SourceAttributionFinalizationStatus::Unresolved => "unresolved",
    }
}
fn exercise(case: &Case) -> (&'static str, Value) {
    let (mut artifact, selection, review_pairs) = cases(&case.scenario);
    let authority = TrustedSourceReviewAuthority::new("host-test-authority").unwrap();
    let signed = review_pairs
        .iter()
        .map(|&(a, b)| {
            record_trusted_source_equivalence(
                &artifact,
                &authority,
                &format!("c-{a}"),
                &format!("c-{b}"),
            )
            .unwrap()
        })
        .collect::<Vec<TrustedSourceCompatibilityReview>>();
    if case.scenario == "stale" {
        artifact.evidence[0]
            .observation
            .push_str(" Correction appended after the original quote.");
        assert!(validate_source_attribution_state(&artifact).is_empty());
    }
    let original_bytes = serde_json::to_vec(&artifact).unwrap();
    let baseline = finalize_source_attributed_answer(&artifact, &selection);
    let result =
        reconcile_source_attributed_targets(&artifact, &selection, Some(&authority), &signed);
    assert_eq!(original_bytes, serde_json::to_vec(&artifact).unwrap());
    if case.expected_result == "reject" {
        assert!(result.is_err(), "case {} accepted unsafe output", case.id);
        return ("reject", json!({"rejected":true}));
    }
    let presentation = result.unwrap();
    assert_eq!(
        presentation.contract_id,
        SOURCE_RECONCILIATION_TARGET_ANSWER_CONTRACT_ID
    );
    assert_eq!(presentation.original, baseline.unwrap());
    assert_eq!(
        case.expected_original_status,
        original_status(presentation.original.status)
    );
    let observed = presentation
        .target_answers
        .iter()
        .map(|answer| status(answer.status))
        .collect::<Vec<_>>();
    assert_eq!(case.expected_target_statuses, observed, "{}", case.id);
    let coverage = presentation
        .target_answers
        .iter()
        .map(|answer| answer.original.citations.len())
        .collect::<Vec<_>>();
    assert_eq!(case.expected_target_citations, coverage, "{}", case.id);
    let reconstructed_claims = presentation
        .target_answers
        .iter()
        .flat_map(|answer| answer.original.claim_ids.iter().cloned())
        .collect::<Vec<_>>();
    assert_eq!(reconstructed_claims, presentation.original.claim_ids);
    let reconstructed_citations = presentation
        .target_answers
        .iter()
        .flat_map(|answer| answer.original.citations.iter().cloned())
        .collect::<Vec<_>>();
    assert_eq!(reconstructed_citations, presentation.original.citations);
    assert!(artifact.claims.is_empty());
    assert!(artifact.verification_receipts.is_empty());
    // Repeated calculation never creates a new provider/source effect and
    // source binding snapshots are re-validated on deserialized replay.
    let restored_artifact: ReasoningArtifact = serde_json::from_slice(&original_bytes).unwrap();
    let restored_reviews: Vec<TrustedSourceCompatibilityReview> =
        serde_json::from_slice(&serde_json::to_vec(&signed).unwrap()).unwrap();
    let second = reconcile_source_attributed_targets(
        &restored_artifact,
        &selection,
        Some(&authority),
        &restored_reviews,
    )
    .unwrap();
    assert_eq!(second, presentation, "{} replay drift", case.id);
    (
        "ok",
        json!({
            "target_statuses":observed,
            "target_citations":coverage,
            "legacy_status":original_status(presentation.original.status),
            "legacy_citations":presentation.original.citations.len(),
            "replay_equal":true
        }),
    )
}
#[test]
fn frozen_target_local_presentation_scenarios() {
    let suite = suite();
    assert_eq!(suite.schema, "engine-0.7-target-presentation-dev-v1-spec");
    assert_eq!(suite.issue, 490);
    assert_eq!(suite.kind, "synthetic_development_only");
    assert_eq!(suite.cases.len(), 14);
    for case in &suite.cases {
        let (observed, details) = exercise(case);
        assert_eq!(observed, case.expected_result);
        println!(
            "ENGINE_070_TARGET_PRESENTATION_CASE {}",
            json!({"id":case.id,"expected":case.expected_result,"observed":observed,"details":details})
        );
    }
    println!(
        "ENGINE_070_TARGET_PRESENTATION_SUMMARY {}",
        json!({"cases":14,"passed":14,"model_calls":0,"provider_attempts":0,
          "external_acquisition":0,"truth_promotions":0})
    );
}
