use std::{collections::BTreeSet, fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceLocalBlockingReason, EvidenceLocalIdentityScope, EvidenceLocalQualificationV6,
    EvidenceLocalRelationScope, EvidenceRelevanceBinding, EvidenceRelevanceBindingProposal,
    EvidenceRelevanceCandidate, EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy,
    derive_effective_evidence_local_qualification_v16, materialize_evidence_relevance_v29,
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
    PreserveAbsence,
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
    tokens.windows(size).map(|w| w.join(" ")).collect()
}

#[test]
fn successor_v11_manifest_is_fresh_and_precommitted() {
    let fresh: Manifest =
        load("fixtures/evidence-relevance-successor-v11-development/manifest.json");
    assert_eq!(
        fresh.suite_id,
        "evidence-relevance-successor-v11-development"
    );
    assert_eq!(fresh.issue, 468);
    assert_eq!(fresh.status, "fresh_independent_development");
    assert_eq!(
        fresh.annotation_protocol_id,
        "evidence-relevance-effective-qualification-v16-materialization-v29"
    );
    assert_eq!(
        fresh.fixed_core_id,
        "evidence-relevance-fixed-core-successor-v11-development-v1"
    );
    assert_eq!(
        fresh.source_rule,
        "independently_authored_after_holdout_v11_observation_no_holdout_v1_v2_v3_v4_v5_v6_v7_v8_v9_v10_v11_or_successor_v5_v6_v7_v8_v9_v10_development_case_entity_task_signal_or_8token_surface_reuse"
    );
    assert_eq!(fresh.cases.len(), 22);

    let mut counts = [0usize; 5];
    for case in &fresh.cases {
        counts[match case.authority_expectation {
            AuthorityExpectation::RequireDifferent => 0,
            AuthorityExpectation::RequireRequested => 1,
            AuthorityExpectation::ForbidDifferent => 2,
            AuthorityExpectation::PreserveRisk => 3,
            AuthorityExpectation::PreserveAbsence => 4,
        }] += 1;
    }
    assert_eq!(counts, [8, 4, 6, 2, 2]);

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
        "fixtures/evidence-relevance-holdout-v10/manifest.json",
        "fixtures/evidence-relevance-holdout-v11/manifest.json",
        "fixtures/evidence-relevance-successor-v5-development/manifest.json",
        "fixtures/evidence-relevance-successor-v6-development/manifest.json",
        "fixtures/evidence-relevance-successor-v7-development/manifest.json",
        "fixtures/evidence-relevance-successor-v7-development-v2/manifest.json",
        "fixtures/evidence-relevance-successor-v8-development/manifest.json",
        "fixtures/evidence-relevance-successor-v9-development/manifest.json",
        "fixtures/evidence-relevance-successor-v10-development/manifest.json",
        "fixtures/evidence-relevance-successor-v10-development-v2/manifest.json",
        "fixtures/evidence-relevance-successor-v10-development-v3/manifest.json",
        "fixtures/evidence-relevance-successor-v10-development-v4/manifest.json",
    ];
    let priors = prior_paths
        .iter()
        .map(|path| load::<PriorManifest>(path))
        .collect::<Vec<_>>();
    let prior_cases = priors
        .iter()
        .flat_map(|m| m.cases.iter())
        .collect::<Vec<_>>();

    let old_ids = prior_cases
        .iter()
        .map(|c| c.id.as_str())
        .collect::<BTreeSet<_>>();
    let old_entities = prior_cases
        .iter()
        .filter_map(|c| c.policy.entity.as_ref())
        .map(|e| normalized(&e.canonical_name))
        .collect::<BTreeSet<_>>();
    let old_tasks = prior_cases
        .iter()
        .map(|c| {
            if c.task.is_empty() {
                normalized(&c.policy.target_question)
            } else {
                normalized(&c.task)
            }
        })
        .collect::<BTreeSet<_>>();
    let old_signals = prior_cases
        .iter()
        .flat_map(|c| c.candidate.signals.iter())
        .map(|s| normalized(&s.text))
        .collect::<BTreeSet<_>>();
    let old_windows = prior_cases
        .iter()
        .flat_map(|c| c.candidate.signals.iter())
        .flat_map(|s| windows(&s.text, 8))
        .collect::<BTreeSet<_>>();

    let mut ids = BTreeSet::new();
    for case in &fresh.cases {
        assert!(ids.insert(case.id.as_str()), "duplicate id: {}", case.id);
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
fn successor_v11_expected_path_matches_v16_v29_contract() {
    let fresh: Manifest =
        load("fixtures/evidence-relevance-successor-v11-development/manifest.json");

    for case in fresh.cases {
        let effective = derive_effective_evidence_local_qualification_v16(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .expect("derive v16");
        let assessment = materialize_evidence_relevance_v29(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .expect("materialize v29");

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
                "risk changed: {}",
                case.id
            ),
            AuthorityExpectation::PreserveAbsence => assert_eq!(
                effective.relation_scope,
                EvidenceLocalRelationScope::RelationAbsent,
                "strict absence changed: {}",
                case.id
            ),
        }
    }
}

#[test]
fn successor_v11_harness_cues_recover_under_conservative_model_outputs() {
    let fresh: Manifest =
        load("fixtures/evidence-relevance-successor-v11-development/manifest.json");
    let proposal = EvidenceRelevanceBindingProposal {
        target_binding: EvidenceRelevanceBinding::Exact,
        relation_binding: EvidenceRelevanceBinding::Unresolved,
    };
    let raw = EvidenceLocalQualificationV6 {
        identity_scope: EvidenceLocalIdentityScope::ExactTarget,
        relation_scope: EvidenceLocalRelationScope::RelationAbsent,
        scope_risk: EvidenceLocalBlockingReason::None,
    };

    for case in fresh.cases.iter().filter(|case| {
        matches!(
            case.authority_expectation,
            AuthorityExpectation::RequireDifferent
        )
    }) {
        let effective = derive_effective_evidence_local_qualification_v16(
            &case.policy,
            &case.candidate,
            Some(&proposal),
            Some(&raw),
        )
        .expect("derive conservative v16");
        let assessment = materialize_evidence_relevance_v29(
            &case.policy,
            &case.candidate,
            Some(&proposal),
            Some(&raw),
        )
        .expect("materialize conservative v29");
        assert_eq!(
            effective.relation_scope,
            EvidenceLocalRelationScope::DifferentRelation,
            "Harness cue failed under conservative model outputs: {}",
            case.id
        );
        assert_eq!(
            assessment.disposition,
            EvidenceRelevanceDisposition::Irrelevant,
            "Harness cue failed to reject: {}",
            case.id
        );
    }
}

#[test]
fn successor_v11_generic_controls_resist_adversarial_model_different_votes() {
    let fresh: Manifest =
        load("fixtures/evidence-relevance-successor-v11-development/manifest.json");
    let proposal = EvidenceRelevanceBindingProposal {
        target_binding: EvidenceRelevanceBinding::Exact,
        relation_binding: EvidenceRelevanceBinding::Different,
    };
    let raw = EvidenceLocalQualificationV6 {
        identity_scope: EvidenceLocalIdentityScope::ExactTarget,
        relation_scope: EvidenceLocalRelationScope::DifferentRelation,
        scope_risk: EvidenceLocalBlockingReason::None,
    };

    for case in fresh.cases.iter().filter(|case| {
        matches!(
            case.authority_expectation,
            AuthorityExpectation::ForbidDifferent
        )
    }) {
        let effective = derive_effective_evidence_local_qualification_v16(
            &case.policy,
            &case.candidate,
            Some(&proposal),
            Some(&raw),
        )
        .expect("derive adversarial v16");
        let assessment = materialize_evidence_relevance_v29(
            &case.policy,
            &case.candidate,
            Some(&proposal),
            Some(&raw),
        )
        .expect("materialize adversarial v29");

        assert_ne!(
            effective.relation_scope,
            EvidenceLocalRelationScope::DifferentRelation,
            "generic material retained model-only negative authority: {}",
            case.id
        );
        assert_eq!(
            assessment.disposition,
            EvidenceRelevanceDisposition::Ambiguous,
            "generic material became terminal reject: {}",
            case.id
        );
    }
}

#[test]
fn successor_v11_strict_absence_survives_adversarial_model_different_votes() {
    let fresh: Manifest =
        load("fixtures/evidence-relevance-successor-v11-development/manifest.json");
    let proposal = EvidenceRelevanceBindingProposal {
        target_binding: EvidenceRelevanceBinding::Exact,
        relation_binding: EvidenceRelevanceBinding::Different,
    };
    let raw = EvidenceLocalQualificationV6 {
        identity_scope: EvidenceLocalIdentityScope::ExactTarget,
        relation_scope: EvidenceLocalRelationScope::DifferentRelation,
        scope_risk: EvidenceLocalBlockingReason::None,
    };

    for case in fresh.cases.iter().filter(|case| {
        matches!(
            case.authority_expectation,
            AuthorityExpectation::PreserveAbsence
        )
    }) {
        let effective = derive_effective_evidence_local_qualification_v16(
            &case.policy,
            &case.candidate,
            Some(&proposal),
            Some(&raw),
        )
        .expect("derive strict absence v16");
        let assessment = materialize_evidence_relevance_v29(
            &case.policy,
            &case.candidate,
            Some(&proposal),
            Some(&raw),
        )
        .expect("materialize strict absence v29");

        assert_eq!(
            effective.relation_scope,
            EvidenceLocalRelationScope::RelationAbsent,
            "strict absence authority changed: {}",
            case.id
        );
        assert_eq!(
            assessment.disposition,
            EvidenceRelevanceDisposition::Irrelevant,
            "strict absence failed to reject: {}",
            case.id
        );
    }
}
