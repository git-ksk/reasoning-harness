use std::{collections::BTreeSet, fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceLocalBlockingReason, EvidenceLocalIdentityScope, EvidenceLocalQualificationV6,
    EvidenceLocalRelationScope, EvidenceRelevanceBinding, EvidenceRelevanceBindingProposal,
    EvidenceRelevanceCandidate, EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy,
    derive_effective_evidence_local_qualification_v11,
    derive_effective_evidence_local_qualification_v14, materialize_evidence_relevance_v23,
    materialize_evidence_relevance_v26,
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
    #[serde(default)]
    task: String,
    authority_expectation: AuthorityExpectation,
    policy: EvidenceRelevanceTargetPolicy,
    candidate: EvidenceRelevanceCandidate,
    expected_proposal: EvidenceRelevanceBindingProposal,
    expected_local_qualification: EvidenceLocalQualificationV6,
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
    #[serde(default)]
    expected_proposal: Option<EvidenceRelevanceBindingProposal>,
    #[serde(default)]
    expected_local_qualification: Option<EvidenceLocalQualificationV6>,
    #[serde(default)]
    expected_disposition: Option<EvidenceRelevanceDisposition>,
}

#[derive(Debug, Deserialize)]
struct ProviderResult {
    observations: Vec<ProviderObservation>,
}

#[derive(Debug, Deserialize)]
struct ProviderObservation {
    id: String,
    observed_proposal: Option<EvidenceRelevanceBindingProposal>,
    observed_local_qualification: Option<EvidenceLocalQualificationV6>,
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
fn successor_v10_v2_manifest_is_fresh_and_precommitted() {
    let fresh: Manifest =
        load("fixtures/evidence-relevance-successor-v10-development-v2/manifest.json");
    assert_eq!(
        fresh.suite_id,
        "evidence-relevance-successor-v10-development-v2"
    );
    assert_eq!(fresh.issue, 468);
    assert_eq!(fresh.status, "fresh_independent_development");
    assert_eq!(
        fresh.annotation_protocol_id,
        "evidence-relevance-effective-qualification-v14-materialization-v26"
    );
    assert_eq!(
        fresh.fixed_core_id,
        "evidence-relevance-fixed-core-successor-v10-development-v2"
    );
    assert_eq!(
        fresh.source_rule,
        "independently_authored_after_holdout_v10_and_successor_v10_development_v1_observation_no_case_entity_task_signal_or_8token_surface_reuse"
    );
    assert_eq!(fresh.cases.len(), 18);

    let mut counts = [0usize; 4];
    for case in &fresh.cases {
        counts[match case.authority_expectation {
            AuthorityExpectation::RequireDifferent => 0,
            AuthorityExpectation::RequireRequested => 1,
            AuthorityExpectation::ForbidDifferent => 2,
            AuthorityExpectation::PreserveRisk => 3,
        }] += 1;
    }
    assert_eq!(counts, [8, 6, 3, 1]);

    let prior_manifests: [PriorManifest; 2] = [
        load("fixtures/evidence-relevance-holdout-v10/manifest.json"),
        load("fixtures/evidence-relevance-successor-v10-development/manifest.json"),
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
            "duplicate fresh id: {}",
            case.id
        );
        assert!(
            !old_ids.contains(case.id.as_str()),
            "reused id: {}",
            case.id
        );
        let entity = case.policy.entity.as_ref().expect("fresh case entity");
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
fn successor_v10_v2_harness_owned_cue_is_monotone_on_fresh_surface() {
    let fresh: Manifest =
        load("fixtures/evidence-relevance-successor-v10-development-v2/manifest.json");

    for case in &fresh.cases {
        let expected_effective = derive_effective_evidence_local_qualification_v14(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .expect("derive expected v14 qualification");
        let expected_assessment = materialize_evidence_relevance_v26(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .expect("materialize expected v26 disposition");
        assert_eq!(
            expected_assessment.disposition, case.expected_disposition,
            "precommitted expected path mismatch: {}",
            case.id
        );

        match case.authority_expectation {
            AuthorityExpectation::RequireDifferent => {
                assert_eq!(
                    expected_effective.relation_scope,
                    EvidenceLocalRelationScope::DifferentRelation,
                    "expected different-relation authority missing: {}",
                    case.id
                );

                let conservative_proposal = EvidenceRelevanceBindingProposal {
                    target_binding: EvidenceRelevanceBinding::Exact,
                    relation_binding: EvidenceRelevanceBinding::Unresolved,
                };
                let conservative_raw = EvidenceLocalQualificationV6 {
                    identity_scope: EvidenceLocalIdentityScope::ExactTarget,
                    relation_scope: EvidenceLocalRelationScope::RelationAbsent,
                    scope_risk: EvidenceLocalBlockingReason::None,
                };
                let baseline = materialize_evidence_relevance_v23(
                    &case.policy,
                    &case.candidate,
                    Some(&conservative_proposal),
                    Some(&conservative_raw),
                )
                .expect("v23 conservative baseline");
                let successor = materialize_evidence_relevance_v26(
                    &case.policy,
                    &case.candidate,
                    Some(&conservative_proposal),
                    Some(&conservative_raw),
                )
                .expect("v26 conservative successor");
                assert_eq!(
                    baseline.disposition,
                    EvidenceRelevanceDisposition::Ambiguous,
                    "fresh recovery case was already terminal under v23: {}",
                    case.id
                );
                assert_eq!(
                    successor.disposition,
                    EvidenceRelevanceDisposition::Irrelevant,
                    "fresh recovery case was not recovered by v26: {}",
                    case.id
                );
            }
            AuthorityExpectation::RequireRequested => assert_eq!(
                expected_effective.relation_scope,
                EvidenceLocalRelationScope::RequestedRelation,
                "requested relation lost: {}",
                case.id
            ),
            AuthorityExpectation::ForbidDifferent => assert_ne!(
                expected_effective.relation_scope,
                EvidenceLocalRelationScope::DifferentRelation,
                "negative control gained different-relation authority: {}",
                case.id
            ),
            AuthorityExpectation::PreserveRisk => {
                assert_eq!(
                    expected_effective.scope_risk, case.expected_local_qualification.scope_risk,
                    "scope risk changed: {}",
                    case.id
                );
                assert_ne!(
                    expected_effective.relation_scope,
                    EvidenceLocalRelationScope::DifferentRelation,
                    "risk case gained different-relation authority: {}",
                    case.id
                );
            }
        }
    }
}

#[test]
fn successor_v10_v2_replays_frozen_expected_contracts_without_change() {
    for relative in [
        "fixtures/evidence-relevance-successor-v9-development/manifest.json",
        "fixtures/evidence-relevance-holdout-v10/manifest.json",
    ] {
        let manifest: PriorManifest = load(relative);
        for case in manifest.cases {
            let proposal = case
                .expected_proposal
                .expect("historical expected proposal");
            let raw = case
                .expected_local_qualification
                .expect("historical expected local qualification");
            let expected_disposition = case
                .expected_disposition
                .expect("historical expected disposition");

            let v11 = derive_effective_evidence_local_qualification_v11(
                &case.policy,
                &case.candidate,
                Some(&proposal),
                Some(&raw),
            )
            .expect("historical v11 qualification");
            let v14 = derive_effective_evidence_local_qualification_v14(
                &case.policy,
                &case.candidate,
                Some(&proposal),
                Some(&raw),
            )
            .expect("historical v14 qualification");
            assert_eq!(
                v14, v11,
                "historical expected qualification changed: {}",
                case.id
            );

            let v23 = materialize_evidence_relevance_v23(
                &case.policy,
                &case.candidate,
                Some(&proposal),
                Some(&raw),
            )
            .expect("historical v23 materialization");
            let v26 = materialize_evidence_relevance_v26(
                &case.policy,
                &case.candidate,
                Some(&proposal),
                Some(&raw),
            )
            .expect("historical v26 materialization");
            assert_eq!(
                v23.disposition, expected_disposition,
                "historical expected v23 contract mismatch before successor: {}",
                case.id
            );
            assert_eq!(
                v26.disposition, v23.disposition,
                "historical expected materialization changed: {}",
                case.id
            );
        }
    }
}

#[test]
fn successor_v10_v2_historical_v10_observations_fix_only_the_known_terminal_miss() {
    let manifest: PriorManifest = load("fixtures/evidence-relevance-holdout-v10/manifest.json");
    let cases = manifest
        .cases
        .into_iter()
        .map(|case| (case.id.clone(), case))
        .collect::<std::collections::BTreeMap<_, _>>();

    let mut baseline_misses = Vec::new();
    let mut successor_misses = Vec::new();
    let mut changed = Vec::new();

    for provider in ["mistral", "google", "groq"] {
        let result: ProviderResult = load(&format!(
            "fixtures/evidence-relevance-holdout-v10-result/{provider}-run-37085574010.json"
        ));
        for observation in result.observations {
            let case = cases.get(&observation.id).expect("historical case");
            let proposal = observation
                .observed_proposal
                .as_ref()
                .expect("historical proposal");
            let raw = observation
                .observed_local_qualification
                .as_ref()
                .expect("historical qualification");
            let expected = case
                .expected_disposition
                .expect("historical expected disposition");

            let baseline = materialize_evidence_relevance_v23(
                &case.policy,
                &case.candidate,
                Some(proposal),
                Some(raw),
            )
            .expect("historical v23 replay")
            .disposition;
            let successor = materialize_evidence_relevance_v26(
                &case.policy,
                &case.candidate,
                Some(proposal),
                Some(raw),
            )
            .expect("historical v26 replay")
            .disposition;

            if baseline != expected {
                baseline_misses.push(format!("{provider}:{}", observation.id));
            }
            if successor != expected {
                successor_misses.push(format!("{provider}:{}", observation.id));
            }
            if successor != baseline {
                changed.push(format!(
                    "{provider}:{}:{baseline:?}->{successor:?}",
                    observation.id
                ));
            }
        }
    }

    assert_eq!(
        baseline_misses,
        vec!["groq:v10h18_negative_exact_target_numeric_non_frame"]
    );
    assert!(
        successor_misses.is_empty(),
        "v26 left historical v10 misses: {successor_misses:?}"
    );
    assert_eq!(
        changed,
        vec!["groq:v10h18_negative_exact_target_numeric_non_frame:Ambiguous->Irrelevant"]
    );
}
