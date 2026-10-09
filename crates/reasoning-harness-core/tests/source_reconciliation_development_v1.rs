//! Frozen-before-code 22-case development comparison for the additive #490 lane.
//! This is NOT the fresh independent Engine 0.7 holdout.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

use reasoning_harness_core::{
    Evidence, EvidenceMetadata, ReasoningArtifact, ScopeCoverage,
    SourceAttributionAuthorityCeiling, SourceAttributionBinding,
    SourceAttributionFinalizationStatus, SourceAttributionProposal, SourceAttributionState,
    SourceAttributionTargetPolicy, SourceAttributionTransformKind, SourceReconciliationStatus,
    SourceTextSpan, TrustedSourceCompatibilityReview, TrustedSourceReviewAuthority,
    append_source_attributed_claim, finalize_source_attributed_answer,
    materialize_source_attributed_claim, reconcile_source_attributed_answer,
    record_trusted_source_equivalence, validate_source_attribution_state,
};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
struct FixtureSet {
    contract: String,
    expected_cases: usize,
    cases: Vec<Fixture>,
}

#[derive(Debug, Deserialize)]
struct Fixture {
    id: String,
    description: String,
    statements: Vec<String>,
    targets: Vec<String>,
    review_pairs: Vec<Vec<usize>>,
    mutation: Option<String>,
    expected: String,
    reviewer_policy_id: String,
    versions: Vec<Option<String>>,
    scopes: Vec<Option<BTreeMap<String, String>>>,
}

fn fixtures() -> FixtureSet {
    serde_json::from_slice(
        &fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
                "../../fixtures/engine-0.7-source-reconciliation-development-v1/manifest.json",
            ),
        )
        .unwrap(),
    )
    .unwrap()
}

fn artifact(fixture: &Fixture) -> ReasoningArtifact {
    assert!(!fixture.statements.is_empty());
    assert_eq!(fixture.statements.len(), fixture.targets.len());
    assert_eq!(fixture.statements.len(), fixture.versions.len());
    assert_eq!(fixture.statements.len(), fixture.scopes.len());
    let mut result = ReasoningArtifact {
        task: "Quote only what sources state about a named feature.".into(),
        ..Default::default()
    };
    let mut state = SourceAttributionState::default();
    for i in 0..fixture.statements.len() {
        let target = &fixture.targets[i];
        if !state.targets.iter().any(|p| p.target_id == *target) {
            state.targets.push(SourceAttributionTargetPolicy {
                policy_id: format!("reconciliation-target-{target}"),
                target_id: target.clone(),
                target_question: "What do these sources state?".into(),
                authority_ceiling: SourceAttributionAuthorityCeiling::SourceLocal,
                hard_verification_required: false,
            });
        }
        let statement = fixture.statements[i].clone();
        let source_id = format!("source-{i}");
        let evidence_id = format!("evidence-{i}");
        let binding_id = format!("binding-{i}");
        let claim_id = format!("claim-{i}");
        result.evidence.push(Evidence {
            id: evidence_id.clone(),
            source: source_id.clone(),
            observation: statement.clone(),
            facts: Default::default(),
            metadata: EvidenceMetadata {
                scope: fixture.scopes[i].clone().map(|values| {
                    values
                        .into_iter()
                        .map(|(key, value)| {
                            (
                                key,
                                ScopeCoverage::Values {
                                    values: BTreeSet::from([value]),
                                },
                            )
                        })
                        .collect()
                }),
                ..Default::default()
            },
        });
        state.bindings.push(SourceAttributionBinding {
            id: binding_id.clone(),
            target_id: target.clone(),
            evidence_id,
            source_id,
            source_url: None,
            locator: None,
            retrieved_at_unix_seconds: Some(1_800_000_000 + i as i64),
            source_version: fixture.versions[i].clone(),
            span: SourceTextSpan {
                start_byte: 0,
                end_byte: statement.len(),
            },
        });
        result.source_attribution = state;
        let proposal = SourceAttributionProposal {
            target_id: target.clone(),
            binding_ids: vec![binding_id],
            transform_kind: SourceAttributionTransformKind::ExactQuote,
            transformed_statement: None,
            source_language: Some("en".into()),
            output_language: Some("en".into()),
        };
        let (_, claim) =
            materialize_source_attributed_claim(&result, claim_id, None, &proposal, None).unwrap();
        append_source_attributed_claim(&mut result, None, claim).unwrap();
        state = result.source_attribution.clone();
    }
    assert!(validate_source_attribution_state(&result).is_empty());
    result
}

fn targets(fixture: &Fixture) -> Vec<String> {
    fixture
        .targets
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn reviews_for(
    artifact: &ReasoningArtifact,
    fixture: &Fixture,
) -> Result<Vec<TrustedSourceCompatibilityReview>, String> {
    let claimed_authority =
        TrustedSourceReviewAuthority::new(fixture.reviewer_policy_id.clone()).unwrap();
    let mut reviews = Vec::new();
    for pair in &fixture.review_pairs {
        assert_eq!(pair.len(), 2);
        match record_trusted_source_equivalence(
            artifact,
            &claimed_authority,
            &format!("claim-{}", pair[0]),
            &format!("claim-{}", pair[1]),
        ) {
            Ok(review) => reviews.push(review),
            // Rejected incompatible pair stays on the original Conflict path.
            Err(error) if fixture.expected == "conflict" => {
                assert!(
                    matches!(
                    error,
                    reasoning_harness_core::SourceReconciliationError::DeterministicConflict
                    | reasoning_harness_core::SourceReconciliationError::DifferentEvidenceContext
                ),
                    "{}: unexpected review rejection: {error}",
                    fixture.id
                );
                return Ok(Vec::new());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(reviews)
}

fn exercise(fixture: &Fixture) -> (String, Value) {
    let mut artifact = artifact(fixture);
    let original_state = serde_json::to_value(&artifact).unwrap();
    let selection = targets(fixture);
    let original = finalize_source_attributed_answer(&artifact, &selection).unwrap();
    let reviews = reviews_for(&artifact, fixture);
    let authority = TrustedSourceReviewAuthority::new("trusted-review").unwrap();
    if fixture.expected == "error" && reviews.is_err() {
        return ("error".into(), json!({"review_rejected":true}));
    }
    let reviews = reviews.expect("valid review formation");
    if let Some(mutation) = &fixture.mutation {
        match mutation.as_str() {
            "observation" => {
                // Keep the original quote span valid to isolate *snapshot*
                // validation from v1 source-attribution validation.
                artifact.evidence[0]
                    .observation
                    .push_str(" Added trailing source information.");
            }
            "binding_version" => {
                artifact.source_attribution.bindings[0].source_version = Some("changed-r2".into());
            }
            other => panic!("unknown mutation {other}"),
        }
        assert!(validate_source_attribution_state(&artifact).is_empty());
    }
    let stable_before = serde_json::to_value(&artifact).unwrap();
    let reconciled =
        reconcile_source_attributed_answer(&artifact, &selection, Some(&authority), &reviews);
    assert_eq!(serde_json::to_value(&artifact).unwrap(), stable_before);
    if fixture.mutation.is_none() {
        assert_eq!(
            stable_before, original_state,
            "normal source state was mutated"
        );
    }
    match reconciled {
        Err(error) => {
            assert_eq!(fixture.expected, "error", "{}: {error}", fixture.id);
            ("error".into(), json!({"error":error.to_string()}))
        }
        Ok(view) => {
            assert_ne!(fixture.expected, "error", "{} must reject", fixture.id);
            // No human/machine truth status changed; original source-local
            // citations, conflict state and text are completely retained.
            assert_eq!(view.original, original);
            assert!(artifact.verification_receipts.is_empty());
            assert!(artifact.claims.is_empty());
            assert_eq!(view.original.citations.len(), fixture.statements.len());
            let observed = match view.status {
                SourceReconciliationStatus::Qualified => "qualified",
                SourceReconciliationStatus::ReviewedCompatible => "reviewed_compatible",
                SourceReconciliationStatus::Conflict => "conflict",
                SourceReconciliationStatus::Unresolved => "unresolved",
            };
            assert_eq!(observed, fixture.expected, "{}", fixture.id);
            if view.status == SourceReconciliationStatus::ReviewedCompatible {
                assert_eq!(
                    view.original.status,
                    SourceAttributionFinalizationStatus::Conflict
                );
                assert_eq!(view.remaining_conflict_target_ids.len(), 0);
                assert_eq!(view.reviewed_compatible_target_ids, vec!["t1"]);
            }
            // The trusted *host* must supply its authority AGAIN on replay.
            // Deserialized reviews alone are not valid authorization.
            let artifact_copy: ReasoningArtifact =
                serde_json::from_value(serde_json::to_value(&artifact).unwrap()).unwrap();
            let reviews_copy: Vec<TrustedSourceCompatibilityReview> =
                serde_json::from_value(serde_json::to_value(&reviews).unwrap()).unwrap();
            let reread = reconcile_source_attributed_answer(
                &artifact_copy,
                &selection,
                Some(&authority),
                &reviews_copy,
            )
            .unwrap();
            assert_eq!(
                view, reread,
                "replay changed reviewed disposition: {}",
                fixture.id
            );
            if !reviews_copy.is_empty() {
                assert!(
                    reconcile_source_attributed_answer(
                        &artifact_copy,
                        &selection,
                        None,
                        &reviews_copy
                    )
                    .is_err(),
                    "persisted reviews must never self-authorize"
                );
            }
            (
                observed.into(),
                json!({
                    "citations": view.original.citations.len(),
                    "legacy_conflict_targets": view.original.conflict_target_ids.len(),
                    "reviewed_targets": view.reviewed_compatible_target_ids,
                    "remaining_conflicts": view.remaining_conflict_target_ids,
                    "hard_receipts": artifact.verification_receipts.len(),
                    "replay_equal": true,
                    "legacy_text": view.original.text
                }),
            )
        }
    }
}

#[test]
fn precommitted_development_controls_are_fail_closed() {
    let spec = fixtures();
    assert_eq!(
        spec.contract,
        "engine-0.7-source-reconciliation-development-v1-spec"
    );
    assert_eq!(spec.expected_cases, 22);
    assert_eq!(spec.cases.len(), 22);
    let mut seen = BTreeSet::new();
    let mut passed = 0;
    for fixture in spec.cases {
        assert!(seen.insert(fixture.id.clone()), "duplicate fixture id");
        let (status, details) = exercise(&fixture);
        assert_eq!(status, fixture.expected, "{}", fixture.id);
        passed += 1;
        println!(
            "ENGINE_070_RECONCILIATION_CASE {}",
            json!({
                "id":fixture.id, "description":fixture.description,
                "expected":fixture.expected, "observed":status,
                "details":details
            })
        );
    }
    println!(
        "ENGINE_070_RECONCILIATION_SUMMARY {}",
        json!({
            "cases":passed,"passed":passed,"provider_attempts":0,"model_calls":0,
            "external_calls":0,"hard_authority_promotions":0,
            "source_state_mutations":0
        })
    );
}

#[test]
fn trusted_review_reordering_does_not_change_view() {
    let spec = fixtures();
    let fixture = spec.cases.iter().find(|case| case.id == "safe-09").unwrap();
    let artifact = artifact(fixture);
    let authority = TrustedSourceReviewAuthority::new("trusted-review").unwrap();
    let selection = targets(fixture);
    let mut reviews = reviews_for(&artifact, fixture).unwrap();
    let original =
        reconcile_source_attributed_answer(&artifact, &selection, Some(&authority), &reviews)
            .unwrap();
    reviews.reverse();
    let reordered =
        reconcile_source_attributed_answer(&artifact, &selection, Some(&authority), &reviews)
            .unwrap();
    assert_eq!(original, reordered);
}

#[test]
fn modal_temporal_and_scope_strengthening_cannot_be_reviewed_away() {
    let spec = fixtures();
    let fixture = spec
        .cases
        .iter()
        .find(|case| case.id == "equiv-01")
        .unwrap();
    let authority = TrustedSourceReviewAuthority::new("trusted-review").unwrap();
    // Extra development controls added after v1 spec freeze, not a replacement
    // for any frozen v1 observation or independent holdout.
    for (left, right) in [
        (
            "Service Basil may be available.",
            "Service Basil is available.",
        ),
        ("Service Basil was in beta.", "Service Basil is in beta."),
        (
            "Service Basil will be available.",
            "Service Basil is available.",
        ),
        (
            "Service Basil is available to some users.",
            "Service Basil is available to all users.",
        ),
        (
            "Service Basil is available if approved.",
            "Service Basil is available.",
        ),
        (
            "Service Basil is only available in region JP.",
            "Service Basil is available in region JP.",
        ),
        (
            "Service Basil is currently enabled.",
            "Service Basil is enabled.",
        ),
    ] {
        let mut generated = Fixture {
            id: "additional-negative-development".into(),
            description: "Post-v1 guard regression".into(),
            statements: vec![left.into(), right.into()],
            targets: vec!["t1".into(), "t1".into()],
            review_pairs: vec![vec![0, 1]],
            mutation: None,
            expected: "conflict".into(),
            reviewer_policy_id: "trusted-review".into(),
            versions: vec![None, None],
            scopes: vec![None, None],
        };
        // Materialize only the frozen 0.6.1 exact-quote source lane.
        let materialized = artifact(&generated);
        assert!(
            record_trusted_source_equivalence(&materialized, &authority, "claim-0", "claim-1")
                .is_err(),
            "must reject modality/time/scope change: {left} vs {right}"
        );
        let reconciled = reconcile_source_attributed_answer(
            &materialized,
            &targets(&generated),
            Some(&authority),
            &[],
        )
        .unwrap();
        assert_eq!(reconciled.status, SourceReconciliationStatus::Conflict);
        assert_eq!(reconciled.original.citations.len(), 2);
        generated.statements.clear();
    }
    assert_eq!(fixture.expected, "reviewed_compatible");
}

#[test]
fn reviewer_policy_and_serialized_anchor_tampering_fail_closed() {
    let spec = fixtures();
    let fixture = spec
        .cases
        .iter()
        .find(|case| case.id == "equiv-01")
        .unwrap();
    let artifact = artifact(fixture);
    let authority = TrustedSourceReviewAuthority::new("trusted-review").unwrap();
    let review =
        record_trusted_source_equivalence(&artifact, &authority, "claim-0", "claim-1").unwrap();
    assert!(
        reconcile_source_attributed_answer(&artifact, &targets(fixture), None, &[review.clone()])
            .is_err()
    );
    let mut wrong_policy = review.clone();
    wrong_policy.reviewer_policy_id = "model-output".into();
    assert!(
        reconcile_source_attributed_answer(
            &artifact,
            &targets(fixture),
            Some(&authority),
            &[wrong_policy]
        )
        .is_err()
    );
    let mut wrong_quote = review.clone();
    wrong_quote.first.claim.statement.push('!');
    assert!(
        reconcile_source_attributed_answer(
            &artifact,
            &targets(fixture),
            Some(&authority),
            &[wrong_quote]
        )
        .is_err()
    );
    let mut wrong_source = review.clone();
    wrong_source.second.evidence[0].source = "invented-source".into();
    assert!(
        reconcile_source_attributed_answer(
            &artifact,
            &targets(fixture),
            Some(&authority),
            &[wrong_source]
        )
        .is_err()
    );
    let mut wrong_contract = review.clone();
    wrong_contract.contract_id = "harness-source-compatibility-trusted-review-v999".into();
    assert!(
        reconcile_source_attributed_answer(
            &artifact,
            &targets(fixture),
            Some(&authority),
            &[wrong_contract]
        )
        .is_err()
    );
    assert!(
        reconcile_source_attributed_answer(
            &artifact,
            &["t1".into(), "t1".into()],
            Some(&authority),
            &[review.clone()]
        )
        .is_err()
    );
    assert!(
        reconcile_source_attributed_answer(&artifact, &["t2".into()], Some(&authority), &[review])
            .is_err()
    );
}

#[test]
fn no_review_and_empty_target_lists_have_no_side_effects() {
    let spec = fixtures();
    let fixture = spec
        .cases
        .iter()
        .find(|case| case.id == "equiv-01")
        .unwrap();
    let original = artifact(fixture);
    let bytes_before = serde_json::to_vec(&original).unwrap();
    let result = reconcile_source_attributed_answer(&original, &[], None, &[]).unwrap();
    assert_eq!(result.status, SourceReconciliationStatus::Unresolved);
    assert_eq!(result.original.citations.len(), 0);
    let result = reconcile_source_attributed_answer(&original, &["t1".into()], None, &[]).unwrap();
    assert_eq!(result.status, SourceReconciliationStatus::Conflict);
    assert_eq!(result.original.citations.len(), 2);
    assert_eq!(serde_json::to_vec(&original).unwrap(), bytes_before);
}
