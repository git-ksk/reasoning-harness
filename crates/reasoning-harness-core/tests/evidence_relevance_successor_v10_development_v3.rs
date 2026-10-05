use std::{collections::BTreeSet, fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceLocalRelationScope, EvidenceRelevanceCandidate, EvidenceRelevanceDisposition,
    EvidenceRelevanceTargetPolicy, derive_effective_evidence_local_qualification_v14,
    materialize_evidence_relevance_v27,
};
use serde::Deserialize;

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
    task: String,
    authority_expectation: AuthorityExpectation,
    policy: EvidenceRelevanceTargetPolicy,
    candidate: EvidenceRelevanceCandidate,
    expected_proposal: reasoning_harness_core::EvidenceRelevanceBindingProposal,
    expected_local_qualification: reasoning_harness_core::EvidenceLocalQualificationV6,
    expected_disposition: EvidenceRelevanceDisposition,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AuthorityExpectation {
    RequireDifferent,
    RequireRequested,
    ForbidDifferent,
    PreserveRisk,
}

#[derive(Debug, Deserialize)]
struct PriorManifest {
    cases: Vec<PriorCase>,
}

#[derive(Debug, Deserialize)]
struct PriorCase {
    id: String,
    #[serde(default)]
    task: String,
    policy: EvidenceRelevanceTargetPolicy,
    candidate: EvidenceRelevanceCandidate,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn load<T: serde::de::DeserializeOwned>(relative: &str) -> T {
    serde_json::from_slice(
        &fs::read(repo_root().join(relative)).expect("read evidence relevance fixture"),
    )
    .expect("parse evidence relevance fixture")
}

fn normalized(value: &str) -> String {
    value
        .chars()
        .flat_map(char::to_lowercase)
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn windows(value: &str, size: usize) -> BTreeSet<String> {
    let normalized = normalized(value);
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();
    if tokens.len() < size {
        return BTreeSet::new();
    }
    tokens
        .windows(size)
        .map(|window| window.join(" "))
        .collect()
}

#[test]
fn successor_v10_v3_manifest_is_fresh_and_precommitted() {
    let fresh: Manifest =
        load("fixtures/evidence-relevance-successor-v10-development-v3/manifest.json");
    assert_eq!(
        fresh.suite_id,
        "evidence-relevance-successor-v10-development-v3"
    );
    assert_eq!(fresh.issue, 468);
    assert_eq!(fresh.status, "fresh_independent_development");
    assert_eq!(
        fresh.annotation_protocol_id,
        "evidence-relevance-effective-qualification-v14-materialization-v27"
    );
    assert_eq!(
        fresh.fixed_core_id,
        "evidence-relevance-fixed-core-successor-v10-development-v3"
    );
    assert_eq!(
        fresh.source_rule,
        "independently_authored_after_holdout_v10_and_successor_v10_development_v1_v2_observation_no_case_entity_task_signal_or_8token_surface_reuse"
    );
    assert_eq!(fresh.cases.len(), 20);

    let mut counts = [0usize; 4];
    for case in &fresh.cases {
        counts[match case.authority_expectation {
            AuthorityExpectation::RequireDifferent => 0,
            AuthorityExpectation::RequireRequested => 1,
            AuthorityExpectation::ForbidDifferent => 2,
            AuthorityExpectation::PreserveRisk => 3,
        }] += 1;
    }
    assert_eq!(counts, [8, 8, 3, 1]);

    let prior_manifests: [PriorManifest; 3] = [
        load("fixtures/evidence-relevance-holdout-v10/manifest.json"),
        load("fixtures/evidence-relevance-successor-v10-development/manifest.json"),
        load("fixtures/evidence-relevance-successor-v10-development-v2/manifest.json"),
    ];
    let prior_cases = prior_manifests
        .iter()
        .flat_map(|manifest| manifest.cases.iter())
        .collect::<Vec<_>>();

    let old_ids = prior_cases
        .iter()
        .map(|case| case.id.as_str())
        .collect::<BTreeSet<_>>();
    let old_entities = prior_cases
        .iter()
        .filter_map(|case| case.policy.entity.as_ref())
        .map(|entity| normalized(&entity.canonical_name))
        .collect::<BTreeSet<_>>();
    let old_tasks = prior_cases
        .iter()
        .map(|case| {
            if case.task.is_empty() {
                normalized(&case.policy.target_question)
            } else {
                normalized(&case.task)
            }
        })
        .collect::<BTreeSet<_>>();
    let old_signals = prior_cases
        .iter()
        .flat_map(|case| case.candidate.signals.iter())
        .map(|signal| normalized(&signal.text))
        .collect::<BTreeSet<_>>();
    let old_windows = prior_cases
        .iter()
        .flat_map(|case| case.candidate.signals.iter())
        .flat_map(|signal| windows(&signal.text, 8))
        .collect::<BTreeSet<_>>();

    let mut fresh_ids = BTreeSet::new();
    for case in &fresh.cases {
        assert!(
            fresh_ids.insert(case.id.as_str()),
            "duplicate id: {}",
            case.id
        );
        assert!(
            !old_ids.contains(case.id.as_str()),
            "reused id: {}",
            case.id
        );
        let entity = case.policy.entity.as_ref().expect("fresh entity");
        assert!(
            !old_entities.contains(&normalized(&entity.canonical_name)),
            "reused entity: {}",
            case.id
        );
        assert!(
            !old_tasks.contains(&normalized(&case.task)),
            "reused task: {}",
            case.id
        );
        for signal in &case.candidate.signals {
            assert!(
                !old_signals.contains(&normalized(&signal.text)),
                "reused signal: {}",
                case.id
            );
            assert!(
                windows(&signal.text, 8).is_disjoint(&old_windows),
                "reused 8-token surface: {}",
                case.id
            );
        }
    }
}

#[test]
fn successor_v10_v3_expected_path_matches_v14_v27_contract() {
    let fresh: Manifest =
        load("fixtures/evidence-relevance-successor-v10-development-v3/manifest.json");

    for case in fresh.cases {
        let effective = derive_effective_evidence_local_qualification_v14(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .expect("derive v14");
        let assessment = materialize_evidence_relevance_v27(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .expect("materialize v27");

        assert_eq!(
            assessment.disposition, case.expected_disposition,
            "expected disposition mismatch: {}",
            case.id
        );

        match case.authority_expectation {
            AuthorityExpectation::RequireDifferent => assert_eq!(
                effective.relation_scope,
                EvidenceLocalRelationScope::DifferentRelation,
                "different relation missing: {}",
                case.id
            ),
            AuthorityExpectation::RequireRequested => assert_eq!(
                effective.relation_scope,
                EvidenceLocalRelationScope::RequestedRelation,
                "requested relation missing: {}",
                case.id
            ),
            AuthorityExpectation::ForbidDifferent => assert_ne!(
                effective.relation_scope,
                EvidenceLocalRelationScope::DifferentRelation,
                "negative control gained different relation: {}",
                case.id
            ),
            AuthorityExpectation::PreserveRisk => assert_eq!(
                effective.scope_risk, case.expected_local_qualification.scope_risk,
                "scope risk changed: {}",
                case.id
            ),
        }
    }
}
