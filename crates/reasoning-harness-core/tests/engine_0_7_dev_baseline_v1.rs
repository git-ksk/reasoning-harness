//! Frozen, *development-only* residual measurement for Engine 0.6.1.
//! The independent 0.7.0 holdout must use distinct, later-authored stimuli.
//! This test intentionally does not modify the Engine's semantics or call a model.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

use reasoning_harness_core::{
    Claim, ContextSufficiency, EpistemicState, Evidence, EvidenceAuthorityPolicy, EvidenceMetadata,
    EvidenceNeedMode, EvidenceNeedProposal, EvidenceNeedTargetKind, EvidenceNeedTargetPolicy,
    EvidenceQualificationFindingKind, EvidenceQualificationInspector, EvidenceRequirement,
    ExistingEvidenceReuseStatus, QualifiedStructuredFactVerifier, ReasoningArtifact, ScopeCoverage,
    SourceAttributionAuthorityCeiling, SourceAttributionBinding, SourceAttributionProposal,
    SourceAttributionState, SourceAttributionTargetPolicy, SourceAttributionTransformKind,
    SourceTextSpan, SuppliedContextState, TemporalValidity, Verifier,
    append_source_attributed_claim, finalize_source_attributed_answer, materialize_evidence_need,
    materialize_source_attributed_claim,
};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
struct Spec {
    schema: String,
    engine_version: String,
    exact_engine_baseline_tag: String,
    engine_baseline_commit: String,
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "family", rename_all = "snake_case")]
enum Case {
    Source {
        id: String,
        title: String,
        primary_target: String,
        sources: Vec<SourceCase>,
        expected_baseline: Value,
        gap_hypothesis: String,
    },
    Qualification {
        id: String,
        title: String,
        as_of_unix_seconds: Option<i64>,
        required_scope: Option<BTreeMap<String, Vec<String>>>,
        minimum_authority_class: Option<String>,
        target_value: String,
        records: Vec<FactCase>,
        expected_baseline: Value,
        gap_hypothesis: String,
    },
    Need {
        id: String,
        title: String,
        target_kind: EvidenceNeedTargetKind,
        baseline_mode: EvidenceNeedMode,
        minimum_mode: EvidenceNeedMode,
        existing_evidence: ExistingEvidenceReuseStatus,
        supplied_context: SuppliedContextState,
        context_sufficiency: ContextSufficiency,
        model_proposal: Option<EvidenceNeedMode>,
        trusted_verification_required: bool,
        expected_baseline: Value,
        gap_hypothesis: String,
    },
}

#[derive(Debug, Deserialize, Clone)]
struct SourceCase {
    source_id: String,
    observation: String,
    oracle_origin_group: Option<String>,
    source_version: Option<String>,
    retrieved_at_unix_seconds: Option<i64>,
    target_id: String,
}

#[derive(Debug, Deserialize, Clone)]
struct FactCase {
    id: String,
    key: String,
    value: String,
    valid_from: Option<i64>,
    valid_until: Option<i64>,
    provenance_class: Option<String>,
    scope: Option<BTreeMap<String, Vec<String>>>,
}

fn contract_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/engine-0.7-baseline-v1")
}

fn serialize<T: serde::Serialize>(value: &T) -> Value {
    serde_json::to_value(value).expect("result serialization")
}

fn scope(value: Option<BTreeMap<String, Vec<String>>>) -> Option<BTreeMap<String, ScopeCoverage>> {
    value.map(|values| {
        values
            .into_iter()
            .map(|(key, values)| {
                (
                    key,
                    ScopeCoverage::Values {
                        values: values.into_iter().collect::<BTreeSet<_>>(),
                    },
                )
            })
            .collect()
    })
}

fn source_observation(primary_target: &str, sources: Vec<SourceCase>) -> (Value, Value) {
    let mut artifact = ReasoningArtifact {
        task: "Explain what exact bound source passages say, without deciding external truth."
            .into(),
        ..Default::default()
    };
    let mut state = SourceAttributionState::default();
    let mut origin_groups = BTreeSet::new();
    let mut missing_origins = false;

    for (index, source) in sources.into_iter().enumerate() {
        if let Some(group) = source.oracle_origin_group {
            origin_groups.insert(group);
        } else {
            missing_origins = true;
        }
        if !state
            .targets
            .iter()
            .any(|target| target.target_id == source.target_id)
        {
            state.targets.push(SourceAttributionTargetPolicy {
                policy_id: format!("baseline-target-policy-{}", source.target_id),
                target_id: source.target_id.clone(),
                target_question: format!("What do sources say about {}?", source.target_id),
                authority_ceiling: SourceAttributionAuthorityCeiling::SourceLocal,
                hard_verification_required: false,
            });
        }
        let evidence_id = format!("baseline-evidence-{index}");
        let binding_id = format!("baseline-binding-{index}");
        let claim_id = format!("baseline-claim-{index}");
        let span_end = source.observation.len();
        artifact.evidence.push(Evidence {
            id: evidence_id.clone(),
            source: source.source_id.clone(),
            observation: source.observation,
            facts: BTreeMap::new(),
            metadata: EvidenceMetadata::default(),
        });
        state.bindings.push(SourceAttributionBinding {
            id: binding_id,
            target_id: source.target_id.clone(),
            evidence_id,
            source_id: source.source_id,
            source_url: None,
            locator: None,
            retrieved_at_unix_seconds: source.retrieved_at_unix_seconds,
            source_version: source.source_version,
            span: SourceTextSpan {
                start_byte: 0,
                end_byte: span_end,
            },
        });
        // Each source is quoted exactly through the existing Harness-owned constructor.
        // Oracle origin labels are intentionally NEVER passed to the Engine.
        // Use one source per attributed claim to avoid model-authored paraphrases.
        let binding = state.bindings.last().unwrap().clone();
        artifact.source_attribution = state.clone();
        let proposal = SourceAttributionProposal {
            target_id: source.target_id,
            binding_ids: vec![binding.id],
            transform_kind: SourceAttributionTransformKind::ExactQuote,
            transformed_statement: None,
            source_language: Some("en".into()),
            output_language: Some("en".into()),
        };
        let (_, claim) =
            materialize_source_attributed_claim(&artifact, &claim_id, None, &proposal, None)
                .expect("exact source quote should materialize");
        assert_eq!(
            claim.authority_ceiling,
            SourceAttributionAuthorityCeiling::SourceLocal
        );
        append_source_attributed_claim(&mut artifact, None, claim).expect("valid source append");
        state = artifact.source_attribution.clone();
    }
    let result = finalize_source_attributed_answer(&artifact, &[primary_target.to_string()])
        .expect("source-only finalization");
    assert!(artifact.verification_receipts.is_empty());
    assert!(artifact.claims.is_empty());
    assert!(
        result
            .citations
            .iter()
            .all(|c| !c.binding_id.is_empty() && !c.source_id.is_empty())
    );
    let observed = json!({
        "status": result.status,
        "citations": result.citations.len(),
        "conflict_targets": result.conflict_target_ids.len()
    });
    let diagnostic = json!({
        "oracle_origin_count": if missing_origins { None } else { Some(origin_groups.len()) },
        "origin_oracle_passed_to_engine": false,
        "hard_verification_receipts": artifact.verification_receipts.len(),
        "source_local_only": true,
        "exposed_text_len": result.text.as_ref().map_or(0, |text| text.len())
    });
    (observed, diagnostic)
}

fn qualification_observation(
    as_of: Option<i64>,
    required_scope: Option<BTreeMap<String, Vec<String>>>,
    minimum_authority_class: Option<String>,
    target_value: String,
    records: Vec<FactCase>,
) -> (Value, Value) {
    let proposition = reasoning_harness_core::Proposition {
        key: "service".into(),
        value: target_value,
    };
    let requirement = EvidenceRequirement {
        proposition: proposition.clone(),
        as_of_unix_seconds: as_of,
        scope: scope(required_scope),
        minimum_authority_class,
    };
    let authority_policy = EvidenceAuthorityPolicy {
        ranks: BTreeMap::from([("primary".into(), 2), ("secondary".into(), 1)]),
    };
    let evidence = records
        .into_iter()
        .map(|record| {
            let temporal = if record.valid_from.is_some() || record.valid_until.is_some() {
                Some(TemporalValidity {
                    effective_from_unix_seconds: record.valid_from,
                    effective_until_unix_seconds: record.valid_until,
                })
            } else {
                None
            };
            Evidence {
                id: record.id.clone(),
                source: record.id,
                observation: "Exact structured observation from the fixture".into(),
                facts: BTreeMap::from([(record.key, record.value)]),
                metadata: EvidenceMetadata {
                    temporal,
                    scope: scope(record.scope),
                    provenance_class: record.provenance_class,
                },
            }
        })
        .collect::<Vec<_>>();
    let artifact = ReasoningArtifact {
        task: "Determine this exact typed service fact from qualified evidence only".into(),
        evidence,
        evidence_requirements: vec![requirement.clone()],
        authority_policy: authority_policy.clone(),
        ..Default::default()
    };
    let inspection = EvidenceQualificationInspector.inspect(&artifact);
    let statuses = inspection
        .assessments
        .iter()
        .map(|x| serialize(&x.status))
        .collect::<Vec<_>>();
    let conflict = inspection
        .findings
        .iter()
        .any(|x| x.kind == EvidenceQualificationFindingKind::Conflict);
    let verifier = QualifiedStructuredFactVerifier::new(vec![requirement], authority_policy);
    let claim = Claim {
        id: "baseline-claim".into(),
        statement: "Service is enabled".into(),
        state: EpistemicState::Unknown,
        proposition: Some(proposition),
        evidence_ids: vec![],
    };
    let receipt = verifier.verify(&claim, &artifact.evidence);
    let observed = json!({
        "statuses": statuses,
        "receipt": receipt.as_ref().map(|x| serialize(&x.conclusion)).unwrap_or(json!("none")),
        "conflict": conflict
    });
    let diagnostic = json!({
        "qualified_records": inspection.assessments.iter().filter(|x|
            x.status == reasoning_harness_core::EvidenceQualificationStatus::Qualified).count(),
        "hard_findings": inspection.findings.iter().filter(|x|
            x.strength == reasoning_harness_core::FindingStrength::Hard).count(),
        "soft_findings": inspection.findings.iter().filter(|x|
            x.strength == reasoning_harness_core::FindingStrength::Soft).count(),
        "receipt_evidence_ids": receipt.as_ref().map_or(vec![], |r| r.evidence_ids.clone())
    });
    (observed, diagnostic)
}

#[allow(clippy::too_many_arguments)]
fn need_observation(
    kind: EvidenceNeedTargetKind,
    baseline_mode: EvidenceNeedMode,
    minimum_mode: EvidenceNeedMode,
    existing_evidence: ExistingEvidenceReuseStatus,
    supplied_context: SuppliedContextState,
    context_sufficiency: ContextSufficiency,
    model_proposal: Option<EvidenceNeedMode>,
    trusted_verification_required: bool,
) -> (Value, Value) {
    let target_id = "baseline-target".to_string();
    let policy = EvidenceNeedTargetPolicy {
        policy_id: "engine-0.7-baseline-policy-v1".into(),
        target_id: target_id.clone(),
        target_question: "What is the verified service status?".into(),
        target_kind: kind,
        baseline_mode,
        minimum_mode,
        model_downgrade_floor: None,
        allow_no_factual_evidence: true,
        allow_context_only: true,
        allow_external_optional: true,
        explicit_verification_intent: false,
        current_state_required: false,
        trusted_verification_required,
        supplied_context,
        context_sufficiency,
        existing_evidence,
    };
    let proposal = model_proposal.map(|mode| EvidenceNeedProposal { target_id, mode });
    let result = materialize_evidence_need(&policy, proposal.as_ref())
        .expect("valid baseline evidence-need policy");
    (
        json!({"acquisition": result.acquisition}),
        json!({
            "mode": result.mode,
            "reasons": result.reasons,
            "target_id": result.target_id,
            "no_external_acquisition_executed": true
        }),
    )
}

#[test]
fn engine_0_7_dev_baseline_is_measured_from_precommitted_0_6_1_spec() {
    let path = contract_dir().join("manifest.json");
    let raw = fs::read(path).expect("frozen spec manifest");
    let spec: Spec = serde_json::from_slice(&raw).expect("well-typed frozen dev spec");
    assert_eq!(spec.schema, "engine-0.7-baseline-spec-v1");
    assert_eq!(spec.exact_engine_baseline_tag, "engine-v0.6.1");
    assert_eq!(
        spec.engine_baseline_commit,
        "4abee90f0501f0d8fc09b453a0b45c942fbed69e"
    );

    let is_frozen_baseline = reasoning_harness_core::ENGINE_VERSION == spec.engine_version;
    let mut family_counts = BTreeMap::<String, usize>::new();
    let mut matching_predictions = 0;
    let mut gaps = BTreeMap::<String, usize>::new();
    let mut seen = BTreeSet::new();

    for case in spec.cases {
        let (id, title, family, expected, gap_hypothesis, measured) = match case {
            Case::Source {
                id,
                title,
                primary_target,
                sources,
                expected_baseline,
                gap_hypothesis,
            } => (
                id,
                title,
                "source",
                expected_baseline,
                gap_hypothesis,
                source_observation(&primary_target, sources),
            ),
            Case::Qualification {
                id,
                title,
                as_of_unix_seconds,
                required_scope,
                minimum_authority_class,
                target_value,
                records,
                expected_baseline,
                gap_hypothesis,
            } => (
                id,
                title,
                "qualification",
                expected_baseline,
                gap_hypothesis,
                qualification_observation(
                    as_of_unix_seconds,
                    required_scope,
                    minimum_authority_class,
                    target_value,
                    records,
                ),
            ),
            Case::Need {
                id,
                title,
                target_kind,
                baseline_mode,
                minimum_mode,
                existing_evidence,
                supplied_context,
                context_sufficiency,
                model_proposal,
                trusted_verification_required,
                expected_baseline,
                gap_hypothesis,
            } => (
                id,
                title,
                "need",
                expected_baseline,
                gap_hypothesis,
                need_observation(
                    target_kind,
                    baseline_mode,
                    minimum_mode,
                    existing_evidence,
                    supplied_context,
                    context_sufficiency,
                    model_proposal,
                    trusted_verification_required,
                ),
            ),
        };
        assert!(seen.insert(id.clone()), "duplicate fixture id: {id}");
        let (observed, diagnostics) = measured;
        let predicted_match = expected == observed;
        println!(
            "ENGINE_070_BASELINE_CASE {}",
            json!({
                "id": id, "title": title, "family": family,
                "gap_hypothesis": gap_hypothesis,
                "baseline_prediction_matches": predicted_match,
                "predicted_baseline": expected,
                "observed": observed,
                "diagnostics": diagnostics,
            })
        );
        if is_frozen_baseline {
            assert!(
                predicted_match,
                "0.6.1 baseline differs from frozen fixture: {id}"
            );
        }
        matching_predictions += usize::from(predicted_match);
        *family_counts.entry(family.to_string()).or_default() += 1;
        if gap_hypothesis != "none" {
            *gaps.entry(gap_hypothesis).or_default() += 1;
        }
    }

    assert_eq!(seen.len(), 29, "the full frozen baseline suite must run");
    assert_eq!(family_counts.get("source"), Some(&10));
    assert_eq!(family_counts.get("qualification"), Some(&11));
    assert_eq!(family_counts.get("need"), Some(&8));
    println!(
        "ENGINE_070_BASELINE_SUMMARY {}",
        json!({
            "engine_version": reasoning_harness_core::ENGINE_VERSION,
            "compared_to_frozen_0_6_1": is_frozen_baseline,
            "cases": seen.len(),
            "matches_predicted_baseline": matching_predictions,
            "by_family": family_counts,
            "gap_hypotheses": gaps,
            "model_calls": 0, "provider_attempts": 0,
            "external_acquisition_calls": 0,
            "provider_failures": 0
        })
    );
}

#[test]
fn source_order_and_evidence_order_cannot_change_baseline_authority_decisions() {
    let spec: Spec =
        serde_json::from_slice(&fs::read(contract_dir().join("manifest.json")).unwrap()).unwrap();
    let mut varied = 0;

    for case in spec.cases {
        match case {
            Case::Source {
                id,
                primary_target,
                mut sources,
                ..
            } if sources.len() > 1 => {
                let normal = source_observation(&primary_target, sources.clone()).0;
                sources.reverse();
                let reversed = source_observation(&primary_target, sources).0;
                assert_eq!(
                    normal, reversed,
                    "source insertion order changed conflict/citation status in {id}"
                );
                varied += 1;
            }
            Case::Qualification {
                id,
                as_of_unix_seconds,
                required_scope,
                minimum_authority_class,
                target_value,
                mut records,
                ..
            } if records.len() > 1 => {
                let (normal, _) = qualification_observation(
                    as_of_unix_seconds,
                    required_scope.clone(),
                    minimum_authority_class.clone(),
                    target_value.clone(),
                    records.clone(),
                );
                records.reverse();
                let (reversed, _) = qualification_observation(
                    as_of_unix_seconds,
                    required_scope,
                    minimum_authority_class,
                    target_value,
                    records,
                );
                assert_eq!(
                    normal["receipt"], reversed["receipt"],
                    "receipt changed: {id}"
                );
                assert_eq!(
                    normal["conflict"], reversed["conflict"],
                    "conflict changed: {id}"
                );
                let mut normal_statuses: Vec<String> =
                    serde_json::from_value(normal["statuses"].clone()).unwrap();
                let mut reversed_statuses: Vec<String> =
                    serde_json::from_value(reversed["statuses"].clone()).unwrap();
                normal_statuses.sort();
                reversed_statuses.sort();
                assert_eq!(
                    normal_statuses, reversed_statuses,
                    "qualifications changed: {id}"
                );
                varied += 1;
            }
            _ => {}
        }
    }
    assert!(
        varied >= 10,
        "expected cross-source and conflicting-fact controls"
    );
}
