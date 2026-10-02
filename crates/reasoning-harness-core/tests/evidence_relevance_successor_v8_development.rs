use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

use reasoning_harness_core::{
    EvidenceLocalQualificationV6, EvidenceRelevanceBindingProposal, EvidenceRelevanceCandidate,
    EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy,
    derive_effective_evidence_local_qualification_v10,
    derive_effective_evidence_local_qualification_v11, materialize_evidence_relevance_v22,
};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct Manifest {
    suite_id: String,
    status: String,
    source_rule: String,
    annotation_protocol_id: String,
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

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn load() -> Manifest {
    serde_json::from_slice(
        &fs::read(
            root().join("fixtures/evidence-relevance-successor-v8-development/manifest.json"),
        )
        .expect("read successor-v8 development manifest"),
    )
    .expect("parse successor-v8 development manifest")
}

fn load_json(path: &str) -> Value {
    serde_json::from_slice(&fs::read(root().join(path)).expect("read comparison manifest"))
        .expect("parse comparison manifest")
}

fn ngrams(value: &str, width: usize) -> BTreeSet<String> {
    let tokens = value
        .split_whitespace()
        .map(str::to_lowercase)
        .collect::<Vec<_>>();
    if tokens.len() < width {
        return BTreeSet::new();
    }
    tokens
        .windows(width)
        .map(|window| window.join(" "))
        .collect()
}

#[test]
fn successor_v8_development_metadata_and_distribution_are_fixed_before_live_observation() {
    let manifest = load();
    assert_eq!(
        manifest.suite_id,
        "evidence-relevance-successor-v8-development"
    );
    assert_eq!(manifest.status, "reusable_development_calibration");
    assert_eq!(
        manifest.annotation_protocol_id,
        "evidence-relevance-effective-qualification-v10"
    );
    assert_eq!(
        manifest.fixed_core_id,
        "evidence-relevance-fixed-core-successor-v8-development-v1"
    );
    assert!(manifest.source_rule.contains("implementation_checkpoint"));
    assert_eq!(manifest.cases.len(), 14);

    let requested = manifest
        .cases
        .iter()
        .filter(|case| {
            case.expected_local_qualification.relation_scope
                == reasoning_harness_core::EvidenceLocalRelationScope::RequestedRelation
        })
        .count();
    let different = manifest
        .cases
        .iter()
        .filter(|case| {
            case.expected_local_qualification.relation_scope
                == reasoning_harness_core::EvidenceLocalRelationScope::DifferentRelation
        })
        .count();
    let unresolved = manifest
        .cases
        .iter()
        .filter(|case| {
            case.expected_local_qualification.relation_scope
                == reasoning_harness_core::EvidenceLocalRelationScope::Unresolved
        })
        .count();
    assert_eq!((requested, different, unresolved), (6, 7, 1));
}

#[test]
fn successor_v8_development_is_exact_under_v10_with_frozen_v22_materialization() {
    for case in load().cases {
        let effective = derive_effective_evidence_local_qualification_v10(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .unwrap_or_else(|error| panic!("{} effective qualification: {error}", case.id));
        assert_eq!(
            effective, case.expected_local_qualification,
            "{} effective qualification",
            case.id
        );

        let assessment = materialize_evidence_relevance_v22(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .unwrap_or_else(|error| panic!("{} materialization: {error}", case.id));
        assert_eq!(
            assessment.disposition, case.expected_disposition,
            "{} materialization",
            case.id
        );
    }
}

#[test]
fn successor_v8_development_expected_inputs_remain_stable_under_v11_authority_floor() {
    for case in load().cases {
        let effective = derive_effective_evidence_local_qualification_v11(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .unwrap_or_else(|error| panic!("{} v11 effective qualification: {error}", case.id));
        assert_eq!(
            effective, case.expected_local_qualification,
            "{} v11 counterfactual replay",
            case.id
        );
    }
}

#[test]
fn successor_v8_development_has_zero_exact_identity_task_signal_or_8token_reuse() {
    let fresh = load_json("fixtures/evidence-relevance-successor-v8-development/manifest.json");
    let prior_paths = [
        "fixtures/evidence-relevance-holdout-v1/manifest.json",
        "fixtures/evidence-relevance-holdout-v2/manifest.json",
        "fixtures/evidence-relevance-holdout-v3/manifest.json",
        "fixtures/evidence-relevance-holdout-v4/manifest.json",
        "fixtures/evidence-relevance-holdout-v5/manifest.json",
        "fixtures/evidence-relevance-holdout-v6/manifest.json",
        "fixtures/evidence-relevance-holdout-v7/manifest.json",
        "fixtures/evidence-relevance-holdout-v8/manifest.json",
        "fixtures/evidence-relevance-successor-v5-development/manifest.json",
        "fixtures/evidence-relevance-successor-v6-development/manifest.json",
        "fixtures/evidence-relevance-successor-v7-development/manifest.json",
        "fixtures/evidence-relevance-successor-v7-development-v2/manifest.json",
    ];

    let fresh_cases = fresh["cases"].as_array().expect("fresh cases");
    let fresh_ids = fresh_cases
        .iter()
        .filter_map(|case| case["id"].as_str())
        .collect::<BTreeSet<_>>();
    let fresh_entities = fresh_cases
        .iter()
        .filter_map(|case| case.pointer("/policy/entity/canonical_name")?.as_str())
        .collect::<BTreeSet<_>>();
    let fresh_tasks = fresh_cases
        .iter()
        .filter_map(|case| case["task"].as_str())
        .collect::<BTreeSet<_>>();
    let fresh_signals = fresh_cases
        .iter()
        .flat_map(|case| {
            case.pointer("/candidate/signals")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(|signal| signal["text"].as_str())
        .collect::<BTreeSet<_>>();
    let fresh_ngrams = fresh_signals
        .iter()
        .flat_map(|signal| ngrams(signal, 8))
        .collect::<BTreeSet<_>>();

    for path in prior_paths {
        let prior = load_json(path);
        let cases = prior["cases"].as_array().expect("prior cases");
        let prior_ids = cases
            .iter()
            .filter_map(|case| case["id"].as_str())
            .collect::<BTreeSet<_>>();
        let prior_entities = cases
            .iter()
            .filter_map(|case| case.pointer("/policy/entity/canonical_name")?.as_str())
            .collect::<BTreeSet<_>>();
        let prior_tasks = cases
            .iter()
            .filter_map(|case| case["task"].as_str())
            .collect::<BTreeSet<_>>();
        let prior_signals = cases
            .iter()
            .flat_map(|case| {
                case.pointer("/candidate/signals")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
            })
            .filter_map(|signal| signal["text"].as_str())
            .collect::<BTreeSet<_>>();
        let prior_ngrams = prior_signals
            .iter()
            .flat_map(|signal| ngrams(signal, 8))
            .collect::<BTreeSet<_>>();

        assert!(
            fresh_ids.is_disjoint(&prior_ids),
            "case-id reuse against {path}"
        );
        assert!(
            fresh_entities.is_disjoint(&prior_entities),
            "canonical-entity reuse against {path}"
        );
        assert!(
            fresh_tasks.is_disjoint(&prior_tasks),
            "task reuse against {path}"
        );
        assert!(
            fresh_signals.is_disjoint(&prior_signals),
            "signal reuse against {path}"
        );
        assert!(
            fresh_ngrams.is_disjoint(&prior_ngrams),
            "8-token surface reuse against {path}"
        );
    }
}

#[derive(Debug, Deserialize)]
struct DevelopmentReplay {
    source: DevelopmentReplaySource,
    providers: Vec<ProviderReplay>,
}

#[derive(Debug, Deserialize)]
struct DevelopmentReplaySource {
    run_id: u64,
    candidate_commit: String,
    holdout_acceptance_evidence: bool,
    development_gate: String,
    historical_result: String,
}

#[derive(Debug, Deserialize)]
struct ProviderReplay {
    provider: String,
    model: String,
    observations: Vec<ObservationReplay>,
}

#[derive(Debug, Deserialize)]
struct ObservationReplay {
    id: String,
    observed_proposal: Option<EvidenceRelevanceBindingProposal>,
    observed_local_qualification: Option<EvidenceLocalQualificationV6>,
    materialized_disposition: Option<EvidenceRelevanceDisposition>,
}

#[test]
fn successor_v11_replays_v10_development_failure_as_relation_authority_evidence() {
    let manifest = load();
    let replay: DevelopmentReplay = serde_json::from_slice(
        &fs::read(
            root().join(
                "fixtures/evidence-relevance-successor-v8-development/observations-run-36978705359.json",
            ),
        )
        .expect("read v10 development replay"),
    )
    .expect("parse v10 development replay");

    assert_eq!(replay.source.run_id, 36_978_705_359);
    assert_eq!(
        replay.source.candidate_commit,
        "706902c9e7ccb6213b8f021c945159ffe86bff41"
    );
    assert!(!replay.source.holdout_acceptance_evidence);
    assert_eq!(
        replay.source.development_gate,
        "successor_owned_relation_semantics_v10"
    );
    assert_eq!(replay.source.historical_result, "failure");

    let summary: Value = load_json(
        "fixtures/evidence-relevance-successor-v8-development/summary-run-36978705359.json",
    );
    assert_eq!(summary["development_gate_passed"], false);
    assert_eq!(
        summary["providers"]["mistral"]["effective_authority_relation_scope_misses"],
        0
    );
    assert_eq!(
        summary["providers"]["google"]["effective_authority_relation_scope_misses"],
        3
    );

    let cases = manifest
        .cases
        .iter()
        .map(|case| (case.id.as_str(), case))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(cases.len(), 14);
    assert_eq!(replay.providers.len(), 2);

    for provider in &replay.providers {
        assert!(matches!(provider.provider.as_str(), "mistral" | "google"));
        assert!(!provider.model.is_empty());
        assert_eq!(provider.observations.len(), 14, "{}", provider.provider);
        let mut v11_relation_misses = Vec::new();
        let mut false_requested_relation = 0;
        let mut wrong_target_relevant = 0;

        for observed in &provider.observations {
            let case = cases.get(observed.id.as_str()).expect("known replay case");
            let effective = derive_effective_evidence_local_qualification_v11(
                &case.policy,
                &case.candidate,
                observed.observed_proposal.as_ref(),
                observed.observed_local_qualification.as_ref(),
            )
            .unwrap_or_else(|error| {
                panic!("{} {} v11 replay: {error}", provider.provider, observed.id)
            });
            if effective.relation_scope != case.expected_local_qualification.relation_scope {
                v11_relation_misses.push((observed.id.as_str(), effective.relation_scope));
            }
            if case.expected_local_qualification.relation_scope
                == reasoning_harness_core::EvidenceLocalRelationScope::DifferentRelation
                && effective.relation_scope
                    == reasoning_harness_core::EvidenceLocalRelationScope::RequestedRelation
            {
                false_requested_relation += 1;
            }
            assert_eq!(
                effective.identity_scope, case.expected_local_qualification.identity_scope,
                "{} {} identity",
                provider.provider, observed.id
            );
            assert_eq!(
                effective.scope_risk, case.expected_local_qualification.scope_risk,
                "{} {} risk",
                provider.provider, observed.id
            );

            let assessment = materialize_evidence_relevance_v22(
                &case.policy,
                &case.candidate,
                observed.observed_proposal.as_ref(),
                observed.observed_local_qualification.as_ref(),
            )
            .unwrap();
            assert_eq!(
                assessment.disposition, case.expected_disposition,
                "{} {} frozen materialization",
                provider.provider, observed.id
            );
            assert_eq!(
                observed.materialized_disposition,
                Some(case.expected_disposition),
                "{} {} captured materialization",
                provider.provider,
                observed.id
            );
            if case.expected_disposition != EvidenceRelevanceDisposition::Relevant
                && assessment.disposition == EvidenceRelevanceDisposition::Relevant
            {
                wrong_target_relevant += 1;
            }
        }

        if provider.provider == "mistral" {
            assert!(
                v11_relation_misses.is_empty(),
                "mistral should remain exact: {v11_relation_misses:?}"
            );
        } else {
            assert_eq!(
                v11_relation_misses,
                vec![
                    (
                        "sv8d_09_limit_non_frame",
                        reasoning_harness_core::EvidenceLocalRelationScope::Unresolved,
                    ),
                    (
                        "sv8d_10_change_non_frame",
                        reasoning_harness_core::EvidenceLocalRelationScope::Unresolved,
                    ),
                    (
                        "sv8d_12_benefit_non_frame",
                        reasoning_harness_core::EvidenceLocalRelationScope::Unresolved,
                    ),
                ],
                "google should change only the three v10 false-positive authority cases"
            );
        }
        assert_eq!(
            false_requested_relation, 0,
            "{} false RequestedRelation authority",
            provider.provider
        );
        assert_eq!(
            wrong_target_relevant, 0,
            "{} wrong-target Relevant",
            provider.provider
        );
    }
}
