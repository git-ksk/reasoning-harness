use std::{fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceLocalBlockingReason, EvidenceLocalIdentityScope, EvidenceLocalQualificationV6,
    EvidenceLocalRelationScope, EvidenceRelevanceBinding, EvidenceRelevanceBindingProposal,
    EvidenceRelevanceCandidate, EvidenceRelevanceDisposition, EvidenceRelevanceIdentityRequirement,
    EvidenceRelevanceReason, EvidenceRelevanceRelationKind, EvidenceRelevanceSignal,
    EvidenceRelevanceSignalKind, EvidenceRelevanceTargetPolicy, EvidenceTargetEntityIdentity,
    classify_deterministic_local_scope_risk, derive_effective_evidence_local_qualification_v2,
    materialize_evidence_relevance_v16,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Manifest {
    suite_id: String,
    fixed_core_id: String,
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
struct Case {
    id: String,
    policy: EvidenceRelevanceTargetPolicy,
    candidate: EvidenceRelevanceCandidate,
    expected_proposal: EvidenceRelevanceBindingProposal,
    expected_local_qualification: EvidenceLocalQualificationV6,
    expected_disposition: EvidenceRelevanceDisposition,
}

#[derive(Debug, Deserialize)]
struct ReplayFixture {
    source: String,
    cases: Vec<ReplayOverride>,
}

#[derive(Debug, Deserialize)]
struct ReplayOverride {
    id: String,
    observed_proposal: EvidenceRelevanceBindingProposal,
    observed_local_qualification: EvidenceLocalQualificationV6,
}

#[derive(Debug, Deserialize)]
struct V19ReplayFixture {
    source: String,
    candidate_commit: String,
    providers: Vec<V19ProviderReplay>,
}

#[derive(Debug, Deserialize)]
struct V19ProviderReplay {
    provider: String,
    model: String,
    successful_case_ids: Vec<String>,
    overrides: Vec<V19ReplayOverride>,
}

#[derive(Debug, Deserialize)]
struct V19ReplayOverride {
    id: String,
    observed_proposal: Option<EvidenceRelevanceBindingProposal>,
    observed_local_qualification: Option<EvidenceLocalQualificationV6>,
}

#[derive(Debug, Deserialize)]
struct V20ReplayFixture {
    source: String,
    candidate_commit: String,
    providers: Vec<V20ProviderReplay>,
}

#[derive(Debug, Deserialize)]
struct V20ProviderReplay {
    provider: String,
    model: String,
    successful_case_ids: Vec<String>,
    overrides: Vec<V20ReplayOverride>,
}

#[derive(Debug, Deserialize)]
struct V20ReplayOverride {
    id: String,
    observed_proposal: Option<EvidenceRelevanceBindingProposal>,
    observed_local_qualification: Option<EvidenceLocalQualificationV6>,
}

fn replay_fixture() -> ReplayFixture {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../fixtures/evidence-relevance-calibration-v21/v18-mistral-mismatch-overrides.json",
    );
    serde_json::from_slice(&fs::read(path).expect("read immutable v18 replay fixture"))
        .expect("parse immutable v18 replay fixture")
}

fn v19_observation_replay() -> V19ReplayFixture {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/evidence-relevance-calibration-v21/v19-observation-replay.json");
    serde_json::from_slice(&fs::read(path).expect("read immutable v19 replay fixture"))
        .expect("parse immutable v19 replay fixture")
}

fn v20_observation_replay() -> V20ReplayFixture {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/evidence-relevance-calibration-v21/v20-observation-replay.json");
    serde_json::from_slice(&fs::read(path).expect("read immutable v20 replay fixture"))
        .expect("parse immutable v20 replay fixture")
}

fn synthetic_policy(
    target: &str,
    relation: EvidenceRelevanceRelationKind,
) -> EvidenceRelevanceTargetPolicy {
    EvidenceRelevanceTargetPolicy {
        policy_id: format!("property:{}", target.to_lowercase().replace(' ', "-")),
        target_id: format!("target:{}", target.to_lowercase().replace(' ', "-")),
        target_question: match relation {
            EvidenceRelevanceRelationKind::Availability => format!("Where is {target} available?"),
            EvidenceRelevanceRelationKind::Pricing => format!("What does {target} cost?"),
            EvidenceRelevanceRelationKind::Limit => format!("What is the {target} limit?"),
            EvidenceRelevanceRelationKind::General => format!("How does {target} sampling work?"),
            _ => format!("What does the source say about {target}?"),
        },
        entity: Some(EvidenceTargetEntityIdentity {
            canonical_id: format!("entity:{}", target.to_lowercase().replace(' ', "-")),
            canonical_name: target.into(),
            aliases: Vec::new(),
        }),
        relation,
        identity_requirement: EvidenceRelevanceIdentityRequirement::RequireHarnessAnchor,
        assessment_budget: Default::default(),
    }
}

fn synthetic_candidate(
    parts: &[(EvidenceRelevanceSignalKind, &str)],
) -> EvidenceRelevanceCandidate {
    EvidenceRelevanceCandidate {
        evidence_id: "property-evidence".into(),
        source_id: "property-source".into(),
        signals: parts
            .iter()
            .map(|(kind, text)| EvidenceRelevanceSignal {
                kind: *kind,
                text: (*text).into(),
            })
            .collect(),
    }
}

fn manifest() -> Manifest {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/evidence-relevance-calibration-v21/manifest.json");
    serde_json::from_slice(&fs::read(path).expect("read v21 calibration manifest"))
        .expect("parse v21 calibration manifest")
}

#[test]
fn v21_inherits_the_frozen_v18_fixed_core_without_relabeling() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    let v18: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join("evidence-relevance-calibration-v18/manifest.json"))
            .expect("read v18 manifest"),
    )
    .expect("parse v18 manifest");
    let v20: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join("evidence-relevance-calibration-v21/manifest.json"))
            .expect("read v20 manifest"),
    )
    .expect("parse v20 manifest");

    assert_eq!(v20["suite_id"], "evidence-relevance-calibration-v21");
    assert_eq!(v20["fixed_core_id"], "evidence-relevance-fixed-core-v1");
    assert_eq!(v20["cases"], v18["cases"]);
    assert_eq!(v20["cases"].as_array().map(Vec::len), Some(48));

    let manifest = manifest();
    assert_eq!(manifest.suite_id, "evidence-relevance-calibration-v21");
    assert_eq!(manifest.fixed_core_id, "evidence-relevance-fixed-core-v1");
    assert_eq!(manifest.cases.len(), 48);
}

#[test]
fn v21_typed_deterministic_risk_matches_all_48_frozen_expectations() {
    for case in manifest().cases {
        assert_eq!(
            classify_deterministic_local_scope_risk(&case.candidate),
            case.expected_local_qualification.scope_risk,
            "{}",
            case.id
        );
    }
}

#[test]
fn v21_effective_qualification_matches_all_48_frozen_expectations() {
    for case in manifest().cases {
        let effective = derive_effective_evidence_local_qualification_v2(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .unwrap_or_else(|error| panic!("{} effective qualification failed: {error}", case.id));

        assert_eq!(
            effective, case.expected_local_qualification,
            "{} effective qualification drifted",
            case.id
        );
    }
}

#[test]
fn v21_replays_immutable_v18_mistral_misses_to_zero_effective_qualification_misses() {
    let replay = replay_fixture();
    assert_eq!(
        replay.source,
        "v18_mistral_immutable_canonical_mismatch_overrides"
    );
    let overrides = replay
        .cases
        .into_iter()
        .map(|case| (case.id.clone(), case))
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(overrides.len(), 25);

    for case in manifest().cases {
        let (proposal, raw) = if let Some(observed) = overrides.get(&case.id) {
            (
                &observed.observed_proposal,
                &observed.observed_local_qualification,
            )
        } else {
            (&case.expected_proposal, &case.expected_local_qualification)
        };

        let effective = derive_effective_evidence_local_qualification_v2(
            &case.policy,
            &case.candidate,
            Some(proposal),
            Some(raw),
        )
        .unwrap_or_else(|error| panic!("{} replay qualification failed: {error}", case.id));
        assert_eq!(
            effective, case.expected_local_qualification,
            "{} immutable v18 replay effective qualification drifted",
            case.id
        );

        let assessment = materialize_evidence_relevance_v16(
            &case.policy,
            &case.candidate,
            Some(proposal),
            Some(raw),
        )
        .unwrap_or_else(|error| panic!("{} replay materialization failed: {error}", case.id));
        assert_eq!(
            assessment.disposition, case.expected_disposition,
            "{} immutable v18 replay materialization drifted",
            case.id
        );
    }
}

#[test]
fn v21_expected_effective_qualification_materializes_all_48_cases() {
    for case in manifest().cases {
        let assessment = materialize_evidence_relevance_v16(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .unwrap_or_else(|error| panic!("{} materialization failed: {error}", case.id));

        assert_eq!(
            assessment.disposition, case.expected_disposition,
            "{}",
            case.id
        );
    }
}

#[test]
fn v21_deterministic_risk_cannot_be_overridden_by_model_none() {
    let case = manifest()
        .cases
        .into_iter()
        .find(|case| {
            case.expected_local_qualification.scope_risk != EvidenceLocalBlockingReason::None
        })
        .expect("risk case");
    let raw = EvidenceLocalQualificationV6 {
        identity_scope: EvidenceLocalIdentityScope::ExactTarget,
        relation_scope: EvidenceLocalRelationScope::RequestedRelation,
        scope_risk: EvidenceLocalBlockingReason::None,
    };

    let effective = derive_effective_evidence_local_qualification_v2(
        &case.policy,
        &case.candidate,
        Some(&case.expected_proposal),
        Some(&raw),
    )
    .expect("effective qualification");

    assert_ne!(effective.scope_risk, EvidenceLocalBlockingReason::None);
    let assessment = materialize_evidence_relevance_v16(
        &case.policy,
        &case.candidate,
        Some(&case.expected_proposal),
        Some(&raw),
    )
    .expect("materialization");
    assert_eq!(
        assessment.disposition,
        EvidenceRelevanceDisposition::Ambiguous
    );
    assert!(
        assessment
            .reasons
            .contains(&EvidenceRelevanceReason::DeterministicLocalScopeRiskPresent)
    );
}

#[test]
fn v21_raw_spurious_risk_stays_diagnostic_when_effective_positive_is_safe() {
    let case = manifest()
        .cases
        .into_iter()
        .find(|case| {
            case.expected_local_qualification.scope_risk == EvidenceLocalBlockingReason::None
                && case.expected_disposition == EvidenceRelevanceDisposition::Relevant
        })
        .expect("clean positive case");
    let raw = EvidenceLocalQualificationV6 {
        identity_scope: case.expected_local_qualification.identity_scope,
        relation_scope: case.expected_local_qualification.relation_scope,
        scope_risk: EvidenceLocalBlockingReason::ContextGap,
    };

    let effective = derive_effective_evidence_local_qualification_v2(
        &case.policy,
        &case.candidate,
        Some(&case.expected_proposal),
        Some(&raw),
    )
    .expect("effective qualification");
    assert_eq!(effective.scope_risk, EvidenceLocalBlockingReason::None);

    let assessment = materialize_evidence_relevance_v16(
        &case.policy,
        &case.candidate,
        Some(&case.expected_proposal),
        Some(&raw),
    )
    .expect("materialization");
    assert_eq!(
        assessment.disposition,
        EvidenceRelevanceDisposition::Relevant
    );
}

#[test]
fn v21_target_negative_rule_accepts_unresolved_primary_only_for_distinct_target_no_risk() {
    let case = manifest()
        .cases
        .into_iter()
        .find(|case| {
            case.expected_local_qualification.identity_scope
                == EvidenceLocalIdentityScope::DistinctTarget
                && case.expected_local_qualification.scope_risk == EvidenceLocalBlockingReason::None
        })
        .expect("distinct target case");
    let proposal = EvidenceRelevanceBindingProposal {
        target_binding: EvidenceRelevanceBinding::Unresolved,
        relation_binding: EvidenceRelevanceBinding::Different,
    };

    let assessment = materialize_evidence_relevance_v16(
        &case.policy,
        &case.candidate,
        Some(&proposal),
        Some(&case.expected_local_qualification),
    )
    .expect("materialization");

    assert_eq!(
        assessment.disposition,
        EvidenceRelevanceDisposition::Irrelevant
    );
    assert!(
        assessment
            .reasons
            .contains(&EvidenceRelevanceReason::NegativeTargetDistinctEntityConfirmed)
    );
}

#[test]
fn v21_target_negative_rule_never_overrides_exact_primary_target() {
    let case = manifest()
        .cases
        .into_iter()
        .find(|case| {
            case.expected_local_qualification.identity_scope
                == EvidenceLocalIdentityScope::DistinctTarget
                && case.expected_local_qualification.scope_risk == EvidenceLocalBlockingReason::None
        })
        .expect("distinct target case");
    let proposal = EvidenceRelevanceBindingProposal {
        target_binding: EvidenceRelevanceBinding::Exact,
        relation_binding: EvidenceRelevanceBinding::Different,
    };

    let assessment = materialize_evidence_relevance_v16(
        &case.policy,
        &case.candidate,
        Some(&proposal),
        Some(&case.expected_local_qualification),
    )
    .expect("materialization");

    assert_ne!(
        assessment.disposition,
        EvidenceRelevanceDisposition::Relevant
    );
}

#[test]
fn v21_property_risk_classifier_is_name_invariant() {
    for (target, candidate_name) in [
        ("Orchid Trace", "Orchid Lens"),
        ("Copper Queue", "Copper Stream"),
        ("Silver Cache", "Silver Store"),
    ] {
        let mapping_candidate = synthetic_candidate(&[
            (
                EvidenceRelevanceSignalKind::SourceTitle,
                &format!("{candidate_name} availability"),
            ),
            (
                EvidenceRelevanceSignalKind::Excerpt,
                &format!(
                    "{candidate_name} is available in East. The material does not establish whether it succeeds {target}."
                ),
            ),
        ]);
        assert_eq!(
            classify_deterministic_local_scope_risk(&mapping_candidate),
            EvidenceLocalBlockingReason::IdentityMapping
        );

        let ownership_candidate = synthetic_candidate(&[
            (
                EvidenceRelevanceSignalKind::Heading,
                &format!("{target} / {candidate_name} pricing"),
            ),
            (
                EvidenceRelevanceSignalKind::Excerpt,
                "Premium row is shown, but the product column is outside this clipped passage.",
            ),
        ]);
        assert_eq!(
            classify_deterministic_local_scope_risk(&ownership_candidate),
            EvidenceLocalBlockingReason::Multiple
        );

        let clean_candidate = synthetic_candidate(&[
            (
                EvidenceRelevanceSignalKind::Heading,
                &format!("{target} limits"),
            ),
            (
                EvidenceRelevanceSignalKind::StructuredMetadata,
                &format!("Product: {target}; request limit: 700/minute"),
            ),
        ]);
        assert_eq!(
            classify_deterministic_local_scope_risk(&clean_candidate),
            EvidenceLocalBlockingReason::None
        );
    }
}

#[test]
fn v21_property_target_negative_rule_is_generic_and_fail_closed() {
    for (target, sibling) in [
        ("Orchid Trace", "Orchid Metrics"),
        ("Copper Queue", "Copper Stream"),
        ("Silver Cache", "Silver Store"),
    ] {
        let policy = synthetic_policy(target, EvidenceRelevanceRelationKind::General);
        let candidate = synthetic_candidate(&[
            (
                EvidenceRelevanceSignalKind::SourceTitle,
                &format!("{sibling} sampling"),
            ),
            (
                EvidenceRelevanceSignalKind::Excerpt,
                &format!("{sibling} samples telemetry every ten seconds."),
            ),
        ]);
        let proposal = EvidenceRelevanceBindingProposal {
            target_binding: EvidenceRelevanceBinding::Unresolved,
            relation_binding: EvidenceRelevanceBinding::Different,
        };
        let raw_safe = EvidenceLocalQualificationV6 {
            identity_scope: EvidenceLocalIdentityScope::DistinctTarget,
            relation_scope: EvidenceLocalRelationScope::DifferentRelation,
            scope_risk: EvidenceLocalBlockingReason::None,
        };

        let safe = materialize_evidence_relevance_v16(
            &policy,
            &candidate,
            Some(&proposal),
            Some(&raw_safe),
        )
        .expect("safe target-negative materialization");
        assert_eq!(safe.disposition, EvidenceRelevanceDisposition::Irrelevant);

        let raw_risky = EvidenceLocalQualificationV6 {
            scope_risk: EvidenceLocalBlockingReason::ContextGap,
            ..raw_safe
        };
        let blocked = materialize_evidence_relevance_v16(
            &policy,
            &candidate,
            Some(&proposal),
            Some(&raw_risky),
        )
        .expect("risky target-negative materialization");
        assert_eq!(blocked.disposition, EvidenceRelevanceDisposition::Ambiguous);

        let exact_primary = EvidenceRelevanceBindingProposal {
            target_binding: EvidenceRelevanceBinding::Exact,
            relation_binding: EvidenceRelevanceBinding::Different,
        };
        let exact_blocked = materialize_evidence_relevance_v16(
            &policy,
            &candidate,
            Some(&exact_primary),
            Some(&raw_safe),
        )
        .expect("exact-primary boundary");
        assert_ne!(
            exact_blocked.disposition,
            EvidenceRelevanceDisposition::Relevant
        );
        assert_ne!(
            exact_blocked.disposition,
            EvidenceRelevanceDisposition::Irrelevant
        );
    }
}

#[test]
fn v21_property_effective_identity_never_guesses_through_local_risk() {
    for (candidate, expected_risk) in [
        (
            synthetic_candidate(&[
                (
                    EvidenceRelevanceSignalKind::SourceTitle,
                    "Orchid Lens availability",
                ),
                (
                    EvidenceRelevanceSignalKind::Excerpt,
                    "The material does not establish whether Orchid Lens is a rename of Orchid Trace.",
                ),
            ]),
            EvidenceLocalBlockingReason::IdentityMapping,
        ),
        (
            synthetic_candidate(&[
                (
                    EvidenceRelevanceSignalKind::Heading,
                    "Orchid Trace / Orchid Trace Edge pricing",
                ),
                (
                    EvidenceRelevanceSignalKind::Excerpt,
                    "The shared row may belong to either product; the product column is outside this clip.",
                ),
            ]),
            EvidenceLocalBlockingReason::Multiple,
        ),
    ] {
        let policy = synthetic_policy("Orchid Trace", EvidenceRelevanceRelationKind::Availability);
        let proposal = EvidenceRelevanceBindingProposal {
            target_binding: EvidenceRelevanceBinding::Exact,
            relation_binding: EvidenceRelevanceBinding::Exact,
        };
        let raw = EvidenceLocalQualificationV6 {
            identity_scope: EvidenceLocalIdentityScope::ExactTarget,
            relation_scope: EvidenceLocalRelationScope::RequestedRelation,
            scope_risk: EvidenceLocalBlockingReason::None,
        };

        let effective = derive_effective_evidence_local_qualification_v2(
            &policy,
            &candidate,
            Some(&proposal),
            Some(&raw),
        )
        .expect("effective risk qualification");
        assert_eq!(effective.scope_risk, expected_risk);
        assert_eq!(
            effective.identity_scope,
            EvidenceLocalIdentityScope::Unresolved
        );
    }
}

#[test]
fn v21_relation_negative_rule_accepts_independently_different_relation_only_fail_closed() {
    let case = manifest()
        .cases
        .into_iter()
        .find(|case| {
            case.expected_local_qualification.identity_scope
                == EvidenceLocalIdentityScope::ExactTarget
                && case.expected_local_qualification.relation_scope
                    == EvidenceLocalRelationScope::DifferentRelation
                && case.expected_local_qualification.scope_risk == EvidenceLocalBlockingReason::None
                && case.expected_disposition == EvidenceRelevanceDisposition::Irrelevant
        })
        .expect("exact-target different-relation case");

    let unresolved_relation = EvidenceRelevanceBindingProposal {
        target_binding: EvidenceRelevanceBinding::Exact,
        relation_binding: EvidenceRelevanceBinding::Unresolved,
    };
    let accepted = materialize_evidence_relevance_v16(
        &case.policy,
        &case.candidate,
        Some(&unresolved_relation),
        Some(&case.expected_local_qualification),
    )
    .expect("relation-negative materialization");
    assert_eq!(
        accepted.disposition,
        EvidenceRelevanceDisposition::Irrelevant
    );
    assert!(
        accepted
            .reasons
            .contains(&EvidenceRelevanceReason::LocalQualificationRejectsRelation)
    );

    let exact_relation = EvidenceRelevanceBindingProposal {
        target_binding: EvidenceRelevanceBinding::Exact,
        relation_binding: EvidenceRelevanceBinding::Exact,
    };
    let conflicting_primary = materialize_evidence_relevance_v16(
        &case.policy,
        &case.candidate,
        Some(&exact_relation),
        Some(&case.expected_local_qualification),
    )
    .expect("conflicting primary relation boundary");
    assert_eq!(
        conflicting_primary.disposition,
        EvidenceRelevanceDisposition::Irrelevant
    );

    let risky_raw = EvidenceLocalQualificationV6 {
        scope_risk: EvidenceLocalBlockingReason::ContextGap,
        ..case.expected_local_qualification
    };
    let risky = materialize_evidence_relevance_v16(
        &case.policy,
        &case.candidate,
        Some(&unresolved_relation),
        Some(&risky_raw),
    )
    .expect("raw-risk boundary");
    assert_ne!(risky.disposition, EvidenceRelevanceDisposition::Irrelevant);
}

#[test]
fn v21_relation_negative_rule_never_crosses_deterministic_context_gap() {
    let policy = synthetic_policy("Orchid Trace", EvidenceRelevanceRelationKind::Pricing);
    let candidate = synthetic_candidate(&[
        (
            EvidenceRelevanceSignalKind::SourceTitle,
            "Orchid Trace pricing",
        ),
        (
            EvidenceRelevanceSignalKind::Excerpt,
            "The captured passage is truncated after: Orchid Trace includes ... and the pricing detail is omitted.",
        ),
    ]);
    assert_ne!(
        classify_deterministic_local_scope_risk(&candidate),
        EvidenceLocalBlockingReason::None
    );
    let proposal = EvidenceRelevanceBindingProposal {
        target_binding: EvidenceRelevanceBinding::Exact,
        relation_binding: EvidenceRelevanceBinding::Unresolved,
    };
    let raw = EvidenceLocalQualificationV6 {
        identity_scope: EvidenceLocalIdentityScope::ExactTarget,
        relation_scope: EvidenceLocalRelationScope::DifferentRelation,
        scope_risk: EvidenceLocalBlockingReason::None,
    };
    let assessment =
        materialize_evidence_relevance_v16(&policy, &candidate, Some(&proposal), Some(&raw))
            .expect("context-gap boundary");
    assert_eq!(
        assessment.disposition,
        EvidenceRelevanceDisposition::Ambiguous
    );
}

#[test]
fn v21_replays_immutable_v19_successful_observations_without_regression() {
    let replay = v19_observation_replay();
    assert_eq!(replay.source, "v19_immutable_canonical_run_36247789205");
    assert_eq!(
        replay.candidate_commit,
        "471bc11a83ba36dbaf5cb69a5fa4550b0e35b578"
    );

    let cases = manifest()
        .cases
        .into_iter()
        .map(|case| (case.id.clone(), case))
        .collect::<std::collections::BTreeMap<_, _>>();

    for provider in replay.providers {
        let expected_successes = match provider.provider.as_str() {
            "mistral" => {
                assert_eq!(provider.model, "ministral-8b-latest");
                48
            }
            "groq" => {
                assert_eq!(provider.model, "openai/gpt-oss-120b");
                10
            }
            "google" => {
                assert_eq!(provider.model, "gemini-3.5-flash-lite");
                46
            }
            other => panic!("unexpected replay provider {other}"),
        };
        assert_eq!(provider.successful_case_ids.len(), expected_successes);

        let overrides = provider
            .overrides
            .into_iter()
            .map(|entry| (entry.id.clone(), entry))
            .collect::<std::collections::BTreeMap<_, _>>();

        let mut exact = 0usize;
        for id in provider.successful_case_ids {
            let case = cases
                .get(&id)
                .unwrap_or_else(|| panic!("missing frozen case {id}"));
            let observed = overrides.get(&id);
            let proposal = observed
                .and_then(|entry| entry.observed_proposal.as_ref())
                .unwrap_or(&case.expected_proposal);
            let raw = observed
                .and_then(|entry| entry.observed_local_qualification.as_ref())
                .unwrap_or(&case.expected_local_qualification);

            if provider.provider != "google" {
                let effective = derive_effective_evidence_local_qualification_v2(
                    &case.policy,
                    &case.candidate,
                    Some(proposal),
                    Some(raw),
                )
                .unwrap_or_else(|error| {
                    panic!("{} {id} qualification: {error}", provider.provider)
                });
                assert_eq!(
                    effective, case.expected_local_qualification,
                    "{} {id} effective qualification regressed",
                    provider.provider
                );
            }

            let assessment = materialize_evidence_relevance_v16(
                &case.policy,
                &case.candidate,
                Some(proposal),
                Some(raw),
            )
            .unwrap_or_else(|error| panic!("{} {id} materialization: {error}", provider.provider));
            assert_eq!(
                assessment.disposition, case.expected_disposition,
                "{} {id} materialization regressed",
                provider.provider
            );
            exact += 1;
        }
        assert_eq!(exact, expected_successes);
    }
}

#[test]
fn v21_google_case13_is_the_only_intended_v14_to_v15_disposition_change() {
    let replay = v19_observation_replay();
    let google = replay
        .providers
        .into_iter()
        .find(|provider| provider.provider == "google")
        .expect("google replay");
    let overrides = google
        .overrides
        .into_iter()
        .map(|entry| (entry.id.clone(), entry))
        .collect::<std::collections::BTreeMap<_, _>>();
    let cases = manifest()
        .cases
        .into_iter()
        .map(|case| (case.id.clone(), case))
        .collect::<std::collections::BTreeMap<_, _>>();

    let mut changed = Vec::new();
    for id in google.successful_case_ids {
        let case = cases.get(&id).expect("frozen case");
        let observed = overrides.get(&id);
        let proposal = observed
            .and_then(|entry| entry.observed_proposal.as_ref())
            .unwrap_or(&case.expected_proposal);
        let raw = observed
            .and_then(|entry| entry.observed_local_qualification.as_ref())
            .unwrap_or(&case.expected_local_qualification);
        let old = reasoning_harness_core::materialize_evidence_relevance_v14(
            &case.policy,
            &case.candidate,
            Some(proposal),
            Some(raw),
        )
        .expect("v14 replay");
        let new = materialize_evidence_relevance_v16(
            &case.policy,
            &case.candidate,
            Some(proposal),
            Some(raw),
        )
        .expect("v15 replay");
        if old.disposition != new.disposition {
            changed.push((id, old.disposition, new.disposition));
        }
    }

    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].0, "13_same_service_different_feature");
    assert_eq!(changed[0].1, EvidenceRelevanceDisposition::Ambiguous);
    assert_eq!(changed[0].2, EvidenceRelevanceDisposition::Irrelevant);
}

#[test]
fn v21_replays_immutable_v20_successful_observations_under_authority_gate() {
    let replay = v20_observation_replay();
    assert_eq!(replay.source, "v20_immutable_canonical_run_36283988719");
    assert_eq!(
        replay.candidate_commit,
        "e9ca7cce6a60c9b69129375aeea62a60cc3b127f"
    );

    let cases = manifest()
        .cases
        .into_iter()
        .map(|case| (case.id.clone(), case))
        .collect::<std::collections::BTreeMap<_, _>>();

    for provider in replay.providers {
        let expected_successes = match provider.provider.as_str() {
            "mistral" => {
                assert_eq!(provider.model, "ministral-8b-latest");
                48
            }
            "groq" => {
                assert_eq!(provider.model, "openai/gpt-oss-120b");
                39
            }
            "google" => {
                assert_eq!(provider.model, "gemini-3.5-flash-lite");
                48
            }
            other => panic!("unexpected replay provider {other}"),
        };
        assert_eq!(provider.successful_case_ids.len(), expected_successes);

        let overrides = provider
            .overrides
            .into_iter()
            .map(|entry| (entry.id.clone(), entry))
            .collect::<std::collections::BTreeMap<_, _>>();

        let mut materialized_exact = 0usize;
        let mut authority_qualified = 0usize;
        for id in provider.successful_case_ids {
            let case = cases
                .get(&id)
                .unwrap_or_else(|| panic!("missing frozen case {id}"));
            let observed = overrides.get(&id);
            let proposal = observed
                .and_then(|entry| entry.observed_proposal.as_ref())
                .unwrap_or(&case.expected_proposal);
            let raw = observed
                .and_then(|entry| entry.observed_local_qualification.as_ref())
                .unwrap_or(&case.expected_local_qualification);

            let effective = derive_effective_evidence_local_qualification_v2(
                &case.policy,
                &case.candidate,
                Some(proposal),
                Some(raw),
            )
            .unwrap_or_else(|error| panic!("{} {id} qualification: {error}", provider.provider));

            assert_eq!(
                effective.scope_risk, case.expected_local_qualification.scope_risk,
                "{} {id} effective risk regressed",
                provider.provider
            );
            if case.expected_local_qualification.scope_risk == EvidenceLocalBlockingReason::None {
                assert_eq!(
                    effective.identity_scope, case.expected_local_qualification.identity_scope,
                    "{} {id} authority identity regressed",
                    provider.provider
                );
                assert_eq!(
                    effective.relation_scope, case.expected_local_qualification.relation_scope,
                    "{} {id} authority relation regressed",
                    provider.provider
                );
            }
            authority_qualified += 1;

            let assessment = materialize_evidence_relevance_v16(
                &case.policy,
                &case.candidate,
                Some(proposal),
                Some(raw),
            )
            .unwrap_or_else(|error| panic!("{} {id} materialization: {error}", provider.provider));
            assert_eq!(
                assessment.disposition, case.expected_disposition,
                "{} {id} materialization regressed",
                provider.provider
            );
            materialized_exact += 1;
        }

        assert_eq!(authority_qualified, expected_successes);
        assert_eq!(materialized_exact, expected_successes);
    }
}

#[test]
fn v21_groq_case13_is_the_only_intended_v20_terminal_change() {
    let replay = v20_observation_replay();
    let groq = replay
        .providers
        .into_iter()
        .find(|provider| provider.provider == "groq")
        .expect("groq replay");
    let overrides = groq
        .overrides
        .into_iter()
        .map(|entry| (entry.id.clone(), entry))
        .collect::<std::collections::BTreeMap<_, _>>();
    let cases = manifest()
        .cases
        .into_iter()
        .map(|case| (case.id.clone(), case))
        .collect::<std::collections::BTreeMap<_, _>>();

    let mut changed = Vec::new();
    for id in groq.successful_case_ids {
        let case = cases.get(&id).expect("frozen case");
        let observed = overrides.get(&id);
        let proposal = observed
            .and_then(|entry| entry.observed_proposal.as_ref())
            .unwrap_or(&case.expected_proposal);
        let raw = observed
            .and_then(|entry| entry.observed_local_qualification.as_ref())
            .unwrap_or(&case.expected_local_qualification);
        let old = reasoning_harness_core::materialize_evidence_relevance_v15(
            &case.policy,
            &case.candidate,
            Some(proposal),
            Some(raw),
        )
        .expect("v15 replay");
        let new = materialize_evidence_relevance_v16(
            &case.policy,
            &case.candidate,
            Some(proposal),
            Some(raw),
        )
        .expect("v16 replay");
        if old.disposition != new.disposition {
            changed.push((id, old.disposition, new.disposition));
        }
    }

    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].0, "13_same_service_different_feature");
    assert_eq!(changed[0].1, EvidenceRelevanceDisposition::Relevant);
    assert_eq!(changed[0].2, EvidenceRelevanceDisposition::Irrelevant);
}

#[test]
fn v21_relation_conflict_does_not_override_deterministic_requested_relation() {
    let policy = synthetic_policy("Orchid Trace", EvidenceRelevanceRelationKind::Pricing);
    let candidate = synthetic_candidate(&[
        (
            EvidenceRelevanceSignalKind::SourceTitle,
            "Orchid Trace pricing",
        ),
        (
            EvidenceRelevanceSignalKind::Excerpt,
            "Orchid Trace pricing is billed at 3 credits per unit.",
        ),
    ]);
    let proposal = EvidenceRelevanceBindingProposal {
        target_binding: EvidenceRelevanceBinding::Exact,
        relation_binding: EvidenceRelevanceBinding::Exact,
    };
    let raw = EvidenceLocalQualificationV6 {
        identity_scope: EvidenceLocalIdentityScope::ExactTarget,
        relation_scope: EvidenceLocalRelationScope::DifferentRelation,
        scope_risk: EvidenceLocalBlockingReason::None,
    };

    let effective = derive_effective_evidence_local_qualification_v2(
        &policy,
        &candidate,
        Some(&proposal),
        Some(&raw),
    )
    .expect("positive relation counterexample");
    assert_eq!(
        effective.relation_scope,
        EvidenceLocalRelationScope::RequestedRelation
    );
    let assessment =
        materialize_evidence_relevance_v16(&policy, &candidate, Some(&proposal), Some(&raw))
            .expect("positive relation materialization");
    assert_ne!(
        assessment.disposition,
        EvidenceRelevanceDisposition::Irrelevant
    );
}

#[test]
fn v21_relation_conflict_requires_strict_local_identity_anchor() {
    let policy = synthetic_policy("Orchid Trace", EvidenceRelevanceRelationKind::Pricing);
    let candidate = synthetic_candidate(&[(
        EvidenceRelevanceSignalKind::Excerpt,
        "Encryption uses customer-managed keys.",
    )]);
    let proposal = EvidenceRelevanceBindingProposal {
        target_binding: EvidenceRelevanceBinding::Exact,
        relation_binding: EvidenceRelevanceBinding::Exact,
    };
    let raw = EvidenceLocalQualificationV6 {
        identity_scope: EvidenceLocalIdentityScope::ExactTarget,
        relation_scope: EvidenceLocalRelationScope::DifferentRelation,
        scope_risk: EvidenceLocalBlockingReason::None,
    };

    let effective = derive_effective_evidence_local_qualification_v2(
        &policy,
        &candidate,
        Some(&proposal),
        Some(&raw),
    )
    .expect("identity-anchor counterexample");
    assert_ne!(
        effective.relation_scope,
        EvidenceLocalRelationScope::DifferentRelation
    );
    let assessment =
        materialize_evidence_relevance_v16(&policy, &candidate, Some(&proposal), Some(&raw))
            .expect("identity-anchor materialization");
    assert_ne!(
        assessment.disposition,
        EvidenceRelevanceDisposition::Irrelevant
    );
}
