use std::{collections::BTreeSet, fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceLocalQualificationV6, EvidenceRelevanceBindingProposal, EvidenceRelevanceCandidate,
    EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy,
    derive_effective_evidence_local_qualification_v11, materialize_evidence_relevance_v23,
};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct Manifest {
    suite_id: String,
    issue: u64,
    status: String,
    source_rule: String,
    annotation_protocol_id: String,
    fixed_core_id: String,
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
struct Case {
    id: String,
    authority_expectation: String,
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
    let path = root().join("fixtures/evidence-relevance-holdout-v10/manifest.json");
    serde_json::from_slice(&fs::read(path).expect("read holdout v5 manifest"))
        .expect("parse holdout v5 manifest")
}

fn load_json(path: &str) -> Value {
    serde_json::from_slice(&fs::read(root().join(path)).expect("read comparison manifest"))
        .expect("parse comparison manifest")
}

#[test]
fn holdout_v10_identity_and_distribution_are_frozen_preobservation() {
    let manifest = load();
    assert_eq!(manifest.suite_id, "evidence-relevance-holdout-v10");
    assert_eq!(manifest.issue, 468);
    assert_eq!(manifest.status, "fresh_unobserved_holdout");
    assert_eq!(
        manifest.source_rule,
        "independently_authored_after_holdout_v10_runner_freeze_no_holdout_v1_v2_v3_v4_v5_v6_v7_v8_v9_or_successor_v5_v6_v7_v8_v9_development_case_entity_task_signal_or_8token_surface_reuse"
    );
    assert_eq!(
        manifest.annotation_protocol_id,
        "evidence-relevance-effective-qualification-v11"
    );
    assert_eq!(manifest.fixed_core_id, "evidence-relevance-fixed-core-v10");
    assert_eq!(manifest.cases.len(), 26);

    let relevant = manifest
        .cases
        .iter()
        .filter(|case| case.expected_disposition == EvidenceRelevanceDisposition::Relevant)
        .count();
    let irrelevant = manifest
        .cases
        .iter()
        .filter(|case| case.expected_disposition == EvidenceRelevanceDisposition::Irrelevant)
        .count();
    let ambiguous = manifest
        .cases
        .iter()
        .filter(|case| case.expected_disposition == EvidenceRelevanceDisposition::Ambiguous)
        .count();
    assert_eq!((relevant, irrelevant, ambiguous), (8, 10, 8));

    let require_requested = manifest
        .cases
        .iter()
        .filter(|case| case.authority_expectation == "require_requested")
        .count();
    let forbid_requested = manifest
        .cases
        .iter()
        .filter(|case| case.authority_expectation == "forbid_requested")
        .count();
    let preserve_risk = manifest
        .cases
        .iter()
        .filter(|case| case.authority_expectation == "preserve_risk")
        .count();
    assert_eq!(
        (require_requested, forbid_requested, preserve_risk),
        (15, 6, 5)
    );
}

#[test]
fn holdout_v10_expected_observations_are_exact_under_frozen_v11_v23_semantics() {
    for case in load().cases {
        let effective = derive_effective_evidence_local_qualification_v11(
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

        let assessment = materialize_evidence_relevance_v23(
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
fn holdout_v10_has_no_exact_identity_or_surface_reuse_from_prior_observed_corpora() {
    let fresh = load_json("fixtures/evidence-relevance-holdout-v10/manifest.json");
    let prior_paths = [
        "fixtures/evidence-relevance-holdout-v1/manifest.json",
        "fixtures/evidence-relevance-holdout-v2/manifest.json",
        "fixtures/evidence-relevance-holdout-v3/manifest.json",
        "fixtures/evidence-relevance-holdout-v4/manifest.json",
        "fixtures/evidence-relevance-holdout-v5/manifest.json",
        "fixtures/evidence-relevance-holdout-v6/manifest.json",
        "fixtures/evidence-relevance-holdout-v7/manifest.json",
        "fixtures/evidence-relevance-holdout-v8/manifest.json",
        "fixtures/evidence-relevance-holdout-v9/manifest.json",
        "fixtures/evidence-relevance-successor-v5-development/manifest.json",
        "fixtures/evidence-relevance-successor-v6-development/manifest.json",
        "fixtures/evidence-relevance-successor-v7-development/manifest.json",
        "fixtures/evidence-relevance-successor-v7-development-v2/manifest.json",
        "fixtures/evidence-relevance-successor-v8-development/manifest.json",
        "fixtures/evidence-relevance-successor-v9-development/manifest.json",
    ];

    let cases = fresh["cases"].as_array().expect("fresh cases");
    let fresh_ids = cases
        .iter()
        .filter_map(|case| case["id"].as_str())
        .collect::<BTreeSet<_>>();
    let fresh_entities = cases
        .iter()
        .filter_map(|case| case.pointer("/policy/entity/canonical_name")?.as_str())
        .collect::<BTreeSet<_>>();
    let fresh_tasks = cases
        .iter()
        .filter_map(|case| case["task"].as_str())
        .collect::<BTreeSet<_>>();
    let fresh_signals = cases
        .iter()
        .flat_map(|case| {
            case.pointer("/candidate/signals")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(|signal| signal["text"].as_str())
        .collect::<BTreeSet<_>>();

    for path in prior_paths {
        let prior = load_json(path);
        let prior_cases = prior["cases"].as_array().expect("prior cases");
        let prior_ids = prior_cases
            .iter()
            .filter_map(|case| case["id"].as_str())
            .collect::<BTreeSet<_>>();
        let prior_entities = prior_cases
            .iter()
            .filter_map(|case| case.pointer("/policy/entity/canonical_name")?.as_str())
            .collect::<BTreeSet<_>>();
        let prior_tasks = prior_cases
            .iter()
            .filter_map(|case| case["task"].as_str())
            .collect::<BTreeSet<_>>();
        let prior_signals = prior_cases
            .iter()
            .flat_map(|case| {
                case.pointer("/candidate/signals")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
            })
            .filter_map(|signal| signal["text"].as_str())
            .collect::<BTreeSet<_>>();

        assert!(fresh_ids.is_disjoint(&prior_ids), "{path}: case id overlap");
        assert!(
            fresh_entities.is_disjoint(&prior_entities),
            "{path}: canonical entity overlap"
        );
        assert!(
            fresh_tasks.is_disjoint(&prior_tasks),
            "{path}: task overlap"
        );
        assert!(
            fresh_signals.is_disjoint(&prior_signals),
            "{path}: exact signal overlap"
        );

        fn grams8(values: &BTreeSet<&str>) -> BTreeSet<Vec<String>> {
            values
                .iter()
                .flat_map(|text| {
                    let tokens = text
                        .to_ascii_lowercase()
                        .split(|ch: char| !ch.is_ascii_alphanumeric())
                        .filter(|token| !token.is_empty())
                        .map(ToOwned::to_owned)
                        .collect::<Vec<_>>();
                    tokens
                        .windows(8)
                        .map(|window| window.to_vec())
                        .collect::<Vec<_>>()
                })
                .collect()
        }
        assert!(
            grams8(&fresh_signals).is_disjoint(&grams8(&prior_signals)),
            "{path}: exact 8-token signal n-gram overlap"
        );
    }
}

#[test]
fn holdout_v10_preserves_frozen_non_frame_relation_contract_and_operational_budget() {
    let manifest = load();
    let cases = manifest
        .cases
        .iter()
        .map(|case| (case.id.as_str(), case))
        .collect::<std::collections::BTreeMap<_, _>>();

    for id in [
        "v10h15_negative_availability_feature_support_non_frame",
        "v10h18_negative_exact_target_numeric_non_frame",
    ] {
        let case = cases.get(id).expect("contract-guard case");
        assert_eq!(
            case.expected_local_qualification.relation_scope,
            reasoning_harness_core::EvidenceLocalRelationScope::DifferentRelation,
            "{id}"
        );
        assert_eq!(
            case.expected_disposition,
            EvidenceRelevanceDisposition::Irrelevant,
            "{id}"
        );
    }

    for case in manifest.cases {
        assert_eq!(
            case.policy.assessment_budget.max_elapsed_ms, 90_000,
            "{} operational budget",
            case.id
        );
        match case.authority_expectation.as_str() {
            "require_requested" => assert_eq!(
                case.expected_local_qualification.relation_scope,
                reasoning_harness_core::EvidenceLocalRelationScope::RequestedRelation,
                "{} authority mode",
                case.id
            ),
            "forbid_requested" => assert_ne!(
                case.expected_local_qualification.relation_scope,
                reasoning_harness_core::EvidenceLocalRelationScope::RequestedRelation,
                "{} authority mode",
                case.id
            ),
            "preserve_risk" => assert_ne!(
                case.expected_local_qualification.scope_risk,
                reasoning_harness_core::EvidenceLocalBlockingReason::None,
                "{} authority mode",
                case.id
            ),
            other => panic!("{} unknown authority mode {other}", case.id),
        }
    }

    let frozen_dev: Value =
        load_json("fixtures/evidence-relevance-successor-v9-development/manifest.json");
    for id in [
        "sv9d_10_availability_feature_support_non_frame",
        "sv9d_19_exact_target_model_only_limit",
    ] {
        let case = frozen_dev["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["id"] == id)
            .unwrap();
        assert_eq!(
            case["expected_local_qualification"]["relation_scope"], "different_relation",
            "{id} frozen contract"
        );
        assert_eq!(case["expected_disposition"], "irrelevant", "{id}");
    }
}
