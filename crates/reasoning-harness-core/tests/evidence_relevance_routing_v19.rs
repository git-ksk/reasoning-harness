use std::{fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceLocalBlockingReason, EvidenceLocalIdentityScope, EvidenceLocalQualificationV6,
    EvidenceLocalRelationScope, EvidenceRelevanceBinding, EvidenceRelevanceBindingProposal,
    EvidenceRelevanceCandidate, EvidenceRelevanceDisposition, EvidenceRelevanceIdentityRequirement,
    EvidenceRelevanceReason, EvidenceRelevanceRelationKind, EvidenceRelevanceSignal,
    EvidenceRelevanceSignalKind, EvidenceRelevanceTargetPolicy, EvidenceTargetEntityIdentity,
    classify_deterministic_local_scope_risk, derive_effective_evidence_local_qualification_v1,
    materialize_evidence_relevance_v14,
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

fn replay_fixture() -> ReplayFixture {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../fixtures/evidence-relevance-calibration-v19/v18-mistral-mismatch-overrides.json",
    );
    serde_json::from_slice(&fs::read(path).expect("read immutable v18 replay fixture"))
        .expect("parse immutable v18 replay fixture")
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
        .join("../../fixtures/evidence-relevance-calibration-v19/manifest.json");
    serde_json::from_slice(&fs::read(path).expect("read v19 calibration manifest"))
        .expect("parse v19 calibration manifest")
}

#[test]
fn v19_inherits_the_frozen_v18_fixed_core_without_relabeling() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    let v18: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join("evidence-relevance-calibration-v18/manifest.json"))
            .expect("read v18 manifest"),
    )
    .expect("parse v18 manifest");
    let v19: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join("evidence-relevance-calibration-v19/manifest.json"))
            .expect("read v19 manifest"),
    )
    .expect("parse v19 manifest");

    assert_eq!(v19["suite_id"], "evidence-relevance-calibration-v19");
    assert_eq!(v19["fixed_core_id"], "evidence-relevance-fixed-core-v1");
    assert_eq!(v19["cases"], v18["cases"]);
    assert_eq!(v19["cases"].as_array().map(Vec::len), Some(48));

    let manifest = manifest();
    assert_eq!(manifest.suite_id, "evidence-relevance-calibration-v19");
    assert_eq!(manifest.fixed_core_id, "evidence-relevance-fixed-core-v1");
    assert_eq!(manifest.cases.len(), 48);
}

#[test]
fn v19_typed_deterministic_risk_matches_all_48_frozen_expectations() {
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
fn v19_effective_qualification_matches_all_48_frozen_expectations() {
    for case in manifest().cases {
        let effective = derive_effective_evidence_local_qualification_v1(
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
fn v19_replays_immutable_v18_mistral_misses_to_zero_effective_qualification_misses() {
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

        let effective = derive_effective_evidence_local_qualification_v1(
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

        let assessment = materialize_evidence_relevance_v14(
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
fn v19_expected_effective_qualification_materializes_all_48_cases() {
    for case in manifest().cases {
        let assessment = materialize_evidence_relevance_v14(
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
fn v19_deterministic_risk_cannot_be_overridden_by_model_none() {
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

    let effective = derive_effective_evidence_local_qualification_v1(
        &case.policy,
        &case.candidate,
        Some(&case.expected_proposal),
        Some(&raw),
    )
    .expect("effective qualification");

    assert_ne!(effective.scope_risk, EvidenceLocalBlockingReason::None);
    let assessment = materialize_evidence_relevance_v14(
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
fn v19_raw_spurious_risk_stays_diagnostic_when_effective_positive_is_safe() {
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

    let effective = derive_effective_evidence_local_qualification_v1(
        &case.policy,
        &case.candidate,
        Some(&case.expected_proposal),
        Some(&raw),
    )
    .expect("effective qualification");
    assert_eq!(effective.scope_risk, EvidenceLocalBlockingReason::None);

    let assessment = materialize_evidence_relevance_v14(
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
fn v19_target_negative_rule_accepts_unresolved_primary_only_for_distinct_target_no_risk() {
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

    let assessment = materialize_evidence_relevance_v14(
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
fn v19_target_negative_rule_never_overrides_exact_primary_target() {
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

    let assessment = materialize_evidence_relevance_v14(
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
fn v19_property_risk_classifier_is_name_invariant() {
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
fn v19_property_target_negative_rule_is_generic_and_fail_closed() {
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

        let safe = materialize_evidence_relevance_v14(
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
        let blocked = materialize_evidence_relevance_v14(
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
        let exact_blocked = materialize_evidence_relevance_v14(
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
fn v19_property_effective_identity_never_guesses_through_local_risk() {
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

        let effective = derive_effective_evidence_local_qualification_v1(
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
